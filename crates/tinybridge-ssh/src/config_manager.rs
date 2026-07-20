use crate::error::{Result, SshError};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// SSH config entry for an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfigEntry {
    pub env_id: Uuid,
    pub alias: String,
    pub hostname: String,
    pub user: String,
    pub port: u16,
    pub identity_file: PathBuf,
    pub options: HashMap<String, String>,
}

impl SshConfigEntry {
    /// Generate SSH config block for this entry
    pub fn to_config_block(&self) -> String {
        let mut block = format!(
            "Host {}\n  HostName {}\n  User {}\n  Port {}\n  IdentityFile {}\n",
            self.alias,
            self.hostname,
            self.user,
            self.port,
            self.identity_file.display()
        );

        // Add additional options
        for (key, value) in &self.options {
            block.push_str(&format!("  {} {}\n", key, value));
        }

        block.push('\n');
        block
    }

    /// Parse from SSH config block
    pub fn from_config_block(alias: &str, block: &str) -> Result<Self> {
        let mut hostname = String::new();
        let mut user = String::new();
        let mut port = 22u16;
        let mut identity_file = PathBuf::new();
        let mut options = HashMap::new();
        let mut env_id_str = String::new();

        for line in block.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }

            match parts[0].to_lowercase().as_str() {
                "hostname" => hostname = parts[1].to_string(),
                "user" => user = parts[1].to_string(),
                "port" => port = parts[1].parse().unwrap_or(22),
                "identityfile" => identity_file = PathBuf::from(parts[1]),
                "# tinybridge-env-id" => env_id_str = parts[1].to_string(),
                _ => {
                    options.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }

        let env_id = if env_id_str.is_empty() {
            Uuid::nil() // Fallback for backward compatibility
        } else {
            Uuid::parse_str(&env_id_str)
                .map_err(|_| SshError::InvalidConfig("Invalid environment ID".to_string()))?
        };

        Ok(SshConfigEntry {
            env_id,
            alias: alias.to_string(),
            hostname,
            user,
            port,
            identity_file,
            options,
        })
    }
}

/// Manages ~/.ssh/config entries
pub struct SshConfigManager {
    ssh_config_path: PathBuf,
    tinybridge_config_path: PathBuf,
}

impl SshConfigManager {
    /// Create a new SSH config manager
    pub fn new(ssh_config_path: impl AsRef<Path>) -> Self {
        let ssh_config_path = ssh_config_path.as_ref().to_path_buf();
        let tinybridge_config_path = ssh_config_path.parent()
            .map(|p| p.join("config.d/tinybridge-auto.conf"))
            .unwrap_or_else(|| PathBuf::from("tinybridge-auto.conf"));

        Self {
            ssh_config_path,
            tinybridge_config_path,
        }
    }

