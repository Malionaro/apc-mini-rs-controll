import type { CSSProperties } from "react";
import { padRows, padCols, sideKeys, trackKeys, type AppConfig } from "../types";

interface ApcGridProps {
  config: AppConfig;
  selectedNote: number | null;
  activeNote: number | null;
  setSelectedNote: (note: number | null) => void;
  setSelectedFader: (fader: number | null) => void;
}

export function ApcGrid({
  config,
  selectedNote,
  activeNote,
  setSelectedNote,
  setSelectedFader,
}: ApcGridProps) {
  const activePage =
    config.pages.find((page) => page.name === config.active_page) || config.pages[0];

  const renderPad = (index: number, defaultLabel: string, className = "") => {
    const mapping = activePage?.mappings[index.toString()];
    const hasActions = Boolean(mapping && mapping.actions.length > 0);
    const palette = mapping ? `hsl(${mapping.color * 12}, 78%, 58%)` : undefined;

    return (
      <button
        key={index}
        className={[
          "pad",
          className,
          selectedNote === index ? "selected" : "",
          hasActions ? "mapped" : "",
          activeNote === index ? "active-press" : "",
        ].join(" ")}
        style={palette ? ({ "--pad-tone": palette } as CSSProperties) : undefined}
        onClick={() => {
          setSelectedNote(index);
          setSelectedFader(null);
        }}
      >
        <span className="pad-label">{mapping?.label || defaultLabel}</span>
        {hasActions ? <span className="pad-dot" /> : null}
      </button>
    );
  };

  return (
    <div className="apc-hardware">
      <div className="hardware-shell">
        <div className="hardware-grid">
          <div className="grid-main-layout">
            {padRows.map((row, rowIndex) => (
              <div key={row} className="hardware-row">
                {padCols.map((col) => renderPad(row * 8 + col, (row * 8 + col).toString()))}
                {renderPad(sideKeys[rowIndex], `S${rowIndex + 1}`, "round side-btn")}
              </div>
            ))}

            <div className="hardware-row bottom-row track-keys">
              {trackKeys.map((key, index) => renderPad(key, `T${index + 1}`, "rect bottom-btn"))}
              {renderPad(122, "SHF", "round shift-btn")}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
