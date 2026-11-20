//! Input injection and listening trait definitions.

use crate::error::InputError;
use agentdesk_core::{InputEvent, Key, MouseButton};

/// Trait for simulating mouse and keyboard inputs on the local host.
pub trait InputInjector: Send + Sync {
    /// Moves mouse cursor to absolute coordinate (x, y).
    fn move_mouse(&mut self, x: f64, y: f64) -> Result<(), InputError>;

    /// Presses or releases a mouse button at the current cursor position.
    fn click(&mut self, button: MouseButton, down: bool) -> Result<(), InputError>;

    /// Convenience click: press and immediately release.
    fn click_tap(&mut self, button: MouseButton) -> Result<(), InputError> {
        self.click(button, true)?;
        std::thread::sleep(std::time::Duration::from_millis(15));
        self.click(button, false)
    }

    /// Presses or releases a physical keyboard key.
    fn key_press(&mut self, key: Key, down: bool) -> Result<(), InputError>;

    /// Convenience key tap: press and immediately release.
    fn key_tap(&mut self, key: Key) -> Result<(), InputError> {
        self.key_press(key.clone(), true)?;
        std::thread::sleep(std::time::Duration::from_millis(15));
        self.key_press(key, false)
    }

    /// Types an arbitrary UTF-8 text string.
    fn type_text(&mut self, text: &str) -> Result<(), InputError>;

    /// Scrolls the mouse wheel by horizontal and vertical delta units.
    fn mouse_scroll(&mut self, delta_x: i64, delta_y: i64) -> Result<(), InputError>;

    /// Simulates a parsed key combination (e.g. Ctrl+Shift+T).
    fn simulate_combo(&mut self, combo_str: &str) -> Result<(), InputError>;
}

/// Trait for listening to physical user input on the local host.
pub trait InputListener: Send + Sync {
    /// Starts listening to physical events in background and emits them through callback.
    fn start_listening<F>(&self, callback: F) -> Result<(), InputError>
    where
        F: Fn(InputEvent) + Send + 'static;
}
