//! Platform environment detection and preflight check helpers.

use serde::{Deserialize, Serialize};

/// Operating system kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformKind {
    Linux,
    MacOS,
    Windows,
    Other,
}

/// Information about the current runtime platform environment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformEnvironment {
    pub kind: PlatformKind,
    pub os_name: String,
    pub session_type: Option<String>,
    pub is_wayland: bool,
    pub is_x11: bool,
}

/// Detects current platform environment (X11 vs Wayland, macOS, Windows).
pub fn detect_platform_environment() -> PlatformEnvironment {
    let os_name = std::env::consts::OS.to_string();

    #[cfg(target_os = "linux")]
    {
        let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default().to_lowercase();
        let wayland_display = std::env::var("WAYLAND_DISPLAY").is_ok();
        let is_wayland = session.contains("wayland") || wayland_display;
        let is_x11 = session.contains("x11") || std::env::var("DISPLAY").is_ok();

        PlatformEnvironment {
            kind: PlatformKind::Linux,
            os_name,
            session_type: if !session.is_empty() { Some(session) } else { None },
            is_wayland,
            is_x11,
        }
    }

    #[cfg(target_os = "macos")]
    {
        PlatformEnvironment {
            kind: PlatformKind::MacOS,
            os_name,
            session_type: None,
            is_wayland: false,
            is_x11: false,
        }
    }

    #[cfg(target_os = "windows")]
    {
        PlatformEnvironment {
            kind: PlatformKind::Windows,
            os_name,
            session_type: Some("windows-desktop".to_string()),
            is_wayland: false,
            is_x11: false,
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        PlatformEnvironment {
            kind: PlatformKind::Other,
            os_name,
            session_type: None,
            is_wayland: false,
            is_x11: false,
        }
    }
}
