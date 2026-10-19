use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::commands::assets::{default_dest, download_verified};
use crate::image_catalog::{default_catalog_path, ImageCatalog};
use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct ImagesArgs {
    /// Search for images matching this term
    pub search: Option<String>,

    /// Show full details
    #[arg(long)]
    pub details: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Load a Lima-compatible catalog JSON instead of the bundled metadata list
    #[arg(long)]
    pub catalog: Option<PathBuf>,

    /// Download the matched catalog image (requires URL + SHA-256)
    #[arg(long)]
    pub pull: bool,

    /// Destination directory for --pull (default: ~/.tinybridge/assets)
    #[arg(long)]
    pub dest: Option<PathBuf>,
}

#[derive(serde::Serialize)]
struct Image {
    name: String,
    version: String,
    size_mb: u32,
    arch: String,
}

pub async fn execute(args: ImagesArgs, _socket: Option<PathBuf>) -> Result<()> {
    if let Some(catalog_path) = args.catalog {
        let catalog = ImageCatalog::load(&catalog_path)?;
        let selected: Vec<_> = match args.search.as_deref() {
            Some(query) => catalog.find(query),
            None => catalog.images.iter().collect(),
        };
        if selected.is_empty() {
            bail!("no images matched the search");
        }

        if args.pull {
            let query = args
                .search
                .as_deref()
                .context("--pull requires a search term/name")?;
            let image = catalog.resolve_download(query)?;
            let dest = args.dest.unwrap_or_else(default_dest);
            let url = image.url.as_deref().expect("validated URL");
            let sha = image.sha256.as_deref().expect("validated SHA-256");
            let path = download_verified(&image.name, url, sha, &dest)?;
            output::print_success(&format!("{} ready at {}", image.name, path.display()));
            if let Some(template) = &image.cloud_init_template {
                output::print_info(&format!("cloud-init template: {template}"));
            }
            return Ok(());
        }

        if args.json {
            println!("{}", serde_json::to_string_pretty(&selected)?);
        } else {
            output::print_header("Image Catalog");
            for image in selected {
                output::print_info(&format!(
                    "{} {} ({}, {} MB, {})",
                    image.name, image.version, image.arch, image.size_mb, image.format
                ));
                if args.details {
                    if let Some(url) = &image.url {
                        output::print_info(&format!("  url: {url}"));
                    }
                    if let Some(template) = &image.cloud_init_template {
                        output::print_info(&format!("  cloud-init: {template}"));
                    }
                }
            }
        }
        return Ok(());
    }

    if args.pull {
        bail!(
            "--pull requires --catalog <path> (the bundled list is metadata-only; \
             it has no download URLs or checksums)"
        );
    }

    let images = get_available_images();

    let filtered: Vec<_> = images
        .into_iter()
        .filter(|i| {
            args.search
                .as_ref()
                .map(|s| i.name.to_lowercase().contains(&s.to_lowercase()))
                .unwrap_or(true)
        })
        .collect();

    if args.json {
        let json = serde_json::to_string_pretty(&filtered)?;
        println!("{}", json);
    } else {
        output::print_header("Available Linux Images");
        for image in &filtered {
            output::print_image(&image.name, &image.version, image.size_mb, &image.arch);
        }
        output::print_info(&format!(
            "catalog: {} (create one to enable verified pulls)",
            default_catalog_path().display()
        ));
    }

    Ok(())
}

fn get_available_images() -> Vec<Image> {
    vec![
        Image {
            name: "Ubuntu".to_string(),
            version: "24.04 LTS".to_string(),
            size_mb: 2048,
            arch: "arm64".to_string(),
        },
        Image {
            name: "Ubuntu".to_string(),
            version: "22.04 LTS".to_string(),
            size_mb: 1950,
            arch: "arm64".to_string(),
        },
        Image {
            name: "Fedora".to_string(),
            version: "40".to_string(),
            size_mb: 1800,
            arch: "arm64".to_string(),
        },
        Image {
            name: "Debian".to_string(),
            version: "12".to_string(),
            size_mb: 1600,
            arch: "arm64".to_string(),
        },
    ]
}
