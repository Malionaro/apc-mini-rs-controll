pub mod config;
pub mod midi;
pub mod obs;
pub mod actions;
pub mod window_watcher;
pub mod web_server;

use config::{AppConfig, load_config, save_config};
use midi::{MidiState, start_listener, stop_listener};
use obs::ObsState;
use std::sync::{Arc, Mutex};
use tauri::{
    State, Manager, WebviewUrl, WebviewWindowBuilder, AppHandle, Emitter,
    window::Color,
    menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent, MouseButton},
};
use midir::{MidiInput, MidiOutput};

#[tauri::command]
fn get_config(state: State<'_, Arc<MidiState>>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn update_config(new_config: AppConfig, state: State<'_, Arc<MidiState>>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    *config = new_config.clone();
    save_config(&new_config)?;
    Ok(())
}

#[tauri::command]
fn toggle_listener(state: State<'_, Arc<MidiState>>) -> Result<bool, String> {
    let is_listening = { *state.is_listening.lock().unwrap() };
    if is_listening {
        stop_listener(state.inner().clone());
        Ok(false)
    } else {
        start_listener(state.inner().clone())?;
        Ok(true)
    }
}

#[tauri::command]
fn get_listener_status(state: State<'_, Arc<MidiState>>) -> bool {
    *state.is_listening.lock().unwrap()
}

#[tauri::command]
fn get_logs(state: State<'_, Arc<MidiState>>) -> Vec<String> {
    state.logs.lock().unwrap().clone()
}

#[tauri::command]
fn get_midi_ports() -> Vec<String> {
    let midi_in = match MidiInput::new("scan") {
        Ok(m) => m,
        Err(_) => return vec![],
    };
    midi_in.ports().iter()
        .map(|p| midi_in.port_name(p).unwrap_or_else(|_| "Unbekannt".to_string()))
        .collect()
}

#[tauri::command]
fn get_midi_output_ports() -> Vec<String> {
    let midi_out: MidiOutput = match MidiOutput::new("scan") {
        Ok(m) => m,
        Err(_) => return vec![],
    };
    midi_out.ports().iter()
        .map(|p| midi_out.port_name(p).unwrap_or_else(|_| "Unbekannt".to_string()))
        .collect()
}

#[tauri::command]
fn connect_obs(host: String, port: u16, password: Option<String>, state: State<'_, Arc<MidiState>>) -> Result<(), String> {
    let res = state.obs.connect(&host, port, password);
    if res.is_ok() {
        if let Some(handle) = &*state.app_handle.lock().unwrap() {
            let _ = handle.emit("obs-connection-status", true);
        }
    }
    res
}

#[tauri::command]
fn get_obs_status(state: State<'_, Arc<MidiState>>) -> bool {
    state.obs.is_connected()
}

#[tauri::command]
fn get_obs_scenes(state: State<'_, Arc<MidiState>>) -> Result<Vec<String>, String> {
    state.obs.get_scenes()
}

#[tauri::command]
fn get_obs_inputs(state: State<'_, Arc<MidiState>>) -> Result<Vec<String>, String> {
    state.obs.get_inputs()
}

#[tauri::command]
fn get_obs_sources(scene: String, state: State<'_, Arc<MidiState>>) -> Result<Vec<String>, String> {
    state.obs.get_sources(&scene)
}

#[tauri::command]
fn get_obs_filters(source: String, state: State<'_, Arc<MidiState>>) -> Result<Vec<String>, String> {
    state.obs.get_filters(&source)
}

