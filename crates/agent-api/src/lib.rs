//! Local HTTP REST API for AI Agents to view and control the desktop.

pub mod auth;
pub mod error;
pub mod routes;
pub mod server;

pub use error::ApiError;
pub use server::{start_agent_api, AppState};
