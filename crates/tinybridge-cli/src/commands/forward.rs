use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use tinybridge_core::ssh_keys::SshKeyManager;

use crate::output;

#[derive(clap::Parser)]
pub struct ForwardArgs {
    #[arg(help = "Environment name (defaults to 'default')")]
    pub name: Option<String>,

    #[arg(
        long,
        help = "Local port to listen on (default: 8080)",
        default_value_t = 8080
    )]
    pub local_port: u16,

    #[arg(long, help = "Port inside the guest (required)")]
    pub remote_port: u16,

    #[arg(
        long,
        help = "Guest host to forward to (default: localhost)",
        default_value = "localhost"
    )]
    pub remote_host: String,

    #[arg(
        long,
        help = "Additional SSH options (e.g., '-o StrictHostKeyChecking=no')"
    )]
    pub options: Option<String>,
}

pub fn build_ssh_args(
    private_key: &str,
    environment: &str,
    local_port: u16,
    remote_host: &str,
    remote_port: u16,
    options: Option<&str>,
) -> Vec<String> {
    let mut args = vec![
        "-i".to_string(),
        private_key.to_string(),
        "-N".to_string(),
        "-L".to_string(),
        format!("{local_port}:{remote_host}:{remote_port}"),
    ];
    if let Some(options) = options {
        args.extend(options.split_whitespace().map(|value| value.to_string()));
    }
    args.push(format!("vm@{environment}"));
    args
}

pub async fn execute(args: ForwardArgs, _socket: Option<PathBuf>) -> Result<()> {
    let env_name = args.name.as_deref().unwrap_or("default");
    let private_key = SshKeyManager::get_private_key_path()?;

    output::print_info(&format!(
        "Forwarding localhost:{} -> {}:{} in {} (Ctrl-C to stop)",
        args.local_port, args.remote_host, args.remote_port, env_name
    ));

    let status = Command::new("ssh")
        .args(build_ssh_args(
            &private_key.to_string_lossy(),
            env_name,
            args.local_port,
            &args.remote_host,
            args.remote_port,
            args.options.as_deref(),
        ))
        .status()?;

    if !status.success() && status.code() == Some(255) {
        output::print_error(&format!(
            "SSH forwarding failed. Make sure the environment is running and SSH is reachable:\n  tinybridge status {env_name}\n  tinybridge ssh {env_name}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_a_local_forward_with_the_guest_host() {
        let args = build_ssh_args("/keys/id", "demo", 8080, "localhost", 80, None);
        assert_eq!(
            args,
            vec!["-i", "/keys/id", "-N", "-L", "8080:localhost:80", "vm@demo"]
        );
    }

    #[test]
    fn passes_extra_ssh_options() {
        let args = build_ssh_args(
            "/keys/id",
            "demo",
            5432,
            "db.internal",
            5432,
            Some("-o StrictHostKeyChecking=no"),
        );
        assert!(args
            .windows(2)
            .any(|w| w == ["-o", "StrictHostKeyChecking=no"]));
        assert_eq!(args.last().unwrap(), "vm@demo");
    }
}
