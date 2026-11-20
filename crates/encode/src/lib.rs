//! Video encoding and decoding library for AgentDesk.

pub mod error;
pub mod hardware;
pub mod mock;
pub mod openh264_decoder;
pub mod openh264_encoder;
pub mod traits;
pub mod yuv;

pub use error::EncodeError;
pub use hardware::{HardwareEncoder, HardwareType};
pub use mock::{MockDecoder, MockEncoder};
pub use openh264_decoder::OpenH264Decoder;
pub use openh264_encoder::OpenH264Encoder;
pub use traits::{EncodedPacket, VideoDecoder, VideoEncoder};
