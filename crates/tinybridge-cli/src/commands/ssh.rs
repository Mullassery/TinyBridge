use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tinybridge_core::ssh_keys::SshKeyManager;

use crate::output;

#[derive(clap::Parser)]
pub struct SshArgs {
    #[arg(help = "Environment name")]
    pub name: Option<String>,

    #[arg(help = "Command to execute (optional, runs interactive shell if not provided)")]
    pub command: Option<String>,

    #[arg(long, help = "SSH options (e.g., '-p 2222')")]
    pub options: Option<String>,
}

pub async fn execute(args: SshArgs, _socket: Option<PathBuf>) -> Result<()> {
    let env_name = args.name.as_deref().unwrap_or("default");

    // Get or create SSH key
    let private_key = SshKeyManager::get_private_key_path()?;

    // SSH connection details
    let user = "vm";
    let host = env_name;
    let ssh_addr = format!("{}@{}", user, host);

    // Build SSH command
    let mut ssh_cmd = Command::new("ssh");

    // Use TinyBridge's private key
    ssh_cmd.arg("-i").arg(&private_key);

    // Add custom options if provided
    if let Some(opts) = &args.options {
        for opt in opts.split_whitespace() {
            ssh_cmd.arg(opt);
        }
    }

    // Add connection address
    ssh_cmd.arg(&ssh_addr);

    // Add command if provided, otherwise interactive shell
    if let Some(cmd) = &args.command {
        ssh_cmd.arg(cmd);
    }

    // Execute SSH
    let status = ssh_cmd.status()?;

    if !status.success() && status.code() == Some(255) {
        output::print_error(&format!(
            "SSH connection failed. Make sure the environment is running:\n  tinybridge up {}",
            env_name
        ));
    }

    Ok(())
}
