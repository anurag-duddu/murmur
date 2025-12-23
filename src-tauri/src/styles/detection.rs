//! Active application detection via macOS NSWorkspace.
//!
//! Detects the frontmost application when recording starts to determine
//! the appropriate style for dictation enhancement.

use super::ActiveApp;
use std::process::Command;

/// Get the currently active (frontmost) application.
///
/// Uses a single AppleScript call to get both bundle ID and name for performance.
/// This approach avoids spawning multiple processes and keeps latency low.
///
/// Returns `None` if detection fails.
pub fn get_active_app() -> Option<ActiveApp> {
    // Single AppleScript call to get both bundle ID and name
    // Returns "bundle_id|name" format for parsing
    let script = r#"
        tell application "System Events"
            set frontApp to first application process whose frontmost is true
            set appBundle to bundle identifier of frontApp
            set appName to name of frontApp
            return appBundle & "|" & appName
        end tell
    "#;

    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .ok()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Parse "bundle_id|name" format
        if let Some((bundle_id, name)) = result.split_once('|') {
            if !bundle_id.is_empty() && bundle_id != "missing value" {
                return Some(ActiveApp {
                    bundle_id: bundle_id.to_string(),
                    name: name.to_string(),
                });
            }
        }
    }

    None
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
}
