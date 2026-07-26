pub mod error;
pub mod template_engine;
pub mod templates;

pub use error::TemplateError;
pub use template_engine::TemplateEngine;
pub use templates::{Template, TemplateCategory};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LaunchProfile {
    Minimal,
    Development,
    Enterprise,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_profiles() {
        assert_eq!(LaunchProfile::Development, LaunchProfile::Development);
    }
}
