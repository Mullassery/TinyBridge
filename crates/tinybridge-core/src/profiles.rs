/// Configuration Profiles
/// Phase 4.0.1: Config Foundation
///
/// Predefined profiles for different use cases (development, production, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name (development, production, minimal, etc.)
    pub name: String,
    /// Profile description
    pub description: String,
    /// Logging level
    pub log_level: String,
    /// CPU cores
    pub cpus: u32,
    /// Memory in GB
    pub memory: u32,
    /// Disk in GB
    pub disk: u32,
    /// Enable GPU
    pub gpu: bool,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Optimization flags
    pub optimizations: Vec<String>,
}

impl Profile {
    /// Get the built-in development profile
    pub fn development() -> Self {
        Profile {
            name: "development".to_string(),
            description: "Development profile: verbose logging, iterative building".to_string(),
            log_level: "debug".to_string(),
            cpus: 4,
            memory: 8,
            disk: 40,
            gpu: false,
            env_vars: {
                let mut vars = HashMap::new();
                vars.insert("RUST_LOG".to_string(), "debug".to_string());
                vars.insert("RUST_BACKTRACE".to_string(), "1".to_string());
                vars
            },
            optimizations: vec!["fast_rebuild".to_string(), "incremental_builds".to_string()],
        }
    }

    /// Get the built-in production profile
    pub fn production() -> Self {
        Profile {
            name: "production".to_string(),
            description: "Production profile: optimized, minimal logging".to_string(),
            log_level: "warn".to_string(),
            cpus: 8,
            memory: 16,
            disk: 100,
            gpu: true,
            env_vars: {
                let mut vars = HashMap::new();
                vars.insert("RUST_LOG".to_string(), "error".to_string());
                vars.insert("RUST_BACKTRACE".to_string(), "0".to_string());
                vars
            },
            optimizations: vec![
                "lto".to_string(),
                "parallel_compilation".to_string(),
                "memory_caching".to_string(),
            ],
        }
    }

    /// Get the built-in testing profile
    pub fn testing() -> Self {
        Profile {
            name: "testing".to_string(),
            description: "Testing profile: balanced resources, deterministic".to_string(),
            log_level: "info".to_string(),
            cpus: 2,
            memory: 4,
            disk: 30,
            gpu: false,
            env_vars: {
                let mut vars = HashMap::new();
                vars.insert("RUST_LOG".to_string(), "info".to_string());
                vars.insert("RUST_BACKTRACE".to_string(), "1".to_string());
                vars
            },
            optimizations: vec!["deterministic_builds".to_string()],
        }
    }

    /// Get the built-in minimal profile (for CI/CD)
    pub fn minimal() -> Self {
        Profile {
            name: "minimal".to_string(),
            description: "Minimal profile: smallest footprint for CI/CD".to_string(),
            log_level: "warn".to_string(),
            cpus: 1,
            memory: 2,
            disk: 20,
            gpu: false,
            env_vars: {
                let mut vars = HashMap::new();
                vars.insert("RUST_LOG".to_string(), "error".to_string());
                vars
            },
            optimizations: vec!["minimal_logging".to_string()],
        }
    }

    /// Get a profile by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "development" => Some(Self::development()),
            "production" => Some(Self::production()),
            "testing" => Some(Self::testing()),
            "minimal" => Some(Self::minimal()),
            _ => None,
        }
    }

    /// List all built-in profile names
    pub fn builtin_names() -> Vec<&'static str> {
        vec!["development", "production", "testing", "minimal"]
    }

    /// Check if this profile has a specific optimization
    pub fn has_optimization(&self, opt: &str) -> bool {
        self.optimizations.iter().any(|o| o == opt)
    }

    /// Get environment variables from profile
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        self.env_vars.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_profile() {
        let profile = Profile::development();
        assert_eq!(profile.name, "development");
        assert_eq!(profile.cpus, 4);
        assert_eq!(profile.memory, 8);
        assert_eq!(profile.log_level, "debug");
        assert!(profile.has_optimization("fast_rebuild"));
    }

    #[test]
    fn test_production_profile() {
        let profile = Profile::production();
        assert_eq!(profile.name, "production");
        assert_eq!(profile.cpus, 8);
        assert_eq!(profile.memory, 16);
        assert_eq!(profile.log_level, "warn");
        assert!(profile.gpu);
    }

    #[test]
    fn test_profile_from_name() {
        assert!(Profile::from_name("development").is_some());
        assert!(Profile::from_name("production").is_some());
        assert!(Profile::from_name("testing").is_some());
        assert!(Profile::from_name("minimal").is_some());
        assert!(Profile::from_name("invalid").is_none());
    }

    #[test]
    fn test_builtin_names() {
        let names = Profile::builtin_names();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"development"));
        assert!(names.contains(&"production"));
    }

    #[test]
    fn test_env_vars_for_profile() {
        let profile = Profile::development();
        let env_vars = profile.get_env_vars();
        assert_eq!(env_vars.get("RUST_LOG"), Some(&"debug".to_string()));
        assert_eq!(env_vars.get("RUST_BACKTRACE"), Some(&"1".to_string()));
    }
}
