//! Input simulation and listening library for AgentDesk.

pub mod combo;
pub mod error;
pub mod mock;
pub mod preflight;
pub mod rdev_impl;
pub mod traits;

pub use combo::parse_key_combo;
pub use error::InputError;
pub use mock::{MockInjector, MockListener};
pub use preflight::check_input_permissions;
pub use rdev_impl::{RdevInjector, RdevListener};
pub use traits::{InputInjector, InputListener};
