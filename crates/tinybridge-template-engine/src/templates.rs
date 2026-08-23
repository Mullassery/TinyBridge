use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub base_image: String,
    pub packages: Vec<String>,
    pub post_install: Option<String>,
    pub ports: Option<Vec<u16>>,
    pub workspace_mount: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateCategory {
    Base,
    Dev,
    Robotics,
    AI,
    Enterprise,
}

impl Template {
    pub fn new(name: &str, description: &str, category: TemplateCategory) -> Self {
        Template {
            name: name.to_string(),
            description: description.to_string(),
            category,
            base_image: "ubuntu:24.04".to_string(),
            packages: vec![],
            post_install: None,
            ports: None,
            workspace_mount: "/workspace".to_string(),
        }
    }

    pub fn with_packages(mut self, packages: Vec<&str>) -> Self {
        self.packages = packages.into_iter().map(|p| p.to_string()).collect();
        self
    }

    pub fn with_post_install(mut self, script: &str) -> Self {
        self.post_install = Some(script.to_string());
        self
    }

    pub fn with_ports(mut self, ports: Option<Vec<u16>>) -> Self {
        self.ports = ports;
        self
    }
}

pub fn get_builtin_templates() -> Vec<Template> {
    vec![
        // Base OS templates
        Template::new(
            "ubuntu",
            "Ubuntu 24.04 LTS (default)",
            TemplateCategory::Base,
        )
        .with_packages(vec!["build-essential", "git", "curl"]),
        Template::new("fedora", "Fedora 40", TemplateCategory::Base)
            .with_packages(vec!["gcc", "git", "curl"]),
        Template::new("debian", "Debian 12 (stable)", TemplateCategory::Base).with_packages(vec![
            "build-essential",
            "git",
            "curl",
        ]),
        // Development templates
        Template::new(
            "rust",
            "Rust development environment",
            TemplateCategory::Dev,
        )
        .with_packages(vec!["build-essential", "curl", "git", "rustc", "cargo"])
        .with_post_install("rustup self update && cargo --version"),
        Template::new("python", "Python 3.12 + Jupyter", TemplateCategory::Dev).with_packages(
            vec!["python3.12", "python3-pip", "jupyter", "numpy", "pandas"],
        ),
        Template::new("node", "Node.js 20 + npm", TemplateCategory::Dev)
            .with_packages(vec!["nodejs", "npm", "git"])
            .with_ports(Some(vec![3000, 8000, 8080])),
        Template::new("golang", "Go 1.22 development", TemplateCategory::Dev).with_packages(vec![
            "golang-1.22",
            "git",
            "build-essential",
        ]),
        // Robotics templates
        Template::new(
            "ros2-humble",
            "ROS 2 Humble + Gazebo",
            TemplateCategory::Robotics,
        )
        .with_packages(vec![
            "ros-humble-desktop",
            "ros-humble-gazebo",
            "python3-colcon-common-extensions",
        ])
        .with_post_install(". /opt/ros/humble/setup.bash"),
        Template::new(
            "ros2-jazzy",
            "ROS 2 Jazzy + Nav2",
            TemplateCategory::Robotics,
        )
        .with_packages(vec![
            "ros-jazzy-desktop",
            "ros-jazzy-nav2",
            "ros-jazzy-rviz2",
        ])
        .with_post_install(". /opt/ros/jazzy/setup.bash"),
        // AI/ML templates
        Template::new("ai", "PyTorch + transformers", TemplateCategory::AI).with_packages(vec![
            "python3.12",
            "python3-pip",
            "pytorch",
            "transformers",
            "langchain",
        ]),
        Template::new(
            "jupyter",
            "Jupyter Lab + data science",
            TemplateCategory::AI,
        )
        .with_packages(vec![
            "python3.12",
            "jupyter-lab",
            "numpy",
            "pandas",
            "scikit-learn",
            "matplotlib",
        ])
        .with_ports(Some(vec![8888])),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = Template::new("test", "Test template", TemplateCategory::Dev);
        assert_eq!(template.name, "test");
        assert_eq!(template.category, TemplateCategory::Dev);
    }

    #[test]
    fn test_template_with_packages() {
        let template =
            Template::new("test", "Test", TemplateCategory::Dev).with_packages(vec!["git", "curl"]);
        assert_eq!(template.packages.len(), 2);
    }

    #[test]
    fn test_builtin_templates() {
        let templates = get_builtin_templates();
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.name == "rust"));
        assert!(templates
            .iter()
            .any(|t| t.category == TemplateCategory::Robotics));
    }
}
