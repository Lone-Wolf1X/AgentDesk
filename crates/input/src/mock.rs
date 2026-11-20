//! Mock injector and listener for automated test suites.

use crate::combo::parse_key_combo;
use crate::error::InputError;
use crate::traits::{InputInjector, InputListener};
use agentdesk_core::{InputEvent, Key, MouseButton};
use std::sync::{Arc, Mutex};

/// In-memory mock input injector that records simulated operations.
#[derive(Clone, Default)]
pub struct MockInjector {
    pub history: Arc<Mutex<Vec<InputEvent>>>,
}

impl MockInjector {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn events(&self) -> Vec<InputEvent> {
        self.history.lock().unwrap().clone()
    }
}

impl InputInjector for MockInjector {
    fn move_mouse(&mut self, x: f64, y: f64) -> Result<(), InputError> {
        self.history
            .lock()
            .unwrap()
            .push(InputEvent::MouseMove { x, y });
        Ok(())
    }

    fn click(&mut self, button: MouseButton, down: bool) -> Result<(), InputError> {
        let event = if down {
            InputEvent::MouseDown { button, x: 0.0, y: 0.0 }
        } else {
            InputEvent::MouseUp { button, x: 0.0, y: 0.0 }
        };
        self.history.lock().unwrap().push(event);
        Ok(())
    }

    fn key_press(&mut self, key: Key, down: bool) -> Result<(), InputError> {
        let event = if down {
            InputEvent::KeyDown { key }
        } else {
            InputEvent::KeyUp { key }
        };
        self.history.lock().unwrap().push(event);
        Ok(())
    }

    fn type_text(&mut self, text: &str) -> Result<(), InputError> {
        self.history.lock().unwrap().push(InputEvent::TextInput {
            text: text.to_string(),
        });
        Ok(())
    }

    fn mouse_scroll(&mut self, delta_x: i64, delta_y: i64) -> Result<(), InputError> {
        self.history
            .lock()
            .unwrap()
            .push(InputEvent::MouseWheel { delta_x, delta_y });
        Ok(())
    }

    fn simulate_combo(&mut self, combo_str: &str) -> Result<(), InputError> {
        let combo = parse_key_combo(combo_str)?;
        let mut history = self.history.lock().unwrap();
        for m in combo.modifiers {
            history.push(InputEvent::KeyDown { key: m.clone() });
            history.push(InputEvent::KeyUp { key: m });
        }
        history.push(InputEvent::KeyDown { key: combo.key.clone() });
        history.push(InputEvent::KeyUp { key: combo.key });
        Ok(())
    }
}

/// Mock input listener for test verification.
#[derive(Default)]
pub struct MockListener;

impl MockListener {
    pub fn new() -> Self {
        Self
    }
}

impl InputListener for MockListener {
    fn start_listening<F>(&self, _callback: F) -> Result<(), InputError>
    where
        F: Fn(InputEvent) + Send + 'static,
    {
        Ok(())
    }
}
