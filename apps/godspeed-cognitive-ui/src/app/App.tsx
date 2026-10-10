import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from 'react'
import { createArtifactService, useArtifact, type ArtifactService } from '../artifacts/service'
import { loadedRenderers } from '../artifacts/registry'
import { causalMembers, childrenOf } from '../layout/layout'
import { createStore, expandedArtifact, focusOf, initialState, isPast, nowRevision, snapshotOf, type Store } from '../model/store'
import type { Actor, Id, ObjectAction, Theme, UiState, WorldHistory } from '../model/types'
import { findArtifact, sideLabel, worldView } from '../model/view'
import { explain } from '../narrative/conduct'
import { startNarrativePlayer } from '../narrative/player'
import type { CaseworkPort, CognitiveArtifact, NarrationPort, SessionIdentity, TemplateSourcePort, XSnapshot } from '../ports/contract'
import { loadCaseHistory } from '../ports/project'
import { SceneRuntime } from '../scene/runtime'
import { Scene, snapshotView, type SceneServices } from '../scene/Scene'
import { ArtifactDock } from '../ui/ArtifactDock'
import { CaseDesignPanel } from '../ui/CaseDesignPanel'
import { Brand, Breadcrumb, CoreAnchorLabel, StatusBar, UserMark } from '../ui/Chrome'
import { CompareBar } from '../ui/CompareBar'
import { Composer } from '../ui/Composer'
import { DiscretionaryDrawer } from '../ui/DiscretionaryDrawer'
import { ExecutionPanel } from '../ui/ExecutionPanel'
import { ExecutionPill } from '../ui/ExecutionPill'
import { JudgmentPanel } from '../ui/JudgmentPanel'
import { OutlineView, type OutlineItem } from '../ui/OutlineView'
import { SessionBadge } from '../ui/SessionBadge'
import { TemplateDesignPanel } from '../ui/TemplateDesignPanel'
import { TimeStrip } from '../ui/TimeStrip'
import { UnavailableActions } from '../ui/UnavailableActions'
import { WorkbenchChrome } from '../ui/WorkbenchChrome'
import { runCommand } from './commands'
import { designModel, isDirty, moveItem, toggleRequired } from './design'
import { createIntentPath, judgmentChoices, type IntentPath } from './intents'
import { connectLive } from './live'
import { createProposalFlow, emptyWorldOffersDesign, loadAndFocusCase } from './proposals'

export interface AppProps {
  port: CaseworkPort
  agent: NarrationPort | null
  history: WorldHistory
  raw: XSnapshot[]
  summaries: Record<string, string>
  human: Actor
  agentActor: Actor
  /** The session the adapter resolved (T09). The chrome names it; intents act as it. */
  session: SessionIdentity | null
  onLogout?(): void
}

interface Env {
  store: Store
  runtime: SceneRuntime
  artifacts: ArtifactService
  intents: IntentPath
}

function boot(p: AppProps): Env {
  const params = new URLSearchParams(location.search)
  const state = initialState(p.history)
  const theme = params.get('theme') as Theme | null
  if (theme === 'dark' || theme === 'light') state.theme = theme
  if (!p.agent) state.agent = 'unavailable'
  const store = createStore(state)
  const runtime = new SceneRuntime(store)
  runtime.expandedSource = (ref) => store.getState().artifacts.find((a) => a.id === ref)?.source ?? findArtifact(store.getState(), ref)?.boundObject
  const artifacts = createArtifactService(p.port)
  const intents = createIntentPath(store, p.port)
  startNarrativePlayer(store, undefined, Number(params.get('beatPace') ?? 1) || 1)

  // Deep links (verification and E2E setup only; every one of these is also reachable by UI).
  const focus = params.get('focus')
  if (focus) store.dispatch({ type: 'focus', id: focus })
  const surface = params.get('surface')
  if (surface === 'causal' || surface === 'orbital') store.dispatch({ type: 'arrange', surface })
  const time = params.get('time')
  if (time) store.dispatch({ type: 'setTime', revision: time })
  if (params.get('timeline')) store.dispatch({ type: 'openTimeline', open: true })
  if (params.get('awake')) store.dispatch({ type: 'wake', awake: true })
  const judge = params.get('judge')
  if (judge) {
    const action = snapshotOf(store.getState()).objects[judge]?.actions?.find((a) => a.consequential)
    if (action) store.dispatch({ type: 'invoke', object: judge, action })
  }
  if (focus || surface || judge) runtime.camera = { ...runtime.result.camera }
  runtime.flight = null
  const w = window as unknown as { __gs: unknown }
  w.__gs = { store, runtime, loadedRenderers, artifacts, worldView: () => worldView(store.getState()), ready: true }
  return { store, runtime, artifacts, intents }
}

