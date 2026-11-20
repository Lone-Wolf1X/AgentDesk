// AgentDesk — Free & Open Source Remote Desktop
// Copyright (c) 2025 Next Gen Innovations Private Limited
// Developer: Abhishek Kumar Paswan
// Website: https://nextgeninnovations.com.np/
// License: MIT — see LICENSE file
// Made in Nepal 🇳🇵

pub mod app_state;
pub mod host_panel;
pub mod theme;
pub mod unified_app;
pub mod viewer_panel;

pub use app_state::{AppState, IncomingRequest, SessionStats};
pub use host_panel::HostGuiState;
pub use unified_app::{AgentDeskApp, UiAction};
pub use viewer_panel::ViewerGuiState;
