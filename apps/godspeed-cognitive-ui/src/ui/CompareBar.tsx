import './compare.css';

export interface CompareBarProps {
  visible: boolean;
  aLabel: string;
  bLabel: string;
  counts: { changed: number; added: number; removed: number; same: number };
  onSwap?(): void;
  onExit(): void;
}

export function CompareBar(p: CompareBarProps): JSX.Element {
  return (
    <div
      className={`compare-bar ${p.visible ? 'visible' : ''}`}
      data-testid="compare-bar"
      role="region"
      aria-label="Comparison"
    >
      <div className="compare-bar__text">
        Comparing A · <span className="compare-bar__label">{p.aLabel}</span>
        <span className="compare-bar__separator"> ↔ </span>
        B · <span className="compare-bar__label">{p.bLabel}</span>
      </div>

      <div className="compare-bar__legend">
        <div className="compare-bar__chip compare-bar__chip--changed">
          <span className="compare-bar__count">{p.counts.changed}</span> changed
        </div>
        <div className="compare-bar__chip compare-bar__chip--added">
          <span className="compare-bar__count">{p.counts.added}</span> added
        </div>
        <div className="compare-bar__chip compare-bar__chip--removed">
          <span className="compare-bar__count">{p.counts.removed}</span> removed
        </div>
        <div className="compare-bar__chip compare-bar__chip--same">
          <span className="compare-bar__count">{p.counts.same}</span> unchanged
        </div>
      </div>

      <div className="compare-bar__actions">
        {p.onSwap && (
          <button
            className="compare-bar__button"
            onClick={p.onSwap}
            aria-label="Swap comparison sides"
          >
            Swap
          </button>
        )}
        <button
          className="compare-bar__button"
          data-testid="compare-exit"
          onClick={p.onExit}
          aria-label="Exit compare mode"
        >
          Exit compare
        </button>
      </div>
    </div>
  );
}
