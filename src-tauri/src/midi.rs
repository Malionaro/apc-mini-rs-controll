use crate::config::AppConfig;
use crate::obs::ObsState;
use crate::actions::{execute_action, faders::handle_fader_move};
use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;
use tauri::{AppHandle, Emitter};
use obws::events::Event;
use serde_json::json;

pub struct MidiState {
    pub is_listening: Arc<Mutex<bool>>,
    pub config: Arc<Mutex<AppConfig>>,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub last_note_pressed: Arc<Mutex<Option<u8>>>,
    pub app_handle: Arc<Mutex<Option<AppHandle>>>,
    pub obs: Arc<ObsState>,
    pub is_recording: Arc<Mutex<bool>>,
    pub is_streaming: Arc<Mutex<bool>>,
    pub is_discord_muted: Arc<Mutex<bool>>,
    pub is_discord_deafened: Arc<Mutex<bool>>,
    pub is_media_playing: Arc<Mutex<bool>>,
    pub page_history: Arc<Mutex<Vec<String>>>,
}

pub fn add_log(state: &Arc<MidiState>, msg: String) {
    let mut logs = state.logs.lock().unwrap();
    let entry = format!("[{}] {}", Local::now().format("%H:%M:%S"), msg);
    logs.push(entry.clone());
    if logs.len() > 100 {
        logs.remove(0);
    }
    if let Some(handle) = &*state.app_handle.lock().unwrap() {
        let _ = handle.emit("new-log", entry);
    }
}

pub fn start_listener(state: Arc<MidiState>) -> Result<(), String> {
    let mut is_listening = state.is_listening.lock().unwrap();
    if *is_listening {
        return Err("Engine läuft".to_string());
    }
    *is_listening = true;
    drop(is_listening);

    add_log(&state, "MIDI Engine START".to_string());
    
    // Smart Profiles Window Watcher starten
    crate::window_watcher::start_window_watcher(state.clone());
    
    // Web Companion Server starten
    crate::web_server::start_web_server(state.clone());
    
    let config_snap = state.config.lock().unwrap().clone();
    if config_snap.obs.auto_connect && !config_snap.obs.host.is_empty() {
        let obs_state = state.obs.clone();
        let state_clone = state.clone();
        thread::spawn(move || {
            if obs_state.connect(
                &config_snap.obs.host,
                config_snap.obs.port,
                config_snap.obs.password.clone(),
            ).is_ok() {
                if let Some(handle) = &*state_clone.app_handle.lock().unwrap() {
                    let _ = handle.emit("obs-connection-status", true);
                }
            }
        });
    }

    let state_clone = state.clone();
    thread::spawn(move || {
        while *state_clone.is_listening.lock().unwrap() {
            if let Err(e) = run_midi_loop(state_clone.clone()) {
                add_log(&state_clone, format!("MIDI Fehler: {}. Retry...", e));
                thread::sleep(Duration::from_secs(2));
            } else {
                break;
            }
        }
    });
    Ok(())
}

pub fn stop_listener(state: Arc<MidiState>) {
    let mut is_listening = state.is_listening.lock().unwrap();
    *is_listening = false;
}