export function App(p: AppProps) {
  const env = useMemo(() => boot(p), [p])
  const { store, runtime, artifacts, intents } = env
  const state = useSyncExternalStore(store.subscribe, store.getState)
  const [hint, setHint] = useState<string | undefined>()
  const [outlineOpen, setOutlineOpen] = useState(false)
  const [savedDraft, setSavedDraft] = useState(false)

  // The event stream follows the case the history shows (PROPOSE_CASE moves it to the new case).
  const caseId = state.history.caseId
  const rawByCase = useRef(new Map<string, XSnapshot[]>([[p.history.caseId, p.raw]]))
  const summariesRef = useRef(p.summaries)
  useEffect(() => {
    const disconnect = connectLive(p.port, store, caseId, rawByCase.current.get(caseId) ?? [], summariesRef.current)
    return disconnect
  }, [caseId, p.port, store])

  /** The template surface, when this connection offers one (additive port; T08/T09). */
  const templateSource = 'getTemplates' in p.port && 'preflightTemplate' in p.port ? (p.port as CaseworkPort & TemplateSourcePort) : null

  const actorFor = (): Actor =>
    p.session
      ? { id: p.session.actor_id, role: p.session.role, name: p.session.display_name ?? p.session.user ?? p.session.actor_id, kind: p.session.kind ?? 'human' }
      : p.human

  /** Loads a case's history and focuses it (used after PROPOSE_CASE commits a new case). */
  const focusCase = async (newId: Id) => {
    const raw = await loadAndFocusCase(p.port, store, newId, actorFor())
    rawByCase.current.set(newId, raw)
    Object.assign(summariesRef.current, Object.fromEntries(raw.map((s) => [s.cursor, s.summary.phase])))
  }
  // The case-design proposal flow (T09): template picker → parameters → preflight → commit.
  const proposalFlow = useMemo(
    () => createProposalFlow({ store, port: p.port, actor: actorFor, focusCase }),
    // The flow closes over stable store/port references; actor/focusCase are re-created per
    // render but the store reads them through deps at call time.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [store, p.port],
  )

  // Templates load when the picker opens (status returns to 'loading').
  useEffect(() => {
    if (state.proposal?.status !== 'loading') return
    if (!templateSource) {
      store.dispatch({ type: 'proposalsUnavailable', error: 'This connection offers no template source.' })
      return
    }
    let alive = true
    templateSource
      .getTemplates()
      .then((templates) => {
        if (alive) store.dispatch({ type: 'proposalsLoaded', templates })
      })
      .catch((e: unknown) => {
        if (alive) store.dispatch({ type: 'proposalsUnavailable', error: e instanceof Error ? e.message : String(e) })
      })
    return () => {
      alive = false
    }
  }, [state.proposal?.status, templateSource, store])

  // Discretionary drawer outcome tracking (the drawer is not the judgment panel).
  const [drawerPending, setDrawerPending] = useState(false)
  const [drawerOutcome, setDrawerOutcome] = useState<{ state: 'accepted' | 'refused'; note?: string; code?: string } | null>(null)
  const drawerKey = state.drawer ? `${state.drawer.object}:${state.drawer.action.id}` : null
  useEffect(() => {
    if (drawerKey) setDrawerOutcome(null)
  }, [drawerKey])

  const submitDiscretionary = async (payload: { title: string; summary?: string; justification: string }) => {
    const drawer = store.getState().drawer
    if (!drawer) return
    setDrawerPending(true)
    try {
      const rec = await intents.submit(actorFor(), drawer.object, drawer.action, {
        justification: payload.justification,
        judgment: false,
        parameters: {
          case_id: store.getState().history.caseId,
          stage_id: drawer.object,
          kind: 'work_item',
          title: payload.title,
          ...(payload.summary ? { summary: payload.summary } : {}),
          justification: payload.justification,
        },
      })
      setDrawerOutcome(rec.state === 'accepted' ? { state: 'accepted', note: rec.note } : rec.state === 'refused' ? { state: 'refused', note: rec.note, code: rec.code } : null)
    } finally {
      setDrawerPending(false)
    }
  }

  useEffect(() => {
    document.documentElement.dataset.theme = state.theme
  }, [state.theme])

  const expand = (ref: string) => {
    if (store.getState().narrative?.status === 'playing') store.dispatch({ type: 'pauseNarrative' })
    store.dispatch({ type: 'expandArtifact', id: ref, snapshot: snapshotView(store, runtime) })
  }

  const enterDesign = async (): Promise<boolean> => {
    const s = store.getState()
    const snap = snapshotOf(s)
    // The focused object or its nearest ancestor that has a designable template.
    let id: Id | null | undefined = focusOf(s)
    while (id && !snap.objects[id]?.templateCaseId) id = snap.objects[id]?.parent
    const tpl = id ? snap.objects[id]?.templateCaseId : undefined
    if (!tpl) {
      setHint('Focus a case to design its template.')
      return false
    }
    const { history } = await loadCaseHistory(p.port, tpl, { actor_id: p.human.id, role: p.human.role as never, display_name: p.human.name, kind: 'human' }, 'local-contract', { withCore: false })
    setSavedDraft(false)
    store.dispatch({ type: 'enterDesign', design: { caseId: tpl, history, revision: nowRevision(history), draft: null, returnFocus: s.focusStack, selected: null } })
    return true
  }

  const compareDefault = (): boolean => {
    const s = store.getState()
    if (s.timeMarks.length === 2) {
      store.dispatch({ type: 'openCompare', a: s.timeMarks[0]!, b: s.timeMarks[1]! })
      return true
    }
    const revs = s.history.revisions
    const i = revs.findIndex((r) => r.id === s.revision)
    const a = i > 0 ? revs[i - 1]!.id : null
    if (!a) return false
    store.dispatch({ type: 'openCompare', a, b: s.revision })
    return true
  }

  const agentAttempt = (object: Id, action: ObjectAction) => {
    const s = store.getState()
    if (!s.judgment || s.judgment.object !== object) store.dispatch({ type: 'invoke', object, action })
    void intents.submit(p.agentActor, object, action, { judgment: true })
  }

  const services: SceneServices = {
    artifacts,
    invoke(object, action) {
      if (action.intent === 'OPEN_ARTIFACT' || action.intent === 'RESOLVE_SOURCE') {
        const ref = snapshotOf(store.getState()).objects[object]?.artifacts?.[0]
        if (ref) expand(ref)
        return
      }
      if (action.consequential) store.dispatch({ type: 'invoke', object, action })
      else void intents.submit(p.human, object, action)
    },
    centerActions(s) {
      const v = worldView(s)
      const f = focusOf(s)
      if (!f && s.mode === 'world' && !s.judgment && templateSource && emptyWorldOffersDesign(v.snap.objects)) {
        return [{ id: 'design', label: 'Design case', onClick: () => void proposalFlow.open() }]
      }
      if (!f || s.mode !== 'world' || s.judgment || expandedArtifact(s)) return []
      const obj = v.snap.objects[f]
      const out: ReturnType<SceneServices['centerActions']> = []
      if (causalMembers(v.snap, f).length >= 2 || s.surface === 'causal') {
        out.push({ id: 'causal', label: s.surface === 'causal' ? 'Orbital view' : 'Causal view', pressed: s.surface === 'causal', onClick: () => store.dispatch({ type: 'arrange', surface: s.surface === 'causal' ? 'orbital' : 'causal' }) })
      }
      out.push({ id: 'history', label: s.timeline ? 'Hide history' : 'History', pressed: s.timeline, onClick: () => store.dispatch({ type: 'openTimeline', open: !s.timeline }) })
      if (!s.compare && s.history.revisions.length > 1) out.push({ id: 'compare', label: 'Compare with earlier', onClick: () => void compareDefault() })
      // Design entry (T09): a fixture-linked template world opens the design surface; otherwise
      // the template picker loads from the port (the live path).
      if (obj?.templateCaseId) out.push({ id: 'design', label: 'Design case', onClick: () => void enterDesign() })
      else out.push({ id: 'design', label: 'Design case', onClick: () => void proposalFlow.open() })
      return out
    },
  }

  // Keyboard map (§25): Esc back, H home, arrows pan, +/- zoom, D theme, T history, [ ] time, O outline.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const typing = (e.target as HTMLElement).closest('input, textarea, [contenteditable]')
      if (e.key === 'Escape') {
        store.dispatch({ type: 'back' })
        return
      }
      if (e.key.toLowerCase() === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        document.querySelector<HTMLInputElement>('input[aria-label="Ask GodSpeed"]')?.focus()
        return
      }
      if (typing) return
      const s = store.getState()
      const pan = 60
      switch (e.key) {
        case 'h':
        case 'Home':
          store.dispatch({ type: 'home' })
          break
        case 'ArrowLeft':
          runtime.input.pan(pan, 0)
          break
        case 'ArrowRight':
          runtime.input.pan(-pan, 0)
          break
        case 'ArrowUp':
          runtime.input.pan(0, pan)
          break
        case 'ArrowDown':
          runtime.input.pan(0, -pan)
          break
        case '+':
        case '=':
          runtime.input.zoom(1.25, window.innerWidth / 2, window.innerHeight * 0.46)
          break
        case '-':
          runtime.input.zoom(0.8, window.innerWidth / 2, window.innerHeight * 0.46)
          break
        case 'd':
          store.dispatch({ type: 'setTheme', theme: s.theme === 'dark' ? 'light' : 'dark' })
          break
        case 't':
          store.dispatch({ type: 'openTimeline', open: !s.timeline })
          break
        case 'o':
          setOutlineOpen((v) => !v)
          break
        case '[':
        case ']': {
          const revs = s.history.revisions
          const i = revs.findIndex((r) => r.id === s.revision)
          const next = revs[Math.max(0, Math.min(revs.length - 1, i + (e.key === '[' ? -1 : 1)))]!
          store.dispatch({ type: 'setTime', revision: next.id })
          break
        }
        case ' ': {
          const nodeId = (e.target as HTMLElement).closest('[data-node]')?.getAttribute('data-node')
          if (nodeId) {
            e.preventDefault()
            store.dispatch({ type: 'focus', id: nodeId })
            break
          }
          if (s.narrative) {
            e.preventDefault()
            store.dispatch({ type: s.narrative.status === 'playing' ? 'pauseNarrative' : 'resumeNarrative' })
          }
          break
        }
        case 'Enter': {
          const id = (e.target as HTMLElement).closest('[data-node]')?.getAttribute('data-node')
          if (id) store.dispatch({ type: 'focus', id })
          break
        }
        default:
          return
      }
      runtime.input.activity()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [store, runtime])

  const view = worldView(state)
  const focus = focusOf(state)
  const home = focus === null
  const workbench = state.mode !== 'world'
  const chromeVisible = !workbench && (!home || state.awake || state.theme === 'dark')
  const snapNow = snapshotOf(state)
  const path = ['You', ...(home ? ['Home'] : state.focusStack.map((id) => snapNow.objects[id]?.title ?? id))]
  const past = isPast(state)
  const nowRev = state.history.revisions.find((r) => r.id === state.revision)!
  const exp = expandedArtifact(state)
  const expDescriptor = exp ? findArtifact(state, exp.id) : null
  const paused = state.narrative?.status === 'paused'
  const narr = state.narrative
  const judgment = state.judgment
  const judgedObj = judgment ? snapNow.objects[judgment.object] : undefined
  const liveSnap = state.history.snapshots[nowRevision(state.history)]!

  // Execution shown in the pill/panel: the most recent one that is not yet settled, else the last.
  const execs = Object.values(state.executions)
  const exec = execs.find((e) => e.state === 'running' || e.state === 'executed') ?? execs.at(-1) ?? null
  const execTarget = exec ? liveSnap.objects[exec.object] : undefined
  const execEvidence = exec
    ? childrenOf(liveSnap, exec.object).filter((c) => c.contractKind === 'evidence_record').flatMap((c) => (c.artifacts ?? []).map((ref) => ({ ref, title: liveSnap.artifacts[ref]?.title ?? ref })))
    : []

  // A decision the backend accepted closes the panel after a beat so the outcome can be read.
  useEffect(() => {
    if (judgment?.outcome?.state !== 'accepted') return
    const h = setTimeout(() => store.dispatch({ type: 'closeJudgment' }), 2200)
    return () => clearTimeout(h)
  }, [judgment?.outcome?.state, store])

  const composerHint =
    hint ??
    (narr?.error
      ? 'The agent disconnected — explore directly, or ask again later'
      : paused
        ? 'Paused — type “continue” to resume'
        : undefined)

  // Judgment context: evidence bound to the judged object and to what it depends on.
  const judgmentContext = (() => {
    if (!judgedObj) return []
    const refs = new Set<string>(judgedObj.artifacts ?? [])
    for (const r of snapNow.relationships) if (r.from === judgedObj.id && r.kind === 'depends') for (const a of snapNow.objects[r.to]?.artifacts ?? []) refs.add(a)
    return [...refs].slice(0, 3).flatMap((ref) => {
      const d = snapNow.artifacts[ref]
      if (!d) return []
      const icon: 'doc' | 'chart' | 'people' = d.kind === 'data_table' || d.kind === 'test_report' ? 'chart' : d.kind === 'decision_record' ? 'people' : 'doc'
      return [{ icon, label: d.title, value: KIND_LABEL[d.kind] ?? d.kind, onOpen: () => expand(ref) }]
    })
  })()

  // Case design.
  const design = state.design
  const designSnap = view.design ? view.snap : null
  const designVersions = design
    ? [...design.history.revisions.map((r) => ({ id: r.id, label: r.summary || r.label })), ...(design.draft ? [{ id: 'draft', label: 'Local draft' }] : [])]
    : []
  const editDraft = (edit: (snap: NonNullable<typeof designSnap>) => NonNullable<typeof designSnap>) => {
    const d = store.getState().design
    if (!d) return
    const base = d.draft ?? d.history.snapshots[nowRevision(d.history)]!
    setSavedDraft(false)
    store.dispatch({ type: 'designDraft', draft: edit(base) })
  }

  const compareCounts = view.compare
    ? Object.values(view.compare.changes).reduce(
        (acc, c) => {
          const inView = true
          if (inView) acc[c.change === 'same' ? 'same' : c.change]++
          return acc
        },
        { changed: 0, added: 0, removed: 0, same: 0 },
      )
    : { changed: 0, added: 0, removed: 0, same: 0 }

  return (
    <>
      <Scene store={store} runtime={runtime} state={state} services={services} />
      {/* Screen readers hear the current thought and anything that needs a decision. */}
      <div className="visually-hidden" aria-live="polite" role="status" data-testid="live-region">
        {state.overrides.caption ?? ''}
        {judgment && !judgment.outcome ? ` Needs your judgment: ${judgedObj?.title ?? ''}.` : ''}
        {judgment?.outcome ? ` Decision ${judgment.outcome.state}. ${judgment.outcome.note ?? ''}` : ''}
        {past ? ` Viewing an earlier state (read-only).` : ''}
        {view.compare ? ` Comparing ${view.compare.a.label} with ${view.compare.b.label}.` : ''}
      </div>
      <Brand visible={chromeVisible} />
      <Breadcrumb path={path} visible={chromeVisible} />
      <CoreAnchorLabel visible={!home && !workbench} onHome={() => store.dispatch({ type: 'home' })} />
      <StatusBar state={past ? 'past' : state.history.provenance === 'local-contract' ? 'local' : 'live'} when={formatWhen(nowRev.at)} visible={chromeVisible} />
      <UserMark visible={chromeVisible} />
      <div className="chrome-element session-badge-container" data-visible={chromeVisible}>
        <SessionBadge session={p.session} onLogout={p.onLogout ? () => p.onLogout!() : undefined} />
      </div>
      <UnavailableActions visible={outlineOpen || workbench} />
      <OutlineView
        visible={outlineOpen}
        title={snapNow.objects[focus ?? 'core']?.title ?? 'Home'}
        caption={state.overrides.caption}
        timeLabel={past ? `${formatWhen(nowRev.at)} (read-only)` : 'Now'}
        items={outlineItems(state, focus)}
        onFocus={(id) => store.dispatch({ type: 'focus', id })}
        onAction={(id, actionId) => {
          const action = snapNow.objects[id]?.actions?.find((a) => a.id === actionId)
          if (action) services.invoke(id, action)
        }}
        onClose={() => setOutlineOpen(false)}
      />
      <TimeStrip
        revisions={state.history.revisions}
        current={state.revision}
        visible={state.mode === 'world' && (past || state.timeline || (!!state.compare && state.compare.source === 'world'))}
        marks={state.timeMarks}
        comparing={state.compare?.source === 'world' ? { a: state.compare.a, b: state.compare.b } : null}
        onSeek={(id) => store.dispatch({ type: 'setTime', revision: id })}
        onReturnToNow={() => store.dispatch({ type: 'returnToNow' })}
        onMark={(id) => store.dispatch({ type: 'markTime', revision: id })}
        onCompare={(a, b) => store.dispatch({ type: 'openCompare', a, b })}
        onClose={() => store.dispatch({ type: 'openTimeline', open: false })}
      />
      <CompareBar
        visible={!!view.compare}
        aLabel={view.compare?.a.label ?? ''}
        bLabel={view.compare?.b.label ?? ''}
        counts={compareCounts}
        onExit={() => store.dispatch({ type: 'closeCompare' })}
      />
      <WorkbenchChrome
        visible={workbench}
        brandSub={state.mode === 'case-design' ? ['Complexity in context.', 'Work with clarity.'] : ['Casework in context.', 'Work with clarity.']}
        activeRail="cases"
        breadcrumb={
          state.mode === 'case-design'
            ? ['Cases', ...state.focusStack.map((id) => snapNow.objects[id]?.title ?? id), 'Design']
            : ['Cases', ...state.focusStack.map((id) => snapNow.objects[id]?.title ?? id)]
        }
        title={state.mode === 'case-design' ? (designSnap ? designModel(designSnap, '').name : 'Case') : execTarget?.title ?? 'Execution'}
        subtitle={state.mode === 'case-design' ? 'Case template' : exec ? `${exec.runId} · ${exec.phase}` : 'No execution'}
        badge={state.mode === 'case-design' ? { label: 'Design Mode', tone: 'progress' } : undefined}
        status={
          state.mode === 'case-design'
            ? { label: design?.draft ? 'Local draft' : 'Published', tone: design?.draft ? 'attention' : 'ok' }
            : { label: exec?.state === 'settled' ? 'Settled' : exec?.state === 'executed' ? 'Awaiting settlement' : exec?.state === 'failed' ? 'Stopped' : 'In progress', tone: exec?.state === 'settled' ? 'ok' : exec?.state === 'failed' ? 'attention' : 'progress' }
        }
        rail={[
          { id: 'home', label: 'Home', onSelect: () => store.dispatch({ type: 'home' }) },
          { id: 'search', label: 'Search' },
          { id: 'cases', label: 'Cases', onSelect: () => store.dispatch({ type: 'setMode', mode: 'world' }) },
          { id: 'inbox', label: 'Inbox' },
          { id: 'agents', label: 'Agents' },
          { id: 'artifacts', label: 'Artifacts', onSelect: () => setOutlineOpen(true) },
          {
            id: 'timeline',
            label: 'Timeline',
            onSelect: () => {
              store.dispatch({ type: 'setMode', mode: 'world' })
              store.dispatch({ type: 'openTimeline', open: true })
            },
          },
          { id: 'governance', label: 'Governance' },
          { id: 'settings', label: 'Settings' },
        ]}
        tabs={[{ label: state.mode === 'case-design' ? 'Graph' : 'Run', onSelect: () => undefined }]}
        activeTab={state.mode === 'case-design' ? 'Graph' : 'Run'}
        onTab={() => undefined}
        onClose={() => store.dispatch({ type: 'setMode', mode: 'world' })}
        right={
          state.mode === 'case-design' && design && designSnap ? (
            <CaseDesignPanel
              model={designModel(designSnap, design.revision === 'draft' ? 'Local draft' : design.history.revisions.find((r) => r.id === design.revision)?.summary || sideLabel(design.history, design.revision))}
              isDraft={design.revision === 'draft'}
              dirty={isDirty(design.draft, design.history) && !savedDraft}
              versions={designVersions}
              showing={design.revision}
              selected={design.selected}
              onShowVersion={(id) => store.dispatch({ type: 'designShow', revision: id })}
              onCompare={(a, b) => store.dispatch({ type: 'openCompare', a, b, source: 'design' })}
              onMoveStage={(id, dir) => editDraft((s) => moveItem(s, id, dir))}
              onToggleEvidence={(id) => editDraft((s) => toggleRequired(s, id))}
              onSaveDraft={() => setSavedDraft(true)}
              onDiscardDraft={() => {
                setSavedDraft(false)
                store.dispatch({ type: 'designDraft', draft: null })
              }}
              onClose={() => store.dispatch({ type: 'setMode', mode: 'world' })}
              submitDisabledReason="Submitting template proposals needs a contract operation that does not exist yet; the draft stays local."
            />
          ) : exec ? (
            <ExecutionPanel
              title={execTarget?.title ?? exec.object}
              runId={exec.runId}
              phase={exec.phase}
              progress={exec.progress}
              state={exec.state}
              connection={state.connection}
              log={exec.log}
              evidence={execEvidence}
              settlement={
                execTarget?.settlement
                  ? { decision: execTarget.settlement.decision, summary: execTarget.settlement.consequence_summary, at: formatWhen(execTarget.settlement.settled_at) }
                  : undefined
              }
              onOpenEvidence={(ref) => {
                store.dispatch({ type: 'setMode', mode: 'world' })
                expand(ref)
              }}
            />
          ) : (
            <div className="execution-empty">Nothing is executing.</div>
          )
        }
      />
      <Composer
        variant={home ? 'home' : 'focused'}
        placeholder={
          composerHint ??
          (state.mode === 'case-design' ? 'Ask, navigate, or build something...' : workbench ? 'Ask, navigate, or give the next instruction...' : undefined)
        }
        hint={undefined}
        onSubmit={(text) => {
          const h = runCommand({ store, expand, explain: (q) => store.getState().agent === 'available' && explain(store, p.agent, q), enterDesign, compareDefault, agentAttempt }, text)
          setHint(h ?? undefined)
          if (h) setTimeout(() => setHint(undefined), 5000)
        }}
      />
      <JudgmentPanel
        visible={!!judgment && !past}
        question={judgedObj && judgment ? `${judgment.action.label}: ${judgedObj.title}${judgedObj.subtitle ? ` ${judgedObj.subtitle}` : ''}?` : ''}
        description={judgedObj?.status?.label ?? ''}
        reference={judgedObj?.subtitle}
        options={judgmentChoices(judgedObj?.actions ?? [])}
        context={judgmentContext}
        pending={!!judgment?.pending}
        outcome={judgment?.outcome ? { state: judgment.outcome.state, note: judgment.outcome.note, code: judgment.outcome.code, by: judgment.outcome.by.name } : undefined}
        authorityNote={`Authority decides · ${state.history.provenance === 'local-contract' ? 'local contract adapter (not the Go service)' : 'Go system front end'}`}
        onChoose={(optionId, justification) => {
          const action = judgedObj?.actions?.find((a) => a.id === optionId)
          if (judgment && action) void intents.submit(p.human, judgment.object, action, { justification, judgment: true })
        }}
        onClose={() => store.dispatch({ type: 'closeJudgment' })}
      />
      <DiscretionaryDrawer
        visible={!!state.drawer}
        targetTitle={state.drawer ? snapNow.objects[state.drawer.object]?.title ?? state.drawer.object : ''}
        proposer={p.session ? actorFor() : p.human}
        pending={drawerPending}
        outcome={drawerOutcome}
        onSubmit={(payload) => void submitDiscretionary(payload)}
        onClose={() => store.dispatch({ type: 'closeDrawer' })}
      />
      {state.proposal && (
        <TemplateDesignPanel
          proposal={state.proposal}
          onSelectTemplate={(ref) => proposalFlow.select(ref)}
          onSetParam={(name, value) => proposalFlow.setParam(name, value)}
          onPreflight={() => void proposalFlow.preflight()}
          onSubmit={() => void proposalFlow.submit()}
          onClose={() => proposalFlow.close()}
        />
      )}
      <ExecutionPill
        visible={(!!exec || state.connection !== 'live') && state.mode === 'world' && !judgment}
        label={execTarget?.title ?? 'Live updates'}
        progress={exec?.progress ?? null}
        state={exec?.state ?? 'running'}
        connection={state.connection}
        onOpen={() => store.dispatch({ type: 'setMode', mode: 'execution-inspect' })}
      />
      {exp && expDescriptor && <DockHost key="dock" state={state} store={store} artifacts={artifacts} descriptor={expDescriptor} pinned={exp.pinned} source={exp.source} />}
      {exp && !expDescriptor && (
        <div className="artifact-missing" role="alert" data-testid="artifact-error">
          Artifact {exp.id} is not in this world. <button onClick={() => store.dispatch({ type: 'collapseArtifact' })}>Close</button>
        </div>
      )}
    </>
  )
}

