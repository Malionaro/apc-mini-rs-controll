use crate::midi::MidiState;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub fn start_web_server(state: Arc<MidiState>) {
    let port = { state.config.lock().unwrap().web_companion_port };
    let enabled = { state.config.lock().unwrap().web_companion_enabled };

    if !enabled {
        return;
    }

    let state_clone = state.clone();
    thread::spawn(move || {
        let address = format!("0.0.0.0:{}", port);
        let listener = match TcpListener::bind(&address) {
            Ok(l) => {
                crate::midi::add_log(
                    &state_clone,
                    format!("Web Companion gestartet auf http://localhost:{}", port),
                );
                l
            }
            Err(e) => {
                crate::midi::add_log(
                    &state_clone,
                    format!("Fehler beim Starten des Web Companions: {}", e),
                );
                return;
            }
        };

        for stream in listener.incoming() {
            if !*state_clone.is_listening.lock().unwrap() {
                break;
            }
            if let Ok(stream) = stream {
                let state_inner = state_clone.clone();
                thread::spawn(move || {
                    handle_connection(stream, state_inner);
                });
            }
        }
    });
}

fn handle_connection(mut stream: TcpStream, state: Arc<MidiState>) {
    let mut buffer = [0; 4096];
    if let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            return;
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let first_line = request.lines().next().unwrap_or_default();
        let parts: Vec<&str> = first_line.split_whitespace().collect();

        if parts.len() >= 2 {
            let path = parts[1];

            if path == "/" || path == "/index.html" {
                serve_html(stream, state);
            } else if path == "/api/config" {
                serve_config(stream, state);
            } else if path.starts_with("/api/press") {
                // Handle button press
                // Format: /api/press?note=X
                let note_val = path
                    .split("note=")
                    .nth(1)
                    .unwrap_or_default()
                    .parse::<u8>()
                    .unwrap_or(255);

                if note_val != 255 {
                    // Trigger pad interaction in background to prevent blocking
                    let state_clone = state.clone();
                    thread::spawn(move || {
                        // In standard Tauri app, we don't have a direct raw output connection from web,
                        // but we can pass a dummy/empty Arc Mutex or let it trigger execute_action directly.
                        // Let's call execute_action for all actions of the mapping!
                        let mapping_opt = {
                            let mut config = state_clone.config.lock().unwrap();
                            let active_page_name = config.active_page.clone();
                            let page = config.pages.iter_mut().find(|p| p.name == active_page_name);
                            if let Some(p) = page {
                                p.mappings.get_mut(&note_val.to_string()).map(|m| {
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
                            crate::midi::add_log(
                                &state_clone,
                                format!("Web Companion: Taste {} gedrückt", note_val),
                            );
                            for action in &mapping.actions {
                                crate::actions::execute_action(action, &state_clone);
                            }
                        }
                    });

                    let response = "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}";
                    let _ = stream.write_all(response.as_bytes());
                } else {
                    let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                }
            } else {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                let _ = stream.write_all(response.as_bytes());
            }
        }
    }
}

fn serve_config(mut stream: TcpStream, state: Arc<MidiState>) {
    let config_json = {
        let config = state.config.lock().unwrap();
        serde_json::to_string(&*config).unwrap_or_default()
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        config_json.len(),
        config_json
    );
    let _ = stream.write_all(response.as_bytes());
}
fn serve_html(mut stream: TcpStream, state: Arc<MidiState>) {
    let active_page_name = { state.config.lock().unwrap().active_page.clone() };

    let html_template = r##"<!DOCTYPE html>
<html lang="de">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>APC Mini Web Companion</title>
    <style>
        body {
            background-color: #0c0c0f;
            color: #e2e8f0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            display: flex;
            flex-direction: column;
            align-items: center;
        }
        h1 {
            color: #38bdf8;
            font-size: 24px;
            margin-bottom: 5px;
            text-shadow: 0 0 10px rgba(56, 189, 248, 0.3);
        }
        .subtitle {
            color: #94a3b8;
            font-size: 14px;
            margin-bottom: 20px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(8, 1fr);
            gap: 8px;
            max-width: 450px;
            width: 100%;
            background: rgba(255, 255, 255, 0.03);
            padding: 15px;
            border-radius: 16px;
            border: 1px solid rgba(255, 255, 255, 0.05);
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.3);
        }
        .btn {
            aspect-ratio: 1;
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 6px;
            color: white;
            font-size: 10px;
            font-weight: bold;
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            transition: all 0.1s ease;
            user-select: none;
            -webkit-tap-highlight-color: transparent;
            text-align: center;
            overflow: hidden;
            word-break: break-all;
        }
        .btn:active {
            transform: scale(0.92);
            background: #38bdf8;
            border-color: #38bdf8;
            box-shadow: 0 0 15px rgba(56, 189, 248, 0.8);
        }
        .active-btn {
            border-color: #38bdf8;
            box-shadow: 0 0 8px rgba(56, 189, 248, 0.4);
        }
    </style>
</head>
<body>
    <h1>APC MINI // COMPANION</h1>
    <div class="subtitle">Aktive Seite: <span id="page-name">{active_page_name}</span></div>
    
    <div class="grid" id="button-grid">
        <!-- Buttons will be generated by JS -->
    </div>

    <script>
        const grid = document.getElementById("button-grid");
        
        // Fetch current configuration
        async function loadCompanion() {
            try {
                const res = await fetch("/api/config");
                const config = await res.json();
                const activePage = config.pages.find(p => p.name === config.active_page) || config.pages[0];
                
                grid.innerHTML = "";
                
                // APC Mini mk2 grid has 8 rows (7 to 0) and 8 columns (0 to 7)
                for (let r = 7; r >= 0; r--) {
                    for (let c = 0; c < 8; c++) {
                        const note = r * 8 + c;
                        const mapping = activePage.mappings[note.toString()];
                        
                        const btn = document.createElement("div");
                        btn.className = "btn";
                        if (mapping) {
                            btn.innerText = mapping.label || note;
                            btn.classList.add("active-btn");
                            
                            // Simple mapping state background glow
                            if (mapping.state) {
                                btn.style.background = "rgba(56, 189, 248, 0.2)";
                                btn.style.borderColor = "#38bdf8";
                            }
                        } else {
                            btn.innerText = note;
                            btn.style.opacity = "0.2";
                        }
                        
                        btn.onclick = () => {
                            fetch("/api/press?note=" + note);
                        };
                        
                        grid.appendChild(btn);
                    }
                }
            } catch(e) {
                grid.innerHTML = '<div style="grid-column: span 8; text-align: center; color: #ef4444;">Ladefehler</div>';
            }
        }

        loadCompanion();
        // Refresh grid status every 2 seconds
        setInterval(loadCompanion, 2000);
    </script>
</body>
</html>
"##;

    let html = html_template.replace("{active_page_name}", &active_page_name);

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());
}
