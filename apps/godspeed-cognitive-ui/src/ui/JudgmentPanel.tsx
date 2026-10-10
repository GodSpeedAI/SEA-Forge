import { useEffect, useRef, useState } from 'react';
import type { JSX } from 'react';
import './judgment-panel.css';

export interface JudgmentOption {
  id: string;
  label: string;
  variant?: 'PRIMARY' | 'SECONDARY' | 'DANGER' | 'WARNING' | 'GHOST';
  requiresJustification?: boolean;
}

export interface JudgmentPanelProps {
  visible: boolean;
  question: string;
  description: string;
  reference?: string;
  options: JudgmentOption[];
  context: {
    icon: 'doc' | 'chart' | 'people';
    label: string;
    value: string;
    onOpen: () => void;
  }[];
  pending: boolean;
  outcome?: { state: 'accepted' | 'refused'; note?: string; code?: string; by: string } | null;
  authorityNote: string;
  onChoose(optionId: string, justification?: string): void;
  onClose(): void;
}

/**
 * The reason typed into the panel belongs to ONE decision. The panel stays mounted between
 * decisions, so the text is stored with the decision it was typed for and read back only for that
 * decision: a reason written for one judgment is never sent with the next (a governed record would
 * carry the wrong justification).
 */
export interface ReasonDraft {
  key: string;
  text: string;
}

export const decisionKeyOf = (question: string, options: readonly { id: string }[]): string =>
  `${question}\u0000${options.map((o) => o.id).join(',')}`;

export const reasonFor = (draft: ReasonDraft | null, key: string): string => (draft && draft.key === key ? draft.text : '');