const KIND_LABEL: Record<string, string> = {
  code_diff: 'Code change',
  test_report: 'Checks',
  decision_record: 'Record',
  document: 'Document',
  data_table: 'Data',
}

function DockHost(p: { state: UiState; store: Store; artifacts: ArtifactService; descriptor: CognitiveArtifact; pinned: boolean; source?: Id }) {
  const { state, store, descriptor } = p
  const st = useArtifact(p.artifacts, descriptor.ref)
  const snap = worldView(state).snap
  const bound = snap.objects[p.source ?? descriptor.boundObject] ?? snap.objects[descriptor.boundObject]
  const siblings = (bound?.artifacts ?? [descriptor.ref]).map((r) => snap.artifacts[r] ?? findArtifact(state, r)).filter((x): x is CognitiveArtifact => !!x)
  const crumbs = state.focusStack.map((id) => snap.objects[id]?.title ?? id)
  if (bound && !state.focusStack.includes(bound.id)) crumbs.push(bound.title)
  return (
    <ArtifactDock
      descriptor={descriptor}
      state={st}
      siblings={siblings.length ? siblings : [descriptor]}
      breadcrumb={crumbs.length ? crumbs : ['Home']}
      pinned={p.pinned}
      onSelectSibling={(ref) => {
        if (bound) store.dispatch({ type: 'materializeArtifact', id: ref, source: bound.id })
        store.dispatch({ type: 'expandArtifact', id: ref, snapshot: store.getState().expandedFrom! })
      }}
      onClose={() => store.dispatch({ type: 'collapseArtifact' })}
      onPin={(pinned) => store.dispatch({ type: 'pinArtifact', id: descriptor.ref, pinned })}
      onFocusObject={(id) => {
        store.dispatch({ type: 'collapseArtifact' })
        store.dispatch({ type: 'focus', id })
      }}
      onSetTime={(cursor) => {
        store.dispatch({ type: 'collapseArtifact' })
        store.dispatch({ type: 'setTime', revision: cursor })
      }}
    />
  )
}

/** Non-spatial equivalent of what is on screen: the center, its parts, and their parts. */
function outlineItems(state: UiState, focus: string | null): OutlineItem[] {
  const snap = worldView(state).snap
  const center = focus ?? Object.values(snap.objects).find((o) => o.kind === 'core')?.id ?? ''
  const out: OutlineItem[] = []
  const add = (id: string, depth: number) => {
    const o = snap.objects[id]
    if (!o) return
    out.push({
      id,
      title: o.title,
      subtitle: o.metric ?? o.subtitle,
      status: o.status,
      depth,
      isCenter: depth === 0,
      actions: isPast(state) ? [] : (o.actions ?? []).map((a) => ({ id: a.id, label: a.label })),
    })
    if (depth < 2) for (const c of childrenOf(snap, id)) add(c.id, depth + 1)
  }
  add(center, 0)
  return out
}

function formatWhen(iso: string) {
  const d = new Date(iso)
  const date = d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  const time = d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })
  return `${date}   ${time}`
}
