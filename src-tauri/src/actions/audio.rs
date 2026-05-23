use std::fs::File;
use std::io::BufReader;
use std::thread;

pub fn play_sound(path: String, volume: f32) {
    thread::spawn(move || {
        use rodio::{Decoder, OutputStream, Sink};
        if let Ok(f) = File::open(path) {
            if let Ok(s) = Decoder::new(BufReader::new(f)) {
                if let Ok((_stream, handle)) = OutputStream::try_default() {
                    if let Ok(sink) = Sink::try_new(&handle) {
                        sink.set_volume(volume);
                        sink.append(s);
                        sink.sleep_until_end();
                    }
                }
            }
        }
    });
}
