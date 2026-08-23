use crate::config::TinyBridgeConfig;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, warn};

/// Migrate from legacy (~/.tinybridge) to Apple-convention paths.
/// This function is idempotent: safe to call on every startup.
/// Best-effort: log warnings on errors, don't block startup.
pub fn migrate_legacy_layout() -> Result<()> {
    debug!("Checking for legacy TinyBridge layout to migrate");

    migrate_launchd_agent();
    migrate_data_directories()?;

    Ok(())
}

/// Attempt to unload and remove the old mislabeled LaunchDaemon plist.
fn migrate_launchd_agent() {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let old_launchd_path = format!("{}/Library/LaunchDaemons/com.tinybridge.daemon.plist", home);

    if !Path::new(&old_launchd_path).exists() {
        return;
    }

    debug!(
        "Found legacy LaunchDaemon plist at {}, attempting cleanup",
        old_launchd_path
    );

    // Try both possible launchctl domains (old formula's load didn't qualify a domain)
    let domains = ["system/com.tinybridge.daemon", "gui/com.tinybridge.daemon"];
    for domain in &domains {
        let _ = Command::new("launchctl")
            .args(&["bootout", domain])
            .output();
    }

    // Remove the plist file
    if let Err(e) = fs::remove_file(&old_launchd_path) {
        warn!(
            "Failed to remove old LaunchDaemon plist {}: {}",
            old_launchd_path, e
        );
    } else {
        debug!("Removed old LaunchDaemon plist");
    }
}

/// Migrate data from ~/.tinybridge to new Apple-convention paths.
fn migrate_data_directories() -> Result<()> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Ok(()),
    };
    let legacy_dir = format!("{}/.tinybridge", home);
    let legacy_path = Path::new(&legacy_dir);

    // Only migrate if legacy dir exists AND new dir doesn't exist yet
    if !legacy_path.exists() {
        return Ok(());
    }

    let new_data_dir = TinyBridgeConfig::data_dir();
    let new_cache_dir = TinyBridgeConfig::cache_dir();

    if new_data_dir.exists() && new_cache_dir.exists() {
        debug!("Migration already complete (new data/cache dirs exist)");
        return Ok(());
    }

    debug!("Migrating legacy ~/.tinybridge to Apple-convention paths");

    // Create new directories
    fs::create_dir_all(&new_data_dir)?;
    fs::create_dir_all(&new_cache_dir)?;

    // Migrate keys (identity material → Application Support)
    let legacy_keys = Path::new(&legacy_dir).join("keys");
    if legacy_keys.exists() {
        let new_keys = new_data_dir.join("keys");
        if !new_keys.exists() {
            if let Err(e) = fs::rename(&legacy_keys, &new_keys) {
                warn!("Failed to migrate keys dir: {}", e);
            } else {
                debug!("Migrated keys to Application Support");
            }
        }
    }

    // Migrate SSH config metadata (persistent → Application Support)
    let legacy_ssh = Path::new(&legacy_dir).join("ssh");
    if legacy_ssh.exists() {
        let new_ssh = new_data_dir.join("ssh");
        if !new_ssh.exists() {
            if let Err(e) = fs::rename(&legacy_ssh, &new_ssh) {
                warn!("Failed to migrate ssh dir: {}", e);
            } else {
                debug!("Migrated SSH metadata to Application Support");
            }
        }
    }

    // Migrate assets (downloadable → Caches)
    let legacy_assets = Path::new(&legacy_dir).join("assets");
    if legacy_assets.exists() {
        let new_assets = new_cache_dir.join("assets");
        if !new_assets.exists() {
            if let Err(e) = fs::rename(&legacy_assets, &new_assets) {
                warn!("Failed to migrate assets dir: {}", e);
            } else {
                debug!("Migrated assets to Caches");
            }
        }
    }

    // Don't migrate shells (ephemeral, regenerated on demand)
    // Leave legacy_dir in place if migration was partial; user can manual cleanup

    Ok(())
}
