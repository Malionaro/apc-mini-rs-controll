use crate::midi::MidiState;
use std::sync::Arc;
use std::time::{Duration, Instant};

static mut LAST_FADER_TIME: Option<Instant> = None;

pub fn handle_fader_move(cc: u8, val: u8, state: &Arc<MidiState>) {
    unsafe {
        if let Some(last) = LAST_FADER_TIME {
            if last.elapsed() < Duration::from_millis(40) {
                return;
            }
        }
        LAST_FADER_TIME = Some(Instant::now());
    }

    let fader_idx = if cc >= 48 && cc <= 56 {
        cc - 48
    } else {
        return;
    };
    
    let mapping = {
        state.config.lock().unwrap().fader_mappings.get(&fader_idx.to_string()).cloned()
    };

    if let Some(m) = mapping {
        if m.action_type == "volume" {
            let vol = (val as f32 / 127.0 * 100.0) as u8;
            if let Ok(device) = volumecontrol::AudioDevice::from_default() {
                let _ = device.set_vol(vol);
            }
        } else if m.action_type == "obs_volume" {
            if let Some(target) = &m.target {
                let vol = (val as f32 / 127.0) * 100.0;
                let target_str = format!("{}|{}", target, vol);
                let _ = state.obs.execute("SetVolume", Some(target_str));
            }
        } else if m.action_type == "app_volume" {
            if let Some(target) = &m.target {
                let vol_percent = val as f32 / 127.0; // 0.0 to 1.0
                set_app_volume(target, vol_percent);
            }
        }
    }
}

pub fn set_app_volume(process_name: &str, volume: f32) {
    let target_lower = process_name.to_lowercase();
    std::thread::spawn(move || {
        unsafe {
            let winmix = winmix::WinMix::default();
            if let Ok(sessions) = winmix.enumerate() {
                for session in sessions {
                    let path = session.path;
                    let filename = std::path::Path::new(&path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_lowercase();
                    if filename == target_lower || filename.replace(".exe", "") == target_lower {
                        let _ = session.vol.set_master_volume(volume);
                    }
                }
            }
        }
    });
}
