import type { XObject } from '../ports/contract'
import type { ObjectKind, Tone } from './types'

// Derived presentation (T09, binding decision D-PRESENTATION): presentation is a UI concern, not
// kernel truth. Live snapshots carry contract objects WITHOUT the UI's `x` extensions — no
// presentation, tone or sentry explanation — so the UI derives them here from kernel fields
// (kind, status, depends_on + the dependencies' standing). Snapshots that DO carry `x` (the
// local fixture) keep the authored override: derivation applies only when `x` is silent.
// Nothing here recognises particular ids or names, and nothing here mutates anything.

/** Kernel object kind → presentation class. Unknown kinds read as items. */
const PRESENTATION_BY_KIND: Record<string, ObjectKind> = {
  work_item: 'item',
  stage: 'facet',
  milestone: 'facet',
  decision_gate: 'facet',
  discretionary_opportunity: 'item',
  execution_trace: 'run',
  evidence_record: 'item',
}

/**
 * Kernel status → semantic tone. Colour is never the only carrier; every tone also carries a
 * label (statusPhrase). ACTIVE/ready work is progress, settled work is ok, blocked or waiting
 * work needs attention, failed work is critical, optional work is hypothesis.
 */
const TONE_BY_STATUS: Record<string, Tone> = {
  ACTIVE: 'progress',
  IN_PROGRESS: 'progress',
  READY_TO_BEGIN: 'progress',
  COMPLETED: 'ok',
  ACTION_REQUIRED: 'attention',
  BLOCKED: 'attention',
  WAITING: 'attention',
  WAITING_ON_OTHERS: 'muted',
  FAILED: 'critical',
  REJECTED: 'critical',
  AVAILABLE_TO_ADD: 'hypothesis',
}

/** Kernel status → honest phrase, used only when the backend sent no badge (spec 04 axiom 1). */
const PHRASE_BY_STATUS: Record<string, string> = {
  ACTIVE: 'In progress',
  IN_PROGRESS: 'In progress',
  READY_TO_BEGIN: 'Ready to begin',
  COMPLETED: 'Completed',
  ACTION_REQUIRED: 'Needs your attention',
  BLOCKED: 'Blocked',
  WAITING: 'Waiting',
  WAITING_ON_OTHERS: 'Waiting on others',
  FAILED: 'Failed',
  REJECTED: 'Rejected',
  AVAILABLE_TO_ADD: 'Available to add if needed',
}

export function presentationOf(kind: string): ObjectKind {
  return PRESENTATION_BY_KIND[kind] ?? 'item'
}

export function statusTone(status: string): Tone {
  return TONE_BY_STATUS[status] ?? 'muted'
}

export function statusPhrase(status: string): string {
  return PHRASE_BY_STATUS[status] ?? status
}

/** A dependency whose standing is not COMPLETED still blocks this object. */
function unmetDependencies(o: XObject, byId: Record<string, XObject>): XObject[] {
  if (o.status === 'COMPLETED') return []
  return (o.depends_on ?? [])
    .map((id) => byId[id])
    .filter((d): d is XObject => !!d && d.status !== 'COMPLETED')
}

/**
 * Sentry transparency: an explanation derived from depends_on plus each dependency's standing,
 * e.g. "Waiting on Rollout (in progress)". Undefined when nothing unmet blocks the object (or
 * when the object itself is settled). The backend's own `explanation`, when present, wins.
 */
export function sentryExplanation(o: XObject, byId: Record<string, XObject>): string | undefined {
  const unmet = unmetDependencies(o, byId)
  if (!unmet.length) return undefined
  const parts = unmet.map((d) => `${d.name} (${statusPhrase(d.status).toLowerCase()})`)
  return `Waiting on ${parts.join(', ')}`
}

/** A usable tone for an object: the authored extension wins, else kernel status derives it. */
export function toneOf(o: XObject): Tone {
  return (o.x?.tone as Tone | undefined) ?? statusTone(o.status)
}
