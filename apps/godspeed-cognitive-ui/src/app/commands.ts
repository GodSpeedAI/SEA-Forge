import { causalMembers } from '../layout/layout'
import { focusOf, isPast, snapshotOf, type Store } from '../model/store'
import type { Id, ObjectAction, WorldObject } from '../model/types'
import { normalize } from '../narrative/localAgent'

// Composer routing. Everything resolves to the shared action vocabulary; questions go to the
// agent (if one is connected). Nothing here knows particular objects: targets are found in the
// current world by what the backend says about them (their actions, titles, artifacts).
// Returns a short hint when nothing matched.

export interface CommandContext {
  store: Store
  expand(ref: string): void
  explain(question: string): boolean
  enterDesign(): Promise<boolean>
  compareDefault(): boolean
  agentAttempt(object: Id, action: ObjectAction): void
}

/** The first object in the live world offering a consequential action of this intent. */
function offering(objects: Record<Id, WorldObject>, intent: ObjectAction['intent']) {
  for (const o of Object.values(objects)) {
    const a = o.actions?.find((x) => x.consequential && x.intent === intent)
    if (a) return { obj: o, action: a }
  }
  return null
}

export function runCommand(ctx: CommandContext, raw: string): string | null {
  const { store } = ctx
  const s = store.getState()
  const q = normalize(raw)
  if (!q) return null
  const dispatch = store.dispatch

  if (/^(continue|resume|go on|keep going)$/.test(q)) {
    if (s.narrative?.status === 'paused') dispatch({ type: 'resumeNarrative' })
    return s.narrative ? null : 'Nothing to continue.'
  }
  if (/^(home|core|go home)$/.test(q)) return dispatch({ type: 'home' }), null
  if (/^(back|go back|up)$/.test(q)) return dispatch({ type: 'back' }), null
  if (/\b(dark)( mode)?\b/.test(q) && q.length < 20) return dispatch({ type: 'setTheme', theme: 'dark' }), null
  if (/\b(light)( mode)?\b/.test(q) && q.length < 20) return dispatch({ type: 'setTheme', theme: 'light' }), null
  if (/^(now|return to now|back to now|live|return to live)$/.test(q)) return dispatch({ type: 'returnToNow' }), null
  if (/^(history|timeline|time|show time|show history)$/.test(q)) return dispatch({ type: 'openTimeline', open: true }), null
  if (/^(compare|compare versions|what changed since)\b/.test(q)) return ctx.compareDefault() ? null : 'Nothing to compare here yet.'
  if (/^(causal|causal view|show causes|cause and effect)$/.test(q)) {
    const f = focusOf(s)
    if (f && causalMembers(snapshotOf(s), f).length >= 2) return dispatch({ type: 'arrange', surface: 'causal' }), null
    return 'No causal reading is available here.'
  }
  if (/\b(design|edit the case|case design)\b/.test(q)) {
    void ctx.enterDesign().then((ok) => ok || undefined)
    return null
  }
  if (/\b(execution|the run|agent run|inspect run|what is the agent doing)\b/.test(q)) {
    if (!Object.keys(s.executions).length) return 'Nothing is executing.'
    dispatch({ type: 'setMode', mode: 'execution-inspect' })
    return null
  }
  // "let the agent approve" goes through the same intent path as a human decision (VAR-007).
  if (/\bagent\b.*\b(approve|sign off)\b|\b(approve|sign off)\b.*\bagent\b/.test(q)) {
    if (isPast(s)) return 'The past is read-only.'
    const hit = offering(snapshotOf(s).objects, 'APPROVE_HUMAN_TASK')
    if (!hit) return 'Nothing is waiting for approval.'
    ctx.agentAttempt(hit.obj.id, hit.action)
    return null
  }
  if (/\b(approve|sign off|release decision)\b/.test(q)) {
    if (isPast(s)) return 'The past is read-only.'
    const hit = offering(snapshotOf(s).objects, 'APPROVE_HUMAN_TASK')
    if (!hit) return 'Nothing is waiting for your approval.'
    if (hit.obj.parent && focusOf(s) !== hit.obj.parent) dispatch({ type: 'focus', id: hit.obj.parent })
    dispatch({ type: 'invoke', object: hit.obj.id, action: hit.action })
    return null
  }

  // "open release", "secondary coverage", "focus release" → focus by title.
  const target = q.replace(/^(open|focus|go to|show|show me|zoom into|enter)\s+/, '')
  const snap = snapshotOf(s)
  const hit =
    Object.values(snap.objects).find((o) => normalize(o.title) === target) ??
    Object.values(snap.objects).find((o) => o.kind !== 'core' && normalize(o.title).startsWith(target) && target.length >= 4)
  if (hit) {
    dispatch({ type: 'focus', id: hit.id })
    return null
  }
  const art = Object.values(snap.artifacts).find((a) => normalize(a.title) === target || (target.length >= 5 && normalize(a.title).includes(target)))
  if (art) {
    ctx.expand(art.ref)
    return null
  }
  if (ctx.explain(raw)) return null
  return s.agent === 'unavailable'
    ? 'The agent is unavailable. Everything else still works: click, zoom, open artifacts, use the time strip.'
    : 'Try “Why did the pilot fail?”, “What’s the evidence?”, “history”, or an object’s name.'
}
