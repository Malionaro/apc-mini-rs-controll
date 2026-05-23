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
            <h2>Controller</h2>
            <span className="chip">{config.device_name || "Auto Scan"}</span>
          </div>
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
            <h2>OBS Studio</h2>
            <span className={`chip ${config.obs.auto_connect ? "chip-on" : ""}`}>
              {config.obs.auto_connect ? "Auto-Link an" : "Auto-Link aus"}
            </span>
          </div>

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
            <h2>Webhook Sync</h2>
            <span className="chip">Cloud Config</span>
          </div>

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
      </div>

      <div className="utility-footer">
        <span className="field-hint">Änderungen werden beim Tippen gespeichert.</span>
      </div>
    </div>
  );
}
