import { useState } from 'react'
import type { JSX } from 'react'
import type { Actor } from '../model/types'
import './discretionary.css'

// The discretionary drawer (T09): proposes optional work anchored to a case/stage through
// ADD_DISCRETIONARY_WORK. Title and justification are mandatory; the proposer badge shows the
// actor the session resolved (never a client-asserted identity). The backend decides: what is
// submitted here is a proposal, not a mutation.

export interface DiscretionaryDrawerProps {
  visible: boolean
  /** Title of the case/stage the work anchors to. */
  targetTitle: string
  proposer: Actor | null
  pending: boolean
  outcome?: { state: 'accepted' | 'refused'; note?: string; code?: string } | null
  onSubmit(payload: { title: string; summary?: string; justification: string }): void
  onClose(): void
}

export function DiscretionaryDrawer(p: DiscretionaryDrawerProps): JSX.Element {
  const [title, setTitle] = useState('')
  const [summary, setSummary] = useState('')
  const [justification, setJustification] = useState('')
  const [validation, setValidation] = useState('')

  const submit = () => {
    if (!title.trim()) {
      setValidation('A title is required.')
      return
    }
    if (!justification.trim()) {
      setValidation('A justification is required: state why this work is needed.')
      return
    }
    setValidation('')
    p.onSubmit({ title: title.trim(), summary: summary.trim() || undefined, justification: justification.trim() })
  }

  const accepted = p.outcome?.state === 'accepted'
  return (
    <aside
      className={`discretionary-drawer ${p.visible ? 'visible' : ''}`}
      data-testid="discretionary-drawer"
      role="dialog"
      aria-label="Add discretionary work"
    >
      <div className="discretionary-header">
        <span className="discretionary-title">Add discretionary work</span>
        <button type="button" className="discretionary-close" data-testid="discretionary-close" aria-label="Close" onClick={p.onClose}>
          ×
        </button>
      </div>
      <div className="discretionary-body">
        <p className="discretionary-target">
          Anchored to <strong>{p.targetTitle}</strong>
        </p>

        {p.proposer && (
          <div className="discretionary-proposer" data-testid="discretionary-proposer">
            Proposed by <strong>{p.proposer.name}</strong> <span className="discretionary-proposer-id">{p.proposer.id}</span>
            <span className="discretionary-proposer-role">{p.proposer.role}</span>
          </div>
        )}

        {!accepted && (
          <>
            <label className="discretionary-field">
              <span>
                Title <em>required</em>
              </span>
              <input
                type="text"
                data-testid="discretionary-title"
                value={title}
                onChange={(e) => {
                  setTitle(e.currentTarget.value)
                  setValidation('')
                }}
                disabled={p.pending}
              />
            </label>
            <label className="discretionary-field">
              <span>Summary (optional)</span>
              <input
                type="text"
                data-testid="discretionary-summary"
                value={summary}
                onChange={(e) => setSummary(e.currentTarget.value)}
                disabled={p.pending}
              />
            </label>
            <label className="discretionary-field">
              <span>
                Justification <em>required</em>
              </span>
              <textarea
                data-testid="discretionary-justification"
                rows={3}
                value={justification}
                placeholder="Why is this work needed?"
                onChange={(e) => {
                  setJustification(e.currentTarget.value)
                  setValidation('')
                }}
                disabled={p.pending}
              />
            </label>
            {validation && (
              <div className="discretionary-validation" role="alert" data-testid="discretionary-validation">
                {validation}
              </div>
            )}
          </>
        )}

        {p.pending && (
          <div className="discretionary-waiting" role="status">
            Waiting for authority…
          </div>
        )}

        {p.outcome && (
          <div className="discretionary-outcome" data-testid="discretionary-outcome" data-state={p.outcome.state}>
            <strong>{accepted ? 'Proposed' : 'Not permitted'}</strong>
            {p.outcome.code && <code className="discretionary-outcome-code">{p.outcome.code}</code>}
            {p.outcome.note && <span>{p.outcome.note}</span>}
          </div>
        )}
      </div>
      <div className="discretionary-footer">
        {!accepted ? (
          <button type="button" className="discretionary-submit" data-testid="discretionary-submit" onClick={submit} disabled={p.pending}>
            Propose work
          </button>
        ) : (
          <button type="button" className="discretionary-submit" data-testid="discretionary-done" onClick={p.onClose}>
            Done
          </button>
        )}
        <span className="discretionary-note">A proposal crosses authority; the kernel decides.</span>
      </div>
    </aside>
  )
}
