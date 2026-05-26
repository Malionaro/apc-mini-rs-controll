use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::sync::{Arc, Mutex, OnceLock};
use rodio::{Decoder, OutputStream, Sink};

fn get_active_sinks() -> &'static Mutex<Vec<Arc<Sink>>> {
    static SINKS: OnceLock<Mutex<Vec<Arc<Sink>>>> = OnceLock::new();
    SINKS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn play_sound(path: String, volume: f32) {
    thread::spawn(move || {
        if let Ok(f) = File::open(path) {
            if let Ok(s) = Decoder::new(BufReader::new(f)) {
                if let Ok((_stream, handle)) = OutputStream::try_default() {
                    if let Ok(sink) = Sink::try_new(&handle) {
                        sink.set_volume(volume);
                        sink.append(s);
                        
                        let sink_arc = Arc::new(sink);
                        {
                            let mut sinks = get_active_sinks().lock().unwrap();
                            sinks.push(sink_arc.clone());
                        }
                        
                        sink_arc.sleep_until_end();
                        
                        let mut sinks = get_active_sinks().lock().unwrap();
                        sinks.retain(|x| !Arc::ptr_eq(x, &sink_arc));
                    }
                }
            }
        }
    });
}

pub fn panic_stop_all_sounds() {
    let mut sinks = get_active_sinks().lock().unwrap();
    for sink in sinks.iter() {
        sink.stop();
    }
    sinks.clear();
}
