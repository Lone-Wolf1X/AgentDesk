//! Keyboard combination parser.
//!
//! Converts strings such as `"ctrl+shift+t"`, `"alt+f4"`, `"cmd+space"` into structured
//! modifiers and target key sequences.

use crate::error::InputError;
use agentdesk_core::{Key, KeyCombo};

/// Parses human-readable keyboard combo string into modifier keys and a target key.
///
/// Example: `"ctrl+shift+t"` -> modifiers: `[ControlLeft, ShiftLeft]`, target: `KeyT`.
pub fn parse_key_combo(input: &str) -> Result<KeyCombo, InputError> {
    let parts: Vec<&str> = input
        .split('+')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(InputError::InvalidKeyCombo("Empty key combo string".to_string()));
    }

    let mut modifiers = Vec::new();

    for &part in &parts[..parts.len() - 1] {
        let mod_key = match_modifier(part)
            .ok_or_else(|| InputError::InvalidKeyCombo(format!("Unknown modifier '{part}' in '{input}'")))?;
        modifiers.push(mod_key);
    }

    let target_str = parts.last().unwrap();
    let target_key = match_key(target_str)
        .ok_or_else(|| InputError::InvalidKeyCombo(format!("Unknown key '{target_str}' in '{input}'")))?;

    Ok(KeyCombo {
        modifiers,
        key: target_key,
    })
}

fn match_modifier(s: &str) -> Option<Key> {
    match s.to_lowercase().as_str() {
        "ctrl" | "control" => Some(Key::ControlLeft),
        "shift" => Some(Key::ShiftLeft),
        "alt" | "opt" | "option" => Some(Key::Alt),
        "super" | "win" | "windows" | "cmd" | "command" | "meta" => Some(Key::MetaLeft),
        _ => None,
    }
}

pub fn match_key(s: &str) -> Option<Key> {
    match s.to_lowercase().as_str() {
        // Modifiers if used directly as target key
        "ctrl" | "control" => Some(Key::ControlLeft),
        "shift" => Some(Key::ShiftLeft),
        "alt" | "opt" | "option" => Some(Key::Alt),
        "super" | "win" | "windows" | "cmd" | "command" | "meta" => Some(Key::MetaLeft),

        // Letters
        "a" => Some(Key::KeyA),
        "b" => Some(Key::KeyB),
        "c" => Some(Key::KeyC),
        "d" => Some(Key::KeyD),
        "e" => Some(Key::KeyE),
        "f" => Some(Key::KeyF),
        "g" => Some(Key::KeyG),
        "h" => Some(Key::KeyH),
        "i" => Some(Key::KeyI),
        "j" => Some(Key::KeyJ),
        "k" => Some(Key::KeyK),
        "l" => Some(Key::KeyL),
        "m" => Some(Key::KeyM),
        "n" => Some(Key::KeyN),
        "o" => Some(Key::KeyO),
        "p" => Some(Key::KeyP),
        "q" => Some(Key::KeyQ),
        "r" => Some(Key::KeyR),
        "s" => Some(Key::KeyS),
        "t" => Some(Key::KeyT),
        "u" => Some(Key::KeyU),
        "v" => Some(Key::KeyV),
        "w" => Some(Key::KeyW),
        "x" => Some(Key::KeyX),
        "y" => Some(Key::KeyY),
        "z" => Some(Key::KeyZ),

        // Numbers
        "0" => Some(Key::Num0),
        "1" => Some(Key::Num1),
        "2" => Some(Key::Num2),
        "3" => Some(Key::Num3),
        "4" => Some(Key::Num4),
        "5" => Some(Key::Num5),
        "6" => Some(Key::Num6),
        "7" => Some(Key::Num7),
        "8" => Some(Key::Num8),
        "9" => Some(Key::Num9),

        // Function keys
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),

        // Common special keys
        "enter" | "return" => Some(Key::Return),
        "esc" | "escape" => Some(Key::Escape),
        "backspace" => Some(Key::Backspace),
        "tab" => Some(Key::Tab),
        "space" => Some(Key::Space),
        "delete" | "del" => Some(Key::Delete),
        "insert" | "ins" => Some(Key::Insert),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" | "pgup" => Some(Key::PageUp),
        "pagedown" | "pgdn" => Some(Key::PageDown),
        "up" | "uparrow" => Some(Key::UpArrow),
        "down" | "downarrow" => Some(Key::DownArrow),
        "left" | "leftarrow" => Some(Key::LeftArrow),
        "right" | "rightarrow" => Some(Key::RightArrow),

        // Punctuation
        "-" | "minus" => Some(Key::Minus),
        "=" | "equal" => Some(Key::Equal),
        "[" => Some(Key::LeftBracket),
        "]" => Some(Key::RightBracket),
        "\\" => Some(Key::Backslash),
        ";" => Some(Key::SemiColon),
        "'" => Some(Key::Quote),
        "`" => Some(Key::Grave),
        "," => Some(Key::Comma),
        "." => Some(Key::Dot),
        "/" => Some(Key::Slash),

        _ => None,
    }
}
