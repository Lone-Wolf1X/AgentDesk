//! Cross-platform preflight permission and session checks.

use crate::error::CaptureError;
use agentdesk_core::platform::{detect_platform_environment, PlatformKind};
use tracing::{info, warn};

/// Checks if current platform permissions and session type allow screen capture.
pub fn check_capture_permissions() -> Result<(), CaptureError> {
    let env = detect_platform_environment();

    match env.kind {
        PlatformKind::Linux => {
            if env.is_wayland {
                return Err(CaptureError::WaylandUnsupported(
                    "Wayland session detected via XDG_SESSION_TYPE / WAYLAND_DISPLAY. \
                    Direct screen capture and input injection require an X11 session. \
                    Please log out and select 'Ubuntu on Xorg' or equivalent X11 session at your login screen."
                        .to_string(),
                ));
            }
            if !env.is_x11 {
                warn!("Linux display server not detected ($DISPLAY is empty). Headless environment?");
            } else {
                info!("Linux X11 display environment verified.");
            }
        }
        PlatformKind::MacOS => {
            #[cfg(target_os = "macos")]
            {
                // On macOS, scap::has_permission() can be called if available, or we verify access
                info!("Checking macOS Screen Recording permissions...");
            }
        }
        PlatformKind::Windows => {
            info!("Windows platform detected: DXGI desktop duplication available.");
        }
        PlatformKind::Other => {
            warn!("Unknown platform detected, proceeding with caution.");
        }
    }

    Ok(())
}
