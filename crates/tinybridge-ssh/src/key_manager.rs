use crate::error::{Result, SshError};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ssh_key::PrivateKey;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// SSH key type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
    #[serde(rename = "rsa")]
    Rsa4096,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ed25519 => write!(f, "ed25519"),
            KeyType::Rsa4096 => write!(f, "rsa"),
        }
    }
}

/// SSH key pair information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyPair {
    pub env_id: Uuid,
    pub env_name: String,
    pub key_type: KeyType,
    pub fingerprint: String,
    pub created_at: chrono::DateTime<Utc>,
    pub rotated_at: Option<chrono::DateTime<Utc>>,
    pub private_key_path: PathBuf,
    pub public_key_path: PathBuf,
}

impl SshKeyPair {
    /// Get the public key content
    pub fn public_key(&self) -> Result<String> {
        fs::read_to_string(&self.public_key_path).map_err(|e| SshError::KeyReadError(e.to_string()))
    }

    /// Get the private key content
    pub fn private_key(&self) -> Result<String> {
        fs::read_to_string(&self.private_key_path)
            .map_err(|e| SshError::KeyReadError(e.to_string()))
    }
}

/// Manages SSH key generation, storage, and lifecycle
pub struct SshKeyManager {
    keys_dir: PathBuf,
}

impl SshKeyManager {
    /// Create a new SSH key manager
    pub fn new(keys_base_dir: impl AsRef<Path>) -> Self {
        Self {
            keys_dir: keys_base_dir.as_ref().to_path_buf(),
        }
    }

