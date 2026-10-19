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
    pub details: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Render a built-in template as cloud-init user-data and exit
    #[arg(long, value_name = "TEMPLATE")]
    pub cloud_init: Option<String>,
}

pub async fn execute(args: TemplatesArgs, _socket: Option<PathBuf>) -> Result<()> {
    if let Some(name) = &args.cloud_init {
        let engine = tinybridge_template_engine::TemplateEngine::new();
        let template = engine
            .get_template(name)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("{}", render_cloud_init(&template));
        return Ok(());
    }
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

        if args.details {
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

/// Render a cloud-init `#cloud-config` user-data document from a template.
pub fn render_cloud_init(template: &tinybridge_template_engine::Template) -> String {
    let mut lines = vec!["#cloud-config".to_string()];
    if !template.packages.is_empty() {
        lines.push("package_update: true".to_string());
        lines.push("packages:".to_string());
        for package in &template.packages {
            lines.push(format!("  - {}", package));
        }
    }
    if let Some(script) = &template.post_install {
        lines.push("runcmd:".to_string());
        lines.push(format!("  - [sh, -c, {:?}]", script));
    }
    if let Some(ports) = &template.ports {
        lines.push("write_files:".to_string());
        lines.push(format!(
            "  - path: /etc/tinybridge/ports\n    content: {:?}",
            ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    lines.join("\n") + "\n"
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

#[cfg(test)]
mod cloud_init_tests {
    use super::*;

    #[test]
    fn renders_packages_and_runcmd() {
        let engine = tinybridge_template_engine::TemplateEngine::new();
        let template = engine.get_template("rust").unwrap();
        let yaml = render_cloud_init(&template);
        assert!(yaml.starts_with("#cloud-config"));
        assert!(yaml.contains("packages:"));
        assert!(yaml.contains("runcmd:"));
    }

    #[test]
    fn unknown_template_is_an_error() {
        let engine = tinybridge_template_engine::TemplateEngine::new();
        assert!(engine.get_template("does-not-exist").is_err());
    }
}
