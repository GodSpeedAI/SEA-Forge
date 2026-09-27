import type { JSX } from 'react';
import './execution-pill.css';

export interface ExecutionPillProps {
  visible: boolean;
  label: string;
  /** 0..1 when the stream reported progress; null when only snapshot standing is known (never guessed). */
  progress: number | null;
  state: 'running' | 'executed' | 'settled' | 'rejected';
  /** True while the event stream is interrupted: the pill shows Reconnecting, not progress. */
  reconnecting?: boolean;
  onOpen(): void;
}

export function ExecutionPill(p: ExecutionPillProps): JSX.Element {
  // While reconnecting, the last snapshot standing is shown without a percentage: nothing is
  // known about the run beyond what the last revision said, and no progress may be fabricated.
  const percent = p.progress === null || p.reconnecting ? null : Math.round(p.progress * 100);
  const statusLabel = p.reconnecting
    ? `${p.label} · reconnecting`
    : p.state === 'running'
      ? `${p.label} · running${percent !== null ? ` ${percent}%` : ''}`
      : p.state === 'executed'
        ? `${p.label} · executed · awaiting settlement`
        : p.state === 'settled'
          ? `${p.label} · settled`
          : `${p.label} · settlement rejected`;

  return (
    <div
      className={`execution-pill ${p.visible ? 'visible' : ''} ${p.reconnecting ? 'reconnecting' : ''}`}
      data-testid="execution-pill"
      data-state={p.reconnecting ? 'reconnecting' : p.state}
      role="status"
      aria-label={p.label}
    >
      <button className="execution-pill__button" onClick={p.onOpen}>
        {statusLabel}
      </button>
    </div>
  );
}
