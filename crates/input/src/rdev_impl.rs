//! Production input injector and listener implementation using `rdev`.

use crate::combo::parse_key_combo;
use crate::error::InputError;
use crate::preflight::check_input_permissions;
use crate::traits::{InputInjector, InputListener};
use agentdesk_core::{InputEvent, Key, MouseButton};
use rdev::{simulate, EventType};
use std::thread;
use std::time::Duration;
use tracing::{debug, error};

/// Production Input Injector using `rdev`.
pub struct RdevInjector;

impl RdevInjector {
    /// Initializes injector after verifying platform permissions.
    pub fn new() -> Result<Self, InputError> {
        check_input_permissions()?;
        Ok(Self)
    }
}

impl InputInjector for RdevInjector {
    fn move_mouse(&mut self, x: f64, y: f64) -> Result<(), InputError> {
        simulate(&EventType::MouseMove { x, y })
            .map_err(|e| InputError::SimulationFailed(format!("MouseMove ({x}, {y}) failed: {e:?}")))
    }

    fn click(&mut self, button: MouseButton, down: bool) -> Result<(), InputError> {
        let rdev_btn = map_mouse_button(button);
        let event_type = if down {
            EventType::ButtonPress(rdev_btn)
        } else {
            EventType::ButtonRelease(rdev_btn)
        };

        simulate(&event_type)
            .map_err(|e| InputError::SimulationFailed(format!("Click {button:?} (down={down}) failed: {e:?}")))
    }

    fn key_press(&mut self, key: Key, down: bool) -> Result<(), InputError> {
        let rdev_key = map_key(&key);
        let event_type = if down {
            EventType::KeyPress(rdev_key)
        } else {
            EventType::KeyRelease(rdev_key)
        };

        simulate(&event_type)
            .map_err(|e| InputError::SimulationFailed(format!("Key {key:?} (down={down}) failed: {e:?}")))
    }

