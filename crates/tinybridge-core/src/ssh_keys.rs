use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct SshKeyManager;

impl SshKeyManager {
    /// Get or create SSH key pair for TinyBridge VMs
    pub fn get_or_create_keys() -> Result<(PathBuf, PathBuf)> {
        let ssh_dir = crate::TinyBridgeConfig::data_dir().join("ssh");
        fs::create_dir_all(&ssh_dir)?;

        let private_key = ssh_dir.join("tinybridge_rsa");
        let public_key = ssh_dir.join("tinybridge_rsa.pub");

        if !private_key.exists() {
            Self::generate_key_pair(&private_key, &public_key)?;
        }

        Ok((private_key, public_key))
    }

    /// Generate SSH key pair using ssh-keygen
    fn generate_key_pair(private_key: &Path, public_key: &Path) -> Result<()> {
        let output = std::process::Command::new("ssh-keygen")
            .arg("-t")
            .arg("rsa")
            .arg("-b")
            .arg("4096")
            .arg("-f")
            .arg(private_key)
            .arg("-N")
            .arg("") // Empty passphrase
            .arg("-C")
            .arg("tinybridge@vm")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to generate SSH key: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Set proper permissions on private key
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(private_key, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Get the public key content
    pub fn get_public_key() -> Result<String> {
        let (_, public_key) = Self::get_or_create_keys()?;
        Ok(fs::read_to_string(public_key)?)
    }

    /// Get the private key path
    pub fn get_private_key_path() -> Result<PathBuf> {
        let (private_key, _) = Self::get_or_create_keys()?;
        Ok(private_key)
    }
}