fn run_midi_loop(state: Arc<MidiState>) -> Result<(), String> {
    let mut midi_in = MidiInput::new("APC In").map_err(|e| e.to_string())?;
    midi_in.ignore(midir::Ignore::None);
    let in_ports = midi_in.ports();
    let target_device = state.config.lock().unwrap().device_name.clone();

    let in_port = in_ports.iter().find(|p| {
        let name = midi_in.port_name(p).unwrap_or_default();
        if !target_device.is_empty() && name == target_device {
            return true;
        }
        let low = name.to_lowercase();
        low.contains("apc") || low.contains("akai") || low.contains("mini")
    }).ok_or("Gerät nicht gefunden")?;

    let midi_out = MidiOutput::new("APC Out").map_err(|e| e.to_string())?;
    let out_ports = midi_out.ports();
    let target_output = state.config.lock().unwrap().output_device_name.clone();

    let out_port = out_ports.iter().find(|p| {
        let name = midi_out.port_name(p).unwrap_or_default();
        if !target_output.is_empty() && name == target_output {
            return true;
        }
        let low = name.to_lowercase();
        low.contains("apc") || low.contains("akai") || low.contains("mini")
    }).ok_or("Ausgang nicht gefunden")?;

    let mut conn_out = midi_out.connect(out_port, "apc-mini-out").map_err(|e| e.to_string())?;
    refresh_leds(&mut conn_out, &state);

    let conn_out_mtx = Arc::new(Mutex::new(conn_out));
    let state_cb = state.clone();
    let conn_out_cb = conn_out_mtx.clone();

    let mut obs_rx = state.obs.event_tx.subscribe();
    let conn_out_obs = conn_out_mtx.clone();
    let state_obs = state.clone();
    
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            while let Ok(event) = obs_rx.recv().await {
                handle_obs_event(event, &state_obs, &conn_out_obs);
            }
        });
    });

    let conn_out_blinker = conn_out_mtx.clone();
    let state_blinker = state.clone();
    thread::spawn(move || {
        let mut toggle = false;
        while *state_blinker.is_listening.lock().unwrap() {
            let is_rec = *state_blinker.is_recording.lock().unwrap();
            let is_str = *state_blinker.is_streaming.lock().unwrap();
            
            if is_rec || is_str {
                let config = state_blinker.config.lock().unwrap();
                let active_page_name = config.active_page.clone();
                if let Some(page) = config.pages.iter().find(|p| p.name == active_page_name) {
                    if let Ok(mut out) = conn_out_blinker.lock() {
                        for (note_str, mapping) in &page.mappings {
                            if let Ok(note) = note_str.parse::<u8>() {
                                let mut should_blink_rec = false;
                                let mut should_blink_str = false;
                                for action in &mapping.actions {
                                    if action.obs_action.as_deref() == Some("StartStopRecord") {
                                        should_blink_rec = true;
                                    }
                                    if action.obs_action.as_deref() == Some("StartStopStream") {
                                        should_blink_str = true;
                                    }
                                }
                                
                                if (is_rec && should_blink_rec) || (is_str && should_blink_str) {
                                    let color = if toggle {
                                        mapping.on_color.unwrap_or(mapping.color)
                                    } else {
                                        0
                                    };
                                    let _ = out.send(&[0x90, note, color]);
                                }
                            }
                        }
                    }
                }
            }
            
            toggle = !toggle;
            thread::sleep(Duration::from_millis(500));
        }
    });

    let _conn_in = midi_in.connect(in_port, "apc-mini-in", move |_stamp, message, _| {
        if message.len() >= 3 {
            let (status, data1, data2) = (message[0], message[1], message[2]);
            let _ = state_cb.app_handle.lock().unwrap().as_ref().map(|h| {
                let _ = h.emit("midi-interaction", (status, data1, data2));
            });

            match status & 0xF0 {
                0x90 if data2 > 0 => handle_interaction(data1, &state_cb, &conn_out_cb),
                0xB0 => handle_fader_move(data1, data2, &state_cb),
                _ => {}
            }
        }
    }, ()).map_err(|e| e.to_string())?;

    if let Some(h) = &*state.app_handle.lock().unwrap() {
        let _ = h.emit("connection-status", true);
    }

    let mut last_page = "".to_string();
    let mut tick_counter = 0;
    while *state.is_listening.lock().unwrap() {
        let current_page = state.config.lock().unwrap().active_page.clone();
        if current_page != last_page {
            if let Ok(mut out) = conn_out_mtx.lock() {
                refresh_leds(&mut out, &state);
            }
            last_page = current_page;
        }
        
        tick_counter += 1;
        if tick_counter >= 10 { // Every 1 second (10 * 100ms)
            tick_counter = 0;
            if let Ok(mut out) = conn_out_mtx.lock() {
                update_media_session_leds(&mut out, &state);
                update_discord_leds(&mut out, &state);
            }
        }
        
        thread::sleep(Duration::from_millis(100));
    }

    if let Ok(mut out) = conn_out_mtx.lock() {
        for n in 0..127 {
            let _ = out.send(&[0x90, n, 0]);
        }
    }
    if let Some(h) = &*state.app_handle.lock().unwrap() {
        let _ = h.emit("connection-status", false);
    }
    Ok(())
}

