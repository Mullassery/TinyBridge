use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::output;

#[derive(Parser)]
pub struct AssetsArgs {
    #[command(subcommand)]
    pub command: AssetsCommand,
}

#[derive(Subcommand)]
pub enum AssetsCommand {
    /// Download and SHA-256 verify guest assets from a JSON manifest.
    Fetch {
        #[arg(long, help = "Path to the asset manifest JSON")]
        manifest: PathBuf,
        #[arg(long, help = "Destination directory (default: ~/.tinybridge/assets)")]
        dest: Option<PathBuf>,
    },
}

#[derive(Debug, Deserialize)]
pub struct AssetManifest {
    pub assets: Vec<AssetEntry>,
}

#[derive(Debug, Deserialize)]
pub struct AssetEntry {
    pub name: String,
    pub url: String,
    pub sha256: String,
}

pub fn parse_manifest(text: &str) -> Result<AssetManifest> {
    let manifest: AssetManifest = serde_json::from_str(text).context("invalid asset manifest")?;
    if manifest.assets.is_empty() {
        bail!("asset manifest contains no assets");
    }
    for asset in &manifest.assets {
        if asset.name.is_empty()
            || asset.name.contains('/')
            || asset.name.contains('\\')
            || asset.name == "."
            || asset.name == ".."
        {
            bail!("invalid asset name '{}'", asset.name);
        }
        if asset.url.trim().is_empty() {
            bail!("asset '{}' has an empty URL", asset.name);
        }
        if asset.sha256.len() != 64 || !asset.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("asset '{}' has an invalid sha256", asset.name);
        }
    }
    Ok(manifest)
}

pub fn sha256_hex(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn default_dest() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".tinybridge").join("assets")
}

pub async fn execute(args: AssetsArgs) -> Result<()> {
    match args.command {
        AssetsCommand::Fetch { manifest, dest } => {
            let text = fs::read_to_string(&manifest)
                .with_context(|| format!("cannot read manifest {}", manifest.display()))?;
            let manifest = parse_manifest(&text)?;
            let dest = dest.unwrap_or_else(default_dest);
            fs::create_dir_all(&dest)?;

            for asset in manifest.assets {
                let target = dest.join(&asset.name);
                if target.is_file() {
                    if sha256_hex(&target)?.eq_ignore_ascii_case(&asset.sha256) {
                        output::print_info(&format!("{} already verified", asset.name));
                        continue;
                    }
                    output::print_warning(&format!(
                        "{} checksum mismatch; re-downloading",
                        asset.name
                    ));
                    fs::remove_file(&target)?;
                }

                let tmp = dest.join(format!(".{}.part", asset.name));
                output::print_info(&format!("downloading {}", asset.name));
                let status = Command::new("curl")
                    .arg("-fL")
                    .arg("--output")
                    .arg(&tmp)
                    .arg(&asset.url)
                    .status()
                    .context("failed to run curl")?;
                if !status.success() {
                    let _ = fs::remove_file(&tmp);
                    bail!("download failed for {}", asset.name);
                }
                let actual = sha256_hex(&tmp)?;
                if !actual.eq_ignore_ascii_case(&asset.sha256) {
                    let _ = fs::remove_file(&tmp);
                    bail!(
                        "checksum mismatch for {}: expected {}, got {}",
                        asset.name,
                        asset.sha256,
                        actual
                    );
                }
                fs::rename(&tmp, &target)?;
                output::print_success(&format!("{} -> {}", asset.name, target.display()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_manifest() {
        let manifest = parse_manifest(
            r#"{"assets":[{"name":"vmlinux","url":"https://example.test/vmlinux","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}"#,
        )
        .unwrap();
        assert_eq!(manifest.assets.len(), 1);
        assert_eq!(manifest.assets[0].name, "vmlinux");
    }

    #[test]
    fn rejects_path_traversal_names_and_bad_hashes() {
        assert!(parse_manifest(
            r#"{"assets":[{"name":"../x","url":"https://x","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}"#
        )
        .is_err());
        assert!(
            parse_manifest(r#"{"assets":[{"name":"x","url":"https://x","sha256":"nope"}]}"#)
                .is_err()
        );
        assert!(parse_manifest(r#"{"assets":[]}"#).is_err());
    }
}
