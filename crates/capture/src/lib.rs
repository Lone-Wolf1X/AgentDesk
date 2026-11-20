//! Screen capture abstraction and cross-platform capturers.

pub mod error;
pub mod mock;
pub mod preflight;
pub mod scap_capturer;
pub mod traits;

pub use error::CaptureError;
pub use mock::MockCapturer;
pub use preflight::check_capture_permissions;
pub use scap_capturer::ScapCapturer;
pub use traits::ScreenCapturer;
