use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String, // "app", "url", "hotkey", "wait", "midi", "media", "obs", "audio", "text", "system", "navigation"
    pub path: Option<String>,
    pub url: Option<String>,
    pub keys: Option<Vec<String>>,
    pub delay_ms: Option<u64>,
    
    // MIDI Felder
    pub midi_type: Option<String>,
    pub midi_note: Option<u8>,
    pub midi_value: Option<u8>,
    pub midi_channel: Option<u8>,
    pub midi_device: Option<String>,
    
    // Media Felder
    pub media_key: Option<String>,
    
    // OBS Felder
    pub obs_action: Option<String>,
    pub obs_target: Option<String>,
    
    // Audio/Soundboard
    pub audio_path: Option<String>,
    pub audio_volume: Option<f32>,
    
    // Text Inserter
    pub text_content: Option<String>,
    
    // System
    pub system_command: Option<String>, // "Lock", "Shutdown", "WindowNextMonitor", "ToggleAlwaysOnTop"
    
    // Navigation
    pub target_page: Option<String>,

    // Webhook Felder
    pub webhook_url: Option<String>,
    pub webhook_method: Option<String>, // "GET" oder "POST"
    pub webhook_payload: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmartProfileMapping {
    pub process_name: String, // z. B. "chrome.exe", "obs64.exe"
    pub target_page: String,  // z. B. "Browser", "OBS-Steuerung"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mapping {
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub is_toggle: bool,
    #[serde(default)]
    pub color: u8,
    pub on_color: Option<u8>,
    #[serde(default)]
    pub state: bool,
    pub label: Option<String>,
    // Für Multi-Schritt-Makros (Toggle / Sequenz)
    #[serde(default)]
    pub current_step: usize,
    #[serde(default)]
    pub is_sequence: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FaderMapping {
    #[serde(rename = "type")]
    pub action_type: String,
    pub target: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObsConfig {
    #[serde(default = "default_obs_host")]
    pub host: String,
    #[serde(default = "default_obs_port")]
    pub port: u16,
    pub password: Option<String>,
    #[serde(default)]
    pub auto_connect: bool,
}

fn default_obs_host() -> String { "localhost".to_string() }
fn default_obs_port() -> u16 { 4455 }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub name: String,
    pub mappings: HashMap<String, Mapping>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_device_name")]
    pub device_name: String,
    #[serde(default)]
    pub output_device_name: String,
    #[serde(default)]
    pub pages: Vec<Page>,
    #[serde(default = "default_page_name")]
    pub active_page: String,
    #[serde(default)]
    pub fader_mappings: HashMap<String, FaderMapping>,
    #[serde(default)]
    pub obs: ObsConfig,
    #[serde(default)]
    pub config_url: String,

    // Neue Advanced Features
    #[serde(default)]
    pub smart_profiles_enabled: bool,
    #[serde(default)]
    pub smart_profiles: Vec<SmartProfileMapping>,
    #[serde(default = "default_web_companion_enabled")]
    pub web_companion_enabled: bool,
    #[serde(default = "default_web_companion_port")]
    pub web_companion_port: u16,
    #[serde(default)]
    pub obs_peak_meter_enabled: bool,
    pub obs_peak_meter_source: Option<String>,
    pub obs_peak_meter_column: Option<u8>,
    #[serde(default)]
    pub ripple_effect_enabled: bool,

    // Spotify & Discord Integration
    #[serde(default)]
    pub media_progress_enabled: bool,
    #[serde(default)]
    pub media_progress_row: u8, // 0-7 (Standard: 0)
    #[serde(default = "default_media_control_note")]
    pub media_control_note: u8, // Standard: 112 (side key 1)
    #[serde(default = "default_discord_mute_note")]
    pub discord_mute_note: u8, // Standard: 113 (side key 2)
    #[serde(default = "default_discord_deafen_note")]
    pub discord_deafen_note: u8, // Standard: 114 (side key 3)
}

fn default_device_name() -> String { "APC mini mk2".to_string() }
fn default_page_name() -> String { "Main".to_string() }
fn default_web_companion_enabled() -> bool { false }
fn default_web_companion_port() -> u16 { 1421 }
fn default_media_control_note() -> u8 { 112 }
fn default_discord_mute_note() -> u8 { 113 }
fn default_discord_deafen_note() -> u8 { 114 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_name: default_device_name(),
            output_device_name: String::new(),
            pages: vec![Page { name: default_page_name(), mappings: HashMap::new() }],
            active_page: default_page_name(),
            fader_mappings: HashMap::new(),
            obs: ObsConfig::default(),
            config_url: String::new(),
            smart_profiles_enabled: false,
            smart_profiles: vec![],
            web_companion_enabled: false,
            web_companion_port: default_web_companion_port(),
            obs_peak_meter_enabled: false,
            obs_peak_meter_source: None,
            obs_peak_meter_column: None,
            ripple_effect_enabled: false,
            media_progress_enabled: false,
            media_progress_row: 0,
            media_control_note: default_media_control_note(),
            discord_mute_note: default_discord_mute_note(),
            discord_deafen_note: default_discord_deafen_note(),
        }
    }
}

impl Default for ObsConfig {
    fn default() -> Self {
        Self {
            host: default_obs_host(),
            port: default_obs_port(),
            password: None,
            auto_connect: false,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push("streamdeck_config.json");
    path
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            // Check if it's the old format first
            if let Ok(config) = serde_json::from_str::<AppConfig>(&contents) {
                if config.pages.is_empty() {
                    // Try to migrate if mappings exists in root (legacy)
                    #[derive(Deserialize)]
                    struct LegacyConfig {
                        pub device_name: String,
                        pub mappings: HashMap<String, Mapping>,
                        pub fader_mappings: HashMap<String, FaderMapping>,
                        pub obs: ObsConfig,
                    }
                    if let Ok(legacy) = serde_json::from_str::<LegacyConfig>(&contents) {
                        return AppConfig {
                            device_name: legacy.device_name,
                            output_device_name: String::new(),
                            pages: vec![Page { name: "Main".to_string(), mappings: legacy.mappings }],
                            active_page: "Main".to_string(),
                            fader_mappings: legacy.fader_mappings,
                            obs: legacy.obs,
                            config_url: String::new(),
                            smart_profiles_enabled: false,
                            smart_profiles: vec![],
                            web_companion_enabled: false,
                            web_companion_port: default_web_companion_port(),
                            obs_peak_meter_enabled: false,
                            obs_peak_meter_source: None,
                            obs_peak_meter_column: None,
                            ripple_effect_enabled: false,
                            media_progress_enabled: false,
                            media_progress_row: 0,
                            media_control_note: default_media_control_note(),
                            discord_mute_note: default_discord_mute_note(),
                            discord_deafen_note: default_discord_deafen_note(),
                        };
                    }
                }
                return config;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
