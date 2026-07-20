use std::io::Cursor;
use tinybridge_core::{Arch, EnvYaml};

#[test]
fn test_env_yaml_minimal() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: test-env
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 4GB
  disk: 20GB
"#;
    let env: EnvYaml = serde_yaml::from_str(yaml).expect("should parse minimal env.yaml");
    assert_eq!(env.api_version, "tinybridge/v1");
    assert_eq!(env.kind, "Environment");
    assert_eq!(env.metadata.name, "test-env");
    assert_eq!(env.metadata.version, "1.0.0");
    assert_eq!(env.substrate.os, "ubuntu-24.04");
    assert_eq!(env.resources.cpu, 2);
    assert_eq!(env.resources.memory_bytes, 4 * 1024_u64.pow(3));
    assert_eq!(env.resources.disk_bytes, 20 * 1024_u64.pow(3));
}

#[test]
fn test_env_yaml_full() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: my-project
  version: "1.0.0"
  description: "My robotics environment"
substrate:
  os: ubuntu-24.04
  kernel: "6.6-lts"
  arch:
    - arm64
    - amd64
resources:
  cpu: 4
  memory: 8GB
  disk: 50GB
native:
  tools:
    - rust@1.87
    - python@3.11
    - cmake
    - docker
"#;
    let env: EnvYaml = serde_yaml::from_str(yaml).expect("should parse full env.yaml");
    assert_eq!(env.metadata.name, "my-project");
    assert_eq!(
        env.metadata.description,
        Some("My robotics environment".to_string())
    );
    assert_eq!(env.substrate.arch, vec![Arch::Arm64, Arch::Amd64]);
    assert_eq!(env.substrate.kernel, Some("6.6-lts".to_string()));
    assert_eq!(env.resources.cpu, 4);
    assert_eq!(env.native.tools.len(), 4);
    assert_eq!(env.native.tools[0].name, "rust");
    assert_eq!(env.native.tools[0].version, Some("1.87".to_string()));
    assert_eq!(env.native.tools[1].name, "python");
    assert_eq!(env.native.tools[1].version, Some("3.11".to_string()));
    assert_eq!(env.native.tools[2].name, "cmake");
    assert_eq!(env.native.tools[2].version, None);
}

#[test]
fn test_env_yaml_default_arch() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: test
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 4GB
  disk: 20GB
"#;
    let env: EnvYaml = serde_yaml::from_str(yaml).expect("should parse");
    assert_eq!(env.substrate.arch, vec![Arch::Arm64]);
}

#[test]
fn test_env_yaml_from_reader() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: test
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 4GB
  disk: 20GB
"#;
    let cursor = Cursor::new(yaml.as_bytes());
    let env = EnvYaml::from_reader(cursor).expect("should parse from reader");
    assert_eq!(env.metadata.name, "test");
}

#[test]
fn test_env_yaml_roundtrip() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: test
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 4GB
  disk: 20GB
"#;
    let env: EnvYaml = serde_yaml::from_str(yaml).expect("should parse");
    let json = serde_json::to_string(&env).expect("should serialize to json");
    let env2: EnvYaml = serde_json::from_str(&json).expect("should deserialize from json");
    assert_eq!(env.metadata.name, env2.metadata.name);
    assert_eq!(env.resources.cpu, env2.resources.cpu);
}

#[test]
fn test_memory_size_gb() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: test
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 16GB
  disk: 100GB
"#;
    let env: EnvYaml = serde_yaml::from_str(yaml).expect("should parse");
    assert_eq!(env.resources.memory_bytes, 16 * 1024_u64.pow(3));
    assert_eq!(env.resources.disk_bytes, 100 * 1024_u64.pow(3));
}

#[test]
fn test_memory_size_integer() {
    let json = r#"
{
  "apiVersion": "tinybridge/v1",
  "kind": "Environment",
  "metadata": {
    "name": "test",
    "version": "1.0.0"
  },
  "substrate": {
    "os": "ubuntu-24.04"
  },
  "resources": {
    "cpu": 2,
    "memory": 8589934592,
    "disk": 53687091200
  }
}
"#;
    let env: EnvYaml = serde_json::from_str(json).expect("should parse raw bytes");
    assert_eq!(env.resources.memory_bytes, 8589934592);
}

#[test]
fn test_invalid_api_version() {
    let yaml = r#"
apiVersion: tinybridge/v2
kind: Environment
metadata:
  name: test
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 4GB
  disk: 20GB
"#;
    let result = EnvYaml::from_reader(Cursor::new(yaml.as_bytes()));
    assert!(result.is_err());
}

#[test]
fn test_invalid_kind() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Pod
metadata:
  name: test
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 4GB
  disk: 20GB
"#;
    let result = EnvYaml::from_reader(Cursor::new(yaml.as_bytes()));
    assert!(result.is_err());
}

#[test]
fn test_native_tools_empty() {
    let yaml = r#"
apiVersion: tinybridge/v1
kind: Environment
metadata:
  name: test
  version: "1.0.0"
substrate:
  os: ubuntu-24.04
resources:
  cpu: 2
  memory: 4GB
  disk: 20GB
"#;
    let env: EnvYaml = serde_yaml::from_str(yaml).expect("should parse");
    assert_eq!(env.native.tools.len(), 0);
}

#[test]
fn test_sample_asset_file() {
    let content = include_str!("../../../assets/sample-env.yaml");
    let env: EnvYaml = serde_yaml::from_str(content).expect("sample-env.yaml should parse");
    assert_eq!(env.metadata.name, "my-project");
    assert_eq!(env.resources.cpu, 4);
    assert_eq!(env.resources.memory_bytes, 8 * 1024_u64.pow(3));
}
