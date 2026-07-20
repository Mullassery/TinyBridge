use crate::error::{Result, SshError};
use serde::{Deserialize, Serialize};

/// Supported Linux distributions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Fedora,
    CentOS,
    Rhel,
    Rocky,
    Alpine,
    Arch,
    Generic,
}

impl LinuxDistro {
    /// Detect distro from common identifiers
    pub fn detect(os_name: &str, _version_id: Option<&str>) -> Self {
        let lower = os_name.to_lowercase();

        match lower.as_str() {
            s if s.contains("ubuntu") => LinuxDistro::Ubuntu,
            s if s.contains("debian") => LinuxDistro::Debian,
            s if s.contains("fedora") => LinuxDistro::Fedora,
            s if s.contains("centos") => LinuxDistro::CentOS,
            s if s.contains("rhel") => LinuxDistro::Rhel,
            s if s.contains("rocky") => LinuxDistro::Rocky,
            s if s.contains("alpine") => LinuxDistro::Alpine,
            s if s.contains("arch") => LinuxDistro::Arch,
            _ => LinuxDistro::Generic,
        }
    }
}

/// SSH provisioning method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvisioningMethod {
    /// Cloud-init (Ubuntu, Debian, Fedora, CentOS, RHEL, Rocky)
    CloudInit,
    /// Ignition (CoreOS, Fedora CoreOS)
    Ignition,
    /// Custom script delivery
    CustomScript,
}

/// SSH provisioning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionConfig {
    pub distro: LinuxDistro,
    pub username: String,
    pub public_key: String,
    pub method: ProvisioningMethod,
}

/// SSH provisioning payload
#[derive(Debug, Clone, Serialize)]
pub struct ProvisioningPayload {
    pub content: String,
    pub filename: String,
    pub path: String,
    pub format: String, // "cloud-init", "ignition", "script"
}

/// Handles SSH key provisioning for guest OS
pub struct SshProvisioner;

impl SshProvisioner {
    /// Generate provisioning payload for a distribution
    pub fn generate_payload(config: &ProvisionConfig) -> Result<ProvisioningPayload> {
        match config.method {
            ProvisioningMethod::CloudInit => Self::generate_cloud_init(config),
            ProvisioningMethod::Ignition => Self::generate_ignition(config),
            ProvisioningMethod::CustomScript => Self::generate_custom_script(config),
        }
    }

    /// Generate cloud-init user-data for SSH key injection
    fn generate_cloud_init(config: &ProvisionConfig) -> Result<ProvisioningPayload> {
        let yaml = format!(
            r#"#cloud-config
# TinyBridge SSH key provisioning
users:
  - name: {}
    ssh_authorized_keys:
      - {}
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash

# Ensure SSH is enabled
packages:
  - openssh-server

runcmd:
  - systemctl enable ssh
  - systemctl start ssh
  - echo "TinyBridge SSH provisioning complete"
"#,
            config.username,
            config.public_key.trim()
        );

        Ok(ProvisioningPayload {
            content: yaml,
            filename: "user-data".to_string(),
            path: "/var/lib/cloud/instance/user-data.txt".to_string(),
            format: "cloud-init".to_string(),
        })
    }

    /// Generate Ignition configuration for SSH key injection
    fn generate_ignition(config: &ProvisionConfig) -> Result<ProvisioningPayload> {
        let json = serde_json::json!({
            "ignition": {
                "version": "3.3.0"
            },
            "storage": {
                "files": [
                    {
                        "path": format!("/home/{}/.ssh/authorized_keys", config.username),
                        "contents": {
                            "inline": config.public_key.trim()
                        },
                        "mode": 384,
                        "user": {
                            "name": config.username
                        }
                    }
                ]
            },
            "systemd": {
                "units": [
                    {
                        "name": "sshd.service",
                        "enabled": true,
                        "contents": "[Unit]\nDescription=OpenSSH\nAfter=network-online.target\n\n[Service]\nType=notify\nExecStart=/usr/sbin/sshd -D\nRestart=on-failure\n\n[Install]\nWantedBy=multi-user.target\n"
                    }
                ]
            }
        });

        Ok(ProvisioningPayload {
            content: serde_json::to_string_pretty(&json)
                .map_err(|e| SshError::ProvisioningError(e.to_string()))?,
            filename: "config.ign".to_string(),
            path: "/etc/ignition/config.ign".to_string(),
            format: "ignition".to_string(),
        })
    }

