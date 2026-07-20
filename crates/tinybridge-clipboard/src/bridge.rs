use crate::error::Result;
use crate::linux::LinuxClipboard;
use crate::macos::MacosPasteboard;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::{debug, error, warn};

/// Clipboard bridge state
#[derive(Debug, Clone)]
struct BridgeState {
    last_macos_text: Option<String>,
    last_macos_change_count: u64,
    last_linux_text: Option<String>,
}

/// Bidirectional clipboard bridge between macOS and Linux
pub struct ClipboardBridge {
    linux_clipboard: LinuxClipboard,
    state: Arc<Mutex<BridgeState>>,
    sync_interval: Duration,
}

impl ClipboardBridge {
    /// Create a new clipboard bridge
    pub fn new(linux_clipboard: LinuxClipboard, sync_interval_ms: u64) -> Self {
        Self {
            linux_clipboard,
            state: Arc::new(Mutex::new(BridgeState {
                last_macos_text: None,
                last_macos_change_count: 0,
                last_linux_text: None,
            })),
            sync_interval: Duration::from_millis(sync_interval_ms),
        }
    }

    /// Sync clipboard from macOS to Linux
    #[cfg(target_os = "macos")]
    pub async fn sync_macos_to_linux(&self) -> Result<()> {
        let current_change_count = MacosPasteboard::change_count()?;

        let mut state = self.state.lock().await;

        if current_change_count > state.last_macos_change_count {
            if let Ok(Some(text)) = MacosPasteboard::read_text() {
                if state.last_macos_text.as_ref() != Some(&text) {
                    debug!("Syncing macOS clipboard to Linux: {} bytes", text.len());
                    if let Err(e) = self.linux_clipboard.write_text(&text).await {
                        error!("Failed to sync macOS to Linux: {}", e);
                        return Err(e);
                    }
                    state.last_macos_text = Some(text);
                }
            }
            state.last_macos_change_count = current_change_count;
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub async fn sync_macos_to_linux(&self) -> Result<()> {
        Ok(())
    }

    /// Sync clipboard from Linux to macOS
    pub async fn sync_linux_to_macos(&self) -> Result<()> {
        if let Ok(Some(text)) = self.linux_clipboard.read_text().await {
            let mut state = self.state.lock().await;

            if state.last_linux_text.as_ref() != Some(&text) {
                debug!("Syncing Linux clipboard to macOS: {} bytes", text.len());
                MacosPasteboard::write_text(&text)?;
                state.last_linux_text = Some(text);
            }
        }

        Ok(())
    }

    /// Start continuous clipboard sync in background
    pub async fn start_sync(&self) -> Result<()> {
        debug!("Starting clipboard bridge sync (interval: {:?})", self.sync_interval);

        let mut sync_timer = interval(self.sync_interval);

        loop {
            sync_timer.tick().await;

            // Try macOS → Linux sync
            if let Err(e) = self.sync_macos_to_linux().await {
                warn!("macOS to Linux sync failed: {}", e);
            }

            // Try Linux → macOS sync
            if let Err(e) = self.sync_linux_to_macos().await {
                warn!("Linux to macOS sync failed: {}", e);
            }
        }
    }

    /// Perform a single sync cycle in both directions
    pub async fn sync_once(&self) -> Result<()> {
        debug!("Performing single clipboard sync");

        // Sync both directions (errors don't block each other)
        let macos_result = self.sync_macos_to_linux().await;
        let linux_result = self.sync_linux_to_macos().await;

        // Return first error if any
        macos_result?;
        linux_result?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let clipboard = LinuxClipboard::local_vm("user");
        let bridge = ClipboardBridge::new(clipboard, 1000);
        assert_eq!(bridge.sync_interval, Duration::from_millis(1000));
    }

    #[tokio::test]
    async fn test_sync_once_creates_state() {
        let clipboard = LinuxClipboard::local_vm("user");
        let bridge = ClipboardBridge::new(clipboard, 500);

        let state = bridge.state.lock().await;
        assert_eq!(state.last_macos_text, None);
        assert_eq!(state.last_linux_text, None);
    }
}
