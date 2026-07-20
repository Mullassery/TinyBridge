use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;

use crate::client::DaemonClient;
use crate::commands::ShellArgs;
use crate::output;
use crate::terminal::TerminalHandler;

pub async fn execute(args: ShellArgs, socket: Option<PathBuf>) -> Result<()> {
    let env_name = args.name.as_deref().unwrap_or("default");

    if let Some(cmd) = &args.command {
        // Non-interactive: run command and exit
        execute_command(cmd.clone(), env_name, socket).await
    } else {
        // Interactive: full keyboard support
        execute_interactive_shell(env_name, socket).await
    }
}

async fn execute_command(
    cmd: String,
    env_name: &str,
    socket: Option<PathBuf>,
) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    output::print_info(&format!("Executing in {}: {}", env_name, cmd));

    let params = json!({
        "name": env_name,
        "command": cmd,
    });

    client.shell_command(params).await
}

async fn execute_interactive_shell(env_name: &str, socket: Option<PathBuf>) -> Result<()> {
    output::print_info(&format!(
        "Opening shell in environment: {}",
        env_name
    ));
    output::print_info("Keyboard support: All keys including arrows, Ctrl+C, Ctrl+D, function keys");

    let mut client = DaemonClient::new(socket)?;

    // Request shell session from daemon
    let shell_socket = client.open_shell(env_name).await?;

    // Set up terminal with full keyboard support
    let mut terminal = TerminalHandler::new()?;
    terminal.handle_interactive_shell(shell_socket).await
}