    /// Create key manager with default path (~/.tinybridge/keys)
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| SshError::InvalidEnvironment("HOME directory not found".to_string()))?;
        let keys_dir = home.join(".tinybridge/keys");
        Ok(Self::new(keys_dir))
    }

    /// Generate a new SSH key pair for an environment
    pub async fn generate_key(
        &self,
        env_id: Uuid,
        env_name: &str,
        key_type: KeyType,
    ) -> Result<SshKeyPair> {
        // Create environment directory
        let env_dir = self.keys_dir.join(env_id.to_string());
        fs::create_dir_all(&env_dir).map_err(|e| SshError::KeyWriteError(e.to_string()))?;

        // Generate key based on type
        let (private_key, public_key_str, fingerprint) = match key_type {
            KeyType::Ed25519 => self.generate_ed25519()?,
            KeyType::Rsa4096 => self.generate_rsa4096()?,
        };

        // Write private key (in PEM format)
        let private_key_path = env_dir.join("id_ed25519");
        let private_key_pem = private_key
            .to_openssh(ssh_key::LineEnding::LF)
            .map_err(|e| SshError::KeyWriteError(e.to_string()))?;
        fs::write(&private_key_path, &private_key_pem)
            .map_err(|e| SshError::KeyWriteError(e.to_string()))?;

        // Set restrictive permissions on private key (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&private_key_path, fs::Permissions::from_mode(0o600))
                .map_err(|e| SshError::KeyWriteError(e.to_string()))?;
        }

        // Write public key
        let public_key_path = env_dir.join("id_ed25519.pub");
        fs::write(&public_key_path, &public_key_str)
            .map_err(|e| SshError::KeyWriteError(e.to_string()))?;

        // Write metadata
        let keypair = SshKeyPair {
            env_id,
            env_name: env_name.to_string(),
            key_type,
            fingerprint,
            created_at: Utc::now(),
            rotated_at: None,
            private_key_path,
            public_key_path,
        };

        self.write_metadata(&keypair)?;

        Ok(keypair)
    }

    /// Load an existing key pair
    pub fn load_key(&self, env_id: Uuid) -> Result<SshKeyPair> {
        let env_dir = self.keys_dir.join(env_id.to_string());
        let metadata_path = env_dir.join("metadata.json");

        let metadata_content = fs::read_to_string(&metadata_path)
            .map_err(|_| SshError::KeyNotFound(env_id.to_string()))?;

        serde_json::from_str(&metadata_content).map_err(|e| SshError::KeyReadError(e.to_string()))
    }

    /// List all key pairs
    pub fn list_keys(&self) -> Result<Vec<SshKeyPair>> {
        let mut keypairs = Vec::new();

        if !self.keys_dir.exists() {
            return Ok(keypairs);
        }

        for entry in
            fs::read_dir(&self.keys_dir).map_err(|e| SshError::KeyReadError(e.to_string()))?
        {
            let entry = entry.map_err(|e| SshError::KeyReadError(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if let Ok(env_id) = uuid::Uuid::parse_str(dir_name) {
                        if let Ok(keypair) = self.load_key(env_id) {
                            keypairs.push(keypair);
                        }
                    }
                }
            }
        }

        Ok(keypairs)
    }

    /// Rotate an existing key pair
    pub async fn rotate_key(&self, env_id: Uuid) -> Result<SshKeyPair> {
        let keypair = self.load_key(env_id)?;
        let env_name = keypair.env_name.clone();

        // Generate new key
        let new_keypair = self
            .generate_key(env_id, &env_name, keypair.key_type)
            .await?;

        // Update rotation timestamp in new keypair
        let mut updated = new_keypair;
        updated.rotated_at = Some(Utc::now());
        self.write_metadata(&updated)?;

        Ok(updated)
    }

    /// Delete a key pair (archive it)
    pub fn delete_key(&self, env_id: Uuid) -> Result<()> {
        let env_dir = self.keys_dir.join(env_id.to_string());
        let archive_dir = self.keys_dir.join("archived");

        fs::create_dir_all(&archive_dir).map_err(|e| SshError::KeyWriteError(e.to_string()))?;

        let archived_name = format!("{}-{}", env_id, Utc::now().timestamp());
        let archive_path = archive_dir.join(archived_name);

        fs::rename(&env_dir, &archive_path).map_err(|e| SshError::KeyWriteError(e.to_string()))?;

        Ok(())
    }

    // Private methods

    fn generate_ed25519(&self) -> Result<(PrivateKey, String, String)> {
        let private_key =
            ssh_key::PrivateKey::random(&mut rand::thread_rng(), ssh_key::Algorithm::Ed25519)
                .map_err(|e| SshError::KeyGenerationError(e.to_string()))?;

        let public_key = private_key.public_key();
        let public_key_openssh = public_key
            .to_openssh()
            .map_err(|e| SshError::KeyGenerationError(e.to_string()))?;
        let public_key_str = format!("{} {}\n", public_key.algorithm(), public_key_openssh);

        // Generate fingerprint
        let fingerprint = public_key.fingerprint(ssh_key::HashAlg::Sha256).to_string();

        Ok((private_key, public_key_str, fingerprint))
    }

    fn generate_rsa4096(&self) -> Result<(PrivateKey, String, String)> {
        // RSA key generation (would require additional dependencies)
        // For now, fall back to Ed25519
        self.generate_ed25519()
    }

    fn write_metadata(&self, keypair: &SshKeyPair) -> Result<()> {
        let env_dir = keypair
            .private_key_path
            .parent()
            .ok_or_else(|| SshError::KeyWriteError("Invalid key path".to_string()))?;

        let metadata_path = env_dir.join("metadata.json");
        let metadata_json =
            serde_json::to_string_pretty(keypair).map_err(|e| SshError::SerializationError(e))?;

        fs::write(&metadata_path, metadata_json)
            .map_err(|e| SshError::KeyWriteError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SshKeyManager::new(temp_dir.path());

        let env_id = Uuid::new_v4();
        let keypair = manager
            .generate_key(env_id, "test-env", KeyType::Ed25519)
            .await
            .unwrap();

        assert_eq!(keypair.env_id, env_id);
        assert_eq!(keypair.env_name, "test-env");
        assert_eq!(keypair.key_type, KeyType::Ed25519);
        assert!(!keypair.fingerprint.is_empty());
        assert!(keypair.private_key_path.exists());
        assert!(keypair.public_key_path.exists());
    }

    #[tokio::test]
    async fn test_key_loading() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SshKeyManager::new(temp_dir.path());

        let env_id = Uuid::new_v4();
        manager
            .generate_key(env_id, "test-env", KeyType::Ed25519)
            .await
            .unwrap();

        let loaded = manager.load_key(env_id).unwrap();
        assert_eq!(loaded.env_id, env_id);
        assert_eq!(loaded.env_name, "test-env");
    }

    #[tokio::test]
    async fn test_list_keys() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SshKeyManager::new(temp_dir.path());

        let _key1 = manager
            .generate_key(Uuid::new_v4(), "env1", KeyType::Ed25519)
            .await
            .unwrap();
        let _key2 = manager
            .generate_key(Uuid::new_v4(), "env2", KeyType::Ed25519)
            .await
            .unwrap();

        let keys = manager.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
    }
}
