import type { RefObject } from "react";

interface LogsViewProps {
  logs: string[];
  setLogs: (logs: string[]) => void;
  logEndRef: RefObject<HTMLDivElement | null>;
}

export function LogsView({ logs, setLogs, logEndRef }: LogsViewProps) {
  return (
    <div className="utility-window utility-logs animate-in">
      <div className="utility-hero">
        <div>
          <div className="utility-kicker">Live Stream</div>
          <h1>Terminal</h1>
          <p>Alle Ereignisse kompakt und lesbar auf einen Blick.</p>
        </div>
        <button type="button" onClick={() => setLogs([])}>
          Leeren
        </button>
      </div>
      <div className="log-panel">
        {logs.map((entry, index) => (
          <div key={index} className="log-line">
            {entry}
          </div>
        ))}
        <div ref={logEndRef} />
      </div>
    </div>
  );
}
