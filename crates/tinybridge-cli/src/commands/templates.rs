use anyhow::Result;
use std::path::PathBuf;

use crate::output;
use clap::Parser;

#[derive(Parser)]
pub struct TemplatesArgs {
    /// Search for templates matching this term
    pub search: Option<String>,

    /// Show full descriptions
    #[arg(long)]
    pub verbose: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub async fn execute(args: TemplatesArgs, _socket: Option<PathBuf>) -> Result<()> {
    let templates = get_available_templates();

    let filtered: Vec<_> = templates
        .into_iter()
        .filter(|t| {
            args.search
                .as_ref()
                .map(|s| t.name.to_lowercase().contains(&s.to_lowercase()))
                .unwrap_or(true)
        })
        .collect();

    if args.json {
        let json = serde_json::to_string_pretty(&filtered)?;
        println!("{}", json);
    } else {
        output::print_header("Available Templates");

        for template in &filtered {
            output::print_template(&template.name, &template.description);
        }

        if args.verbose {
            println!();
            for template in &filtered {
                output::print_info(&format!("{}:", template.name));
                output::print_info(&format!("  Description: {}", template.description));
                output::print_info(&format!("  Category: {}", template.category));
            }
        }
    }

    Ok(())
}

#[derive(serde::Serialize, Clone)]
struct Template {
    name: String,
    description: String,
    category: String,
}

fn get_available_templates() -> Vec<Template> {
    vec![
        // Base OS templates
        Template {
            name: "ubuntu".to_string(),
            description: "Ubuntu 24.04 LTS (default)".to_string(),
            category: "base".to_string(),
        },
        Template {
            name: "fedora".to_string(),
            description: "Fedora 40".to_string(),
            category: "base".to_string(),
        },
        Template {
            name: "debian".to_string(),
            description: "Debian 12 (stable)".to_string(),
            category: "base".to_string(),
        },
        // Development templates
        Template {
            name: "rust".to_string(),
            description: "Rust development environment".to_string(),
            category: "dev".to_string(),
        },
        Template {
            name: "python".to_string(),
            description: "Python 3.12 + Jupyter + common packages".to_string(),
            category: "dev".to_string(),
        },
        Template {
            name: "node".to_string(),
            description: "Node.js 20 + npm + common tools".to_string(),
            category: "dev".to_string(),
        },
        Template {
            name: "golang".to_string(),
            description: "Go 1.22 development environment".to_string(),
            category: "dev".to_string(),
        },
        // Robotics templates
        Template {
            name: "ros2-humble".to_string(),
            description: "ROS 2 Humble + Gazebo + RViz".to_string(),
            category: "robotics".to_string(),
        },
        Template {
            name: "ros2-jazzy".to_string(),
            description: "ROS 2 Jazzy + Gazebo + RViz + Nav2".to_string(),
            category: "robotics".to_string(),
        },
        // AI/ML templates
        Template {
            name: "ai".to_string(),
            description: "PyTorch + transformers + LangChain".to_string(),
            category: "ai".to_string(),
        },
        Template {
            name: "jupyter".to_string(),
            description: "Jupyter Lab + common data science packages".to_string(),
            category: "ai".to_string(),
        },
    ]
}