    /// Create config manager with default ~/.ssh/config path
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| SshError::InvalidEnvironment("HOME directory not found".to_string()))?;
        let ssh_config_path = home.join(".ssh/config");
        Ok(Self::new(ssh_config_path))
    }

    /// Add or update an SSH config entry
    pub fn add_entry(&self, entry: &SshConfigEntry) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.tinybridge_config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| SshError::ConfigWriteError(e.to_string()))?;
        }

        // Read existing entries
        let mut entries = self.load_entries()?;

        // Update or insert
        entries.insert(entry.alias.clone(), entry.clone());

        // Write back
        self.save_entries(&entries)?;

        // Ensure main ssh/config includes our file
        self.ensure_include()?;

        Ok(())
    }

    /// Remove an SSH config entry
    pub fn remove_entry(&self, alias: &str) -> Result<()> {
        let mut entries = self.load_entries()?;
        entries.remove(alias);
        self.save_entries(&entries)?;

        Ok(())
    }

    /// Get an entry by alias
    pub fn get_entry(&self, alias: &str) -> Result<Option<SshConfigEntry>> {
        let entries = self.load_entries()?;
        Ok(entries.get(alias).cloned())
    }

    /// List all TinyBridge entries
    pub fn list_entries(&self) -> Result<Vec<SshConfigEntry>> {
        let entries = self.load_entries()?;
        Ok(entries.values().cloned().collect())
    }

    /// Update hostname for an entry (e.g., IP change)
    pub fn update_hostname(&self, alias: &str, new_hostname: &str) -> Result<()> {
        let mut entries = self.load_entries()?;

        if let Some(entry) = entries.get_mut(alias) {
            entry.hostname = new_hostname.to_string();
            self.save_entries(&entries)?;
        } else {
            return Err(SshError::InvalidConfig(format!("Entry not found: {}", alias)));
        }

        Ok(())
    }

    // Private methods

    fn load_entries(&self) -> Result<HashMap<String, SshConfigEntry>> {
        let mut entries = HashMap::new();

        if !self.tinybridge_config_path.exists() {
            return Ok(entries);
        }

        let content = fs::read_to_string(&self.tinybridge_config_path)
            .map_err(|e| SshError::ConfigReadError(e.to_string()))?;

        let mut current_alias = String::new();
        let mut current_block = String::new();
        let host_regex = Regex::new(r"^Host\s+(\S+)").unwrap();

        for line in content.lines() {
            if let Some(caps) = host_regex.captures(line) {
                if !current_alias.is_empty() {
                    if let Ok(entry) = SshConfigEntry::from_config_block(&current_alias, &current_block) {
                        entries.insert(current_alias.clone(), entry);
                    }
                }

                current_alias = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                current_block = String::new();
            }

            current_block.push_str(line);
            current_block.push('\n');
        }

        // Don't forget the last entry
        if !current_alias.is_empty() {
            if let Ok(entry) = SshConfigEntry::from_config_block(&current_alias, &current_block) {
                entries.insert(current_alias, entry);
            }
        }

        Ok(entries)
    }

    fn save_entries(&self, entries: &HashMap<String, SshConfigEntry>) -> Result<()> {
        let mut content = String::from("# TinyBridge SSH Configuration - DO NOT EDIT\n");
        content.push_str("# Auto-generated by TinyBridge. Manual edits will be lost.\n");
        content.push_str("# To add custom entries, create ~/.ssh/config.d/custom.conf\n\n");

        let mut aliases: Vec<_> = entries.keys().cloned().collect();
        aliases.sort();

        for alias in aliases {
            if let Some(entry) = entries.get(&alias) {
                content.push_str(&format!("# TinyBridge Environment ID: {}\n", entry.env_id));
                content.push_str(&entry.to_config_block());
            }
        }

        fs::write(&self.tinybridge_config_path, content)
            .map_err(|e| SshError::ConfigWriteError(e.to_string()))?;

        Ok(())
    }

    fn ensure_include(&self) -> Result<()> {
        let ssh_dir = self.ssh_config_path.parent()
            .ok_or_else(|| SshError::ConfigWriteError("Invalid SSH config path".to_string()))?;

        // Ensure .ssh directory exists
        fs::create_dir_all(ssh_dir)
            .map_err(|e| SshError::ConfigWriteError(e.to_string()))?;

        // Read main ssh/config
        let config_content = if self.ssh_config_path.exists() {
            fs::read_to_string(&self.ssh_config_path)
                .map_err(|e| SshError::ConfigReadError(e.to_string()))?
        } else {
            String::new()
        };

        // Check if include is already present
        let include_line = format!("Include config.d/tinybridge-auto.conf");
        if config_content.contains(&include_line) {
            return Ok(());
        }

        // Append include directive
        let mut new_content = config_content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(&include_line);
        new_content.push('\n');

        fs::write(&self.ssh_config_path, new_content)
            .map_err(|e| SshError::ConfigWriteError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_entry_generation() {
        let entry = SshConfigEntry {
            env_id: Uuid::new_v4(),
            alias: "myvm".to_string(),
            hostname: "192.168.64.2".to_string(),
            user: "user".to_string(),
            port: 22,
            identity_file: PathBuf::from("~/.tinybridge/keys/id_ed25519"),
            options: Default::default(),
        };

        let block = entry.to_config_block();
        assert!(block.contains("Host myvm"));
        assert!(block.contains("HostName 192.168.64.2"));
        assert!(block.contains("User user"));
    }

    #[test]
    fn test_config_manager_add_entry() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".ssh/config");
        let manager = SshConfigManager::new(&config_path);

        let entry = SshConfigEntry {
            env_id: Uuid::new_v4(),
            alias: "myvm".to_string(),
            hostname: "192.168.64.2".to_string(),
            user: "user".to_string(),
            port: 22,
            identity_file: PathBuf::from("~/.tinybridge/keys/id_ed25519"),
            options: Default::default(),
        };

        assert!(manager.add_entry(&entry).is_ok());
        assert!(manager.get_entry("myvm").is_ok());
    }
}
