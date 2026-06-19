import type { LogEntry } from "../types";

interface LogPanelProps {
  logs: LogEntry[];
  logRef: React.RefObject<HTMLDivElement>;
}

export default function LogPanel({ logs, logRef }: LogPanelProps) {
  if (logs.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">📝</div>
        <div className="empty-state-text">暂无日志记录</div>
      </div>
    );
  }

  return (
    <div className="log-panel" ref={logRef}>
      {logs.map((log, index) => (
        <div key={index} className="log-entry">
          <span className="timestamp">[{log.timestamp}]</span>
          <span className={`log-${log.level}`}>{log.message}</span>
        </div>
      ))}
    </div>
  );
}
