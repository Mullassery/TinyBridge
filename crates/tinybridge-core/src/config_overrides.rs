/// Configuration Override Engine
/// Phase 4.0.1: Config Foundation
///
/// Handles overrides from environment variables and CLI flags

use std::collections::HashMap;

/// Configuration override sources (priority order: CLI > ENV > YAML)
#[derive(Debug, Default, Clone)]
pub struct OverrideEngine {
    /// CLI flag overrides (highest priority)
    pub cli_overrides: CliOverrides,
    /// Environment variable overrides (medium priority)
    pub env_overrides: EnvOverrides,
}

#[derive(Debug, Default, Clone)]
pub struct CliOverrides {
    /// --cpus flag
    pub cpus: Option<u32>,
    /// --memory flag
    pub memory: Option<u32>,
    /// --disk flag
    pub disk: Option<u32>,
    /// --profile flag
    pub profile: Option<String>,
    /// --env-var flags (can be multiple)
    pub env_vars: HashMap<String, String>,
    /// --gpu flag
    pub gpu: Option<bool>,
}

#[derive(Debug, Default, Clone)]
pub struct EnvOverrides {
    /// TB_CPUS environment variable
    pub cpus: Option<u32>,
    /// TB_MEMORY environment variable
    pub memory: Option<u32>,
    /// TB_DISK environment variable
    pub disk: Option<u32>,
    /// TB_PROFILE environment variable
    pub profile: Option<String>,
    /// TB_PROFILE_<NAME> environment variables for custom profiles
    pub custom_profiles: HashMap<String, String>,
    /// TB_<KEY>=<VALUE> for arbitrary overrides
    pub custom_vars: HashMap<String, String>,
}

impl OverrideEngine {
    /// Create a new override engine
    pub fn new() -> Self {
        OverrideEngine::default()
    }

    /// Load environment variable overrides
    pub fn load_from_env() -> Self {
        let mut engine = OverrideEngine::new();

        // Load standard overrides
        if let Ok(cpus) = std::env::var("TB_CPUS") {
            if let Ok(cpus) = cpus.parse::<u32>() {
                engine.env_overrides.cpus = Some(cpus);
            }
        }

        if let Ok(memory) = std::env::var("TB_MEMORY") {
            if let Ok(memory) = memory.parse::<u32>() {
                engine.env_overrides.memory = Some(memory);
            }
        }

        if let Ok(disk) = std::env::var("TB_DISK") {
            if let Ok(disk) = disk.parse::<u32>() {
                engine.env_overrides.disk = Some(disk);
            }
        }

        if let Ok(profile) = std::env::var("TB_PROFILE") {
            engine.env_overrides.profile = Some(profile);
        }

        // Load custom variables (TB_<KEY>=<VALUE>)
        for (key, value) in std::env::vars() {
            if key.starts_with("TB_") && !key.starts_with("TB_PROFILE_") {
                let var_name = key.strip_prefix("TB_").unwrap();
                // Skip already-parsed standard vars
                if !matches!(var_name, "CPUS" | "MEMORY" | "DISK" | "PROFILE") {
                    engine.env_overrides.custom_vars.insert(var_name.to_string(), value);
                }
            }
        }

        engine
    }

    /// Get CPU override (CLI > ENV > None)
    pub fn get_cpus(&self) -> Option<u32> {
        self.cli_overrides.cpus.or(self.env_overrides.cpus)
    }

    /// Get memory override (CLI > ENV > None)
    pub fn get_memory(&self) -> Option<u32> {
        self.cli_overrides.memory.or(self.env_overrides.memory)
    }

    /// Get disk override (CLI > ENV > None)
    pub fn get_disk(&self) -> Option<u32> {
        self.cli_overrides.disk.or(self.env_overrides.disk)
    }

    /// Get profile override (CLI > ENV > None)
    pub fn get_profile(&self) -> Option<String> {
        self.cli_overrides
            .profile
            .clone()
            .or_else(|| self.env_overrides.profile.clone())
    }

    /// Get GPU override (CLI > ENV > None)
    pub fn get_gpu(&self) -> Option<bool> {
        self.cli_overrides.gpu.or_else(|| {
            if let Ok(val) = std::env::var("TB_GPU") {
                match val.to_lowercase().as_str() {
                    "true" | "1" | "yes" => Some(true),
                    "false" | "0" | "no" => Some(false),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    /// Get all environment variable overrides (CLI + ENV)
    pub fn get_all_env_vars(&self) -> HashMap<String, String> {
        let mut combined = self.env_overrides.custom_vars.clone();
        for (key, value) in &self.cli_overrides.env_vars {
            combined.insert(key.clone(), value.clone());
        }
        combined
    }

    /// Merge two override engines (self takes priority over other)
    pub fn merge_with(&self, other: &OverrideEngine) -> OverrideEngine {
        let mut merged = OverrideEngine::new();

        // CLI takes priority over other's CLI
        merged.cli_overrides = CliOverrides {
            cpus: self.cli_overrides.cpus.or(other.cli_overrides.cpus),
            memory: self.cli_overrides.memory.or(other.cli_overrides.memory),
            disk: self.cli_overrides.disk.or(other.cli_overrides.disk),
            profile: self
                .cli_overrides
                .profile
                .clone()
                .or_else(|| other.cli_overrides.profile.clone()),
            env_vars: {
                let mut combined = other.cli_overrides.env_vars.clone();
                for (key, value) in &self.cli_overrides.env_vars {
                    combined.insert(key.clone(), value.clone());
                }
                combined
            },
            gpu: self.cli_overrides.gpu.or(other.cli_overrides.gpu),
        };

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_override_engine_creation() {
        let engine = OverrideEngine::new();
        assert!(engine.get_cpus().is_none());
        assert!(engine.get_memory().is_none());
    }

    #[test]
    fn test_cli_override_priority() {
        let mut engine = OverrideEngine::new();
        engine.cli_overrides.cpus = Some(8);
        engine.env_overrides.cpus = Some(4);

        // CLI should win
        assert_eq!(engine.get_cpus(), Some(8));
    }

    #[test]
    fn test_env_override_fallback() {
        let mut engine = OverrideEngine::new();
        engine.env_overrides.memory = Some(16);

        // ENV should be used when CLI is not set
        assert_eq!(engine.get_memory(), Some(16));
    }

    #[test]
    fn test_merge_overrides() {
        let mut engine1 = OverrideEngine::new();
        engine1.cli_overrides.cpus = Some(8);

        let mut engine2 = OverrideEngine::new();
        engine2.cli_overrides.memory = Some(16);

        let merged = engine1.merge_with(&engine2);
        assert_eq!(merged.get_cpus(), Some(8));
        assert_eq!(merged.get_memory(), Some(16));
    }

    #[test]
    fn test_env_vars_combination() {
        let mut engine = OverrideEngine::new();
        engine.env_overrides.custom_vars.insert("FOO".to_string(), "bar".to_string());
        engine.cli_overrides.env_vars.insert("BAZ".to_string(), "qux".to_string());

        let combined = engine.get_all_env_vars();
        assert_eq!(combined.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(combined.get("BAZ"), Some(&"qux".to_string()));
    }

    #[test]
    fn test_gpu_parsing() {
        let mut engine = OverrideEngine::new();
        engine.cli_overrides.gpu = Some(true);
        assert_eq!(engine.get_gpu(), Some(true));

        engine.cli_overrides.gpu = Some(false);
        assert_eq!(engine.get_gpu(), Some(false));
    }
}
