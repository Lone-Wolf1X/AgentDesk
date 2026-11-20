//! Hardware encoder trait abstractions and stubs.
//!
//! Provides extension points for future hardware acceleration:
//! - NVENC (NVIDIA GPUs on Windows/Linux)
//! - VideoToolbox (Apple Silicon / Intel on macOS)
//! - VAAPI (Intel / AMD on Linux)

use crate::error::EncodeError;
use crate::traits::VideoEncoder;

/// Identifiers for supported hardware encoding pipelines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareType {
    Nvenc,
    VideoToolbox,
    Vaapi,
}

/// Abstract trait for hardware-accelerated video encoders.
pub trait HardwareEncoder: VideoEncoder {
    /// Returns the hardware acceleration type.
    fn hardware_type(&self) -> HardwareType;

    /// Checks if the underlying GPU hardware is currently available.
    fn is_available() -> bool
    where
        Self: Sized;
}

/// NVENC hardware encoder stub.
pub struct NvencEncoderStub;

impl NvencEncoderStub {
    pub fn new(_width: u32, _height: u32) -> Result<Self, EncodeError> {
        #[cfg(feature = "hardware-encode-nvenc")]
        {
            todo!("NVENC encoder implementation for Phase 2")
        }
        #[cfg(not(feature = "hardware-encode-nvenc"))]
        {
            Err(EncodeError::HardwareNotSupported("NVENC".to_string()))
        }
    }
}

/// Apple VideoToolbox hardware encoder stub.
pub struct VideoToolboxEncoderStub;

impl VideoToolboxEncoderStub {
    pub fn new(_width: u32, _height: u32) -> Result<Self, EncodeError> {
        #[cfg(feature = "hardware-encode-videotoolbox")]
        {
            todo!("VideoToolbox encoder implementation for Phase 2")
        }
        #[cfg(not(feature = "hardware-encode-videotoolbox"))]
        {
            Err(EncodeError::HardwareNotSupported("VideoToolbox".to_string()))
        }
    }
}

/// Linux VAAPI hardware encoder stub.
pub struct VaapiEncoderStub;

impl VaapiEncoderStub {
    pub fn new(_width: u32, _height: u32) -> Result<Self, EncodeError> {
        #[cfg(feature = "hardware-encode-vaapi")]
        {
            todo!("VAAPI encoder implementation for Phase 2")
        }
        #[cfg(not(feature = "hardware-encode-vaapi"))]
        {
            Err(EncodeError::HardwareNotSupported("VAAPI".to_string()))
        }
    }
}
