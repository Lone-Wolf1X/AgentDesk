//! Core types representing video frames, screen geometry, input devices, and events.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Raw screen frame captured from the display.
#[derive(Clone, Debug)]
pub struct Frame {
    /// Width of the frame in pixels.
    pub width: u32,
    /// Height of the frame in pixels.
    pub height: u32,
    /// Row stride in bytes (bytes per row, e.g. width * 4 for 32-bit colors).
    pub stride: u32,
    /// Raw pixel data in BGRA or RGBA 8-bit format.
    pub bgra: Vec<u8>,
    /// Instant when the frame was captured.
    pub timestamp: Instant,
}

impl Frame {
    /// Creates a new Frame instance.
    pub fn new(width: u32, height: u32, stride: u32, bgra: Vec<u8>) -> Self {
        Self {
            width,
            height,
            stride,
            bgra,
            timestamp: Instant::now(),
        }
    }

    /// Converts BGRA byte slice to RGBA in-place or returning a new buffer.
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut rgba = self.bgra.clone();
        for chunk in rgba.chunks_exact_mut(4) {
            let b = chunk[0];
            let r = chunk[2];
            chunk[0] = r;
            chunk[2] = b;
        }
        rgba
    }
}

/// Screen display resolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

/// Metadata about an available display.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    pub scale_factor: u32,
}

/// A 2D rectangle specifying a crop region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Mouse button identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Supported physical and virtual keyboard keys.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Key {
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Return,
    Escape,
    Backspace,
    Tab,
    Space,
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    SemiColon,
    Quote,
    Grave,
    Comma,
    Dot,
    Slash,
    CapsLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    Home,
    PageUp,
    Delete,
    End,
    PageDown,
    RightArrow,
    LeftArrow,
    DownArrow,
    UpArrow,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    Alt,
    AltGraph,
    MetaLeft,
    MetaRight,
    Unknown(u32),
}

/// Key combination parsed from human-readable strings like "ctrl+shift+t".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyCombo {
    pub modifiers: Vec<Key>,
    pub key: Key,
}

/// High-level input event (mouse or keyboard).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum InputEvent {
    MouseMove { x: f64, y: f64 },
    MouseDown { button: MouseButton, x: f64, y: f64 },
    MouseUp { button: MouseButton, x: f64, y: f64 },
    MouseWheel { delta_x: i64, delta_y: i64 },
    KeyDown { key: Key },
    KeyUp { key: Key },
    TextInput { text: String },
}
