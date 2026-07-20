use crate::error::{ClipboardError, Result};
use std::process::Command;
use tokio::process::Command as TokioCommand;

/// Access Linux clipboard via SSH
pub struct LinuxClipboard {
    host: String,
    port: u16,
    user: String,
}

impl LinuxClipboard {
    /// Create a new Linux clipboard accessor
    ///
    /// # Arguments
    /// * `host` - SSH host address
    /// * `port` - SSH port (default 22)
    /// * `user` - SSH username
    pub fn new(host: impl Into<String>, port: u16, user: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            user: user.into(),
        }
    }

    /// Default constructor for local VM (assumes 127.0.0.1:2222)
    pub fn local_vm(user: impl Into<String>) -> Self {
        Self::new("127.0.0.1", 2222, user)
    }

    /// Read text from Linux clipboard via SSH (async)
    pub async fn read_text(&self) -> Result<Option<String>> {
        let ssh_cmd = format!("{}@{}", self.user, self.host);

        let output = TokioCommand::new("ssh")
            .arg("-p")
            .arg(self.port.to_string())
            .arg(&ssh_cmd)
            .arg("xclip -selection clipboard -o 2>/dev/null || xsel --clipboard --output 2>/dev/null || echo ''")
            .output()
            .await
            .map_err(|e| ClipboardError::SshError(e.to_string()))?;

        if output.status.success() {
            let text = String::from_utf8(output.stdout)?;
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed))
            }
        } else {
            Err(ClipboardError::ReadError(
                "Failed to read from Linux clipboard".to_string(),
            ))
        }
    }

    /// Write text to Linux clipboard via SSH (async)
    pub async fn write_text(&self, text: &str) -> Result<()> {
        let ssh_cmd = format!("{}@{}", self.user, self.host);

        let mut child = TokioCommand::new("ssh")
            .arg("-p")
            .arg(self.port.to_string())
            .arg(&ssh_cmd)
            .arg("xclip -selection clipboard 2>/dev/null || xsel --clipboard --input 2>/dev/null")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ClipboardError::SshError(e.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(text.as_bytes())
                .await
                .map_err(|e| ClipboardError::WriteError(e.to_string()))?;
        }

        Ok(())
    }

    /// Check if xclip or xsel is available
    pub fn is_available(&self) -> Result<bool> {
        let ssh_cmd = format!("{}@{}", self.user, self.host);

        let output = Command::new("ssh")
            .arg("-p")
            .arg(self.port.to_string())
            .arg(&ssh_cmd)
            .arg("which xclip > /dev/null 2>&1 || which xsel > /dev/null 2>&1")
            .output()
            .map_err(|e| ClipboardError::SshError(e.to_string()))?;

        Ok(output.status.success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_clipboard_new() {
        let clipboard = LinuxClipboard::new("localhost", 2222, "user");
        assert_eq!(clipboard.host, "localhost");
        assert_eq!(clipboard.port, 2222);
        assert_eq!(clipboard.user, "user");
    }

    #[test]
    fn test_local_vm_default() {
        let clipboard = LinuxClipboard::local_vm("user");
        assert_eq!(clipboard.host, "127.0.0.1");
        assert_eq!(clipboard.port, 2222);
        assert_eq!(clipboard.user, "user");
    }
}
