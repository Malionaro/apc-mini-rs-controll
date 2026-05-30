use crate::actions::hotkeys::trigger_hotkey;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::process::Command;

pub fn trigger_media(k: &str) {
    let k_string = k.to_string();
    std::thread::spawn(move || {
        if let Ok(mut e) = Enigo::new(&Settings::default()) {
            let key = match k_string.as_str() {
                "play_pause" => Key::MediaPlayPause,
                "next" => Key::MediaNextTrack,
                "prev" => Key::MediaPrevTrack,
                "vol_up" => Key::VolumeUp,
                "vol_down" => Key::VolumeDown,
                "mute" => Key::VolumeMute,
                _ => return,
            };
            let _ = e.key(key, Direction::Click);
        }
    });
}

pub fn trigger_system(c: &str) {
    match c {
        "Lock" => {
            let _ = Command::new("rundll32.exe")
                .args(&["user32.dll,LockWorkStation"])
                .spawn();
        }
        "Shutdown" => {
            let _ = Command::new("shutdown").args(&["/s", "/t", "0"]).spawn();
        }
        "Screenshot" => {
            trigger_hotkey(&["win".to_string(), "shift".to_string(), "s".to_string()]);
        }
        _ => {}
    }
}

pub fn trigger_mouse(action: &str, target: &str) {
    let action_str = action.to_string();
    let target_str = target.to_string();
    std::thread::spawn(move || {
        use enigo::{Axis, Button, Coordinate, Direction, Enigo, Mouse, Settings};
        if let Ok(mut e) = Enigo::new(&Settings::default()) {
            match action_str.as_str() {
                "mouse_click" => {
                    let button = match target_str.as_str() {
                        "Right" => Button::Right,
                        "Middle" => Button::Middle,
                        _ => Button::Left,
                    };
                    if target_str == "DoubleLeft" {
                        let _ = e.button(Button::Left, Direction::Click);
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        let _ = e.button(Button::Left, Direction::Click);
                    } else {
                        let _ = e.button(button, Direction::Click);
                    }
                }
                "mouse_move" => {
                    let parts: Vec<&str> = target_str.split(',').collect();
                    if parts.len() == 2 {
                        let x = parts[0].trim().parse::<i32>().unwrap_or(0);
                        let y = parts[1].trim().parse::<i32>().unwrap_or(0);
                        let _ = e.move_mouse(x, y, Coordinate::Rel);
                    }
                }
                "mouse_scroll" => {
                    let scroll_amount = target_str.trim().parse::<i32>().unwrap_or(0);
                    let _ = e.scroll(scroll_amount, Axis::Vertical);
                }
                _ => {}
            }
        }
    });
}
