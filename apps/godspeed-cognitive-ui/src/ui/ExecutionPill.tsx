import type { JSX } from 'react';
import './execution-pill.css';

export interface ExecutionPillProps {
  visible: boolean;
  label: string;
  progress: number;
  state: 'running' | 'executed' | 'settled' | 'rejected';
  onOpen(): void;
}

export function ExecutionPill(p: ExecutionPillProps): JSX.Element {
  const statusLabel =
    p.state === 'running'
      ? `${p.label} · running ${Math.round(p.progress * 100)}%`
      : p.state === 'executed'
        ? `${p.label} · executed · awaiting settlement`
        : p.state === 'settled'
          ? `${p.label} · settled`
          : `${p.label} · settlement rejected`;

  return (
    <div
      className={`execution-pill ${p.visible ? 'visible' : ''}`}
      data-testid="execution-pill"
      data-state={p.state}
      role="status"
      aria-label={p.label}
    >
      <button className="execution-pill__button" onClick={p.onOpen}>
        {statusLabel}
      </button>
    </div>
  );
}
