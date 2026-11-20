//! Error types for input operations.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Wayland is not supported yet for input injection. Please log into an X11 session. Details: {0}")]
    WaylandUnsupported(String),

    #[error("Permission denied for {0}. Enable accessibility permissions.")]
    PermissionDenied(String),

    #[error("Input simulation failed: {0}")]
    SimulationFailed(String),

    #[error("Failed to start input listener: {0}")]
    ListenFailed(String),

    #[error("Invalid key combination: '{0}'")]
    InvalidKeyCombo(String),

    #[error("Internal error: {0}")]
    Other(String),
}
