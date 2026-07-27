use anyhow::Result;
use std::path::PathBuf;
use serde_json::json;

use crate::client::DaemonClient;
use crate::output;

#[derive(clap::Parser)]
pub struct SnapshotArgs {
    #[arg(help = "Environment name")]
    pub name: String,

    #[command(subcommand)]
    pub action: SnapshotAction,
}

#[derive(clap::Subcommand)]
pub enum SnapshotAction {
    /// Create a new snapshot
    Create {
        #[arg(help = "Snapshot name")]
        snapshot_name: String,

        #[arg(long, help = "Description of the snapshot")]
        description: Option<String>,
    },
    /// Restore from a snapshot
    Restore {
        #[arg(help = "Snapshot name to restore")]
        snapshot_name: String,

        #[arg(long, help = "Force restore even if environment is running")]
        force: bool,
    },
    /// List snapshots
    List,
    /// Delete a snapshot
    Delete {
        #[arg(help = "Snapshot name")]
        snapshot_name: String,
    },
}

pub async fn execute(args: SnapshotArgs, socket: Option<PathBuf>) -> Result<()> {
    let mut client = DaemonClient::new(socket)?;

    match args.action {
        SnapshotAction::Create {
            snapshot_name,
            description,
        } => {
            let params = json!({
                "name": args.name,
                "snapshot_name": snapshot_name,
                "description": description,
            });
            client.call("environment.snapshot.create", params).await?;
            output::print_success(&format!(
                "Snapshot '{}' created for environment '{}'",
                snapshot_name, args.name
            ));
        }
        SnapshotAction::Restore {
            snapshot_name,
            force,
        } => {
            let params = json!({
                "name": args.name,
                "snapshot_name": snapshot_name,
                "force": force,
            });
            client.call("environment.snapshot.restore", params).await?;
            output::print_success(&format!(
                "Environment '{}' restored from snapshot '{}'",
                args.name, snapshot_name
            ));
        }
        SnapshotAction::List => {
            let params = json!({"name": args.name});
            let result = client.call("environment.snapshot.list", params).await?;
            if let Some(snapshots) = result.get("snapshots").and_then(|s| s.as_array()) {
                if snapshots.is_empty() {
                    output::print_info(&format!(
                        "No snapshots found for environment '{}'",
                        args.name
                    ));
                } else {
                    println!("\n📸 Snapshots for '{}':", args.name);
                    for snapshot in snapshots {
                        if let Some(name) = snapshot.get("name").and_then(|n| n.as_str()) {
                            let created = snapshot
                                .get("created_at")
                                .and_then(|c| c.as_str())
                                .unwrap_or("unknown");
                            println!("  • {} (created: {})", name, created);
                        }
                    }
                    println!();
                }
            }
        }
        SnapshotAction::Delete { snapshot_name } => {
            let params = json!({
                "name": args.name,
                "snapshot_name": snapshot_name,
            });
            client.call("environment.snapshot.delete", params).await?;
            output::print_success(&format!(
                "Snapshot '{}' deleted from environment '{}'",
                snapshot_name, args.name
            ));
        }
    }

    Ok(())
}
