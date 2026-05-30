use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub fn parse_command_line(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
            }
            ' ' | '\t' if !in_double_quotes && !in_single_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

pub fn trigger_hotkey(keys: &[String]) {
    let keys_vec = keys.to_vec();
    std::thread::spawn(move || {
        if let Ok(mut e) = Enigo::new(&Settings::default()) {
            for k in &keys_vec {
                if let Some(pk) = parse_key(k) {
                    let _ = e.key(pk, Direction::Press);
                }
            }
            for k in keys_vec.iter().rev() {
                if let Some(pk) = parse_key(k) {
                    let _ = e.key(pk, Direction::Release);
                }
            }
        }
    });
}

pub fn trigger_text(t: &str) {
    let t_string = t.to_string();
    std::thread::spawn(move || {
        if let Ok(mut e) = Enigo::new(&Settings::default()) {
            let _ = e.text(&t_string);
        }
    });
}

pub fn parse_key(s: &str) -> Option<Key> {
    match s.to_lowercase().as_str() {
        "ctrl" | "control" => Some(Key::Control),
        "shift" => Some(Key::Shift),
        "alt" => Some(Key::Alt),
        "win" | "meta" => Some(Key::Meta),
        "enter" | "return" => Some(Key::Return),
        "space" => Some(Key::Space),
        "tab" => Some(Key::Tab),
        "backspace" => Some(Key::Backspace),
        "esc" | "escape" => Some(Key::Escape),
        "delete" | "del" => Some(Key::Delete),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" | "pgup" => Some(Key::PageUp),
        "pagedown" | "pgdn" => Some(Key::PageDown),
        "capslock" => Some(Key::CapsLock),
        "up" | "uparrow" => Some(Key::UpArrow),
        "down" | "downarrow" => Some(Key::DownArrow),
        "left" | "leftarrow" => Some(Key::LeftArrow),
        "right" | "rightarrow" => Some(Key::RightArrow),
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
        "f13" => Some(Key::F13),
        "f14" => Some(Key::F14),
        "f15" => Some(Key::F15),
        "f16" => Some(Key::F16),
        "f17" => Some(Key::F17),
        "f18" => Some(Key::F18),
        "f19" => Some(Key::F19),
        "f20" => Some(Key::F20),
        k if k.len() == 1 => Some(Key::Unicode(k.chars().next().unwrap())),
        _ => None,
    }
}
