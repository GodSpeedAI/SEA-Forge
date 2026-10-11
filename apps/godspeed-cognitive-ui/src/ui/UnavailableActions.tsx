import type { JSX } from 'react'
import './unavailable.css'

// Honest availability (T09): CJ10–CJ12 are not wired in this deployment. They render as static
// descriptors — plain list items, never buttons — so they cannot be dispatched, and nothing can
// accidentally treat them as offered actions. They disappear only when their journeys are proven
// live (T13).

export const UNAVAILABLE_ACTIONS: readonly { label: string; note: string }[] = [
  { label: 'Federation export / import', note: 'CJ10 · not wired in this deployment' },
  { label: 'Case memory promotion', note: 'CJ11 · not wired in this deployment' },
  { label: 'Gauntlet execution feed', note: 'CJ12 · not wired in this deployment' },
]

export interface UnavailableActionsProps {
  visible?: boolean
}

export function UnavailableActions(p: UnavailableActionsProps): JSX.Element {
  return (
    <div className="unavailable-actions" data-testid="unavailable-actions" data-visible={p.visible ?? true}>
      <span className="unavailable-heading">Not available here</span>
      <ul className="unavailable-list">
        {UNAVAILABLE_ACTIONS.map((a) => (
          <li key={a.label} className="unavailable-item" data-testid="unavailable-action" data-label={a.label}>
            <span className="unavailable-label">{a.label}</span>
            <span className="unavailable-note">{a.note}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