#[tauri::command]
async fn pick_file(handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
    handle.dialog().file().pick_file(move |f| { let _ = tx.send(f.map(|p| p.to_string())); });
    rx.await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn pick_folder(handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
    handle.dialog().file().pick_folder(move |f| { let _ = tx.send(f.map(|p| p.to_string())); });
    rx.await.map_err(|e| e.to_string())
}

#[tauri::command]
fn open_log_window(handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = handle.get_webview_window("logs") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = WebviewWindowBuilder::new(&handle, "logs", WebviewUrl::App("index.html?window=logs".into()))
            .title("System Diagnostics")
            .inner_size(500.0, 400.0)
            .background_color(Color(11, 16, 32, 255))
            .build()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_settings_window(handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = handle.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = WebviewWindowBuilder::new(&handle, "settings", WebviewUrl::App("index.html?window=settings".into()))
            .title("Configuration")
            .inner_size(450.0, 600.0)
            .resizable(false)
            .decorations(true)
            .background_color(Color(11, 16, 32, 255))
            .build()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn set_active_page(page_name: String, state: State<'_, Arc<MidiState>>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();
    if config.pages.iter().any(|p| p.name == page_name) {
        config.active_page = page_name;
        Ok(())
    } else {
        Err("Seite nicht gefunden".to_string())
    }
}

#[tauri::command]
async fn download_and_install_update(download_url: String, filename: String) -> Result<(), String> {
    use std::path::PathBuf;
    use tokio::io::AsyncWriteExt;

    // Download to temp dir
    let temp_dir = std::env::temp_dir();
    let dest_path: PathBuf = temp_dir.join(&filename);

    let response = reqwest::get(&download_url)
        .await
        .map_err(|e| format!("Download fehlgeschlagen: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Server Fehler: {}", response.status()));
    }

    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Lesen fehlgeschlagen: {}", e))?;

    let mut file = tokio::fs::File::create(&dest_path)
        .await
        .map_err(|e| format!("Datei erstellen fehlgeschlagen: {}", e))?;

    file.write_all(&bytes)
        .await
        .map_err(|e| format!("Schreiben fehlgeschlagen: {}", e))?;

    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);

    // Launch installer and exit app
    std::process::Command::new(&dest_path)
        .spawn()
        .map_err(|e| format!("Installer starten fehlgeschlagen: {}", e))?;

    std::process::exit(0);
}

#[tauri::command]
async fn fetch_config(url: String, state: State<'_, Arc<MidiState>>) -> Result<AppConfig, String> {
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("Fehler beim Senden: {}", e))?;
    
    if !resp.status().is_success() {
        return Err(format!("Server Fehler: {}", resp.status()));
    }

    let next_config = resp.json::<AppConfig>()
        .await
        .map_err(|e| format!("Fehler beim Parsen: {}. Ist das Format korrekt?", e))?;
    
    save_config(&next_config)?;
    
    {
        let mut config = state.config.lock().unwrap();
        *config = next_config.clone();
    }
    
    Ok(next_config)
}

#[tauri::command]
fn panic_stop_all_sounds() {
    crate::actions::audio::panic_stop_all_sounds();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_config = load_config();
    
    let midi_state = Arc::new(MidiState {
        is_listening: Arc::new(Mutex::new(false)),
        config: Arc::new(Mutex::new(initial_config)),
        logs: Arc::new(Mutex::new(vec![])),
        last_note_pressed: Arc::new(Mutex::new(None)),
        app_handle: Arc::new(Mutex::new(None)),
        obs: Arc::new(ObsState::new()),
        is_recording: Arc::new(Mutex::new(false)),
        is_streaming: Arc::new(Mutex::new(false)),
        is_discord_muted: Arc::new(Mutex::new(false)),
        is_discord_deafened: Arc::new(Mutex::new(false)),
        is_media_playing: Arc::new(Mutex::new(false)),
        page_history: Arc::new(Mutex::new(vec![])),
    });

    let state_setup = midi_state.clone();

    tauri::Builder::default()
        .setup(move |app| {
            let mut handle = state_setup.app_handle.lock().unwrap();
            let app_handle = app.handle().clone();
            *handle = Some(app_handle.clone());
            
            // Auto-connect to OBS on startup if configured
            let config = state_setup.config.lock().unwrap().clone();
            if config.obs.auto_connect && !config.obs.host.is_empty() {
                let obs = state_setup.obs.clone();
                let app_handle_clone = app_handle.clone();
                std::thread::spawn(move || {
                    if obs.connect(&config.obs.host, config.obs.port, config.obs.password.clone()).is_ok() {
                        let _ = app_handle_clone.emit("obs-connection-status", true);
                    }
                });
            }
            
            let quit_i = MenuItem::with_id(app, "quit", "Exit Console", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Interface", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app: &AppHandle, event| {
                    match event.id.as_ref() {
                        "quit" => { app.exit(0); }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == MouseButton::Left {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;


            Ok(())
        })
        .manage(midi_state)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label();
                if label == "main" || label == "logs" || label == "settings" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            toggle_listener,
            get_listener_status,
            get_logs,
            get_midi_ports,
            get_midi_output_ports,
            connect_obs,
            get_obs_status,
            get_obs_scenes,
            get_obs_inputs,
            get_obs_sources,
            get_obs_filters,
            pick_file,
            pick_folder,
            open_log_window,
            open_settings_window,
            set_active_page,
            fetch_config,
            download_and_install_update,
            panic_stop_all_sounds
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