export function JudgmentPanel(p: JudgmentPanelProps): JSX.Element {
  const panelRef = useRef<HTMLElement>(null);
  const decisionKey = decisionKeyOf(p.question, p.options);
  const [draft, setDraft] = useState<ReasonDraft | null>(null);
  const [errorFor, setErrorFor] = useState<ReasonDraft | null>(null);
  const justification = reasonFor(draft, decisionKey);
  const validationError = reasonFor(errorFor, decisionKey);
  const setJustification = (text: string) => setDraft({ key: decisionKey, text });
  const setValidationError = (text: string) => setErrorFor({ key: decisionKey, text });

  useEffect(() => {
    if (p.visible) panelRef.current?.focus();
    // A closed panel holds no reason: reopening the same decision starts from a blank page.
    else {
      setDraft(null);
      setErrorFor(null);
    }
  }, [p.visible]);

  const iconMap: Record<string, () => JSX.Element> = {
    doc: () => (
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        style={{ color: 'var(--ink-3)' }}
      >
        <path
          d="M3 2h7v10H3V2z"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <line
          x1="5"
          y1="5"
          x2="9"
          y2="5"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinecap="round"
        />
        <line
          x1="5"
          y1="8"
          x2="9"
          y2="8"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinecap="round"
        />
        <line
          x1="5"
          y1="11"
          x2="9"
          y2="11"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinecap="round"
        />
      </svg>
    ),
    chart: () => (
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        style={{ color: 'var(--ink-3)' }}
      >
        <rect
          x="2"
          y="8"
          width="2"
          height="6"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
        />
        <rect
          x="7"
          y="4"
          width="2"
          height="10"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
        />
        <rect
          x="12"
          y="6"
          width="2"
          height="8"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
        />
      </svg>
    ),
    people: () => (
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        style={{ color: 'var(--ink-3)' }}
      >
        <circle
          cx="5"
          cy="4"
          r="2"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
        />
        <path
          d="M2 8c0-1.1.9-2 2-2h2c1.1 0 2 .9 2 2v4H2V8z"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
        <circle
          cx="11"
          cy="4"
          r="2"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
        />
        <path
          d="M8 8c0-1.1.9-2 2-2h2c1.1 0 2 .9 2 2v4H8V8z"
          stroke="currentColor"
          strokeWidth="1.2"
          fill="none"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    ),
  };

  const handleChoose = (optionId: string) => {
    const option = p.options.find((o) => o.id === optionId);
    if (!option) return;

    if (option.requiresJustification && !justification.trim()) {
      setValidationError(`Reason required for "${option.label}"`);
      return;
    }

    setValidationError('');
    p.onChoose(optionId, justification || undefined);
  };

  const requiresJustificationOptions = p.options
    .filter((o) => o.requiresJustification)
    .map((o) => o.label);

  return (
    <aside
      ref={panelRef}
      tabIndex={-1}
      className={`judgment-panel ${p.visible ? 'visible' : ''}`}
      data-testid="judgment-panel"
      role="dialog"
      aria-label="Judgment needed"
    >
      <div className="judgment-panel__header">
        <div className="judgment-panel__header-left">
          <span className="judgment-panel__header-title">Judgment needed</span>
        </div>
        <div className="judgment-panel__header-right">
          {p.reference && (
            <span className="judgment-panel__reference">{p.reference}</span>
          )}
          <button
            className="judgment-panel__close-button"
            data-testid="judgment-close"
            aria-label="Close"
            onClick={p.onClose}
          >
            ×
          </button>
        </div>
      </div>

      <div className="judgment-panel__body">
        <h2 className="judgment-panel__question">{p.question}</h2>
        <p className="judgment-panel__description">{p.description}</p>

        <div className="judgment-panel__divider" />

        {p.pending && (
          <div className="judgment-panel__waiting" role="status">
            Waiting for authority…
          </div>
        )}

        {p.outcome && p.outcome.state === 'refused' && (
          <div className="judgment-panel__outcome" data-testid="judgment-outcome" data-state="refused">
            <div className="judgment-panel__outcome-title">Not permitted</div>
            {p.outcome.code && <div className="judgment-panel__outcome-code">{p.outcome.code}</div>}
            {p.outcome.note && <div className="judgment-panel__outcome-note">{p.outcome.note}</div>}
            <div className="judgment-panel__outcome-by">by {p.outcome.by}</div>
          </div>
        )}

        {p.outcome && p.outcome.state === 'accepted' && (
          <div className="judgment-panel__outcome" data-testid="judgment-outcome" data-state="accepted">
            <div className="judgment-panel__outcome-title">Accepted</div>
            {p.outcome.note && <div className="judgment-panel__outcome-note">{p.outcome.note}</div>}
            <div className="judgment-panel__outcome-by">by {p.outcome.by}</div>
          </div>
        )}

        {/* A refusal leaves the decision open: a fresh attempt is possible. */}
        {p.outcome?.state !== 'accepted' && (
          <>
            <div className="judgment-panel__options">
              {p.options.map((opt) => {
                const variantClass = opt.variant
                  ? opt.variant.toLowerCase()
                  : 'primary';

                return (
                  <button
                    key={opt.id}
                    className={`judgment-panel__option judgment-panel__option--${variantClass}`}
                    data-testid="judgment-option"
                    data-option={opt.id}
                    onClick={() => handleChoose(opt.id)}
                    disabled={p.pending}
                    aria-label={opt.label}
                  >
                    <span className="judgment-panel__option-label">{opt.label}</span>
                  </button>
                );
              })}
            </div>

            {requiresJustificationOptions.length > 0 && (
              <div className="judgment-panel__reason-block">
                <label className="judgment-panel__reason-label">
                  Reason
                  {requiresJustificationOptions.length > 0 && (
                    <span className="judgment-panel__reason-required">
                      (required for {requiresJustificationOptions.join(', ')})
                    </span>
                  )}
                </label>
                <textarea
                  className="judgment-panel__reason-input"
                  data-testid="judgment-reason"
                  value={justification}
                  onChange={(e) => {
                    setJustification(e.currentTarget.value);
                    setValidationError('');
                  }}
                  placeholder="Explain your reasoning…"
                  disabled={p.pending}
                  rows={3}
                />
                {validationError && (
                  <div className="judgment-panel__validation-error">{validationError}</div>
                )}
              </div>
            )}
          </>
        )}

        {p.outcome && p.outcome.state === 'accepted' && (
          <button
            className="judgment-panel__done-button"
            onClick={p.onClose}
          >
            Done
          </button>
        )}

        {p.context.length > 0 && (
          <>
            <div className="judgment-panel__divider" />
            <div className="judgment-panel__context">
              {p.context.map((ctx) => (
                <button
                  key={ctx.label}
                  className="judgment-panel__context-row"
                  onClick={ctx.onOpen}
                >
                  <div className="judgment-panel__context-left">
                    {iconMap[ctx.icon]()}
                    <span className="judgment-panel__context-label">
                      {ctx.label}
                    </span>
                  </div>
                  <div className="judgment-panel__context-right">
                    <span className="judgment-panel__context-value">
                      {ctx.value}
                    </span>
                    <svg
                      width="14"
                      height="14"
                      viewBox="0 0 14 14"
                      fill="none"
                      style={{ color: 'var(--ink-3)' }}
                    >
                      <path
                        d="M2 7h9M9 4l3 3-3 3"
                        stroke="currentColor"
                        strokeWidth="1.4"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                    </svg>
                  </div>
                </button>
              ))}
            </div>
          </>
        )}

        <div className="judgment-panel__footer">
          <div className="judgment-panel__authority-note">{p.authorityNote}</div>
        </div>
      </div>
    </aside>
  );
}
