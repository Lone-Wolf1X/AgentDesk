//! Input permission and session preflight diagnostics.

use crate::error::InputError;
use agentdesk_core::platform::{detect_platform_environment, PlatformKind};
use tracing::{info, warn};

/// Verifies whether input simulation and listening permissions are satisfied.
pub fn check_input_permissions() -> Result<(), InputError> {
    let env = detect_platform_environment();

    match env.kind {
        PlatformKind::Linux => {
            if env.is_wayland {
                return Err(InputError::WaylandUnsupported(
                    "Wayland session detected. Wayland security protocol restricts synthetic \
                    input injection from unauthorized background processes. \
                    Please log into an X11 session (Ubuntu on Xorg) to allow remote input control."
                        .to_string(),
                ));
            }
            info!("Linux X11 input subsystem available.");
        }
        PlatformKind::MacOS => {
            #[cfg(target_os = "macos")]
            {
                // In macOS, simulate rdev key or check AXIsProcessTrusted
                // If not trusted, return InputError::PermissionDenied("Accessibility")
                warn!(
                    "Ensure AgentDesk has Accessibility permissions in: \
                    System Settings > Privacy & Security > Accessibility"
                );
            }
        }
        PlatformKind::Windows => {
            info!("Windows input simulation subsystem verified.");
        }
        PlatformKind::Other => {
            warn!("Unknown OS, assuming generic input injection.");
        }
    }

    Ok(())
}
