//! Active application detection via macOS lsappinfo.
//!
//! Detects the frontmost application when recording starts to determine
//! the appropriate style for dictation enhancement.

use super::ActiveApp;
use std::process::Command;

/// Get the currently active (frontmost) application.
///
/// Uses `lsappinfo` which is ~35x faster than AppleScript (~7ms vs ~250ms).
/// This is critical for instant overlay response.
///
/// Returns `None` if detection fails.
pub fn get_active_app() -> Option<ActiveApp> {
    // Get the frontmost app's ASN (Application Serial Number)
    let front_output = Command::new("lsappinfo").arg("front").output().ok()?;

    if !front_output.status.success() {
        return None;
    }

    let asn = String::from_utf8_lossy(&front_output.stdout)
        .trim()
        .to_string();
    if asn.is_empty() || asn == "-" {
        return None;
    }

    // Get bundle ID and name for this ASN
    let info_output = Command::new("lsappinfo")
        .args(["info", "-only", "bundleid", "-only", "name", &asn])
        .output()
        .ok()?;

    if !info_output.status.success() {
        return None;
    }

    let info = String::from_utf8_lossy(&info_output.stdout);

    // Parse output like:
    // "CFBundleIdentifier"="com.example.app"
    // "CFBundleName"="App Name"
    let mut bundle_id = None;
    let mut name = None;

    for line in info.lines() {
        if let Some(value) = extract_lsappinfo_value(line, "CFBundleIdentifier") {
            bundle_id = Some(value);
        } else if let Some(value) = extract_lsappinfo_value(line, "CFBundleName") {
            name = Some(value);
        } else if let Some(value) = extract_lsappinfo_value(line, "LSDisplayName") {
            // Fallback to display name if bundle name not available
            if name.is_none() {
                name = Some(value);
            }
        }
    }

    match (bundle_id, name) {
        (Some(bid), Some(n)) => Some(ActiveApp {
            bundle_id: bid,
            name: n,
        }),
        (Some(bid), None) => Some(ActiveApp {
            bundle_id: bid.clone(),
            name: bid, // Use bundle ID as fallback name
        }),
        _ => None,
    }
}

/// Extract a value from lsappinfo output format: "Key"="Value"
fn extract_lsappinfo_value(line: &str, key: &str) -> Option<String> {
    let prefix = format!("\"{}\"=", key);
    if line.trim().starts_with(&prefix) {
        let value_part = line.trim().strip_prefix(&prefix)?;
        // Remove surrounding quotes
        let value = value_part.trim_matches('"');
        Some(value.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_active_app_returns_something() {
        // This test will only pass when run on macOS with a GUI
        // In headless CI, it may return None
        let app = get_active_app();
        // Just verify it doesn't panic
        println!("Active app: {:?}", app);
    }

    #[test]
    fn test_extract_lsappinfo_value() {
        assert_eq!(
            extract_lsappinfo_value(
                "\"CFBundleIdentifier\"=\"com.example.app\"",
                "CFBundleIdentifier"
            ),
            Some("com.example.app".to_string())
        );
        assert_eq!(
            extract_lsappinfo_value("\"CFBundleName\"=\"My App\"", "CFBundleName"),
            Some("My App".to_string())
        );
        assert_eq!(
            extract_lsappinfo_value("\"Other\"=\"value\"", "CFBundleIdentifier"),
            None
        );
    }
}
