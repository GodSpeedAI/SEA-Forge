import './workbench.css';

export interface ExecutionPanelProps {
  title: string;
  runId: string;
  phase: string;
  /** 0..1 when the stream reported progress; null when only snapshot standing is known. */
  progress: number | null;
  state: 'running' | 'executed' | 'settled' | 'rejected';
  connection?: 'live' | 'reconnecting' | 'interrupted';
  log: string[];
  evidence: { ref: string; title: string }[];
  settlement?: { decision: string; summary: string; at: string };
  onOpenEvidence(ref: string): void;
}

const phases = ['orchestrator', 'builder', 'critic', 'verifier', 'settling'];

function getPhaseIndex(phase: string): number {
  return phases.indexOf(phase);
}

export function ExecutionPanel(p: ExecutionPanelProps): JSX.Element {
  const currentPhaseIdx = getPhaseIndex(p.phase);
  const progress = p.connection && p.connection !== 'live' ? null : p.progress

  return (
    <div className="execution-panel" data-testid="execution-panel">
      <div className="exec-phases-list">
        {phases.map((phase, idx) => {
          const isDone = idx < currentPhaseIdx;
          const isCurrent = idx === currentPhaseIdx;

          return (
            <div
              key={phase}
              className={`exec-phase-item ${isDone ? 'done' : isCurrent ? 'current' : 'pending'}`}
              data-phase={phase}
            >
              <span className="exec-phase-icon">
                {isDone ? '✓' : isCurrent ? '◉' : '○'}
              </span>
              <span className="exec-phase-label">{phase}</span>
            </div>
          );
        })}
      </div>

      {p.connection === 'reconnecting' && <div role="status">Reconnecting to live updates. Showing the last received standing.</div>}
      {p.connection === 'interrupted' && <div role="alert">Live updates are interrupted. The displayed standing may be stale.</div>}

      <div className="exec-progress-container">
        <div className="exec-progress-bar">
          <div className="exec-progress-fill" style={{ width: progress === null ? '100%' : `${progress * 100}%`, opacity: progress === null ? 0.25 : 1 }} />
        </div>
        <span className="exec-progress-percent">{progress === null ? '—' : `${Math.round(progress * 100)}%`}</span>
      </div>

      <div className="exec-log-section">
        <div className="exec-log-header">Live Log</div>
        <div className="exec-terminal">
          {p.log.slice(-8).map((line, i) => (
            <div key={i} className="exec-log-line">
              <span className="exec-log-text">{line}</span>
            </div>
          ))}
        </div>
      </div>

      {p.evidence.length > 0 && (
        <div className="exec-evidence-section">
          <div className="exec-evidence-header">Evidence</div>
          <div className="exec-evidence-list">
            {p.evidence.map((ev) => (
              <button
                key={ev.ref}
                data-testid="execution-evidence"
                className="exec-evidence-button"
                onClick={() => p.onOpenEvidence(ev.ref)}
              >
                {ev.title}
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="exec-settlement-section" data-testid="settlement">
        {p.state === 'running' && (
          <div className="exec-status-running">Executing…</div>
        )}
        {p.state === 'executed' && (
          <div className="exec-status-executed">
            Executed · awaiting settlement
          </div>
        )}
        {p.state === 'settled' && p.settlement && (
          <div className="exec-status-settled">
            <div className="exec-settlement-decision">Settled · {p.settlement.decision}</div>
            <div className="exec-settlement-summary">{p.settlement.summary}</div>
            <div className="exec-settlement-time">{p.settlement.at}</div>
          </div>
        )}
        {p.state === 'rejected' && (
          <div className="exec-status-rejected">Settlement rejected</div>
        )}
      </div>
    </div>
  );
}
