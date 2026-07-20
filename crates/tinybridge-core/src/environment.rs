use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Top-level parsed env.yaml document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvYaml {
    pub api_version: String,
    pub kind: String,
    pub metadata: EnvMetadata,
    pub substrate: SubstrateConfig,
    pub resources: Resources,
    #[serde(default)]
    pub native: NativeSection,
    #[serde(default)]
    pub execution: ExecutionProfiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstrateConfig {
    pub os: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub kernel: Option<String>,
    #[serde(default = "default_arch")]
    pub arch: Vec<Arch>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    Arm64,
    Amd64,
}

fn default_arch() -> Vec<Arch> {
    vec![Arch::Arm64]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub cpu: u32,
    #[serde(rename = "memory", deserialize_with = "deserialize_size_string")]
    pub memory_bytes: u64,
    #[serde(rename = "disk", deserialize_with = "deserialize_size_string")]
    pub disk_bytes: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativeSection {
    #[serde(default)]
    pub tools: Vec<NativeToolSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NativeToolSpec {
    pub name: String,
    pub version: Option<String>,
}

/// Execution profiles for tier-based workload routing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionProfiles {
    #[serde(default)]
    pub default_tier: Option<ExecutionTier>,
    #[serde(default)]
    pub profiles: Vec<ExecutionProfile>,
}

/// Execution tier preference
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionTier {
    /// Run on native macOS if possible
    Native,
    /// Run in Linux substrate
    Linux,
    /// Run on remote GPU tier
    RemoteGpu,
}

/// Tool-level execution profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProfile {
    /// Tool name pattern (glob)
    pub tool: String,
    /// Preferred tier for this tool
    pub tier: ExecutionTier,
    #[serde(default)]
    pub fallback: Option<ExecutionTier>,
}

impl<'de> Deserialize<'de> for NativeToolSpec {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        if let Some((name, ver)) = s.split_once('@') {
            Ok(NativeToolSpec {
                name: name.to_owned(),
                version: Some(ver.to_owned()),
            })
        } else {
            Ok(NativeToolSpec {
                name: s,
                version: None,
            })
        }
    }
}

/// Runtime representation of a managed environment instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub substrate: SubstrateConfig,
    pub resources: Resources,
    pub native_tools: Vec<NativeToolSpec>,
    pub status: EnvironmentStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum EnvironmentStatus {
    Stopped,
    Starting { progress_pct: u8 },
    Running { uptime_secs: u64 },
    Stopping,
    Error { message: String },
}

impl EnvironmentStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, EnvironmentStatus::Running { .. })
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            EnvironmentStatus::Stopped | EnvironmentStatus::Error { .. }
        )
    }
}

impl EnvYaml {
    pub fn from_reader<R: std::io::Read>(reader: R) -> crate::error::Result<Self> {
        let yaml: EnvYaml = serde_yaml::from_reader(reader)?;
        yaml.validate()?;
        Ok(yaml)
    }

    pub fn from_path(path: &std::path::Path) -> crate::error::Result<Self> {
        let f = std::fs::File::open(path)?;
        Self::from_reader(f)
    }

    fn validate(&self) -> crate::error::Result<()> {
        if self.api_version != "tinybridge/v1" {
            return Err(crate::error::CoreError::InvalidApiVersion(
                self.api_version.clone(),
            ));
        }
        if self.kind != "Environment" {
            return Err(crate::error::CoreError::InvalidKind(self.kind.clone()));
        }
        Ok(())
    }
}

fn deserialize_size_string<'de, D>(d: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SizeValue {
        Integer(u64),
        String(String),
    }

    match SizeValue::deserialize(d)? {
        SizeValue::Integer(n) => Ok(n),
        SizeValue::String(s) => parse_size_string(&s).map_err(D::Error::custom),
    }
}

fn parse_size_string(s: &str) -> std::result::Result<u64, String> {
    let s = s.trim().to_uppercase();

    let (num_str, suffix) = if s.ends_with("TB") {
        (&s[..s.len() - 2], "TB")
    } else if s.ends_with("GB") {
        (&s[..s.len() - 2], "GB")
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], "MB")
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], "KB")
    } else if s.ends_with("B") {
        (&s[..s.len() - 1], "B")
    } else {
        (s.as_str(), "")
    };

    let num: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| format!("Invalid number: {}", num_str))?;

    let multiplier = match suffix {
        "TB" => 1024_u64.pow(4),
        "GB" => 1024_u64.pow(3),
        "MB" => 1024_u64.pow(2),
        "KB" => 1024,
        "B" | "" => 1,
        _ => return Err(format!("Unknown suffix: {}", suffix)),
    };

    Ok((num * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_8gb() {
        assert_eq!(parse_size_string("8GB").unwrap(), 8 * 1024_u64.pow(3));
    }

    #[test]
    fn test_parse_size_16gb() {
        assert_eq!(parse_size_string("16GB").unwrap(), 16 * 1024_u64.pow(3));
    }

    #[test]
    fn test_parse_size_50gb() {
        assert_eq!(parse_size_string("50GB").unwrap(), 50 * 1024_u64.pow(3));
    }

    #[test]
    fn test_native_tool_spec_with_version() {
        let json = r#""rust@1.87""#;
        let spec: NativeToolSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.name, "rust");
        assert_eq!(spec.version, Some("1.87".to_string()));
    }

    #[test]
    fn test_native_tool_spec_no_version() {
        let json = r#""python""#;
        let spec: NativeToolSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.name, "python");
        assert_eq!(spec.version, None);
    }
}
