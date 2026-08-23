use crate::error::TemplateError;
use crate::templates::{get_builtin_templates, Template, TemplateCategory};
use std::collections::HashMap;

pub struct TemplateEngine {
    templates: HashMap<String, Template>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Load built-in templates
        for template in get_builtin_templates() {
            templates.insert(template.name.clone(), template);
        }

        TemplateEngine { templates }
    }

    pub fn list_templates(&self) -> Vec<Template> {
        self.templates.values().cloned().collect()
    }

    pub fn search_templates(&self, query: &str) -> Vec<Template> {
        self.templates
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query.to_lowercase())
                    || t.description.to_lowercase().contains(&query.to_lowercase())
            })
            .cloned()
            .collect()
    }

    pub fn get_template(&self, name: &str) -> Result<Template, TemplateError> {
        self.templates
            .get(name)
            .cloned()
            .ok_or_else(|| TemplateError::NotFound(name.to_string()))
    }

    pub fn get_by_category(&self, category: TemplateCategory) -> Vec<Template> {
        self.templates
            .values()
            .filter(|t| t.category == category)
            .cloned()
            .collect()
    }

    pub fn add_template(&mut self, template: Template) -> Result<(), TemplateError> {
        if self.templates.contains_key(&template.name) {
            return Err(TemplateError::ValidationFailed(format!(
                "Template '{}' already exists",
                template.name
            )));
        }
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }

    pub fn validate_template(&self, template: &Template) -> Result<(), TemplateError> {
        if template.name.is_empty() {
            return Err(TemplateError::ValidationFailed(
                "Template name cannot be empty".to_string(),
            ));
        }

        if template.base_image.is_empty() {
            return Err(TemplateError::ValidationFailed(
                "Base image cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(!engine.templates.is_empty());
    }

    #[test]
    fn test_list_templates() {
        let engine = TemplateEngine::new();
        let templates = engine.list_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_search_templates() {
        let engine = TemplateEngine::new();
        let results = engine.search_templates("rust");
        assert!(results.iter().any(|t| t.name == "rust"));
    }

    #[test]
    fn test_get_template() {
        let engine = TemplateEngine::new();
        let template = engine.get_template("rust").unwrap();
        assert_eq!(template.name, "rust");
    }

    #[test]
    fn test_get_template_not_found() {
        let engine = TemplateEngine::new();
        let result = engine.get_template("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_by_category() {
        let engine = TemplateEngine::new();
        let dev_templates = engine.get_by_category(TemplateCategory::Dev);
        assert!(!dev_templates.is_empty());
    }

    #[test]
    fn test_add_template() {
        let mut engine = TemplateEngine::new();
        let template = Template::new("custom", "Custom template", TemplateCategory::Dev);
        assert!(engine.add_template(template).is_ok());
    }

    #[test]
    fn test_add_duplicate_template() {
        let mut engine = TemplateEngine::new();
        let template = Template::new("rust", "Duplicate", TemplateCategory::Dev);
        assert!(engine.add_template(template).is_err());
    }

    #[test]
    fn test_validate_template() {
        let engine = TemplateEngine::new();
        let valid = Template::new("test", "Test", TemplateCategory::Dev);
        assert!(engine.validate_template(&valid).is_ok());

        let invalid = Template {
            name: String::new(),
            description: String::new(),
            category: TemplateCategory::Dev,
            base_image: "ubuntu".to_string(),
            packages: vec![],
            post_install: None,
            ports: None,
            workspace_mount: "/workspace".to_string(),
        };
        assert!(engine.validate_template(&invalid).is_err());
    }
}
