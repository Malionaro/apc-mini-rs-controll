import { ActionCard } from "./ActionCard";
import type { Action, AppConfig, Mapping } from "../types";

interface InspectorProps {
  selectedNote: number | null;
  selectedFader: number | null;
  config: AppConfig;
  currentMapping: Mapping | null;
  updateActionType: (index: number, type: string) => void;
  updateActionValue: (index: number, value: string) => void;
  updateSelectedActionField: (
    index: number,
    field: keyof Action,
    value: string | number | string[] | undefined
  ) => void;
  updateActionFields: (index: number, fields: Partial<Action>) => void;
  updateSelectedMapping: (updater: (mapping: Mapping) => void) => void;
  clearSelectedMapping: () => void;
  saveConfig: (config: AppConfig) => Promise<void>;
}

export function Inspector({
  selectedNote,
  selectedFader,
  config,
  currentMapping,
  updateActionType,
  updateActionValue,
  updateSelectedActionField,
  updateActionFields,
  updateSelectedMapping,
  clearSelectedMapping,
  saveConfig,
}: InspectorProps) {
  if (selectedNote !== null && currentMapping) {
    return (
      <div className="animate-in config-view">
        <div className="config-header">
          <div>
            <div className="utility-kicker">Inspector</div>
            <h2>Pad {selectedNote}</h2>
          </div>
          <button
            type="button"
            className="accent action-btn"
            onClick={() => {
              updateSelectedMapping((mapping) => {
                mapping.actions.push({ type: "app", path: "" });
              });
            }}
          >
            + Action
          </button>
          <button type="button" className="action-btn" onClick={() => clearSelectedMapping()}>
            Reset
          </button>
        </div>

        <div className="selected-summary">
          <div className="summary-chip">
            <span>Label</span>
            <strong>{currentMapping.label || "Unbenannt"}</strong>
          </div>
          <div className="summary-chip">
            <span>Modus</span>
            <strong>{currentMapping.is_toggle ? "Toggle" : "Momentary"}</strong>
          </div>
          <div className="summary-chip">
            <span>Farbe</span>
            <strong>{currentMapping.color}</strong>
          </div>
          <div className="summary-chip">
            <span>Aktionen</span>
            <strong>{currentMapping.actions.length}</strong>
          </div>
        </div>

        <section className="settings-grid">
          <div className="form-group full">
            <label>Label</label>
            <input
              value={currentMapping.label || ""}
              onChange={(event) => {
                updateSelectedMapping((mapping) => {
                  mapping.label = event.target.value;
                });
              }}
              placeholder="Titel..."
            />
          </div>

          <div className="form-group">
            <label>Mode</label>
            <select
              className="dark-select"
              value={currentMapping.is_toggle ? "toggle" : "momentary"}
              onChange={(event) => {
                updateSelectedMapping((mapping) => {
                  mapping.is_toggle = event.target.value === "toggle";
                });
              }}
            >
              <option value="momentary">Momentary</option>
              <option value="toggle">Toggle</option>
            </select>
          </div>

          <div className="form-group">
            <label>
              Color
              <div className="info-trigger mini">
                <i>i</i>
                <div className="info-tooltip">
                  Der Farbcode (0-127), der auf dem Pad angezeigt wird. 1-3 = Grün/Rot/Gelb.
                </div>
              </div>
            </label>
            <input
              type="number"
              value={currentMapping.color}
              onChange={(event) => {
                updateSelectedMapping((mapping) => {
                  mapping.color = parseInt(event.target.value, 10) || 0;
                });
              }}
            />
          </div>
        </section>

        <div className="actions-header">
          <h3>Aktionen</h3>
          <span className="field-hint">Jede Aktion kann separat konfiguriert werden.</span>
        </div>

        <div className="actions-section">
          {currentMapping.actions.length > 0 ? (
            currentMapping.actions.map((action, index) => (
              <ActionCard
                key={index}
                action={action}
                index={index}
                actionsCount={currentMapping.actions.length}
                updateActionType={updateActionType}
                updateActionValue={updateActionValue}
                updateSelectedActionField={updateSelectedActionField}
                updateActionFields={updateActionFields}
                updateSelectedMapping={updateSelectedMapping}
              />
            ))
          ) : (
            <div className="empty-state">
              <strong>Noch keine Aktion definiert.</strong>
              <span>Füge eine Aktion hinzu, um den Pad direkt nutzbar zu machen.</span>
            </div>
          )}
        </div>
      </div>
    );
  }

  if (selectedFader !== null) {
    return (
      <div className="animate-in config-view">
        <div className="config-header">
          <div>
            <div className="utility-kicker">Inspector</div>
            <h2>Fader {selectedFader + 1}</h2>
          </div>
        </div>

        <section className="settings-grid">
          <div className="form-group full">
            <label>Systemzuweisung</label>
            <select
              className="dark-select"
              value={config.fader_mappings[selectedFader.toString()]?.type || "None"}
              onChange={(event) => {
                const nextConfig = structuredClone(config);
                nextConfig.fader_mappings[selectedFader.toString()] = {
                  type: event.target.value,
                };
                void saveConfig(nextConfig);
              }}
            >
              <option value="None">Nicht zugewiesen</option>
              <option value="volume">Master Volume</option>
            </select>
          </div>
        </section>
      </div>
    );
  }

  return (
    <div className="idle-state">
      <div className="idle-card">
        <div className="utility-kicker">Bereit</div>
        <h2>Wähle einen Pad oder Fader aus</h2>
        <p>Rechts erscheinen dann alle Einstellungen, Aktionen und Statusdaten für das Element.</p>
      </div>
    </div>
  );
}
