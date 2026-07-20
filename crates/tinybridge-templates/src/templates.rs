use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Result, TemplateError};

/// Environment template for common use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template identifier (e.g., "backend", "ml-training")
    pub name: String,

    /// User-friendly description
    pub description: String,

    /// Base OS configuration
    pub os: String,

    /// OS version
    pub os_version: String,

    /// Recommended resource allocation
    pub resources: ResourceConfig,

    /// Pre-installed tools and versions
    pub tools: Vec<ToolSpec>,

    /// Environment variables to set
    pub env_vars: HashMap<String, String>,

    /// Suggested execution tier (if multi-tier routing enabled)
    pub execution_tier: Option<String>,

    /// Use cases this template is optimized for
    pub use_cases: Vec<String>,
}

/// Resource configuration for a template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu: u32,
    pub memory_gb: u32,
    pub disk_gb: u32,
}

/// Tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub version: Option<String>,
}

impl Template {
    /// Create a template for backend development
    pub fn backend() -> Self {
        Template {
            name: "backend".to_string(),
            description: "Backend development environment (Python/Node/Go)".to_string(),
            os: "ubuntu".to_string(),
            os_version: "24.04".to_string(),
            resources: ResourceConfig {
                cpu: 4,
                memory_gb: 8,
                disk_gb: 50,
            },
            tools: vec![
                ToolSpec {
                    name: "python".to_string(),
                    version: Some("3.11".to_string()),
                },
                ToolSpec {
                    name: "nodejs".to_string(),
                    version: Some("20".to_string()),
                },
                ToolSpec {
                    name: "golang".to_string(),
                    version: Some("1.21".to_string()),
                },
                ToolSpec {
                    name: "postgresql".to_string(),
                    version: Some("15".to_string()),
                },
                ToolSpec {
                    name: "redis".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "git".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "docker".to_string(),
                    version: None,
                },
            ],
            env_vars: [
                ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
                ("NODE_ENV".to_string(), "development".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            execution_tier: Some("linux".to_string()),
            use_cases: vec![
                "REST API development".to_string(),
                "microservices".to_string(),
                "web backend".to_string(),
            ],
        }
    }

    /// Create a template for ML/data science work
    pub fn ml_training() -> Self {
        Template {
            name: "ml-training".to_string(),
            description: "ML training environment (PyTorch/TensorFlow/JAX)".to_string(),
            os: "ubuntu".to_string(),
            os_version: "24.04".to_string(),
            resources: ResourceConfig {
                cpu: 8,
                memory_gb: 32,
                disk_gb: 200,
            },
            tools: vec![
                ToolSpec {
                    name: "python".to_string(),
                    version: Some("3.11".to_string()),
                },
                ToolSpec {
                    name: "pytorch".to_string(),
                    version: Some("2.1".to_string()),
                },
                ToolSpec {
                    name: "tensorflow".to_string(),
                    version: Some("2.13".to_string()),
                },
                ToolSpec {
                    name: "jax".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "jupyter".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "pandas".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "scikit-learn".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "cuda".to_string(),
                    version: Some("12.1".to_string()),
                },
            ],
            env_vars: [
                ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
                ("CUDA_VISIBLE_DEVICES".to_string(), "0".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            execution_tier: Some("remote".to_string()),
            use_cases: vec![
                "model training".to_string(),
                "deep learning".to_string(),
                "data analysis".to_string(),
                "experimentation".to_string(),
            ],
        }
    }

    /// Create a template for robotics/ROS development
    pub fn robotics() -> Self {
        Template {
            name: "robotics".to_string(),
            description: "Robotics development environment (ROS 2)".to_string(),
            os: "ubuntu".to_string(),
            os_version: "24.04".to_string(),
            resources: ResourceConfig {
                cpu: 4,
                memory_gb: 16,
                disk_gb: 100,
            },
            tools: vec![
                ToolSpec {
                    name: "ros2".to_string(),
                    version: Some("humble".to_string()),
                },
                ToolSpec {
                    name: "python".to_string(),
                    version: Some("3.11".to_string()),
                },
                ToolSpec {
                    name: "cpp".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "cmake".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "gazebo".to_string(),
                    version: Some("11".to_string()),
                },
                ToolSpec {
                    name: "rviz2".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "colcon".to_string(),
                    version: None,
                },
            ],
            env_vars: [
                ("ROS_DISTRO".to_string(), "humble".to_string()),
                ("ROS_DOMAIN_ID".to_string(), "0".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            execution_tier: Some("linux".to_string()),
            use_cases: vec![
                "robot simulation".to_string(),
                "autonomous systems".to_string(),
                "ROS 2 development".to_string(),
            ],
        }
    }

    /// Create a template for data engineering
    pub fn data_engineering() -> Self {
        Template {
            name: "data-engineering".to_string(),
            description: "Data engineering environment (Spark/Hadoop/Airflow)".to_string(),
            os: "ubuntu".to_string(),
            os_version: "24.04".to_string(),
            resources: ResourceConfig {
                cpu: 8,
                memory_gb: 16,
                disk_gb: 150,
            },
            tools: vec![
                ToolSpec {
                    name: "python".to_string(),
                    version: Some("3.11".to_string()),
                },
                ToolSpec {
                    name: "spark".to_string(),
                    version: Some("3.5".to_string()),
                },
                ToolSpec {
                    name: "hadoop".to_string(),
                    version: Some("3.3".to_string()),
                },
                ToolSpec {
                    name: "airflow".to_string(),
                    version: Some("2.7".to_string()),
                },
                ToolSpec {
                    name: "postgresql".to_string(),
                    version: Some("15".to_string()),
                },
                ToolSpec {
                    name: "kafka".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "dbt".to_string(),
                    version: None,
                },
            ],
            env_vars: [
                ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
                ("SPARK_LOCAL_IP".to_string(), "127.0.0.1".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            execution_tier: Some("linux".to_string()),
            use_cases: vec![
                "ETL pipelines".to_string(),
                "data warehousing".to_string(),
                "streaming analytics".to_string(),
            ],
        }
    }

    /// Create a template for frontend development
    pub fn frontend() -> Self {
        Template {
            name: "frontend".to_string(),
            description: "Frontend development environment (Node/React/Vue)".to_string(),
            os: "ubuntu".to_string(),
            os_version: "24.04".to_string(),
            resources: ResourceConfig {
                cpu: 4,
                memory_gb: 8,
                disk_gb: 50,
            },
            tools: vec![
                ToolSpec {
                    name: "nodejs".to_string(),
                    version: Some("20".to_string()),
                },
                ToolSpec {
                    name: "npm".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "yarn".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "react".to_string(),
                    version: Some("18".to_string()),
                },
                ToolSpec {
                    name: "typescript".to_string(),
                    version: None,
                },
                ToolSpec {
                    name: "webpack".to_string(),
                    version: None,
                },
            ],
            env_vars: [
                ("NODE_ENV".to_string(), "development".to_string()),
                ("BABEL_ENV".to_string(), "development".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            execution_tier: Some("linux".to_string()),
            use_cases: vec![
                "web development".to_string(),
                "SPA development".to_string(),
                "component library work".to_string(),
            ],
        }
    }
}

/// Template registry for managing available templates
pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    /// Create a new registry with all default templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        let default_templates = vec![
            Template::backend(),
            Template::ml_training(),
            Template::robotics(),
            Template::data_engineering(),
            Template::frontend(),
        ];

        for template in default_templates {
            templates.insert(template.name.clone(), template);
        }

        Self { templates }
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Result<Template> {
        self.templates
            .get(name)
            .cloned()
            .ok_or_else(|| TemplateError::TemplateNotFound(name.to_string()))
    }

    /// List all available templates
    pub fn list(&self) -> Vec<TemplateInfo> {
        self.templates
            .values()
            .map(|t| TemplateInfo {
                name: t.name.clone(),
                description: t.description.clone(),
            })
            .collect()
    }

    /// Check if a template exists
    pub fn exists(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of available template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_backend() {
        let template = Template::backend();
        assert_eq!(template.name, "backend");
        assert_eq!(template.resources.cpu, 4);
        assert_eq!(template.resources.memory_gb, 8);
        assert!(!template.tools.is_empty());
    }

    #[test]
    fn test_template_ml_training() {
        let template = Template::ml_training();
        assert_eq!(template.name, "ml-training");
        assert_eq!(template.resources.cpu, 8);
        assert_eq!(template.resources.memory_gb, 32);
        assert!(template.tools.len() > 5);
    }

    #[test]
    fn test_registry_get() {
        let registry = TemplateRegistry::new();
        assert!(registry.get("backend").is_ok());
        assert!(registry.get("robotics").is_ok());
        assert!(registry.get("nonexistent").is_err());
    }

    #[test]
    fn test_registry_list() {
        let registry = TemplateRegistry::new();
        let list = registry.list();
        assert_eq!(list.len(), 5);
        assert!(list.iter().any(|t| t.name == "backend"));
    }

    #[test]
    fn test_registry_exists() {
        let registry = TemplateRegistry::new();
        assert!(registry.exists("backend"));
        assert!(!registry.exists("fake"));
    }
}