    fn type_text(&mut self, text: &str) -> Result<(), InputError> {
        for ch in text.chars() {
            // For ASCII and standard characters, type via keys or virtual events
            if let Some((key, needs_shift)) = char_to_key(ch) {
                if needs_shift {
                    self.key_press(Key::ShiftLeft, true)?;
                    thread::sleep(Duration::from_millis(5));
                }

                self.key_press(key.clone(), true)?;
                thread::sleep(Duration::from_millis(10));
                self.key_press(key, false)?;

                if needs_shift {
                    thread::sleep(Duration::from_millis(5));
                    self.key_press(Key::ShiftLeft, false)?;
                }
            } else {
                debug!("Skipping untypable character: {ch:?}");
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn mouse_scroll(&mut self, delta_x: i64, delta_y: i64) -> Result<(), InputError> {
        simulate(&EventType::Wheel {
            delta_x,
            delta_y,
        })
        .map_err(|e| InputError::SimulationFailed(format!("MouseWheel ({delta_x}, {delta_y}) failed: {e:?}")))
    }

    fn simulate_combo(&mut self, combo_str: &str) -> Result<(), InputError> {
        let combo = parse_key_combo(combo_str)?;

        // Press modifiers in order
        for modifier in &combo.modifiers {
            self.key_press(modifier.clone(), true)?;
            thread::sleep(Duration::from_millis(15));
        }

        // Tap the target key
        self.key_press(combo.key.clone(), true)?;
        thread::sleep(Duration::from_millis(25));
        self.key_press(combo.key, false)?;
        thread::sleep(Duration::from_millis(15));

        // Release modifiers in reverse order
        for modifier in combo.modifiers.into_iter().rev() {
            self.key_press(modifier, false)?;
            thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }
}

/// Production Input Listener using `rdev::listen`.
pub struct RdevListener;

impl RdevListener {
    pub fn new() -> Self {
        Self
    }
}

impl InputListener for RdevListener {
    fn start_listening<F>(&self, callback: F) -> Result<(), InputError>
    where
        F: Fn(InputEvent) + Send + 'static,
    {
        thread::spawn(move || {
            if let Err(e) = rdev::listen(move |event| {
                if let Some(agent_event) = rdev_event_to_agent_event(event.event_type) {
                    callback(agent_event);
                }
            }) {
                error!("rdev::listen terminated with error: {e:?}");
            }
        });

        Ok(())
    }
}

fn map_mouse_button(btn: MouseButton) -> rdev::Button {
    match btn {
        MouseButton::Left => rdev::Button::Left,
        MouseButton::Right => rdev::Button::Right,
        MouseButton::Middle => rdev::Button::Middle,
        MouseButton::Other(n) => rdev::Button::Unknown(n),
    }
}

fn map_key(key: &Key) -> rdev::Key {
    match key {
        Key::KeyA => rdev::Key::KeyA,
        Key::KeyB => rdev::Key::KeyB,
        Key::KeyC => rdev::Key::KeyC,
        Key::KeyD => rdev::Key::KeyD,
        Key::KeyE => rdev::Key::KeyE,
        Key::KeyF => rdev::Key::KeyF,
        Key::KeyG => rdev::Key::KeyG,
        Key::KeyH => rdev::Key::KeyH,
        Key::KeyI => rdev::Key::KeyI,
        Key::KeyJ => rdev::Key::KeyJ,
        Key::KeyK => rdev::Key::KeyK,
        Key::KeyL => rdev::Key::KeyL,
        Key::KeyM => rdev::Key::KeyM,
        Key::KeyN => rdev::Key::KeyN,
        Key::KeyO => rdev::Key::KeyO,
        Key::KeyP => rdev::Key::KeyP,
        Key::KeyQ => rdev::Key::KeyQ,
        Key::KeyR => rdev::Key::KeyR,
        Key::KeyS => rdev::Key::KeyS,
        Key::KeyT => rdev::Key::KeyT,
        Key::KeyU => rdev::Key::KeyU,
        Key::KeyV => rdev::Key::KeyV,
        Key::KeyW => rdev::Key::KeyW,
        Key::KeyX => rdev::Key::KeyX,
        Key::KeyY => rdev::Key::KeyY,
        Key::KeyZ => rdev::Key::KeyZ,
        Key::Num0 => rdev::Key::Num0,
        Key::Num1 => rdev::Key::Num1,
        Key::Num2 => rdev::Key::Num2,
        Key::Num3 => rdev::Key::Num3,
        Key::Num4 => rdev::Key::Num4,
        Key::Num5 => rdev::Key::Num5,
        Key::Num6 => rdev::Key::Num6,
        Key::Num7 => rdev::Key::Num7,
        Key::Num8 => rdev::Key::Num8,
        Key::Num9 => rdev::Key::Num9,
        Key::Return => rdev::Key::Return,
        Key::Escape => rdev::Key::Escape,
        Key::Backspace => rdev::Key::Backspace,
        Key::Tab => rdev::Key::Tab,
        Key::Space => rdev::Key::Space,
        Key::Minus => rdev::Key::Minus,
        Key::Equal => rdev::Key::Equal,
        Key::LeftBracket => rdev::Key::LeftBracket,
        Key::RightBracket => rdev::Key::RightBracket,
        Key::Backslash => rdev::Key::BackSlash,
        Key::SemiColon => rdev::Key::SemiColon,
        Key::Quote => rdev::Key::Quote,
        Key::Grave => rdev::Key::BackQuote,
        Key::Comma => rdev::Key::Comma,
        Key::Dot => rdev::Key::Dot,
        Key::Slash => rdev::Key::Slash,
        Key::CapsLock => rdev::Key::CapsLock,
        Key::F1 => rdev::Key::F1,
        Key::F2 => rdev::Key::F2,
        Key::F3 => rdev::Key::F3,
        Key::F4 => rdev::Key::F4,
        Key::F5 => rdev::Key::F5,
        Key::F6 => rdev::Key::F6,
        Key::F7 => rdev::Key::F7,
        Key::F8 => rdev::Key::F8,
        Key::F9 => rdev::Key::F9,
        Key::F10 => rdev::Key::F10,
        Key::F11 => rdev::Key::F11,
        Key::F12 => rdev::Key::F12,
        Key::PrintScreen => rdev::Key::PrintScreen,
        Key::ScrollLock => rdev::Key::ScrollLock,
        Key::Pause => rdev::Key::Pause,
        Key::Insert => rdev::Key::Insert,
        Key::Home => rdev::Key::Home,
        Key::PageUp => rdev::Key::PageUp,
        Key::Delete => rdev::Key::Delete,
        Key::End => rdev::Key::End,
        Key::PageDown => rdev::Key::PageDown,
        Key::RightArrow => rdev::Key::RightArrow,
        Key::LeftArrow => rdev::Key::LeftArrow,
        Key::DownArrow => rdev::Key::DownArrow,
        Key::UpArrow => rdev::Key::UpArrow,
        Key::ShiftLeft => rdev::Key::ShiftLeft,
        Key::ShiftRight => rdev::Key::ShiftRight,
        Key::ControlLeft => rdev::Key::ControlLeft,
        Key::ControlRight => rdev::Key::ControlRight,
        Key::Alt => rdev::Key::Alt,
        Key::AltGraph => rdev::Key::AltGr,
        Key::MetaLeft => rdev::Key::MetaLeft,
        Key::MetaRight => rdev::Key::MetaRight,
        Key::Unknown(code) => rdev::Key::Unknown(*code),
    }
}

fn rdev_key_to_core(key: rdev::Key) -> Key {
    match key {
        rdev::Key::KeyA => Key::KeyA,
        rdev::Key::KeyB => Key::KeyB,
        rdev::Key::KeyC => Key::KeyC,
        rdev::Key::KeyD => Key::KeyD,
        rdev::Key::KeyE => Key::KeyE,
        rdev::Key::KeyF => Key::KeyF,
        rdev::Key::KeyG => Key::KeyG,
        rdev::Key::KeyH => Key::KeyH,
        rdev::Key::KeyI => Key::KeyI,
        rdev::Key::KeyJ => Key::KeyJ,
        rdev::Key::KeyK => Key::KeyK,
        rdev::Key::KeyL => Key::KeyL,
        rdev::Key::KeyM => Key::KeyM,
        rdev::Key::KeyN => Key::KeyN,
        rdev::Key::KeyO => Key::KeyO,
        rdev::Key::KeyP => Key::KeyP,
        rdev::Key::KeyQ => Key::KeyQ,
        rdev::Key::KeyR => Key::KeyR,
        rdev::Key::KeyS => Key::KeyS,
        rdev::Key::KeyT => Key::KeyT,
        rdev::Key::KeyU => Key::KeyU,
        rdev::Key::KeyV => Key::KeyV,
        rdev::Key::KeyW => Key::KeyW,
        rdev::Key::KeyX => Key::KeyX,
        rdev::Key::KeyY => Key::KeyY,
        rdev::Key::KeyZ => Key::KeyZ,
        rdev::Key::Num0 => Key::Num0,
        rdev::Key::Num1 => Key::Num1,
        rdev::Key::Num2 => Key::Num2,
        rdev::Key::Num3 => Key::Num3,
        rdev::Key::Num4 => Key::Num4,
        rdev::Key::Num5 => Key::Num5,
        rdev::Key::Num6 => Key::Num6,
        rdev::Key::Num7 => Key::Num7,
        rdev::Key::Num8 => Key::Num8,
        rdev::Key::Num9 => Key::Num9,
        rdev::Key::Return => Key::Return,
        rdev::Key::Escape => Key::Escape,
        rdev::Key::Backspace => Key::Backspace,
        rdev::Key::Tab => Key::Tab,
        rdev::Key::Space => Key::Space,
        rdev::Key::Minus => Key::Minus,
        rdev::Key::Equal => Key::Equal,
        rdev::Key::LeftBracket => Key::LeftBracket,
        rdev::Key::RightBracket => Key::RightBracket,
        rdev::Key::BackSlash => Key::Backslash,
        rdev::Key::SemiColon => Key::SemiColon,
        rdev::Key::Quote => Key::Quote,
        rdev::Key::BackQuote => Key::Grave,
        rdev::Key::Comma => Key::Comma,
        rdev::Key::Dot => Key::Dot,
        rdev::Key::Slash => Key::Slash,
        rdev::Key::CapsLock => Key::CapsLock,
        rdev::Key::F1 => Key::F1,
        rdev::Key::F2 => Key::F2,
        rdev::Key::F3 => Key::F3,
        rdev::Key::F4 => Key::F4,
        rdev::Key::F5 => Key::F5,
        rdev::Key::F6 => Key::F6,
        rdev::Key::F7 => Key::F7,
        rdev::Key::F8 => Key::F8,
        rdev::Key::F9 => Key::F9,
        rdev::Key::F10 => Key::F10,
        rdev::Key::F11 => Key::F11,
        rdev::Key::F12 => Key::F12,
        rdev::Key::PrintScreen => Key::PrintScreen,
        rdev::Key::ScrollLock => Key::ScrollLock,
        rdev::Key::Pause => Key::Pause,
        rdev::Key::Insert => Key::Insert,
        rdev::Key::Home => Key::Home,
        rdev::Key::PageUp => Key::PageUp,
        rdev::Key::Delete => Key::Delete,
        rdev::Key::End => Key::End,
        rdev::Key::PageDown => Key::PageDown,
        rdev::Key::RightArrow => Key::RightArrow,
        rdev::Key::LeftArrow => Key::LeftArrow,
        rdev::Key::DownArrow => Key::DownArrow,
        rdev::Key::UpArrow => Key::UpArrow,
        rdev::Key::ShiftLeft => Key::ShiftLeft,
        rdev::Key::ShiftRight => Key::ShiftRight,
        rdev::Key::ControlLeft => Key::ControlLeft,
        rdev::Key::ControlRight => Key::ControlRight,
        rdev::Key::Alt => Key::Alt,
        rdev::Key::AltGr => Key::AltGraph,
        rdev::Key::MetaLeft => Key::MetaLeft,
        rdev::Key::MetaRight => Key::MetaRight,
        rdev::Key::Unknown(u) => Key::Unknown(u),
        _ => Key::Unknown(0),
    }
}

fn rdev_event_to_agent_event(event: EventType) -> Option<InputEvent> {
    match event {
        EventType::MouseMove { x, y } => Some(InputEvent::MouseMove { x, y }),
        EventType::ButtonPress(btn) => Some(InputEvent::MouseDown {
            button: rdev_btn_to_core(btn),
            x: 0.0,
            y: 0.0,
        }),
        EventType::ButtonRelease(btn) => Some(InputEvent::MouseUp {
            button: rdev_btn_to_core(btn),
            x: 0.0,
            y: 0.0,
        }),
        EventType::Wheel { delta_x, delta_y } => Some(InputEvent::MouseWheel { delta_x, delta_y }),
        EventType::KeyPress(key) => Some(InputEvent::KeyDown {
            key: rdev_key_to_core(key),
        }),
        EventType::KeyRelease(key) => Some(InputEvent::KeyUp {
            key: rdev_key_to_core(key),
        }),
    }
}

fn rdev_btn_to_core(btn: rdev::Button) -> MouseButton {
    match btn {
        rdev::Button::Left => MouseButton::Left,
        rdev::Button::Right => MouseButton::Right,
        rdev::Button::Middle => MouseButton::Middle,
        rdev::Button::Unknown(u) => MouseButton::Other(u),
    }
}

fn char_to_key(c: char) -> Option<(Key, bool)> {
    match c {
        'a'..='z' => {
            let offset = c as u8 - b'a';
            let key = match offset {
                0 => Key::KeyA,
                1 => Key::KeyB,
                2 => Key::KeyC,
                3 => Key::KeyD,
                4 => Key::KeyE,
                5 => Key::KeyF,
                6 => Key::KeyG,
                7 => Key::KeyH,
                8 => Key::KeyI,
                9 => Key::KeyJ,
                10 => Key::KeyK,
                11 => Key::KeyL,
                12 => Key::KeyM,
                13 => Key::KeyN,
                14 => Key::KeyO,
                15 => Key::KeyP,
                16 => Key::KeyQ,
                17 => Key::KeyR,
                18 => Key::KeyS,
                19 => Key::KeyT,
                20 => Key::KeyU,
                21 => Key::KeyV,
                22 => Key::KeyW,
                23 => Key::KeyX,
                24 => Key::KeyY,
                25 => Key::KeyZ,
                _ => return None,
            };
            Some((key, false))
        }
        'A'..='Z' => {
            let offset = c as u8 - b'A';
            let key = match offset {
                0 => Key::KeyA,
                1 => Key::KeyB,
                2 => Key::KeyC,
                3 => Key::KeyD,
                4 => Key::KeyE,
                5 => Key::KeyF,
                6 => Key::KeyG,
                7 => Key::KeyH,
                8 => Key::KeyI,
                9 => Key::KeyJ,
                10 => Key::KeyK,
                11 => Key::KeyL,
                12 => Key::KeyM,
                13 => Key::KeyN,
                14 => Key::KeyO,
                15 => Key::KeyP,
                16 => Key::KeyQ,
                17 => Key::KeyR,
                18 => Key::KeyS,
                19 => Key::KeyT,
                20 => Key::KeyU,
                21 => Key::KeyV,
                22 => Key::KeyW,
                23 => Key::KeyX,
                24 => Key::KeyY,
                25 => Key::KeyZ,
                _ => return None,
            };
            Some((key, true))
        }
        '0' => Some((Key::Num0, false)),
        '1' => Some((Key::Num1, false)),
        '2' => Some((Key::Num2, false)),
        '3' => Some((Key::Num3, false)),
        '4' => Some((Key::Num4, false)),
        '5' => Some((Key::Num5, false)),
        '6' => Some((Key::Num6, false)),
        '7' => Some((Key::Num7, false)),
        '8' => Some((Key::Num8, false)),
        '9' => Some((Key::Num9, false)),
        ' ' => Some((Key::Space, false)),
        '\n' | '\r' => Some((Key::Return, false)),
        '\t' => Some((Key::Tab, false)),
        '.' => Some((Key::Dot, false)),
        ',' => Some((Key::Comma, false)),
        '-' => Some((Key::Minus, false)),
        '=' => Some((Key::Equal, false)),
        '/' => Some((Key::Slash, false)),
        _ => None,
    }
}
