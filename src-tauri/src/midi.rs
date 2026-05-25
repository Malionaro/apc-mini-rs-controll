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
    while *state.is_listening.lock().unwrap() {
        let current_page = state.config.lock().unwrap().active_page.clone();
        if current_page != last_page {
            if let Ok(mut out) = conn_out_mtx.lock() {
                refresh_leds(&mut out, &state);
            }
            last_page = current_page;
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
    for (note_str, m) in &page.mappings {
        if let Ok(note) = note_str.parse::<u8>() {
            let color = if m.is_toggle && m.state {
                m.on_color.unwrap_or(m.color)
            } else {
                m.color
            };
            let _ = conn_out.send(&[0x90, note, color]);
        }
    }
}

fn handle_interaction(note: u8, state: &Arc<MidiState>, conn_out: &Arc<Mutex<MidiOutputConnection>>) {
    let mapping_opt = {
        let mut config = state.config.lock().unwrap();
        let active_page_name = config.active_page.clone();
        let page = config.pages.iter_mut().find(|p| p.name == active_page_name);
        if let Some(p) = page {
            p.mappings.get_mut(&note.to_string()).map(|m| {
                if m.is_toggle {
                    m.state = !m.state;
                }
                m.clone()
            })
        } else {
            None
        }
    };

    if let Some(mapping) = mapping_opt {
        add_log(state, format!("Taste {}: {}", note, mapping.label.clone().unwrap_or_default()));
        for action in &mapping.actions {
            execute_action(action, state);
        }
        if let Ok(mut out) = conn_out.lock() {
            let color = if mapping.is_toggle && mapping.state {
                mapping.on_color.unwrap_or(mapping.color)
            } else {
                mapping.color
            };
            let _ = out.send(&[0x90, note, color]);
        }
    }
}