fn send_obs_webhook(state: &Arc<MidiState>, event_type: &str, target: &str, value: serde_json::Value) {
    let config = state.config.lock().unwrap();
    let url = config.config_url.clone();
    if url.starts_with("http://") || url.starts_with("https://") {
        let client = reqwest::Client::new();
        let payload = json!({
            "event": event_type,
            "target": target,
            "value": value,
            "device": config.device_name,
            "active_page": config.active_page
        });
        tokio::spawn(async move {
            let _ = client.post(&url).json(&payload).send().await;
        });
    }
}

fn handle_obs_event(event: Event, state: &Arc<MidiState>, conn_out: &Arc<Mutex<MidiOutputConnection>>) {
    match event {
        Event::CurrentProgramSceneChanged { id } => {
            update_leds_for_obs(state, conn_out, "SetScene", &id.name, true);
            send_obs_webhook(state, "SceneChanged", &id.name, json!(true));
        }
        Event::CurrentPreviewSceneChanged { id } => {
            update_leds_for_obs(state, conn_out, "SetPreviewScene", &id.name, true);
            send_obs_webhook(state, "PreviewSceneChanged", &id.name, json!(true));
        }
        Event::InputMuteStateChanged { id, muted } => {
            update_leds_for_obs(state, conn_out, "ToggleMute", &id.name, muted);
            send_obs_webhook(state, "MuteStateChanged", &id.name, json!(muted));
        }
        Event::StreamStateChanged { active, .. } => {
            *state.is_streaming.lock().unwrap() = active;
            update_leds_for_obs(state, conn_out, "StartStopStream", "", active);
            send_obs_webhook(state, "StreamStateChanged", "", json!(active));
        }
        Event::RecordStateChanged { active, .. } => {
            *state.is_recording.lock().unwrap() = active;
            update_leds_for_obs(state, conn_out, "StartStopRecord", "", active);
            send_obs_webhook(state, "RecordStateChanged", "", json!(active));
        }
        Event::SourceFilterEnableStateChanged { source, filter, enabled } => {
            let target = format!("{}|{}", source, filter);
            update_leds_for_obs(state, conn_out, "ToggleFilter", &target, enabled);
            send_obs_webhook(state, "FilterStateChanged", &target, json!(enabled));
        }
        Event::SceneItemEnableStateChanged { scene, item_id, enabled } => {
            let state_clone = state.clone();
            let conn_out_clone = conn_out.clone();
            tokio::spawn(async move {
                if let Ok(source_name) = state_clone.obs.resolve_scene_item_name(&scene.name, item_id as i64).await {
                    let target = format!("{}|{}", scene.name, source_name);
                    update_leds_for_obs(&state_clone, &conn_out_clone, "ToggleSource", &target, enabled);
                    update_leds_for_obs(&state_clone, &conn_out_clone, "SetSourceVisible", &target, enabled);
                    send_obs_webhook(&state_clone, "SourceVisibilityChanged", &target, json!(enabled));
                } else {
                    let target = format!("{}|{}", scene.name, item_id);
                    send_obs_webhook(&state_clone, "SourceVisibilityChanged", &target, json!(enabled));
                }
            });
        }
        Event::InputVolumeMeters { inputs } => {
            let (meter_enabled, source_name, column) = {
                let config = state.config.lock().unwrap();
                (
                    config.obs_peak_meter_enabled,
                    config.obs_peak_meter_source.clone().unwrap_or_default(),
                    config.obs_peak_meter_column.unwrap_or(7),
                )
            };
            
            if meter_enabled && !source_name.is_empty() {
                if let Some(meter) = inputs.iter().find(|m| m.name == source_name) {
                    if let Some(channel_levels) = meter.levels.get(0) {
                        let val = channel_levels[0]; // multiplier level from 0.0 upwards
                        let level = ((val * 8.0).round() as u8).min(8);
                        
                        if let Ok(mut out) = conn_out.lock() {
                            for row in 0..8u8 {
                                let note = (row * 8 + column) as u8;
                                let color = if row < level {
                                    if row < 5 {
                                        21 // Green
                                    } else if row < 7 {
                                        13 // Yellow/Amber
                                    } else {
                                        121 // Red
                                    }
                                } else {
                                    0 // Off
                                };
                                let _ = out.send(&[0x90, note, color]);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn update_leds_for_obs(
    state: &Arc<MidiState>,
    conn_out: &Arc<Mutex<MidiOutputConnection>>,
    action_type: &str,
    target: &str,
    is_active: bool,
) {
    let mut config = state.config.lock().unwrap();
    let active_page_name = config.active_page.clone();
    let mut updates = Vec::new();

    if let Some(page) = config.pages.iter_mut().find(|p| p.name == active_page_name) {
        for (note_str, mapping) in page.mappings.iter_mut() {
            if let Ok(note) = note_str.parse::<u8>() {
                let mut is_match = false;
                let mut is_exclusive = false;
                
                for action in &mapping.actions {
                    let act_type = match action.action_type.as_str() {
                        "obs_toggle" => "ToggleSource",
                        "obs_filter" => "ToggleFilter",
                        "obs_visible" => "SetSourceVisible",
                        "obs" => action.obs_action.as_deref().unwrap_or(""),
                        _ => "",
                    };
                    
                    if act_type == action_type {
                        if action_type == "SetScene" {
                            is_exclusive = true;
                            if let Some(obs_tgt) = &action.obs_target {
                                if obs_tgt == target {
                                    is_match = true;
                                }
                            }
                        } else {
                            if target.is_empty() {
                                is_match = true;
                            } else if let Some(obs_tgt) = &action.obs_target {
                                if obs_tgt == target {
                                    is_match = true;
                                }
                            }
                        }
                    }
                }

                if is_exclusive {
                    mapping.state = is_match;
                    let color = if mapping.state {
                        mapping.on_color.unwrap_or(mapping.color)
                    } else {
                        mapping.color
                    };
                    updates.push((note, color));
                } else if is_match {
                    mapping.state = is_active;
                    let color = if mapping.state {
                        mapping.on_color.unwrap_or(mapping.color)
                    } else {
                        mapping.color
                    };
                    updates.push((note, color));
                }
            }
        }
    }

    if !updates.is_empty() {
        if let Ok(mut out) = conn_out.lock() {
            for (note, color) in updates {
                let _ = out.send(&[0x90, note, color]);
            }
        }
    }
}

fn refresh_leds(conn_out: &mut MidiOutputConnection, state: &Arc<MidiState>) {
    let config = state.config.lock().unwrap();
    let active_page_name = config.active_page.clone();
    let page = config.pages.iter().find(|p| p.name == active_page_name).unwrap_or(&config.pages[0]);
    for n in 0..127 {
        let _ = conn_out.send(&[0x90, n, 0]);
    }
    for (note_str, _m) in &page.mappings {
        if let Ok(note) = note_str.parse::<u8>() {
            let color = get_mapping_color(note, state);
            let _ = conn_out.send(&[0x90, note, color]);
        }
    }
    // Update active media LEDs and progress bar if enabled
    update_media_session_leds(conn_out, state);
    update_discord_leds(conn_out, state);
}

fn handle_interaction(note: u8, state: &Arc<MidiState>, conn_out: &Arc<Mutex<MidiOutputConnection>>) {
    let (mapping_opt, actions_to_run, ripple_enabled) = {
        let mut config = state.config.lock().unwrap();
        let ripple = config.ripple_effect_enabled;
        let active_page_name = config.active_page.clone();
        let page = config.pages.iter_mut().find(|p| p.name == active_page_name);
        if let Some(p) = page {
            if let Some(m) = p.mappings.get_mut(&note.to_string()) {
                if m.is_toggle {
                    m.state = !m.state;
                }
                
                let run_actions = if m.is_sequence && !m.actions.is_empty() {
                    let step = m.current_step % m.actions.len();
                    m.current_step = (m.current_step + 1) % m.actions.len();
                    vec![m.actions[step].clone()]
                } else if m.is_toggle && m.actions.len() >= 2 {
                    if m.state {
                        vec![m.actions[0].clone()]
                    } else {
                        vec![m.actions[1].clone()]
                    }
                } else {
                    m.actions.clone()
                };
                
                (Some(m.clone()), run_actions, ripple)
            } else {
                (None, vec![], false)
            }
        } else {
            (None, vec![], false)
        }
    };

    if let Some(mapping) = mapping_opt {
        add_log(state, format!("Taste {}: {}", note, mapping.label.clone().unwrap_or_default()));
        
        // Execute actions
        for action in &actions_to_run {
            execute_action(action, state);
        }
        
        // Update local LED color
        if let Ok(mut out) = conn_out.lock() {
            let color = get_mapping_color(note, state);
            let _ = out.send(&[0x90, note, color]);
            
            // Instantly sync any other Discord/Media buttons on the page
            update_media_session_leds(&mut out, state);
            update_discord_leds(&mut out, state);
        }

        // Trigger Ripple LED effect if enabled
        if ripple_enabled {
            trigger_ripple_effect(note, conn_out.clone(), state.clone());
        }
    }
}

fn trigger_ripple_effect(note: u8, conn_out: Arc<Mutex<MidiOutputConnection>>, state: Arc<MidiState>) {
    if note >= 64 {
        return; // Ripple is only for the 8x8 grid
    }
    
    let state_clone = state.clone();
    let conn_out_clone = conn_out.clone();
    thread::spawn(move || {
        let r = (note / 8) as i8;
        let c = (note % 8) as i8;
        
        let mut radius_1 = Vec::new();
        let mut radius_2 = Vec::new();
        
        for row in 0..8i8 {
            for col in 0..8i8 {
                let dist = std::cmp::max((row - r).abs(), (col - c).abs());
                let n = (row * 8 + col) as u8;
                if dist == 1 {
                    radius_1.push(n);
                } else if dist == 2 {
                    radius_2.push(n);
                }
            }
        }
        
        // Step 1: Glow Radius 1 (e.g. Cyan color)
        if let Ok(mut out) = conn_out_clone.lock() {
            for &n in &radius_1 {
                let _ = out.send(&[0x90, n, 45]);
            }
        }
        thread::sleep(Duration::from_millis(80));
        
        // Step 2: Dim Radius 1, Glow Radius 2
        if let Ok(mut out) = conn_out_clone.lock() {
            for &n in &radius_1 {
                let orig = get_mapping_color(n, &state_clone);
                let _ = out.send(&[0x90, n, orig]);
            }
            for &n in &radius_2 {
                let _ = out.send(&[0x90, n, 49]); // Magenta/Purple color
            }
        }
        thread::sleep(Duration::from_millis(80));
        
        // Step 3: Restore Radius 2
        if let Ok(mut out) = conn_out_clone.lock() {
            for &n in &radius_2 {
                let orig = get_mapping_color(n, &state_clone);
                let _ = out.send(&[0x90, n, orig]);
            }
        }
    });
}

fn get_mapping_color(note: u8, state: &Arc<MidiState>) -> u8 {
    let config = state.config.lock().unwrap();
    let active_page_name = config.active_page.clone();
    if let Some(page) = config.pages.iter().find(|p| p.name == active_page_name) {
        if let Some(mapping) = page.mappings.get(&note.to_string()) {
            for action in &mapping.actions {
                match action.action_type.as_str() {
                    "discord" | "discord_mute" => {
                        let is_muted = *state.is_discord_muted.lock().unwrap();
                        return if is_muted { 121 } else { mapping.color };
                    }
                    "discord_deafen" => {
                        let is_deafened = *state.is_discord_deafened.lock().unwrap();
                        return if is_deafened { 13 } else { mapping.color };
                    }
                    "media_play_pause" => {
                        let is_playing = *state.is_media_playing.lock().unwrap();
                        return if is_playing { 21 } else { 121 };
                    }
                    _ => {}
                }
            }
            if mapping.is_toggle && mapping.state {
                return mapping.on_color.unwrap_or(mapping.color);
            } else {
                return mapping.color;
            }
        }
    }
    0
}

pub fn toggle_system_media_play_pause() -> Result<(), String> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    let op = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().map_err(|e| e.to_string())?;
    let manager = op.get().map_err(|e| e.to_string())?;
    if let Ok(session) = manager.GetCurrentSession() {
        let op = session.TryTogglePlayPauseAsync().map_err(|e| e.to_string())?;
        let _ = op.get();
    }
    Ok(())
}

fn update_media_session_leds(out: &mut MidiOutputConnection, state: &Arc<MidiState>) {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    let (enabled, progress_row) = {
        let config = state.config.lock().unwrap();
        (config.media_progress_enabled, config.media_progress_row)
    };

    let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
        Ok(op) => match op.get() {
            Ok(mgr) => mgr,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let session = match manager.GetCurrentSession() {
        Ok(s) => s,
        Err(_) => {
            if enabled {
                for col in 0..8 {
                    let note = progress_row * 8 + col;
                    let _ = out.send(&[0x90, note, 0]);
                }
            }
            let active_page_name = {
                let config = state.config.lock().unwrap();
                config.active_page.clone()
            };
            let config = state.config.lock().unwrap();
            if let Some(page) = config.pages.iter().find(|p| p.name == active_page_name) {
                for (note_str, mapping) in &page.mappings {
                    if let Ok(note) = note_str.parse::<u8>() {
                        for action in &mapping.actions {
                            if action.action_type == "media_play_pause" {
                                let _ = out.send(&[0x90, note, 121]); // Red when no session
                            }
                        }
                    }
                }
            }
            return;
        }
    };

    let is_playing = if let Ok(info) = session.GetPlaybackInfo() {
        if let Ok(status) = info.PlaybackStatus() {
            status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
        } else {
            false
        }
    } else {
        false
    };

    *state.is_media_playing.lock().unwrap() = is_playing;

    let active_page_name = {
        let config = state.config.lock().unwrap();
        config.active_page.clone()
    };
    let config = state.config.lock().unwrap();
    if let Some(page) = config.pages.iter().find(|p| p.name == active_page_name) {
        for (note_str, mapping) in &page.mappings {
            if let Ok(note) = note_str.parse::<u8>() {
                for action in &mapping.actions {
                    if action.action_type == "media_play_pause" {
                        let play_pause_color = if is_playing { 21 } else { 121 }; // Green vs Red
                        let _ = out.send(&[0x90, note, play_pause_color]);
                    }
                }
            }
        }
    }

    if !enabled {
        return;
    }

    if let Ok(timeline) = session.GetTimelineProperties() {
        if let (Ok(pos), Ok(end)) = (timeline.Position(), timeline.EndTime()) {
            let pos_ticks = pos.Duration;
            let end_ticks = end.Duration;
            if end_ticks > 0 {
                let percent = (pos_ticks as f64 / end_ticks as f64).min(1.0).max(0.0);
                let filled_pads = (percent * 8.0).round() as u8;
                for col in 0..8 {
                    let note = progress_row * 8 + col;
                    let color = if col < filled_pads {
                        45 // Cyan
                    } else {
                        0 // Off
                    };
                    let _ = out.send(&[0x90, note, color]);
                }
                return;
            }
        }
    }

    for col in 0..8 {
        let note = progress_row * 8 + col;
        let _ = out.send(&[0x90, note, 0]);
    }
}

fn update_discord_leds(out: &mut MidiOutputConnection, state: &Arc<MidiState>) {
    let active_page_name = {
        let config = state.config.lock().unwrap();
        config.active_page.clone()
    };
    let config = state.config.lock().unwrap();
    let page = match config.pages.iter().find(|p| p.name == active_page_name) {
        Some(p) => p,
        None => return,
    };

    let is_muted = *state.is_discord_muted.lock().unwrap();
    let is_deafened = *state.is_discord_deafened.lock().unwrap();

    for (note_str, mapping) in &page.mappings {
        if let Ok(note) = note_str.parse::<u8>() {
            for action in &mapping.actions {
                if action.action_type == "discord" || action.action_type == "discord_mute" {
                    let color = if is_muted { 121 } else { mapping.color };
                    let _ = out.send(&[0x90, note, color]);
                }
                if action.action_type == "discord_deafen" {
                    let color = if is_deafened { 13 } else { mapping.color };
                    let _ = out.send(&[0x90, note, color]);
                }
            }
        }
    }
}
