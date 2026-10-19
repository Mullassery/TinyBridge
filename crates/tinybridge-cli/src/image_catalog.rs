//! Lima-compatible image catalog.
//!
//! The catalog is deliberately a small JSON contract (`images: []`) so users
//! can generate it from Lima's templates, a CI artifact, or a signed release
//! manifest. TinyBridge bundles a metadata-only catalog; downloading requires
//! an entry with a URL and SHA-256.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageCatalog {
    pub images: Vec<CatalogImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogImage {
    pub name: String,
    pub version: String,
    pub arch: String,
    #[serde(default)]
    pub size_mb: u32,
    #[serde(default = "default_format")]
    pub format: String,
    /// Download URL; optional for listing/planning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Lowercase/uppercase hex SHA-256; required for `--pull`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Optional cloud-init template name shipped by TinyBridge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cloud_init_template: Option<String>,
}

fn default_format() -> String {
    "qcow2".to_string()
}

impl ImageCatalog {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("cannot read image catalog {}", path.display()))?;
        let catalog: Self = serde_json::from_str(&text).context("invalid image catalog JSON")?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn validate(&self) -> Result<()> {
        if self.images.is_empty() {
            bail!("image catalog contains no images");
        }
        for image in &self.images {
            if image.name.trim().is_empty()
                || image.name.contains('/')
                || image.name.contains('\\')
                || image.name == "."
                || image.name == ".."
            {
                bail!("invalid image name '{}'", image.name);
            }
            if image.arch.trim().is_empty() {
                bail!("image '{}' has no architecture", image.name);
            }
            if image.format.trim().is_empty() {
                bail!("image '{}' has no format", image.name);
            }
            if let Some(url) = &image.url {
                if url.trim().is_empty() {
                    bail!("image '{}' has an empty URL", image.name);
                }
            }
            if let Some(sha) = &image.sha256 {
                if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
                    bail!("image '{}' has an invalid sha256", image.name);
                }
            }
        }
        Ok(())
    }

    pub fn find(&self, query: &str) -> Vec<&CatalogImage> {
        let query = query.to_lowercase();
        self.images
            .iter()
            .filter(|image| {
                image.name.to_lowercase().contains(&query)
                    || image.version.to_lowercase().contains(&query)
            })
            .collect()
    }

    /// Resolve a pullable image: exactly one match with URL + SHA-256.
    pub fn resolve_download(&self, query: &str) -> Result<&CatalogImage> {
        let matches = self.find(query);
        if matches.is_empty() {
            bail!("no image matches '{query}'");
        }
        if matches.len() > 1 {
            let names = matches
                .iter()
                .map(|image| image.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            bail!("'{query}' matches multiple images ({names}); use an exact name");
        }
        let image = matches[0];
        if image.url.is_none() || image.sha256.is_none() {
            bail!(
                "image '{}' has no download URL/SHA-256; use a catalog that provides both",
                image.name
            );
        }
        Ok(image)
    }
}

pub fn default_catalog_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".tinybridge").join("images.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog(json: &str) -> ImageCatalog {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn validates_and_finds_images() {
        let c = catalog(
            r#"{"images":[{"name":"ubuntu-24.04","version":"24.04","arch":"arm64","url":"https://example.test/u.qcow2","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}"#,
        );
        c.validate().unwrap();
        assert_eq!(c.find("ubuntu").len(), 1);
        assert!(c.resolve_download("24.04").is_ok());
    }

    #[test]
    fn rejects_traversal_and_bad_hashes() {
        let c = catalog(
            r#"{"images":[{"name":"../x","version":"1","arch":"arm64","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}"#,
        );
        assert!(c.validate().is_err());

        let c =
            catalog(r#"{"images":[{"name":"x","version":"1","arch":"arm64","sha256":"nope"}]}"#);
        assert!(c.validate().is_err());
    }

    #[test]
    fn pull_requires_url_and_hash() {
        let c = catalog(r#"{"images":[{"name":"x","version":"1","arch":"arm64"}]}"#);
        c.validate().unwrap();
        assert!(c.resolve_download("x").is_err());
    }
}
