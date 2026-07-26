use anyhow::Result;
use std::path::PathBuf;

use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct ImagesArgs {
    /// Search for images matching this term
    pub search: Option<String>,

    /// Show full details
    #[arg(long)]
    pub verbose: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(serde::Serialize)]
struct Image {
    name: String,
    version: String,
    size_mb: u32,
    arch: String,
}

pub async fn execute(args: ImagesArgs, _socket: Option<PathBuf>) -> Result<()> {
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
