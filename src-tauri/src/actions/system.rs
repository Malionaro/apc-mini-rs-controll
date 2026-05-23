use enigo::{Enigo, Key, Direction, Settings, Keyboard};
use std::process::Command;
use crate::actions::hotkeys::trigger_hotkey;

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
            let _ = Command::new("shutdown")
                .args(&["/s", "/t", "0"])
                .spawn();
        }
        "Screenshot" => {
            trigger_hotkey(&[
                "win".to_string(),
                "shift".to_string(),
                "s".to_string(),
            ]);
        }
        _ => {}
    }
}
