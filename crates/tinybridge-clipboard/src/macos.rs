#![allow(unexpected_cfgs)]

#[cfg(target_os = "macos")]
use crate::error::{ClipboardError, Result};

#[cfg(target_os = "macos")]
use objc::msg_send;
#[cfg(target_os = "macos")]
use objc::runtime::Class;
#[cfg(target_os = "macos")]
use objc::sel;
#[cfg(target_os = "macos")]
use objc::sel_impl;
#[cfg(target_os = "macos")]
use objc_foundation::{INSString, NSString};

/// Access macOS pasteboard for clipboard operations
pub struct MacosPasteboard;

#[cfg(target_os = "macos")]
impl MacosPasteboard {
    /// Read text from the general pasteboard
    pub fn read_text() -> Result<Option<String>> {
        unsafe {
            let pasteboard_class = Class::get("NSPasteboard").ok_or_else(|| {
                ClipboardError::PasteboardError("NSPasteboard class not found".to_string())
            })?;

            // [NSPasteboard generalPasteboard]
            let pasteboard: *mut objc::runtime::Object =
                msg_send![pasteboard_class, generalPasteboard];
            if pasteboard.is_null() {
                return Err(ClipboardError::PasteboardError(
                    "Failed to get general pasteboard".to_string(),
                ));
            }

            // [pasteboard stringForType:NSPasteboardTypeString]
            let ns_type = NSString::from_str("public.utf8-plain-text");
            let content: *mut objc::runtime::Object = msg_send![pasteboard, stringForType:ns_type];

            if content.is_null() {
                return Ok(None);
            }

            // Convert NSString to Rust String
            let rust_string: *const u8 = msg_send![content, UTF8String];
            if rust_string.is_null() {
                return Ok(None);
            }

            let c_str = std::ffi::CStr::from_ptr(rust_string as *const i8);
            Ok(Some(c_str.to_string_lossy().to_string()))
        }
    }

    /// Write text to the general pasteboard
    pub fn write_text(text: &str) -> Result<()> {
        unsafe {
            let pasteboard_class = Class::get("NSPasteboard").ok_or_else(|| {
                ClipboardError::PasteboardError("NSPasteboard class not found".to_string())
            })?;

            // [NSPasteboard generalPasteboard]
            let pasteboard: *mut objc::runtime::Object =
                msg_send![pasteboard_class, generalPasteboard];
            if pasteboard.is_null() {
                return Err(ClipboardError::PasteboardError(
                    "Failed to get general pasteboard".to_string(),
                ));
            }

            // [pasteboard clearContents]
            let _: () = msg_send![pasteboard, clearContents];

            // Create NSString from our text using trait method
            let ns_string = NSString::from_str(text);
            let ns_string_type = NSString::from_str("public.utf8-plain-text");

            // [pasteboard setString:forType:]
            let success: bool = msg_send![pasteboard, setString:ns_string forType:ns_string_type];

            if success {
                Ok(())
            } else {
                Err(ClipboardError::WriteError(
                    "Failed to set pasteboard content".to_string(),
                ))
            }
        }
    }

    /// Get the change count of the pasteboard (for detecting changes)
    pub fn change_count() -> Result<u64> {
        unsafe {
            let pasteboard_class = Class::get("NSPasteboard").ok_or_else(|| {
                ClipboardError::PasteboardError("NSPasteboard class not found".to_string())
            })?;

            // [NSPasteboard generalPasteboard]
            let pasteboard: *mut objc::runtime::Object =
                msg_send![pasteboard_class, generalPasteboard];
            if pasteboard.is_null() {
                return Err(ClipboardError::PasteboardError(
                    "Failed to get general pasteboard".to_string(),
                ));
            }

            // [pasteboard changeCount]
            let count: i64 = msg_send![pasteboard, changeCount];
            Ok(count as u64)
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl MacosPasteboard {
    pub fn read_text() -> Result<Option<String>> {
        Err(ClipboardError::NotAvailable)
    }

    pub fn write_text(_text: &str) -> Result<()> {
        Err(ClipboardError::NotAvailable)
    }

    pub fn change_count() -> Result<u64> {
        Err(ClipboardError::NotAvailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_read_write_pasteboard() {
        let test_text = "TinyBridge test clipboard";
        assert!(MacosPasteboard::write_text(test_text).is_ok());

        let read = MacosPasteboard::read_text();
        assert!(read.is_ok());
        if let Ok(Some(content)) = read {
            assert_eq!(content, test_text);
        }
    }

    #[test]
    #[ignore]
    fn test_change_count() {
        let count1 = MacosPasteboard::change_count();
        assert!(count1.is_ok());

        let _ = MacosPasteboard::write_text("test");
        let count2 = MacosPasteboard::change_count();
        assert!(count2.is_ok());
    }
}