    /// Generate a custom shell script for SSH key injection
    fn generate_custom_script(config: &ProvisionConfig) -> Result<ProvisioningPayload> {
        let script = format!(
            r#"#!/bin/bash
set -euo pipefail

# TinyBridge SSH provisioning script
echo "TinyBridge: Setting up SSH access for {}"

# Create .ssh directory
mkdir -p /home/{}/.ssh
chmod 700 /home/{}/.ssh

# Add authorized key
echo '{}' >> /home/{}/.ssh/authorized_keys
chmod 600 /home/{}/.ssh/authorized_keys
chown -R {}:{} /home/{}/.ssh

# Ensure SSH server is installed and running
if command -v apk &> /dev/null; then
  # Alpine
  apk add --no-cache openssh
  rc-service sshd start
  rc-update add sshd
elif command -v apt-get &> /dev/null; then
  # Debian/Ubuntu
  apt-get update
  apt-get install -y openssh-server
  systemctl enable ssh
  systemctl start ssh
elif command -v dnf &> /dev/null; then
  # Fedora
  dnf install -y openssh-server openssh-clients
  systemctl enable sshd
  systemctl start sshd
elif command -v yum &> /dev/null; then
  # CentOS/RHEL
  yum install -y openssh-server openssh-clients
  systemctl enable sshd
  systemctl start sshd
elif command -v pacman &> /dev/null; then
  # Arch
  pacman -S --noconfirm openssh
  systemctl enable sshd
  systemctl start sshd
fi

echo "TinyBridge: SSH provisioning complete"
"#,
            config.username,
            config.username,
            config.username,
            config.public_key.trim(),
            config.username,
            config.username,
            config.username,
            config.username,
            config.username
        );

        Ok(ProvisioningPayload {
            content: script,
            filename: "tinybridge-ssh-provision.sh".to_string(),
            path: "/usr/local/bin/tinybridge-ssh-provision.sh".to_string(),
            format: "script".to_string(),
        })
    }

    /// Recommend provisioning method for a distribution
    pub fn recommend_method(distro: LinuxDistro) -> ProvisioningMethod {
        match distro {
            LinuxDistro::Alpine => ProvisioningMethod::CustomScript,
            LinuxDistro::Arch => ProvisioningMethod::CustomScript,
            LinuxDistro::Generic => ProvisioningMethod::CustomScript,
            _ => ProvisioningMethod::CloudInit,
        }
    }

    /// Generate SSH key verification script
    pub fn generate_verify_script(public_key_fingerprint: &str) -> String {
        format!(
            r#"#!/bin/bash
# Verify SSH provisioning
set -euo pipefail

echo "Verifying SSH provisioning..."

# Check SSH server is running
if ! systemctl is-active --quiet sshd && ! systemctl is-active --quiet ssh; then
  echo "ERROR: SSH server not running"
  exit 1
fi

# Check authorized_keys exists and contains our key
if [ ! -f ~/.ssh/authorized_keys ]; then
  echo "ERROR: authorized_keys not found"
  exit 1
fi

echo "SSH provisioning verified successfully"
echo "Fingerprint: {}"
exit 0
"#,
            public_key_fingerprint
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_detection() {
        assert_eq!(LinuxDistro::detect("Ubuntu", None), LinuxDistro::Ubuntu);
        assert_eq!(LinuxDistro::detect("debian", None), LinuxDistro::Debian);
        assert_eq!(LinuxDistro::detect("Alpine", None), LinuxDistro::Alpine);
        assert_eq!(LinuxDistro::detect("Fedora", None), LinuxDistro::Fedora);
    }

    #[test]
    fn test_cloud_init_generation() {
        let config = ProvisionConfig {
            distro: LinuxDistro::Ubuntu,
            username: "user".to_string(),
            public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB...".to_string(),
            method: ProvisioningMethod::CloudInit,
        };

        let payload = SshProvisioner::generate_payload(&config).unwrap();
        assert_eq!(payload.format, "cloud-init");
        assert!(payload.content.contains("users:"));
        assert!(payload.content.contains("user"));
        assert!(payload.content.contains("ssh_authorized_keys"));
    }

    #[test]
    fn test_custom_script_generation() {
        let config = ProvisionConfig {
            distro: LinuxDistro::Alpine,
            username: "user".to_string(),
            public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB...".to_string(),
            method: ProvisioningMethod::CustomScript,
        };

        let payload = SshProvisioner::generate_payload(&config).unwrap();
        assert_eq!(payload.format, "script");
        assert!(payload.content.contains("#!/bin/bash"));
        assert!(payload.content.contains(".ssh/authorized_keys"));
    }

    #[test]
    fn test_verify_script_generation() {
        let script = SshProvisioner::generate_verify_script("SHA256:abc123");
        assert!(script.contains("Verifying SSH provisioning"));
        assert!(script.contains("authorized_keys"));
    }
}
