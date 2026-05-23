import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ACTION_OPTIONS, ACTION_FIELD_MAP, type Action, type Mapping } from "../types";

interface ActionCardProps {
  action: Action;
  index: number;
  actionsCount: number;
  updateActionType: (index: number, type: string) => void;
  updateActionValue: (index: number, value: string) => void;
  updateSelectedActionField: (
    index: number,
    field: keyof Action,
    value: string | number | string[] | undefined
  ) => void;
  updateActionFields: (index: number, fields: Partial<Action>) => void;
  updateSelectedMapping: (updater: (mapping: Mapping) => void) => void;
}

export function ActionCard({
  action,
  index,
  actionsCount,
  updateActionType,
  updateActionValue,
  updateSelectedActionField,
  updateActionFields,
  updateSelectedMapping,
}: ActionCardProps) {
  const [localScenes, setLocalScenes] = useState<string[]>([]);
  const [localInputs, setLocalInputs] = useState<string[]>([]);
  const [localSources, setLocalSources] = useState<string[]>([]);
  const [localFilters, setLocalFilters] = useState<string[]>([]);

  const meta = ACTION_FIELD_MAP[action.type] || ACTION_FIELD_MAP.app;

  // Automatically fetch OBS scenes and inputs when action type is OBS
  useEffect(() => {
    let active = true;

    const fetchScenesAndInputs = async () => {
      if (!action.type.startsWith("obs")) return;
      try {
        const scenes = await invoke<string[]>("get_obs_scenes");
        if (active) setLocalScenes(scenes);
      } catch (err) {
        console.error("Error loading OBS scenes:", err);
        if (active) setLocalScenes([]);
      }
      try {
        const inputs = await invoke<string[]>("get_obs_inputs");
        if (active) setLocalInputs(inputs);
      } catch (err) {
        console.error("Error loading OBS inputs:", err);
        if (active) setLocalInputs([]);
      }
    };

    void fetchScenesAndInputs();

    return () => {
      active = false;
    };
  }, [action.type]);

  // Automatically fetch OBS sources or filters on mount or when related action fields change
  useEffect(() => {
    let active = true;

    const fetchSourcesAndFilters = async () => {
      if (!action.type.startsWith("obs")) return;

      const primaryValue = action.obs_target || "";
      const parts = primaryValue.split("|");

      // 1. Fetch sources if action targets scene/sources
      if (
        (action.type === "obs" &&
          (action.obs_action === "scene" ||
            action.obs_action === "SetScene" ||
            action.obs_action === "SetPreviewScene")) ||
        action.type === "obs_toggle" ||
        action.type === "obs_visible"
      ) {
        const scene = parts[0];
        if (scene) {
          try {
            const sources = await invoke<string[]>("get_obs_sources", { scene });
            if (active) {
              setLocalSources(sources);
            }
          } catch (err) {
            console.error("Error loading OBS sources for scene:", scene, err);
            if (active) setLocalSources([]);
          }
        } else {
          if (active) setLocalSources([]);
        }
      }

      // 2. Fetch filters if action targets filter
      if (action.type === "obs_filter") {
        const source = parts[0];
        if (source) {
          try {
            const filters = await invoke<string[]>("get_obs_filters", { source });
            if (active) {
              setLocalFilters(filters);
            }
          } catch (err) {
            console.error("Error loading OBS filters for source:", source, err);
            if (active) setLocalFilters([]);
          }
        } else {
          if (active) setLocalFilters([]);
        }
      }
    };

    void fetchSourcesAndFilters();

    return () => {
      active = false;
    };
  }, [action.type, action.obs_action, action.obs_target]);

  const getActionPrimaryValue = (act: Action) => {
    return (
      act.path ||
      act.url ||
      act.keys?.join(", ") ||
      act.delay_ms?.toString() ||
      act.midi_type ||
      act.media_key ||
      act.obs_target ||
      act.audio_path ||
      act.text_content ||
      act.system_command ||
      act.target_page ||
      ""
    );
  };

  const value = getActionPrimaryValue(action);

  const handlePickFile = async () => {
    try {
      const path = await invoke<string | null>("pick_file");
      if (path) {
        updateActionValue(index, path);
      }
    } catch (err) {
      console.error("Error picking file:", err);
    }
  };

  return (
    <div className="action-card">
      <div className="action-card-header">
        <div className="action-type-wrap">
          <select
            value={action.type.startsWith("obs") ? "obs" : action.type}
            onChange={(event) => {
              const selectedType = event.target.value;
              if (selectedType === "obs") {
                if (!action.type.startsWith("obs")) {
                  updateActionFields(index, {
                    type: "obs",
                    obs_action: "SetScene",
                    obs_target: ""
                  });
                }
              } else {
                updateActionType(index, selectedType);
              }
            }}
          >
            {ACTION_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.value.includes("obs")
                  ? "🎥 "
                  : option.value === "app"
                  ? "🚀 "
                  : option.value === "url"
                  ? "🌐 "
                  : option.value === "hotkey"
                  ? "⌨️ "
                  : option.value === "audio"
                  ? "🔊 "
                  : option.value === "midi"
                  ? "🎹 "
                  : "🔹 "}
                {option.label}
              </option>
            ))}
          </select>
          <div className="info-trigger">
            <i>i</i>
            <div className="info-tooltip">
              {ACTION_OPTIONS.find((o) => o.value === (action.type.startsWith("obs") ? "obs" : action.type))?.description}
            </div>
          </div>
        </div>
        <div className="action-header-btns">
          <button
            className="reorder-btn"
            disabled={index === 0}
            onClick={() => {
              updateSelectedMapping((mapping) => {
                const act = mapping.actions.splice(index, 1)[0];
                mapping.actions.splice(index - 1, 0, act);
              });
            }}
          >
            ↑
          </button>
          <button
            className="reorder-btn"
            disabled={index === actionsCount - 1}
            onClick={() => {
              updateSelectedMapping((mapping) => {
                const act = mapping.actions.splice(index, 1)[0];
                mapping.actions.splice(index + 1, 0, act);
              });
            }}
          >
            ↓
          </button>
          <button
            className="remove-btn"
            onClick={() => {
              updateSelectedMapping((mapping) => {
                mapping.actions.splice(index, 1);
              });
            }}
          >
            Entfernen
          </button>
        </div>
      </div>

      <div className="action-card-body">
        {action.type.startsWith("obs") ? (
          <div className="action-grid">
            <div className="obs-tabs">
              {[
                { id: "scene", label: "Szenen", icon: "🎬" },
                { id: "audio", label: "Audio", icon: "🔊" },
                { id: "sources", label: "Quellen", icon: "🖼️" },
                { id: "output", label: "Output", icon: "📡" },
              ].map((tab) => {
                const activeTab =
                  action.type === "obs_vol" || (action.type === "obs" && action.obs_action === "ToggleMute")
                    ? "audio"
                    : action.type === "obs_toggle" || action.type === "obs_filter" || action.type === "obs_visible"
                    ? "sources"
                    : action.type === "obs_replay" ||
                      (action.type === "obs" &&
                        (action.obs_action === "StartStopStream" ||
                          action.obs_action === "StartStopRecord" ||
                          action.obs_action === "ReplayBuffer"))
                    ? "output"
                    : "scene";
                
                const isTabActive = activeTab === tab.id;

                return (
                  <button
                    key={tab.id}
                    type="button"
                    className={`obs-tab ${isTabActive ? "active" : ""}`}
                    onClick={() => {
                      if (tab.id === "scene") {
                        updateActionFields(index, {
                          type: "obs",
                          obs_action: "SetScene",
                          obs_target: "",
                        });
                      } else if (tab.id === "audio") {
                        updateActionFields(index, {
                          type: "obs",
                          obs_action: "ToggleMute",
                          obs_target: "",
                        });
                      } else if (tab.id === "sources") {
                        updateActionFields(index, {
                          type: "obs_toggle",
                          obs_action: undefined,
                          obs_target: "",
                        });
                      } else if (tab.id === "output") {
                        updateActionFields(index, {
                          type: "obs",
                          obs_action: "StartStopStream",
                          obs_target: "",
                        });
                      }
                    }}
                  >
                    {tab.icon} {tab.label}
                  </button>
                );
              })}
            </div>
            <div>
              <label className="field-label">Aktion</label>
              <select
                value={
                  action.obs_action === "ReplayBuffer" && value === "toggle"
                    ? "replay_toggle"
                    : action.obs_action === "ReplayBuffer" && value === "save"
                    ? "replay_save"
                    : action.obs_action ||
                      (action.type === "obs_vol"
                        ? "obs_vol"
                        : action.type === "obs_toggle"
                        ? "obs_toggle"
                        : action.type === "obs_filter"
                        ? "obs_filter"
                        : action.type === "obs_visible"
                        ? "obs_visible"
                        : "SetScene")
                }
                onChange={(event) => {
                  const val = event.target.value;
                  if (val === "replay_toggle") {
                    updateActionFields(index, {
                      type: "obs",
                      obs_action: "ReplayBuffer",
                      obs_target: "toggle",
                    });
                  } else if (val === "replay_save") {
                    updateActionFields(index, {
                      type: "obs",
                      obs_action: "ReplayBuffer",
                      obs_target: "save",
                    });
                  } else if (val === "obs_vol") {
                    updateActionType(index, "obs_vol");
                  } else if (val === "obs_toggle") {
                    updateActionType(index, "obs_toggle");
                  } else if (val === "obs_filter") {
                    updateActionType(index, "obs_filter");
                  } else if (val === "obs_visible") {
                    updateActionType(index, "obs_visible");
                  } else {
                    updateActionFields(index, {
                      type: "obs",
                      obs_action: val,
                      obs_target: ["StartStopStream", "StartStopRecord", "ToggleStudioMode", "Transition"].includes(val) ? "" : undefined,
                    });
                  }
                }}
              >
                {(action.type === "obs" &&
                  (action.obs_action === "SetScene" ||
                    action.obs_action === "SetPreviewScene" ||
                    action.obs_action === "Transition" ||
                    action.obs_action === "ToggleStudioMode")) && (
                  <>
                    <option value="SetScene">Szene wechseln (Live)</option>
                    <option value="SetPreviewScene">Szene wechseln (Preview)</option>
                    <option value="Transition">Übergang (Transition)</option>
                    <option value="ToggleStudioMode">Studio Mode An/Aus</option>
                  </>
                )}
                {(action.type === "obs_vol" || (action.type === "obs" && action.obs_action === "ToggleMute")) && (
                  <>
                    <option value="ToggleMute">Mute togglen</option>
                    <option value="obs_vol">Lautstärke setzen</option>
                  </>
                )}
                {(action.type === "obs_toggle" ||
                  action.type === "obs_filter" ||
                  action.type === "obs_visible") && (
                  <>
                    <option value="obs_toggle">Quelle Umschalten</option>
                    <option value="obs_visible">Quelle Sichtbarkeit</option>
                    <option value="obs_filter">Filter Umschalten</option>
                  </>
                )}
                {(action.type === "obs_replay" ||
                  (action.type === "obs" &&
                    ["StartStopStream", "StartStopRecord", "ReplayBuffer"].includes(
                      action.obs_action || ""
                    ))) && (
                  <>
                    <option value="StartStopStream">Stream Start/Stop</option>
                    <option value="StartStopRecord">Record Start/Stop</option>
                    <option value="replay_toggle">Replay Buffer Start/Stop</option>
                    <option value="replay_save">Replay Buffer Speichern</option>
                  </>
                )}
              </select>
            </div>

            {action.type === "obs" &&
            [
              "ToggleStudioMode",
              "Transition",
              "StartStopStream",
              "StartStopRecord",
              "ReplayBuffer",
            ].includes(action.obs_action || "") ? null : (
              <div className="full-width">
                <div className="label-with-btn">
                  <label className="field-label">
                    {action.type === "obs_vol"
                      ? "Quelle & Volume (Name|%)"
                      : action.type === "obs_toggle" || action.type === "obs_visible"
                      ? "Szene & Quelle (Szene|Quelle)"
                      : action.type === "obs_filter"
                      ? "Quelle & Filter (Quelle|Filter)"
                      : "Ziel"}
                  </label>
                </div>

                {((action.type === "obs" &&
                  (action.obs_action === "scene" ||
                    action.obs_action === "SetScene" ||
                    action.obs_action === "SetPreviewScene")) ||
                  action.type === "obs_toggle" ||
                  action.type === "obs_visible") &&
                localScenes.length > 0 ? (
                  <div className="cascading-selects">
                    <select
                      value={value.split("|")[0]}
                      onChange={(event) => {
                        updateActionValue(index, event.target.value + "|");
                      }}
                    >
                      <option value="">Szene wählen...</option>
                      {localScenes.map((s) => (
                        <option key={s} value={s}>
                          {s}
                        </option>
                      ))}
                    </select>
                    {(action.type === "obs_toggle" || action.type === "obs_visible") && (
                      <select
                        value={value.split("|")[1] || ""}
                        onChange={(event) => {
                          const parts = value.split("|");
                          updateActionValue(
                            index,
                            parts[0] +
                              "|" +
                              event.target.value +
                              (parts[2] ? "|" + parts[2] : "")
                          );
                        }}
                      >
                        <option value="">Quelle wählen...</option>
                        {localSources.map((s) => (
                          <option key={s} value={s}>
                            {s}
                          </option>
                        ))}
                      </select>
                    )}
                    {action.type === "obs_visible" && (
                      <select
                        value={value.split("|")[2] || "1"}
                        onChange={(event) => {
                          const parts = value.split("|");
                          updateActionValue(
                            index,
                            parts[0] + "|" + (parts[1] || "") + "|" + event.target.value
                          );
                        }}
                      >
                        <option value="1">Sichtbar (AN)</option>
                        <option value="0">Versteckt (AUS)</option>
                      </select>
                    )}
                  </div>
                ) : action.type === "obs_filter" ? (
                  <div className="cascading-selects">
                    <select
                      value={value.split("|")[0]}
                      onChange={(event) => {
                        updateActionValue(index, event.target.value + "|");
                      }}
                    >
                      <option value="">Quelle wählen...</option>
                      {[...localScenes, ...localInputs].map((s) => (
                        <option key={s} value={s}>
                          {s}
                        </option>
                      ))}
                    </select>
                    <select
                      value={value.split("|")[1] || ""}
                      onChange={(event) => {
                        const parts = value.split("|");
                        updateActionValue(index, parts[0] + "|" + event.target.value);
                      }}
                    >
                      <option value="">Filter wählen...</option>
                      {localFilters.map((f) => (
                        <option key={f} value={f}>
                          {f}
                        </option>
                      ))}
                    </select>
                  </div>
                ) : ((action.type === "obs" &&
                    (action.obs_action === "mute" || action.obs_action === "ToggleMute")) ||
                    action.type === "obs_vol") &&
                  localInputs.length > 0 ? (
                  <div className="cascading-selects">
                    <select
                      value={value.split("|")[0]}
                      onChange={(event) => {
                        const parts = value.split("|");
                        updateActionValue(
                          index,
                          event.target.value + (parts[1] ? "|" + parts[1] : "")
                        );
                      }}
                    >
                      <option value="">Input wählen...</option>
                      {localInputs.map((i) => (
                        <option key={i} value={i}>
                          {i}
                        </option>
                      ))}
                    </select>
                    {action.type === "obs_vol" && (
                      <input
                        type="number"
                        min="0"
                        max="100"
                        value={value.split("|")[1] || 50}
                        onChange={(event) => {
                          const parts = value.split("|");
                          updateActionValue(index, (parts[0] || "") + "|" + event.target.value);
                        }}
                        placeholder="%"
                      />
                    )}
                  </div>
                ) : (
                  <div className="input-with-button">
                    <input
                      value={value}
                      onChange={(event) => updateActionValue(index, event.target.value)}
                      placeholder={meta.placeholder}
                    />
                    {(meta.placeholder.includes("C:") ||
                      ["app", "audio", "system"].includes(action.type)) && (
                      <button
                        type="button"
                        className="small-icon-btn"
                        title="Explorer öffnen"
                        onClick={handlePickFile}
                      >
                        <svg
                          width="20"
                          height="20"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        >
                          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                        </svg>
                      </button>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>
        ) : action.type === "hotkey" ? (
          <div>
            <label className="field-label">
              {meta.label}
              <div className="info-trigger mini">
                <i>i</i>
                <div className="info-tooltip">
                  Gib Tasten wie 'CTRL', 'SHIFT', 'ALT' oder 'WIN' ein. Beispiel: 'CTRL+SHIFT+P'.
                </div>
              </div>
            </label>
            <input
              value={action.keys?.join(", ") || ""}
              onChange={(event) =>
                updateSelectedActionField(
                  index,
                  "keys",
                  event.target.value
                    .split(",")
                    .map((key) => key.trim())
                    .filter(Boolean)
                )
              }
              placeholder={meta.placeholder}
            />
            <p className="field-hint">Kommagetrennt eingeben, z. B. `CTRL`, `ALT`, `L`.</p>
          </div>
        ) : action.type === "wait" ? (
          <div>
            <label className="field-label">
              {meta.label}
              <div className="info-trigger mini">
                <i>i</i>
                <div className="info-tooltip">
                  Die Verzögerung in Millisekunden, bevor die nächste Aktion ausgeführt wird.
                </div>
              </div>
            </label>
            <input
              type="number"
              value={action.delay_ms ?? 0}
              onChange={(event) =>
                updateSelectedActionField(
                  index,
                  "delay_ms",
                  parseInt(event.target.value, 10) || 0
                )
              }
              placeholder={meta.placeholder}
            />
          </div>
        ) : action.type === "midi" ? (
          <div className="action-grid midi-grid">
            <div>
              <label className="field-label">
                MIDI Typ
                <div className="info-trigger mini">
                  <i>i</i>
                  <div className="info-tooltip">
                    Mögliche Werte: 'note_on', 'note_off', 'cc'. Steuert die Art des MIDI-Signals.
                  </div>
                </div>
              </label>
              <input
                value={action.midi_type || ""}
                onChange={(event) =>
                  updateSelectedActionField(index, "midi_type", event.target.value)
                }
                placeholder="note_on"
              />
            </div>
            <div>
              <label className="field-label">MIDI Gerät</label>
              <input
                value={action.midi_device || ""}
                onChange={(event) =>
                  updateSelectedActionField(index, "midi_device", event.target.value)
                }
                placeholder="Device name"
              />
            </div>
            <div>
              <label className="field-label">Kanal</label>
              <input
                type="number"
                value={action.midi_channel ?? ""}
                onChange={(event) =>
                  updateSelectedActionField(
                    index,
                    "midi_channel",
                    parseInt(event.target.value, 10) || 0
                  )
                }
                placeholder="1"
              />
            </div>
            <div>
              <label className="field-label">Note</label>
              <input
                type="number"
                value={action.midi_note ?? ""}
                onChange={(event) =>
                  updateSelectedActionField(
                    index,
                    "midi_note",
                    parseInt(event.target.value, 10) || 0
                  )
                }
                placeholder="60"
              />
            </div>
            <div>
              <label className="field-label">Value</label>
              <input
                type="number"
                value={action.midi_value ?? ""}
                onChange={(event) =>
                  updateSelectedActionField(
                    index,
                    "midi_value",
                    parseInt(event.target.value, 10) || 0
                  )
                }
                placeholder="127"
              />
            </div>
          </div>
        ) : action.type === "audio" ? (
          <div className="action-grid">
            <div className="full-width">
              <label className="field-label">{meta.label}</label>
              <div className="input-with-button">
                <input
                  value={value}
                  onChange={(event) => updateActionValue(index, event.target.value)}
                  placeholder={meta.placeholder}
                />
                <button
                  type="button"
                  className="small-icon-btn"
                  title="Explorer öffnen"
                  onClick={handlePickFile}
                >
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                  </svg>
                </button>
              </div>
            </div>
            <div>
              <label className="field-label">Lautstärke</label>
              <input
                type="number"
                min="0"
                max="100"
                value={action.audio_volume ?? 100}
                onChange={(event) =>
                  updateSelectedActionField(
                    index,
                    "audio_volume",
                    parseInt(event.target.value, 10) || 100
                  )
                }
                placeholder="100"
              />
            </div>
          </div>
        ) : (
          <div>
            <label className="field-label">{meta.label}</label>
            <div className="input-with-button">
              <input
                value={value}
                onChange={(event) => updateActionValue(index, event.target.value)}
                placeholder={meta.placeholder}
              />
              {(meta.placeholder.includes("C:") ||
                ["app", "system"].includes(action.type)) && (
                <button
                  type="button"
                  className="small-icon-btn"
                  title="Explorer öffnen"
                  onClick={handlePickFile}
                >
                  <svg
                    width="20"
                    height="20"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                  </svg>
                </button>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
