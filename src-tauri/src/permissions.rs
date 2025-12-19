use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub microphone: String, // "granted", "denied", "undetermined"
    pub accessibility: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrophoneDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Check if accessibility permission is granted
/// This checks if we can use AppleScript to control System Events
pub fn check_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Try to run a simple AppleScript that requires accessibility
        let result = Command::new("osascript")
            .arg("-e")
            .arg(r#"tell application "System Events" to return name of first process"#)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    return true;
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                // If we get a specific error about not being allowed, it's denied
                !stderr.contains("not allowed") && !stderr.contains("assistive")
            }
            Err(_) => false,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        true // Non-macOS platforms don't need this check
    }
}

/// Check microphone permission status
/// On macOS, this tries to enumerate input devices which requires permission
pub fn check_microphone_permission() -> String {
    #[cfg(target_os = "macos")]
    {
        // Try to access the audio host and enumerate devices
        // This will fail if microphone permission hasn't been granted
        let host = cpal::default_host();

        match host.default_input_device() {
            Some(device) => {
                // Try to get the device config - this confirms we have access
                match device.default_input_config() {
                    Ok(_) => "granted".to_string(),
                    Err(e) => {
                        let error_str = e.to_string().to_lowercase();
                        if error_str.contains("permission") || error_str.contains("denied") {
                            "denied".to_string()
                        } else {
                            // Some other error, but device exists
                            "granted".to_string()
                        }
                    }
                }
            }
            None => {
                // No default device - could be permission denied or no devices
                "undetermined".to_string()
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        "granted".to_string()
    }
}

/// Get list of available microphones
pub fn get_microphone_devices() -> Vec<MicrophoneDevice> {
    let host = cpal::default_host();
    let mut devices = Vec::new();

    // Get the default device name for comparison
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    // Enumerate all input devices
    if let Ok(input_devices) = host.input_devices() {
        for (index, device) in input_devices.enumerate() {
            if let Ok(name) = device.name() {
                let is_default = name == default_name;
                devices.push(MicrophoneDevice {
                    id: format!("device_{}", index),
                    name,
                    is_default,
                });
            }
        }
    }

    // If no devices found, add a default placeholder
    if devices.is_empty() {
        devices.push(MicrophoneDevice {
            id: "default".to_string(),
            name: "Default Microphone".to_string(),
            is_default: true,
        });
    }

    devices
}

/// Request microphone permission by triggering audio device access
/// On macOS, this will prompt the system permission dialog
pub fn request_microphone_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        let host = cpal::default_host();

        // Try to get the default input device - this should trigger the permission prompt
        if let Some(device) = host.default_input_device() {
            // Try to get the config - this might also trigger the prompt
            if device.default_input_config().is_ok() {
                return true;
            }
        }

        // As a fallback, try to build a stream briefly
        // This more reliably triggers the permission dialog
        if let Some(device) = host.default_input_device() {
            if let Ok(config) = device.default_input_config() {
                use cpal::traits::StreamTrait;
                let stream = device.build_input_stream(
                    &config.into(),
                    |_data: &[f32], _: &cpal::InputCallbackInfo| {},
                    |err| eprintln!("Stream error: {}", err),
                    None,
                );

                if let Ok(stream) = stream {
                    // Start briefly and stop to ensure dialog is triggered
                    let _ = stream.play();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    drop(stream);
                    return true;
                }
            }
        }

        false
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Open System Settings to the Accessibility pane
pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|e| format!("Failed to open System Settings: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

/// Check if onboarding has been completed
pub fn is_onboarding_complete() -> bool {
    let config_dir = dirs::config_dir().unwrap_or_default();
    let onboarding_file = config_dir.join("murmur").join("onboarding_complete");
    onboarding_file.exists()
}

/// Mark onboarding as complete
pub fn mark_onboarding_complete() -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("murmur");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let onboarding_file = config_dir.join("onboarding_complete");
    std::fs::write(&onboarding_file, "1")
        .map_err(|e| format!("Failed to write onboarding file: {}", e))?;

    Ok(())
}

/// Get selected microphone device ID
pub fn get_selected_microphone() -> Option<String> {
    let config_dir = dirs::config_dir()?.join("murmur");
    let mic_file = config_dir.join("selected_microphone");

    if mic_file.exists() {
        std::fs::read_to_string(mic_file).ok()
    } else {
        None
    }
}

/// Set selected microphone device ID
pub fn set_selected_microphone(device_id: &str) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("murmur");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let mic_file = config_dir.join("selected_microphone");
    std::fs::write(&mic_file, device_id)
        .map_err(|e| format!("Failed to write microphone selection: {}", e))?;

    Ok(())
}
