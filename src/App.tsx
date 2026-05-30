import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

import { ApcGrid } from "./components/ApcGrid";
import { Inspector } from "./components/Inspector";
import { SettingsView } from "./components/SettingsView";
import { LogsView } from "./components/LogsView";
import { createEmptyMapping, type Action, type AppConfig, type Mapping } from "./types";
import { checkForUpdate, installUpdate, type UpdateStatus } from "./updater";

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [selectedNote, setSelectedNote] = useState<number | null>(null);
  const [selectedFader, setSelectedFader] = useState<number | null>(null);
  const [isListening, setIsListening] = useState(false);
  const [isConnected, setIsConnected] = useState(false);
  const [isObsConnected, setIsObsConnected] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [midiPorts, setMidiPorts] = useState<string[]>([]);
  const [autoSelect, setAutoSelect] = useState(true);
  const [activeNote, setActiveNote] = useState<number | null>(null);
  const [newPageName, setNewPageName] = useState("");
  const [showNewPageInput, setShowNewPageInput] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [midiOutputPorts, setMidiOutputPorts] = useState<string[]>([]);
  const [updateInfo, setUpdateInfo] = useState<UpdateStatus | null>(null);
  const [updateProgress, setUpdateProgress] = useState<number | null>(null);
  
  const logEndRef = useRef<HTMLDivElement | null>(null);
  const autoSelectRef = useRef(autoSelect);
  const windowLabel = getCurrentWindow().label;
  const windowMode = new URLSearchParams(window.location.search).get("window");
  const isSettingsWindow = windowLabel === "settings" || windowMode === "settings";
  const isLogsWindow = windowLabel === "logs" || windowMode === "logs";

  useEffect(() => {
    autoSelectRef.current = autoSelect;
  }, [autoSelect]);

  useEffect(() => {
    let unlistenLog: (() => void) | undefined;
    let unlistenMidi: (() => void) | undefined;
    let unlistenStatus: (() => void) | undefined;
    let unlistenObsStatus: (() => void) | undefined;
    let unlistenActivePage: (() => void) | undefined;

    const setup = async () => {
      await loadConfig();

      if (isSettingsWindow) {
        await fetchMidiPorts();
      }
      if (isLogsWindow) {
        await fetchLogs();
      }

      // Auto-update check on startup (main window only)
      if (!isSettingsWindow && !isLogsWindow) {
        const status = await checkForUpdate();
        if (status.available) {
          setUpdateInfo(status);
        }
      }

      unlistenLog = await listen<string>("new-log", (event) => {
        setLogs((prev) => [...prev.slice(-99), event.payload]);
      });

      unlistenMidi = await listen<[number, number, number]>("midi-interaction", (event) => {
        const [status, data1, data2] = event.payload;

        if ((status & 0xF0) === 0x90 && data2 > 0) {
          setActiveNote(data1);
          window.setTimeout(() => setActiveNote(null), 200);

          if (autoSelectRef.current) {
            setSelectedNote(data1);
            setSelectedFader(null);
          }
        } else if ((status & 0xF0) === 0xB0 && autoSelectRef.current && data1 >= 48 && data1 <= 56) {
          setSelectedFader(data1 - 48);
          setSelectedNote(null);
        }
      });

      unlistenStatus = await listen<boolean>("connection-status", (event) => {
        setIsConnected(event.payload);
      });

      unlistenObsStatus = await listen<boolean>("obs-connection-status", (event) => {
        setIsObsConnected(event.payload);
      });

      unlistenActivePage = await listen<string>("active-page-changed", (event) => {
        setConfig((prevConfig) => {
          if (!prevConfig) return prevConfig;
          if (!prevConfig.pages.some((page) => page.name === event.payload)) return prevConfig;
          return { ...prevConfig, active_page: event.payload };
        });
        setSelectedNote(null);
        setSelectedFader(null);
      });
    };

    void setup();

    return () => {
      unlistenLog?.();
      unlistenMidi?.();
      unlistenStatus?.();
      unlistenObsStatus?.();
      unlistenActivePage?.();
    };
  }, [isSettingsWindow, isLogsWindow]);

  useEffect(() => {
    const checkObsStatus = async () => {
      try {
        const connected = await invoke<boolean>("get_obs_status");
        setIsObsConnected(connected);
      } catch (e) {
        console.error("Error checking OBS status:", e);
      }
    };

    void checkObsStatus();
    const interval = setInterval(checkObsStatus, 3000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: "auto" });
    }
  }, [logs]);

  const loadConfig = async () => {
    try {
      const nextConfig = await invoke<AppConfig>("get_config");
      setConfig(nextConfig);
    } catch (error) {
      console.error(error);
    }
  };

  const fetchLogs = async () => {
    try {
      const nextLogs = await invoke<string[]>("get_logs");
      setLogs(nextLogs);
    } catch (error) {
      console.error(error);
    }
  };

  const fetchMidiPorts = async () => {
    try {
      const nextInPorts = await invoke<string[]>("get_midi_ports");
      const nextOutPorts = await invoke<string[]>("get_midi_output_ports");
      setMidiPorts(nextInPorts);
      setMidiOutputPorts(nextOutPorts);
    } catch (error) {
      console.error(error);
    }
  };



  const saveConfig = async (newConfig: AppConfig) => {
    try {
      await invoke("update_config", { newConfig });
      setConfig(structuredClone(newConfig));
    } catch (error) {
      console.error(error);
    }
  };

  const switchPage = async (name: string) => {
    try {
      await invoke("set_active_page", { pageName: name });
      await loadConfig();
      setSelectedNote(null);
      setSelectedFader(null);
    } catch (error: any) {
      alert(error.toString());
    }
  };

  const toggleListener = async () => {
    try {
      const nextState = await invoke<boolean>("toggle_listener");
      setIsListening(nextState);
    } catch (error) {
      console.error(error);
    }
  };

  const showMessage = (message: string) => {
    setToast(message);
    window.setTimeout(() => setToast(null), 2000);
  };

  const activePage = config?.pages.find((page) => page.name === config.active_page) || config?.pages[0];
  
  const rawMapping =
    selectedNote !== null && activePage
      ? activePage.mappings[selectedNote.toString()] || createEmptyMapping()
      : null;

  const currentMapping = rawMapping
    ? {
        ...rawMapping,
        actions: (rawMapping.actions || [])
          .filter(Boolean)
          .map((act) => ({ ...act, type: act.type || "app" })),
      }
    : null;

  const createPage = async () => {
    const name = newPageName.trim();
    if (!config || !name) {
      return;
    }

    if (config.pages.some((page) => page.name.toLowerCase() === name.toLowerCase())) {
      showMessage("Seite existiert bereits.");
      return;
    }

    const nextConfig = structuredClone(config);
    nextConfig.pages.push({ name, mappings: {} });
    await saveConfig(nextConfig);
    setNewPageName("");
    setShowNewPageInput(false);
    await switchPage(name);
  };

  const updateSelectedMapping = (updater: (mapping: Mapping) => void) => {
    if (selectedNote === null) {
      return;
    }

    setConfig((prevConfig) => {
      if (!prevConfig) return null;
      const nextConfig = structuredClone(prevConfig);
      const pageIndex = nextConfig.pages.findIndex((page) => page.name === nextConfig.active_page);

      if (pageIndex === -1) {
        return prevConfig;
      }

      const page = nextConfig.pages[pageIndex];
      const key = selectedNote.toString();
      if (!page.mappings[key]) {
        page.mappings[key] = createEmptyMapping();
      }

      if (!page.mappings[key].actions) {
        page.mappings[key].actions = [];
      }
      page.mappings[key].actions = page.mappings[key].actions.filter(Boolean);
      page.mappings[key].actions.forEach((act) => {
        if (!act.type) act.type = "app";
      });

      updater(page.mappings[key]);
      void invoke("update_config", { newConfig: nextConfig }).catch(console.error);
      return nextConfig;
    });
  };

  const clearSelectedMapping = () => {
    if (selectedNote === null) {
      return;
    }

    setConfig((prevConfig) => {
      if (!prevConfig) return null;
      const nextConfig = structuredClone(prevConfig);
      const pageIndex = nextConfig.pages.findIndex((page) => page.name === nextConfig.active_page);

      if (pageIndex === -1) {
        return prevConfig;
      }

      delete nextConfig.pages[pageIndex].mappings[selectedNote.toString()];
      void invoke("update_config", { newConfig: nextConfig }).catch(console.error);
      return nextConfig;
    });
  };

  const updateActionType = (index: number, type: string) => {
    updateSelectedMapping((mapping) => {
      const currentAction = mapping.actions[index];
      if (!currentAction) {
        return;
      }

      const value =
        currentAction.path ||
        currentAction.url ||
        currentAction.media_key ||
        currentAction.obs_target ||
        currentAction.audio_path ||
        currentAction.text_content ||
        currentAction.system_command ||
        currentAction.target_page ||
        currentAction.webhook_url ||
        currentAction.keys?.join(", ") ||
        currentAction.midi_type ||
        currentAction.delay_ms?.toString() ||
        "";

      mapping.actions[index] = {
        type,
        path: type === "app" ? value : undefined,
        url: type === "url" ? value : undefined,
        keys: type === "hotkey" ? value.split(",").map((key) => key.trim()).filter(Boolean) : undefined,
        delay_ms: type === "wait" ? parseInt(value, 10) || 0 : undefined,
        midi_type: type === "midi" ? value : undefined,
        midi_note: undefined,
        midi_value: undefined,
        midi_channel: undefined,
        midi_device: type === "midi" ? currentAction.midi_device || "" : undefined,
        media_key: type === "media" ? value : undefined,
        obs_action: type === "obs" ? currentAction.obs_action || "SetScene" : undefined,
        obs_target: type.startsWith("obs") ? value : undefined,
        audio_path: type === "audio" ? value : undefined,
        audio_volume: type === "audio" ? currentAction.audio_volume || 100 : undefined,
        text_content: type === "text" ? value : undefined,
        system_command: (type === "system" || type === "mouse_click" || type === "mouse_move" || type === "mouse_scroll") ? value : undefined,
        target_page: type === "page" ? value : undefined,
        webhook_url: type === "webhook" ? value : undefined,
        webhook_method: type === "webhook" ? currentAction.webhook_method || "POST" : undefined,
        webhook_payload: type === "webhook" ? currentAction.webhook_payload || "" : undefined,
      };
    });
  };

  const updateActionValue = (index: number, value: string) => {
    updateSelectedMapping((mapping) => {
      const currentAction = mapping.actions[index];
      if (!currentAction) {
        return;
      }

      currentAction.path = undefined;
      currentAction.url = undefined;
      currentAction.obs_target = undefined;
      currentAction.audio_path = undefined;
      currentAction.text_content = undefined;
      currentAction.system_command = undefined;
      currentAction.target_page = undefined;
      currentAction.webhook_url = undefined;

      switch (currentAction.type) {
        case "url":
          currentAction.url = value;
          break;
        case "hotkey":
          currentAction.keys = value
            .split(",")
            .map((key) => key.trim())
            .filter(Boolean);
          break;
        case "wait":
          currentAction.delay_ms = parseInt(value, 10) || 0;
          break;
        case "midi":
          currentAction.midi_type = value;
          break;
        case "media":
          currentAction.media_key = value;
          break;
        case "obs":
        case "obs_vol":
        case "obs_toggle":
        case "obs_filter":
        case "obs_visible":
        case "obs_replay":
          currentAction.obs_target = value;
          break;
        case "audio":
          currentAction.audio_path = value;
          break;
        case "text":
          currentAction.text_content = value;
          break;
        case "system":
        case "mouse_click":
        case "mouse_move":
        case "mouse_scroll":
          currentAction.system_command = value;
          break;
        case "page":
          currentAction.target_page = value;
          break;
        case "webhook":
          currentAction.webhook_url = value;
          break;
        case "app":
        default:
          currentAction.path = value;
          break;
      }
    });
  };

  const updateSelectedActionField = (
    index: number,
    field: keyof Action,
    value: string | number | string[] | undefined
  ) => {
    updateSelectedMapping((mapping) => {
      const currentAction = mapping.actions[index];
      if (!currentAction) {
        return;
      }
      (currentAction as Record<keyof Action, unknown>)[field] = value as unknown;
    });
  };

  const updateActionFields = (index: number, fields: Partial<Action>) => {
    updateSelectedMapping((mapping) => {
      const currentAction = mapping.actions[index];
      if (!currentAction) {
        return;
      }
      Object.assign(currentAction, fields);
    });
  };

  if (!config) {
    return (
      <div className="loading-screen">
        <div className="loading-card">
          <div className="loading-title">APC MINI</div>
          <div className="loading-subtitle">Konfiguration wird geladen</div>
        </div>
      </div>
    );
  }

  if (isSettingsWindow) {
    return (
      <SettingsView
        config={config}
        midiPorts={midiPorts}
        midiOutputPorts={midiOutputPorts}
        saveConfig={saveConfig}
        setConfig={setConfig}
        showMessage={showMessage}
      />
    );
  }

  if (isLogsWindow) {
    return <LogsView logs={logs} setLogs={setLogs} logEndRef={logEndRef} />;
  }

  return (
    <div className="container">
      {toast ? <div className="toast">{toast}</div> : null}

      {updateInfo?.available && (
        <div className="update-banner">
          <div className="update-banner-content">
            <span className="update-icon">🚀</span>
            <span>
              <strong>Update verfügbar: v{updateInfo.version}</strong>
              {updateInfo.body ? <span className="update-body"> — {updateInfo.body}</span> : null}
            </span>
          </div>
          <div className="update-banner-actions">
            {updateProgress !== null ? (
              <div className="update-progress">
                <div className="update-progress-bar" style={{ width: `${updateProgress}%` }} />
                <span>{updateProgress === 100 ? "Installiert..." : `${updateProgress}%`}</span>
              </div>
            ) : (
              <>
                {updateInfo.downloadUrl ? (
                  <button
                    className="update-btn install"
                    onClick={() => {
                      setUpdateProgress(0);
                      void installUpdate(
                        updateInfo.downloadUrl!,
                        updateInfo.filename!,
                        (p) => setUpdateProgress(p)
                      );
                    }}
                  >
                    Jetzt installieren
                  </button>
                ) : (
                  <a
                    className="update-btn install"
                    href="https://github.com/Malionaro/apc-mini-rs-controll/releases/latest"
                    target="_blank"
                    rel="noreferrer"
                  >
                    Download öffnen
                  </a>
                )}
                <button className="update-btn dismiss" onClick={() => setUpdateInfo(null)}>
                  Später
                </button>
              </>
            )}
          </div>
        </div>
      )}

      <div className="left-panel">
        <div className="header">
          <div className="brand-block">
            <div>
              <div className="brand">APC MINI // MK2</div>
              <div className="brand-subtitle">Pad-Mapping, Fader und OBS-Steuerung</div>
            </div>
          </div>

          <div className={`connection-pill ${isConnected ? "online" : "offline"}`}>
            <span className="connection-dot" />
            {isConnected ? "Online" : "Offline"}
          </div>
        </div>

        <div className="header-toolbar">
          <div className="page-section">
            <div className="page-select-wrap">
              <select
                className="page-select"
                value={config.active_page}
                onChange={(event) => switchPage(event.target.value)}
              >
                {config.pages.map((page) => (
                  <option key={page.name} value={page.name}>
                    {page.name}
                  </option>
                ))}
              </select>
            </div>

            {!showNewPageInput && (
              <button
                type="button"
                className="icon-btn"
                onClick={() => {
                  setShowNewPageInput(true);
                }}
              >
                + Seite
              </button>
            )}
          </div>

          <div className="header-actions">
            <button
              type="button"
              className="settings-btn"
              onClick={() => void invoke("open_settings_window")}
            >
              Einstellungen
            </button>
            <button
              type="button"
              className="accent listener-btn"
              onClick={() => void toggleListener()}
            >
              {isListening ? "Stop" : "Start"}
            </button>
          </div>
        </div>

        {showNewPageInput && (
          <div className="toolbar-expand">
            <div className="inline-create">
              <input
                autoFocus
                value={newPageName}
                onChange={(event) => setNewPageName(event.target.value)}
                placeholder="Name der neuen Seite"
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    void createPage();
                  }
                  if (event.key === "Escape") {
                    setShowNewPageInput(false);
                    setNewPageName("");
                  }
                }}
              />
              <button
                type="button"
                className="small-button accent"
                onClick={() => void createPage()}
              >
                Erstellen
              </button>
              <button
                type="button"
                className="small-button"
                onClick={() => {
                  setShowNewPageInput(false);
                  setNewPageName("");
                }}
              >
                Abbrechen
              </button>
            </div>
          </div>
        )}

        <ApcGrid
          config={config}
          selectedNote={selectedNote}
          activeNote={activeNote}
          setSelectedNote={setSelectedNote}
          setSelectedFader={setSelectedFader}
        />

        <div className="status-bar">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={autoSelect}
              onChange={(event) => setAutoSelect(event.target.checked)}
            />
            <span>Auto-Select</span>
          </label>

          <div className="status-center">
            <span className={`status-badge ${isObsConnected ? "online" : "offline"}`} />
            <span className="connection-text">{isObsConnected ? "OBS Online" : "OBS Offline"}</span>
          </div>

          <button
            type="button"
            className="terminal-btn"
            onClick={() => void invoke("open_log_window")}
          >
            Terminal
          </button>
        </div>
      </div>

      <div className="right-panel">
        <Inspector
          selectedNote={selectedNote}
          selectedFader={selectedFader}
          config={config}
          currentMapping={currentMapping}
          updateActionType={updateActionType}
          updateActionValue={updateActionValue}
          updateSelectedActionField={updateSelectedActionField}
          updateActionFields={updateActionFields}
          updateSelectedMapping={updateSelectedMapping}
          clearSelectedMapping={clearSelectedMapping}
          saveConfig={saveConfig}
        />
      </div>
    </div>
  );
}

export default App;
