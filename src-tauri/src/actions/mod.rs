pub mod audio;
pub mod faders;
pub mod hotkeys;
pub mod system;

use crate::config::Action;
use crate::midi::MidiState;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use audio::play_sound;
use hotkeys::{parse_command_line, trigger_hotkey, trigger_text};
use system::{trigger_media, trigger_system};

fn emit_active_page(state: &Arc<MidiState>, page_name: &str) {
    if let Some(handle) = &*state.app_handle.lock().unwrap() {
        let _ = handle.emit("active-page-changed", page_name.to_string());
    }
}

fn switch_to_page(state: &Arc<MidiState>, target_page: &str, push_history: bool) {
    let mut next_page = None;
    let mut previous_page = None;

    {
        let mut config = state.config.lock().unwrap();
        if config.active_page == target_page {
            return;
        }

        if config.pages.iter().any(|page| page.name == target_page) {
            previous_page = Some(config.active_page.clone());
            config.active_page = target_page.to_string();
            next_page = Some(config.active_page.clone());
        }
    }

    if let Some(page) = next_page {
        if push_history {
            if let Some(previous) = previous_page {
                state.page_history.lock().unwrap().push(previous);
            }
        }
        crate::midi::add_log(state, format!("Seite gewechselt: {}", page));
        emit_active_page(state, &page);
    } else {
        crate::midi::add_log(state, format!("Seite nicht gefunden: {}", target_page));
    }
}

fn open_app_or_path(path: &str, state: &Arc<MidiState>) {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return;
    }

    let parts = parse_command_line(trimmed);
    if !parts.is_empty() {
        let mut cmd = Command::new(&parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        if cmd.spawn().is_ok() {
            return;
        }
    }

    if let Err(error) = open::that(trimmed) {
        crate::midi::add_log(
            state,
            format!("Programm/Pfad konnte nicht geöffnet werden: {}", error),
        );
    }
}

fn normalize_obs_action(action: &str) -> &str {
    match action {
        "scene" => "SetScene",
        "preview_scene" => "SetPreviewScene",
        "mute" => "ToggleMute",
        "source" => "ToggleSource",
        "filter" => "ToggleFilter",
        "visible" => "SetSourceVisible",
        "stream" => "StartStopStream",
        "record" => "StartStopRecord",
        "replay" => "ReplayBuffer",
        other => other,
    }
}

pub fn execute_action(action: &Action, state: &Arc<MidiState>) {
    match action.action_type.as_str() {
        "app" => {
            if let Some(p) = &action.path {
                open_app_or_path(p, state);
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
                let _ = state
                    .obs
                    .execute(normalize_obs_action(a), action.obs_target.clone());
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
            let _ = state
                .obs
                .execute("SetSourceVisible", action.obs_target.clone());
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
        "audio_panic" | "panic" => {
            audio::panic_stop_all_sounds();
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
        "mouse_click" | "mouse_move" | "mouse_scroll" => {
            if let Some(c) = &action.system_command {
                system::trigger_mouse(action.action_type.as_str(), c);
            }
        }
        "command" => {
            if let Some(c) = &action.system_command {
                let _ = Command::new("cmd").args(&["/C", c]).spawn();
            }
        }
        "navigation" | "page" => {
            if let Some(t) = &action.target_page {
                switch_to_page(state, t, true);
            }
        }
        "page_back" => {
            let prev_page = { state.page_history.lock().unwrap().pop() };
            if let Some(p) = prev_page {
                switch_to_page(state, &p, false);
            } else {
                switch_to_page(state, "Main", false);
            }
        }
        "webhook" => {
            if let Some(u) = &action.webhook_url {
                let client = reqwest::Client::new();
                let method = action
                    .webhook_method
                    .clone()
                    .unwrap_or_else(|| "POST".to_string());
                let payload = action.webhook_payload.clone().unwrap_or_default();
                let url = u.clone();
                tokio::spawn(async move {
                    let req = if method.to_uppercase() == "POST" {
                        client
                            .post(&url)
                            .header("Content-Type", "application/json")
                            .body(payload)
                    } else {
                        client.get(&url)
                    };
                    let _ = req.send().await;
                });
            }
        }
        "discord" | "discord_mute" => {
            let mut muted = state.is_discord_muted.lock().unwrap();
            *muted = !*muted;
            trigger_hotkey(&["ctrl".to_string(), "shift".to_string(), "m".to_string()]);
        }
        "discord_deafen" => {
            let mut deafened = state.is_discord_deafened.lock().unwrap();
            *deafened = !*deafened;
            trigger_hotkey(&["ctrl".to_string(), "shift".to_string(), "d".to_string()]);
        }
        "media_play_pause" => {
            let _ = crate::midi::toggle_system_media_play_pause();
        }
        "media_next" => {
            trigger_media("next");
        }
        "media_prev" => {
            trigger_media("prev");
        }
        _ => {}
    }
}
