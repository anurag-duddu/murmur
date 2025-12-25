//! Platform-specific abstractions.
//!
//! This module provides platform-specific implementations for:
//! - Getting selected text (via Accessibility API on macOS)
//! - Detecting active application
//! - Replacing selected text

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub use macos::selection;

#[cfg(target_os = "linux")]
pub use linux::selection;
