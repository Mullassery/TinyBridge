pub mod error;
pub mod linux;
pub mod macos;
pub mod bridge;

pub use bridge::ClipboardBridge;
pub use error::{ClipboardError, Result};
pub use linux::LinuxClipboard;
pub use macos::MacosPasteboard;
