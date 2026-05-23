pub mod hotkeys;
pub mod audio;
pub mod system;
pub mod faders;

use crate::config::Action;
use crate::midi::MidiState;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::process::Command;

use hotkeys::{parse_command_line, trigger_hotkey, trigger_text};
use audio::play_sound;
use system::{trigger_media, trigger_system};

pub fn execute_action(action: &Action, state: &Arc<MidiState>) {
    match action.action_type.as_str() {
        "app" => {
            if let Some(p) = &action.path {
                let parts = parse_command_line(p);
                if !parts.is_empty() {
                    let mut cmd = Command::new(&parts[0]);
                    if parts.len() > 1 {
                        cmd.args(&parts[1..]);
                    }
                    let _ = cmd.spawn();
                }
            }
        }
        "url" => {
            if let Some(u) = &action.url {
                let formatted_url = if u.starts_with("http://")
                    || u.starts_with("https://")
                    || u.starts_with("mailto:")
                    || u.starts_with("file://")
                {
                    u.clone()
                } else {
                    format!("https://{}", u)
                };
                let _ = open::that(formatted_url);
            }
        }
        "obs" => {
            if let Some(a) = &action.obs_action {
                let _ = state.obs.execute(a, action.obs_target.clone());
            }
        }
        "obs_vol" => {
            let _ = state.obs.execute("SetVolume", action.obs_target.clone());
        }
        "obs_toggle" => {
            let _ = state.obs.execute("ToggleSource", action.obs_target.clone());
        }
        "obs_filter" => {
            let _ = state.obs.execute("ToggleFilter", action.obs_target.clone());
        }
        "obs_visible" => {
            let _ = state.obs.execute("SetSourceVisible", action.obs_target.clone());
        }
        "obs_replay" => {
            let _ = state.obs.execute("ReplayBuffer", action.obs_target.clone());
        }
        "audio" => {
            if let Some(p) = &action.audio_path {
                let volume = action.audio_volume.unwrap_or(100.0) / 100.0;
                play_sound(p.clone(), volume);
            }
        }
        "hotkey" => {
            if let Some(keys) = &action.keys {
                trigger_hotkey(keys);
            }
        }
        "text" => {
            if let Some(t) = &action.text_content {
                trigger_text(t);
            }
        }
        "wait" => {
            if let Some(d) = action.delay_ms {
                thread::sleep(Duration::from_millis(d));
            }
        }
        "media" => {
            if let Some(k) = &action.media_key {
                trigger_media(k);
            }
        }
        "system" => {
            if let Some(c) = &action.system_command {
                trigger_system(c);
            }
        }
        "command" => {
            if let Some(c) = &action.system_command {
                let _ = Command::new("cmd").args(&["/C", c]).spawn();
            }
        }
        "navigation" | "page" => {
            if let Some(t) = &action.target_page {
                state.config.lock().unwrap().active_page = t.clone();
            }
        }
        "discord" => {
            trigger_hotkey(&[
                "ctrl".to_string(),
                "shift".to_string(),
                "m".to_string(),
            ]);
        }
        _ => {}
    }
}
