import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "../types";

interface SettingsViewProps {
  config: AppConfig;
  midiPorts: string[];
  midiOutputPorts: string[];
  saveConfig: (config: AppConfig) => Promise<void>;
  setConfig: (config: AppConfig) => void;
  showMessage: (message: string) => void;
}

export function SettingsView({
  config,
  midiPorts,
  midiOutputPorts,
  saveConfig,
  setConfig,
  showMessage,
}: SettingsViewProps) {
  const [newProcess, setNewProcess] = useState("");
  const [newPage, setNewPage] = useState("");
  const [showInfo, setShowInfo] = useState<Record<string, boolean>>({});
  const toggleInfo = (key: string) => {
    setShowInfo(prev => ({ ...prev, [key]: !prev[key] }));
  };

  const addProfile = () => {
    if (!newProcess.trim() || !newPage.trim()) return;
    const smart_profiles = [...(config.smart_profiles || [])];
    if (smart_profiles.some(p => p.process_name.toLowerCase() === newProcess.trim().toLowerCase())) {
      alert("Prozess existiert bereits!");
      return;
    }
    smart_profiles.push({
      process_name: newProcess.trim(),
      target_page: newPage.trim(),
    });
    void saveConfig({ ...config, smart_profiles });
    setNewProcess("");
    setNewPage("");
  };

  const removeProfile = (processName: string) => {
    const smart_profiles = (config.smart_profiles || []).filter(
      p => p.process_name !== processName
    );
    void saveConfig({ ...config, smart_profiles });
  };

  return (
    <div className="utility-window utility-settings animate-in">
      <div className="utility-hero">
        <div>
          <div className="utility-kicker">System Setup</div>
          <h1>Core Config</h1>
          <p>Hardware, MIDI und OBS sauber einrichten.</p>
        </div>
      </div>

      <div className="utility-grid">
        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["controller"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["controller"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["controller"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("controller")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>Controller</h2>
            </div>
            <span className="chip">{config.device_name || "Auto Scan"}</span>
          </div>

          {showInfo["controller"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 MIDI Controller-Setup</h4>
              Die App kommuniziert über zwei Kanäle mit deinem APC Mini:<br />
              • <b>MIDI Input:</b> Empfängt deine Tastendrücke und Fader-Bewegungen auf dem Controller.<br />
              • <b>MIDI Output:</b> Sendet Farbsignale zurück an deinen Controller, um die LEDs (z. B. OBS-Status, Pegel, Ripple) leuchten zu lassen.<br />
              <i>Tipp:</i> Wenn "Automatisch erkennen" gewählt ist, sucht die App nach Geräten mit "APC", "Akai" oder "Mini" im Namen.
            </div>
          )}

          <div className="grid-2-compact">
            <div>
              <label className="field-label">MIDI Input</label>
              <select
                value={config.device_name}
                onChange={(event) =>
                  void saveConfig({
                    ...config,
                    device_name: event.target.value,
                  })
                }
              >
                <option value="">Automatisch erkennen</option>
                {midiPorts.map((port) => (
                  <option key={port} value={port}>
                    {port}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="field-label">MIDI Output</label>
              <select
                value={config.output_device_name}
                onChange={(event) =>
                  void saveConfig({
                    ...config,
                    output_device_name: event.target.value,
                  })
                }
              >
                <option value="">Automatisch erkennen</option>
                {midiOutputPorts.map((port) => (
                  <option key={port} value={port}>
                    {port}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <p className="field-hint">Wähle Input (für Tasten) und Output (für LEDs) deines Controllers.</p>
        </section>

        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["obs"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["obs"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["obs"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("obs")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>OBS Studio</h2>
            </div>
            <span className={`chip ${config.obs.auto_connect ? "chip-on" : ""}`}>
              {config.obs.auto_connect ? "Auto-Link an" : "Auto-Link aus"}
            </span>
          </div>

          {showInfo["obs"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 OBS Studio-Verbindung</h4>
              Steuere OBS Studio live von deiner APC Mini aus!<br />
              • <b>Aktivierung:</b> Gehe in OBS zu <i>Werkzeuge ➔ WebSocket-Server-Einstellungen</i>.<br />
              • Aktiviere den WebSocket-Server (unter OBS v28+), trage den Port (Standard: 4455) und dein Kennwort ein.<br />
              • <b>Auto-Link:</b> Versucht bei jedem Start der App automatisch die Verbindung im Hintergrund herzustellen.
            </div>
          )}

          <div className="grid-2-compact">
            <div>
              <label className="field-label">Host</label>
              <input
                value={config.obs.host}
                onChange={(event) =>
                  void saveConfig({
                    ...config,
                    obs: { ...config.obs, host: event.target.value },
                  })
                }
                placeholder="127.0.0.1"
              />
            </div>
            <div>
              <label className="field-label">Port</label>
              <input
                type="number"
                value={config.obs.port}
                onChange={(event) =>
                  void saveConfig({
                    ...config,
                    obs: { ...config.obs, port: parseInt(event.target.value, 10) || 4455 },
                  })
                }
              />
            </div>
          </div>

          <label className="field-label">Passwort / Token</label>
          <input
            type="password"
            value={config.obs.password || ""}
            onChange={(event) =>
              void saveConfig({
                ...config,
                obs: { ...config.obs, password: event.target.value },
              })
            }
            placeholder="Security token"
          />

          <div className="utility-actions">
            <button
              type="button"
              className={`toggle-switch ${config.obs.auto_connect ? "on" : ""}`}
              onClick={() =>
                void saveConfig({
                  ...config,
                  obs: { ...config.obs, auto_connect: !config.obs.auto_connect },
                })
              }
            >
              Auto-Link {config.obs.auto_connect ? "ON" : "OFF"}
            </button>
            <button
              type="button"
              className="accent"
              onClick={async () => {
                try {
                  await invoke("connect_obs", {
                    host: config.obs.host,
                    port: config.obs.port,
                    password: config.obs.password,
                  });
                  showMessage("OBS verbunden.");
                } catch (error: any) {
                  alert(error.toString());
                }
              }}
            >
              Verbindung testen
            </button>
          </div>
        </section>

        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["webhook"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["webhook"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["webhook"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("webhook")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>Webhook Sync</h2>
            </div>
            <span className="chip">Cloud Config</span>
          </div>

          {showInfo["webhook"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 Webhook & Cloud Sync</h4>
              Verwalte deine Tastenbelegungen zentral im Web!<br />
              • Trage eine Web-URL zu einer <code>config.json</code> Datei ein (z. B. auf einem Server oder GitHub Gist).<br />
              • Ein Klick lädt das gesamte Setup (Buttons, Farben, Actions) live auf dein System. Perfekt für Backups oder Synchronisation über mehrere PCs!
            </div>
          )}

          <label className="field-label">Webhook URL</label>
          <input
            value={config.config_url || ""}
            onChange={(event) =>
              void saveConfig({
                ...config,
                config_url: event.target.value,
              })
            }
            placeholder="https://deine-api.com/config.json"
          />

          <div className="utility-actions">
            <button
              type="button"
              className="accent full-width"
              onClick={async () => {
                try {
                  const nextConfig = await invoke<AppConfig>("fetch_config", {
                    url: config.config_url,
                  });
                  setConfig(nextConfig);
                  showMessage("Konfiguration erfolgreich geladen!");
                } catch (error: any) {
                  alert(error.toString());
                }
              }}
            >
              Konfiguration jetzt holen
            </button>
          </div>
          <p className="field-hint">Lade dein gesamtes Setup (Buttons, Farben, Actions) direkt aus dem Web.</p>
        </section>

        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["ripple"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["ripple"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["ripple"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("ripple")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>LED Ripple-Effekt</h2>
            </div>
            <span className={`chip ${config.ripple_effect_enabled ? "chip-on" : ""}`}>
              {config.ripple_effect_enabled ? "Aktiviert" : "Deaktiviert"}
            </span>
          </div>

          {showInfo["ripple"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 LED Matrix Ripple-Effekt</h4>
              Ein non-blocking Lichteffekt für deine Tasten!<br />
              • Wenn aktiv, breitet sich beim Drücken eines Pads auf der 8x8 Grid-Matrix eine animierte Farbwelle aus.<br />
              • Zuerst leuchten die direkten Nachbar-Pads in <b>Cyan</b>, danach die äußeren Nachbar-Pads in <b>Magenta</b>, bevor sie wieder ihre Standardfarbe annehmen.
            </div>
          )}

          <div className="utility-actions">
            <button
              type="button"
              className={`toggle-switch ${config.ripple_effect_enabled ? "on" : ""}`}
              onClick={() =>
                void saveConfig({
                  ...config,
                  ripple_effect_enabled: !config.ripple_effect_enabled,
                })
              }
            >
              Ripple-Effekt {config.ripple_effect_enabled ? "An" : "Aus"}
            </button>
          </div>
          <p className="field-hint">Erzeugt beim Drücken eines Pads eine leuchtende Farbwelle auf der physischen LED-Matrix.</p>
        </section>

        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["media"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["media"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["media"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("media")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>Spotify & Mediensteuerung</h2>
            </div>
            <span className={`chip ${config.media_progress_enabled ? "chip-on" : ""}`}>
              {config.media_progress_enabled ? "Aktiv" : "Aus"}
            </span>
          </div>

          {showInfo["media"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 Spotify & System-Medien</h4>
              Echtes Live-Feedback für deine Musik!<br />
              • Nutzt die Windows-Systemsteuerung offline für Spotify, YouTube, VLC, Browser etc. ohne Login!<br />
              • Zeigt den Song-Fortschritt live als Leuchtbalken (in <b>Cyan</b>).<br />
              • Play/Pause-Pads (mit der Aktion <i>Medien: Play/Pause</i>) leuchten bei Wiedergabe <b>Grün</b> und bei Pause <b>Rot</b>.
            </div>
          )}

          <div>
            <label className="field-label">Wiedergabe-Fortschritt (Reihe)</label>
            <select
              value={config.media_progress_row}
              onChange={(event) =>
                void saveConfig({
                  ...config,
                  media_progress_row: parseInt(event.target.value, 10),
                })
              }
              className="full-width"
            >
              <option value={0}>Reihe 1 (Unten)</option>
              <option value={1}>Reihe 2</option>
              <option value={2}>Reihe 3</option>
              <option value={3}>Reihe 4</option>
              <option value={4}>Reihe 5</option>
              <option value={5}>Reihe 6</option>
              <option value={6}>Reihe 7</option>
              <option value={7}>Reihe 8 (Oben)</option>
            </select>
          </div>
          <div className="utility-actions" style={{ marginTop: "15px" }}>
            <button
              type="button"
              className={`toggle-switch ${config.media_progress_enabled ? "on" : ""}`}
              onClick={() =>
                void saveConfig({
                  ...config,
                  media_progress_enabled: !config.media_progress_enabled,
                })
              }
            >
              Fortschrittsanzeige {config.media_progress_enabled ? "ON" : "OFF"}
            </button>
          </div>
          <p className="field-hint">Zeigt den Song-Fortschritt live als leuchtenden Balken auf der ausgewählten Reihe deines APC Mini an.</p>
        </section>

        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["peak"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["peak"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["peak"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("peak")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>OBS Pegelanzeige</h2>
            </div>
            <span className={`chip ${config.obs_peak_meter_enabled ? "chip-on" : ""}`}>
              {config.obs_peak_meter_enabled ? "Aktiv" : "Inaktiv"}
            </span>
          </div>

          {showInfo["peak"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 OBS Pegelanzeige (Volume HUD)</h4>
              Deine OBS-Lautstärke live auf den APC-Pads!<br />
              • Trage den exakten Namen deiner OBS-Audioquelle (z. B. 'Desktop-Audio' oder 'Mic/Aux') ein.<br />
              • Wähle eine Spalte (1-8) auf deinem Controller. Die Pads dieser Spalte fungieren nun als Live-Dezibel-Meter (Grün/Gelb/Rot).
            </div>
          )}

          <div className="grid-2-compact">
            <div>
              <label className="field-label">OBS Audioquelle</label>
              <input
                value={config.obs_peak_meter_source || ""}
                onChange={(event) =>
                  void saveConfig({
                    ...config,
                    obs_peak_meter_source: event.target.value,
                  })
                }
                placeholder="z. B. Mic/Aux"
              />
            </div>
            <div>
              <label className="field-label">APC Fader-Spalte (1-8)</label>
              <input
                type="number"
                min="1"
                max="8"
                value={config.obs_peak_meter_column !== undefined ? config.obs_peak_meter_column + 1 : 8}
                onChange={(event) =>
                  void saveConfig({
                    ...config,
                    obs_peak_meter_column: (parseInt(event.target.value, 10) || 8) - 1,
                  })
                }
              />
            </div>
          </div>
          <div className="utility-actions">
            <button
              type="button"
              className={`toggle-switch ${config.obs_peak_meter_enabled ? "on" : ""}`}
              onClick={() =>
                void saveConfig({
                  ...config,
                  obs_peak_meter_enabled: !config.obs_peak_meter_enabled,
                })
              }
            >
              Peak Meter {config.obs_peak_meter_enabled ? "ON" : "OFF"}
            </button>
          </div>
          <p className="field-hint">Zeigt den Pegel deiner OBS-Quelle live auf einer der Pad-Spalten deines APC Mini.</p>
        </section>

        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["companion"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["companion"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["companion"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("companion")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>Web Companion</h2>
            </div>
            <span className={`chip ${config.web_companion_enabled ? "chip-on" : ""}`}>
              {config.web_companion_enabled ? "Bereit" : "Aus"}
            </span>
          </div>

          {showInfo["companion"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 Web Companion (Mobile Controller)</h4>
              Steuere deinen PC kabellos über dein Smartphone, Tablet oder einen Zweitbildschirm!<br />
              • Wenn der Server aktiv ist, hostet die App eine schlanke, responsive Steuerungsseite im lokalen Netzwerk.<br />
              • Rufe einfach die angezeigte IP-Adresse auf deinem Gerät auf, um alle hinterlegten Tasten virtuell aus der Ferne auszulösen.
            </div>
          )}

          <label className="field-label">Netzwerk-Port</label>
          <input
            type="number"
            value={config.web_companion_port || 1421}
            onChange={(event) =>
              void saveConfig({
                ...config,
                web_companion_port: parseInt(event.target.value, 10) || 1421,
              })
            }
          />
          <div className="utility-actions">
            <button
              type="button"
              className={`toggle-switch ${config.web_companion_enabled ? "on" : ""}`}
              onClick={() =>
                void saveConfig({
                  ...config,
                  web_companion_enabled: !config.web_companion_enabled,
                })
              }
            >
              Server {config.web_companion_enabled ? "Starten" : "Stoppen"}
            </button>
          </div>
          <p className="field-hint">Steuere deine Pads drahtlos über dein Handy oder Tablet unter <b>http://[IP]:{config.web_companion_port || 1421}</b></p>
        </section>

        <section className="utility-card">
          <div className="utility-card-head">
            <div style={{ display: "flex", alignItems: "center" }}>
              <button
                type="button"
                style={{
                  background: showInfo["smart"] ? "rgba(123, 240, 214, 0.15)" : "rgba(255,255,255,0.03)",
                  border: `1px solid ${showInfo["smart"] ? "var(--accent)" : "rgba(255,255,255,0.1)"}`,
                  borderRadius: "50%",
                  width: "24px",
                  height: "24px",
                  display: "inline-flex",
                  alignItems: "center",
                  justifyContent: "center",
                  padding: 0,
                  color: showInfo["smart"] ? "var(--accent)" : "var(--muted)",
                  cursor: "pointer",
                  fontSize: "12px",
                  fontWeight: "bold",
                  marginRight: "10px",
                  transition: "all 0.15s ease",
                }}
                onClick={() => toggleInfo("smart")}
                title="Hilfe anzeigen"
              >
                i
              </button>
              <h2>Smarte Profile</h2>
            </div>
            <span className={`chip ${config.smart_profiles_enabled ? "chip-on" : ""}`}>
              {config.smart_profiles_enabled ? "Aktiv" : "Aus"}
            </span>
          </div>

          {showInfo["smart"] && (
            <div className="info-explanation-box animate-in" style={{
              background: "rgba(123, 240, 214, 0.04)",
              border: "1px solid rgba(123, 240, 214, 0.15)",
              borderRadius: "12px",
              padding: "14px",
              marginBottom: "15px",
              fontSize: "13px",
              lineHeight: "1.5",
              color: "var(--muted)"
            }}>
              <h4 style={{ margin: "0 0 6px 0", color: "var(--accent)" }}>💡 Smarte Profile (Auto-Switch)</h4>
              Automatischer Layout-Wechsel je nach aktivem Windows-Fenster!<br />
              • Trage den Prozessnamen (z. B. <code>chrome.exe</code> oder <code>obs64.exe</code>) und die gewünschte Zielseite ein.<br />
              • Sobald du in Windows dieses Programm anklickst, wechselt dein APC Mini automatisch auf das passende Layout!
            </div>
          )}

          <div className="utility-actions">
            <button
              type="button"
              className={`toggle-switch ${config.smart_profiles_enabled ? "on" : ""}`}
              onClick={() =>
                void saveConfig({
                  ...config,
                  smart_profiles_enabled: !config.smart_profiles_enabled,
                })
              }
            >
              Seitenwechsel {config.smart_profiles_enabled ? "An" : "Aus"}
            </button>
          </div>
          <div style={{ marginTop: "15px" }}>
            <label className="field-label">Neues Profil hinzufügen</label>
            <div className="grid-2-compact" style={{ gap: "5px" }}>
              <input
                value={newProcess}
                onChange={(event) => setNewProcess(event.target.value)}
                placeholder="z. B. chrome.exe"
              />
              <input
                value={newPage}
                onChange={(event) => setNewPage(event.target.value)}
                placeholder="z. B. Web-Seite"
              />
            </div>
            <button
              type="button"
              className="accent full-width"
              style={{ marginTop: "10px", padding: "8px" }}
              onClick={addProfile}
            >
              Profil hinzufügen
            </button>
          </div>
          
          {(config.smart_profiles || []).length > 0 && (
            <div style={{ marginTop: "15px", borderTop: "1px solid rgba(255,255,255,0.05)", paddingTop: "10px" }}>
              <label className="field-label">Aktive Profile</label>
              <div style={{ display: "flex", flexDirection: "column", gap: "6px", marginTop: "5px" }}>
                {(config.smart_profiles || []).map((p) => (
                  <div key={p.process_name} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", background: "rgba(255,255,255,0.02)", padding: "6px 10px", borderRadius: "6px" }}>
                    <span style={{ fontSize: "13px" }}><b>{p.process_name}</b> ➔ {p.target_page}</span>
                    <button
                      type="button"
                      style={{ background: "rgba(239, 68, 68, 0.15)", color: "#ef4444", border: "none", borderRadius: "4px", padding: "3px 8px", cursor: "pointer", fontSize: "11px" }}
                      onClick={() => removeProfile(p.process_name)}
                    >
                      Löschen
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}
          <p className="field-hint" style={{ marginTop: "10px" }}>Wechselt das Layout automatisch, wenn das verknüpfte Programm aktiv ist.</p>
        </section>
      </div>

      <div className="utility-footer">
        <span className="field-hint">Änderungen werden beim Tippen gespeichert.</span>
      </div>
    </div>
  );
}
