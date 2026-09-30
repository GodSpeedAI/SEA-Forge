import { describe, expect, test } from 'bun:test'
import type { CaseworkPort, StreamEvent, XSnapshot } from '../ports/contract'
import { LocalContractAdapter } from '../adapters/local/localAdapter'
import { HttpCaseworkAdapter } from '../adapters/http/httpCaseworkAdapter'
import { createIntentPath } from '../app/intents'
import { connectLive } from '../app/live'
import { createProposalFlow, loadAndFocusCase } from '../app/proposals'
import { nowRevision } from '../model/store'
import { ExecutionPill } from './ExecutionPill'
import { DiscretionaryDrawer } from './DiscretionaryDrawer'
import { JudgmentPanel } from './JudgmentPanel'
import { LoginScreen } from './LoginScreen'
import { SessionBadge } from './SessionBadge'
import { TemplateDesignPanel } from './TemplateDesignPanel'
import { UnavailableActions } from './UnavailableActions'
import { ArtifactDock } from './ArtifactDock'
import { loadedRenderers, RENDERER_KINDS, RendererBoundary, SourceRenderer } from '../artifacts/registry'
import { createArtifactService } from '../artifacts/service'
import {
  ACTOR,
  DEV_ACTOR,
  NORTHSTAR_CASE_ID,
  headObjects,
  makeJourneyAdapter,
  makeStore,
  northstarHistory,
  render,
  tick,
} from './journeys.harness'

// T09 journey wiring tests (bun test, local adapter + fixture data). Per journey: the component
// surface is rendered from REAL store state and every interaction goes through the real paths
// (intent path, proposal flow, connectLive) against the REAL local adapter.

function buttonTag(markup: string, testid: string): string | null {
  return markup.match(new RegExp(`<button[^>]*data-testid="${testid}"[^>]*>`))?.[0] ?? null
}

function findAction(objects: ReturnType<typeof headObjects>, id: string, intent: string) {
  const a = objects[id]?.actions?.find((x) => x.intent === intent)
  if (!a) throw new Error(`${id} does not offer ${intent} (fixture changed?)`)
  return a
}

// ---------------------------------------------------------------------------
// Session (CJ00): login screen, session badge, local session identity.

describe('journey: login/session', () => {
  test('the local adapter self-reports a session; login/logout transitions it', async () => {
    const adapter = makeJourneyAdapter()
    const s = await adapter.session()
    expect(s?.actor_id).toBe('usr-sam')
    expect(s?.role).toBe('case_architect')

    const out = new LocalContractAdapter({ latency: 0, identity: null })
    expect(await out.session()).toBeNull()
    const logged = await out.login('usr-operator', 'secret')
    expect(logged.actor_id).toBe('usr-operator')
    expect((await out.session())?.actor_id).toBe('usr-operator')
    await out.logout()
    expect(await out.session()).toBeNull()
  })

  test('login refuses an empty username typed', async () => {
    const out = new LocalContractAdapter({ latency: 0, identity: null })
    await expect(out.login('', 'x')).rejects.toMatchObject({ code: 'INVALID' })
  })

  test('the login screen renders with fields and an error card', () => {
    const html = render(<LoginScreen sourceLabel="Local contract adapter (not the Go service)" error="login failed (401)" onLogin={() => {}} />)
    expect(html).toContain('data-testid="login-username"')
    expect(html).toContain('data-testid="login-password"')
    expect(html).toContain('data-testid="login-error"')
    expect(html).toContain('login failed (401)')
  })

  test('the session badge shows the resolved actor_id and role', () => {
    const html = render(
      <SessionBadge session={{ actor_id: 'usr-sam', role: 'case_architect', display_name: 'Sam Prime', kind: 'human' }} />,
    )
    expect(html).toContain('data-state="signed-in"')
    expect(html).toContain('usr-sam')
    expect(html).toContain('case_architect')
    const out = render(<SessionBadge session={null} />)
    expect(out).toContain('Signed out')
  })
})

// ---------------------------------------------------------------------------
// Case design (CJ04): template picker → parameters → preflight digest → gated commit → focus.

describe('journey: case design (CJ04)', () => {
  function setup() {
    const adapter = makeJourneyAdapter()
    const { history } = northstarHistory()
    const { store } = makeStore(history)
    const flow = createProposalFlow({
      store,
      port: adapter,
      actor: () => ACTOR,
      focusCase: (id) => loadAndFocusCase(adapter, store, id, ACTOR),
    })
    return { adapter, store, flow }
  }

  test('picker loads fixture templates through TemplateSourcePort', async () => {
    const { store, flow } = setup()
    await flow.open()
    const pr = store.getState().proposal!
    expect(pr.status).toBe('ready')
    expect(pr.templates.map((t) => t.template_ref)).toContain('tpl-release-rollout')
    const html = render(<TemplateDesignPanel proposal={pr} onSelectTemplate={() => {}} onSetParam={() => {}} onPreflight={() => {}} onSubmit={() => {}} onClose={() => {}} />)
    expect(html.match(/data-testid="proposal-template"/g)?.length).toBe(2)
  })

  test('submit is disabled until preflight passes, then the commit focuses the new case', async () => {
    const { store, flow } = setup()
    await flow.open()
    flow.select('tpl-release-rollout')

    // No preflight yet: the gate holds.
    const before = await flow.submit()
    expect(before.accepted).toBe(false)
    expect(store.getState().proposal!.result).toBeUndefined()

    // Missing required parameter: preflight fails, reasons shown, gate still holds.
    await flow.preflight()
    let pr = store.getState().proposal!
    expect(pr.preflight?.passed).toBe(false)
    let html = render(<TemplateDesignPanel proposal={pr} onSelectTemplate={() => {}} onSetParam={() => {}} onPreflight={() => {}} onSubmit={() => {}} onClose={() => {}} />)
    expect(html).toContain('data-passed="false"')
    expect(buttonTag(html, 'proposal-submit') ?? '').toContain('disabled')

    // Fill the required parameter and run preflight: digest shown, gate opens.
    flow.setParam('release_version', 'v0.4.2')
    expect(store.getState().proposal!.preflight).toBeNull() // changed params invalidate preflight
    await flow.preflight()
    pr = store.getState().proposal!
    expect(pr.preflight?.passed).toBe(true)
    expect(pr.preflight?.digest).toMatch(/^sha256:[0-9a-f]{64}$/)
    html = render(<TemplateDesignPanel proposal={pr} onSelectTemplate={() => {}} onSetParam={() => {}} onPreflight={() => {}} onSubmit={() => {}} onClose={() => {}} />)
    expect(html).toContain('data-passed="true"')
    expect(html).toContain('data-testid="proposal-digest"')
    expect(buttonTag(html, 'proposal-submit')).not.toContain('disabled')

    // Commit: PROPOSE_CASE (no client_cursor), focus moves to resulting_object.id.
    const submitted = await flow.submit()
    expect(submitted.accepted).toBe(true)
    expect(submitted.caseId).toMatch(/^case-release-rollout-/)
    expect(store.getState().proposal).toBeNull() // panel closed
    const s = store.getState()
    expect(s.history.caseId).toBe(submitted.caseId!)
    expect(s.focusStack.at(-1)).toBe(submitted.caseId!)
    // The committed case world carries the template plan with a sentry chain.
    const objs = s.history.snapshots[nowRevision(s.history)]!.objects
    expect(Object.values(objs).some((o) => o.title === 'Rollout')).toBe(true)
    expect(Object.values(objs).some((o) => o.title === 'Sign-off')).toBe(true)
  })

  test('PROPOSE_CASE with a digest that no longer matches is refused STALE_PROJECTION', async () => {
    const adapter = makeJourneyAdapter()
    const r = await adapter.dispatchIntent({
      intent_id: `propose-${Math.random()}`,
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'PROPOSE_CASE',
      target_object_id: 'tpl-release-rollout',
      case_id: '',
      client_cursor: '',
      actor: { actor_id: 'usr-sam', role: 'case_architect' },
      parameters: { template_ref: 'tpl-release-rollout', params: { release_version: 'v1' }, preflight_digest: 'sha256:' + '0'.repeat(64) },
    })
    expect(r.success).toBe(false)
    expect(r.refusal?.refusal_kind).toBe('STALE_PROJECTION')
  })
})

// ---------------------------------------------------------------------------
// Discretionary work (CJ05-equivalent in the journey cube): typed drawer, mandatory fields,
// proposer badge, refusal surfaced verbatim.

describe('journey: discretionary add', () => {
  test('the drawer shows the proposer from the session and marks both fields required', () => {
    const html = render(
      <DiscretionaryDrawer
        visible
        targetTitle="Northstar"
        proposer={ACTOR}
        pending={false}
        outcome={null}
        onSubmit={() => {}}
        onClose={() => {}}
      />,
    )
    expect(html).toContain('data-testid="discretionary-proposer"')
    expect(html).toContain('usr-sam')
    expect(html).toContain('case_architect')
    expect(html).toContain('data-testid="discretionary-title"')
    expect(html).toContain('data-testid="discretionary-justification"')
    expect(html).toContain('required')
  })

  test('missing justification is refused JUSTIFICATION_REQUIRED; missing title INVALID; a full proposal lands', async () => {
    const adapter = makeJourneyAdapter()
    const { history, raw } = northstarHistory()
    const { store } = makeStore(history)
    const summaries: Record<string, string> = {}
    const disconnect = connectLive(adapter, store, NORTHSTAR_CASE_ID, raw, summaries)
    const intents = createIntentPath(store, adapter)
    const action = findAction(headObjects(history), 'northstar', 'ADD_DISCRETIONARY_WORK')

    const refused = await intents.submit(ACTOR, 'northstar', action, { judgment: false, parameters: { case_id: NORTHSTAR_CASE_ID, stage_id: 'northstar', kind: 'work_item', title: 'Rollback rehearsal' } })
    expect(refused.state).toBe('refused')
    expect(refused.code).toBe('JUSTIFICATION_REQUIRED')

    const invalid = await intents.submit(ACTOR, 'northstar', action, {
      judgment: false,
      justification: 'The migration needs a rollback rehearsal.',
      parameters: { case_id: NORTHSTAR_CASE_ID, stage_id: 'northstar', kind: 'work_item' },
    })
    expect(invalid.state).toBe('refused')
    expect(invalid.code).toBe('INVALID')

    const ok = await intents.submit(ACTOR, 'northstar', action, {
      judgment: false,
      justification: 'The migration needs a rollback rehearsal before verification.',
      parameters: {
        case_id: NORTHSTAR_CASE_ID,
        stage_id: 'northstar',
        kind: 'work_item',
        title: 'Rollback rehearsal',
        justification: 'The migration needs a rollback rehearsal before verification.',
      },
    })
    expect(ok.state).toBe('accepted')
    await tick()
    // The proposed work arrived as a new snapshot revision (no reload, no local mutation).
    const objs = store.getState().history.snapshots[nowRevision(store.getState().history)]!.objects
    const proposed = Object.values(objs).find((o) => o.title === 'Rollback rehearsal')
    expect(proposed).toBeDefined()
    expect(proposed?.contractStatus).toBe('AVAILABLE_TO_ADD')
    disconnect()

    // The refusal surfaces verbatim in the drawer's outcome card.
    const html = render(
      <DiscretionaryDrawer
        visible
        targetTitle="Northstar"
        proposer={ACTOR}
        pending={false}
        outcome={{ state: 'refused', code: 'JUSTIFICATION_REQUIRED', note: 'This action needs a written justification.' }}
        onSubmit={() => {}}
        onClose={() => {}}
      />,
    )
    expect(html).toContain('data-state="refused"')
    expect(html).toContain('JUSTIFICATION_REQUIRED')
    expect(html).toContain('This action needs a written justification.')
  })
})

// ---------------------------------------------------------------------------
// Execute (CJ06/CJ07): EXECUTE_ITEM drives real stream events; the pill is honest about
// progress and shows Reconnecting from the adapter's error path; downstream standing updates
// through SSE without a reload (sentry transparency).

describe('journey: execute + execution pill + sentry', () => {
  function setup(speed = 200) {
    const adapter = makeJourneyAdapter(speed)
    const { history, raw } = northstarHistory()
    const { store } = makeStore(history)
    const summaries: Record<string, string> = {}
    const disconnect = connectLive(adapter, store, NORTHSTAR_CASE_ID, raw, summaries)
    const intents = createIntentPath(store, adapter)
    return { adapter, store, intents, disconnect }
  }

  test('EXECUTE_ITEM updates standing through snapshot events without a reload', async () => {
    const { store, intents, disconnect } = setup()
    const before = store.getState().history.revisions.length
    const action = findAction(headObjects(store.getState().history), 't-rollout', 'EXECUTE_ITEM')
    const rec = await intents.submit(ACTOR, 't-rollout', action, { judgment: false })
    expect(rec.state).toBe('accepted')
    await tick(600) // execution phases + completion snapshot at speed 200

    const s = store.getState()
    expect(s.history.revisions.length).toBeGreaterThan(before) // revisions arrived via SSE
    const exec = s.executions['t-rollout']
    expect(['executed', 'settled']).toContain(exec?.state) // the local fixture settles quickly
    const objs = s.history.snapshots[nowRevision(s.history)]!.objects
    expect(objs['t-rollout']?.contractStatus).toBe('COMPLETED')
    disconnect()
  })

  test('sentry transparency: a dependent item explains what blocks it, and unlocks when it completes', async () => {
    const { store, intents, disconnect } = setup()
    // t-review depends_on t-rollout (kernel depends_on, no backend explanation sent): the
    // projection derives the sentry explanation from depends_on + standing.
    const before = store.getState().history.snapshots[nowRevision(store.getState().history)]!.objects['t-review']!
    expect(before.subtitle).toContain('Waiting on')
    expect(before.subtitle).toContain('Plan rollout')

    const action = findAction(headObjects(store.getState().history), 't-rollout', 'EXECUTE_ITEM')
    const rec = await intents.submit(ACTOR, 't-rollout', action, { judgment: false })
    expect(rec.state).toBe('accepted')
    await tick(600)

    const after = store.getState().history.snapshots[nowRevision(store.getState().history)]!.objects['t-review']!
    // The sentry released it — via SSE only, no reload, no refetch.
    expect(after.subtitle ?? '').not.toContain('Waiting on')
    disconnect()
  })

  test('the pill shows real reported progress, and Reconnecting hides any number', () => {
    const { store } = setup()
    store.dispatch({
      type: 'execution',
      exec: { object: 't-rollout', runId: 'run-t-rollout-1', phase: 'builder', progress: 0.35, log: ['Applying release'], state: 'running' },
    })
    const live = render(
      <ExecutionPill visible label="Plan rollout" progress={store.getState().executions['t-rollout']!.progress} state="running" reconnecting={false} onOpen={() => {}} />,
    )
    expect(live).toContain('35%')
    expect(live).toContain('data-state="running"')

    // Reconnecting: the stale percentage is hidden — only snapshot standing is known.
    const recon = render(
      <ExecutionPill visible label="Plan rollout" progress={0.35} state="running" reconnecting onOpen={() => {}} />,
    )
    expect(recon).toContain('data-state="reconnecting"')
    expect(recon).toContain('reconnecting')
    expect(recon).not.toContain('%')

    const interrupted = render(
      <ExecutionPill visible label="Live updates" progress={0.35} state="running" connection="interrupted" onOpen={() => {}} />,
    )
    expect(interrupted).toContain('data-state="interrupted"')
    expect(interrupted).toContain('interrupted')
    expect(interrupted).toContain('may be stale')
    expect(interrupted).not.toContain('%')

    // Snapshot standing alone (no stream progress) never shows a fabricated percentage.
    const standing = render(
      <ExecutionPill visible label="Plan rollout" progress={null} state="running" reconnecting={false} onOpen={() => {}} />,
    )
    expect(standing).toContain('running')
    expect(standing).not.toMatch(/\d+%/)
  })

  test('TOOTH: SSE drop flips the store to reconnecting via the live adapter error path, and an event resumes it', async () => {
    // The live adapter's real error path: a gateway that accepts nothing (every stream request
    // fails fast), so the adapter reports the interruption and retries with backoff.
    const broken = Bun.serve({ hostname: '127.0.0.1', port: 0, fetch: () => new Response('gateway broken', { status: 503 }) })
    try {
      const dead = new HttpCaseworkAdapter({ base: `http://127.0.0.1:${broken.port}`, reconnectBaseMs: 20, reconnectMaxMs: 40 })
      const { history, raw } = northstarHistory()
      const { store } = makeStore(history)
      const disconnect = connectLive(dead, store, NORTHSTAR_CASE_ID, raw, {})
      await tick(120)
      expect(store.getState().connection).toBe('reconnecting')
      disconnect()
    } finally {
      broken.stop(true)
    }

    // Resume: a delivered event proves the stream is alive again (the adapter keeps its cursor).
    let onError: ((e: Error) => void) | null = null
    let onEvent: ((e: StreamEvent) => void) | null = null
    const port = {
      getSnapshot: () => Promise.reject(new Error('unused')),
      getSnapshotAt: () => Promise.reject(new Error('unused')),
      dispatchIntent: () => Promise.resolve({ intent_id: 'x', success: true }),
      resolveArtifact: () => Promise.reject(new Error('unused')),
      queryTemporalTrajectory: () => Promise.reject(new Error('unused')),
      subscribeEvents(_caseId: string, _since: string | undefined, ev: (e: StreamEvent) => void, err: (e: Error) => void) {
        onEvent = ev
        onError = err
        return () => undefined
      },
    } as unknown as CaseworkPort
    const resumed = northstarHistory()
    const store2 = makeStore(resumed.history).store
    const disconnect2 = connectLive(port, store2, NORTHSTAR_CASE_ID, resumed.raw, {})
    onError!(new Error('connection interrupted'))
    expect(store2.getState().connection).toBe('reconnecting')
    // While reconnecting the pill hides the last-known percentage (no fake progress).
    const reconPill = render(<ExecutionPill visible label="Plan rollout" progress={0.35} state="running" reconnecting onOpen={() => {}} />)
    expect(reconPill).toContain('reconnecting')
    expect(reconPill).not.toContain('%')
    // The stream returns: first a snapshot (the run exists), then real reported progress.
    const head = resumed.raw.at(-1)!
    const withRun = {
      ...head,
      cursor: '9.0000000009',
      timestamp: new Date().toISOString(),
      visible_objects: [
        ...head.visible_objects,
        { id: 'run-t-rollout-1', kind: 'execution_trace', name: 'Rollout run', status: 'IN_PROGRESS', badge: 'Running', explanation: 'Gauntlet', salience: 0.4, parent_id: 't-rollout', actions: [], x: { presentation: 'run', tone: 'progress' } },
      ],
    } as unknown as XSnapshot
    onEvent!({ event_type: 'snapshot', cursor: withRun.cursor, timestamp: withRun.timestamp, payload: withRun } as StreamEvent)
    expect(store2.getState().connection).toBe('live')
    onEvent!({
      event_type: 'execution_progress',
      cursor: '9.0000000009',
      timestamp: new Date().toISOString(),
      payload: { run_id: 'run-t-rollout-1', phase: 'builder', progress_percent: 0.4, log_line: 'Applying release v0.3.0' },
    } as StreamEvent)
    // Progress resumes from what the stream actually reported.
    expect(store2.getState().executions['t-rollout']?.progress).toBe(0.4)
    disconnect2()
  })
})

// ---------------------------------------------------------------------------
// Human task (CJ06-CJ07-ish): complete + approve/reject with mandatory justification; typed
// refusals surface verbatim.

describe('journey: human task judgment', () => {
  function setup() {
    const adapter = makeJourneyAdapter()
    const { history, raw } = northstarHistory()
    const { store } = makeStore(history)
    const disconnect = connectLive(adapter, store, NORTHSTAR_CASE_ID, raw, {})
    const intents = createIntentPath(store, adapter)
    return { adapter, store, intents, disconnect, history }
  }

  function panelProps(store: ReturnType<typeof makeStore>['store'], objects: ReturnType<typeof headObjects>, id: string) {
    const obj = objects[id]
    return {
      options: (obj?.actions ?? []).filter((a) => a.consequential).map((a) => ({ id: a.id, label: a.label, variant: a.variant, requiresJustification: a.requiresJustification })),
      outcome: store.getState().judgment?.outcome,
    }
  }

  test('COMPLETE_HUMAN_TASK without justification is refused; the panel shows the typed kind', async () => {
    const { store, intents, disconnect } = setup()
    const action = findAction(headObjects(store.getState().history), 't-review', 'COMPLETE_HUMAN_TASK')
    expect(action.requiresJustification).toBe(true)

    // The panel itself blocks an empty reason (component-level gate).
    store.dispatch({ type: 'invoke', object: 't-review', action })
    const html = render(
      <JudgmentPanel
        visible
        question="Complete review: Review PR #491?"
        description="Needs your attention"
        options={panelProps(store, headObjects(store.getState().history), 't-review').options}
        context={[]}
        pending={false}
        outcome={null}
        authorityNote="Authority decides"
        onChoose={() => {}}
        onClose={() => {}}
      />,
    )
    expect(html).toContain('Reason')
    expect(html).toContain('required for')

    // The backend refuses it too, and the refusal surfaces verbatim with the typed kind.
    const rec = await intents.submit(ACTOR, 't-review', action, { judgment: true })
    expect(rec.state).toBe('refused')
    expect(rec.code).toBe('JUSTIFICATION_REQUIRED')
    const refusedHtml = render(
      <JudgmentPanel
        visible
        question="Complete review: Review PR #491?"
        description=""
        options={[]}
        context={[]}
        pending={false}
        outcome={{ state: 'refused', code: rec.code, note: rec.note, by: ACTOR.name }}
        authorityNote=""
        onChoose={() => {}}
        onClose={() => {}}
      />,
    )
    expect(refusedHtml).toContain('JUSTIFICATION_REQUIRED')
    expect(refusedHtml).toContain('This action needs a written justification.')
    disconnect()
  })

  test('a justification completes the task through a snapshot event', async () => {
    const { store, intents, disconnect } = setup()
    const action = findAction(headObjects(store.getState().history), 't-review', 'COMPLETE_HUMAN_TASK')
    const rec = await intents.submit(ACTOR, 't-review', action, { judgment: true, justification: 'Reviewed the fix against the coverage gap.' })
    expect(rec.state).toBe('accepted')
    await tick()
    const objs = store.getState().history.snapshots[nowRevision(store.getState().history)]!.objects
    expect(objs['t-review']?.contractStatus).toBe('COMPLETED')
    disconnect()
  })

  test('role refusals surface verbatim: a developer cannot approve the release gate', async () => {
    const { store, intents, disconnect } = setup()
    const action = findAction(headObjects(store.getState().history), 'ns-release', 'APPROVE_HUMAN_TASK')
    const rec = await intents.submit(DEV_ACTOR, 'ns-release', action, { judgment: true })
    expect(rec.state).toBe('refused')
    expect(rec.code).toBe('UNAUTHORIZED_ROLE')
    expect(rec.note).toContain('cannot decide this')
    const html = render(
      <JudgmentPanel
        visible
        question="Approve release: Release?"
        description=""
        options={[]}
        context={[]}
        pending={false}
        outcome={{ state: 'refused', code: rec.code, note: rec.note, by: DEV_ACTOR.name }}
        authorityNote=""
        onChoose={() => {}}
        onClose={() => {}}
      />,
    )
    expect(html).toContain('UNAUTHORIZED_ROLE')
    disconnect()
  })

  test('an approval by a case architect is accepted and execution begins', async () => {
    const { store, intents, disconnect } = setup()
    const action = findAction(headObjects(store.getState().history), 'ns-release', 'APPROVE_HUMAN_TASK')
    const rec = await intents.submit(ACTOR, 'ns-release', action, { judgment: true })
    expect(rec.state).toBe('accepted')
    await tick(600)
    const exec = store.getState().executions['ns-release']
    expect(exec).toBeDefined()
    disconnect()
  })
})

// ---------------------------------------------------------------------------
// Lifecycle (CJ08/CJ09): Reopen/Terminate render ONLY when the snapshot offers them.

describe('journey: lifecycle (the offered-action tooth)', () => {
  function setup() {
    const adapter = makeJourneyAdapter()
    const { history, raw } = northstarHistory()
    const { store } = makeStore(history)
    const disconnect = connectLive(adapter, store, NORTHSTAR_CASE_ID, raw, {})
    const intents = createIntentPath(store, adapter)
    return { adapter, store, intents, disconnect, history }
  }

  const panelHtml = (objects: ReturnType<typeof headObjects>, id: string) => {
    const obj = objects[id]
    return render(
      <JudgmentPanel
        visible
        question={`${obj?.title}: decide?`}
        description=""
        options={(obj?.actions ?? []).filter((a) => a.consequential).map((a) => ({ id: a.id, label: a.label, variant: a.variant, requiresJustification: a.requiresJustification }))}
        context={[]}
        pending={false}
        outcome={null}
        authorityNote=""
        onChoose={() => {}}
        onClose={() => {}}
      />,
    )
  }

  test('the head snapshot offers Reopen and Terminate on the case, and the panel renders exactly those', () => {
    const { history } = setup()
    const objects = headObjects(history)
    const caseActions = objects['northstar']!.actions!.map((a) => a.intent)
    expect(caseActions).toContain('REOPEN_CASE')
    expect(caseActions).toContain('TERMINATE_CASE')
    const html = panelHtml(objects, 'northstar')
    expect(html).toContain('Reopen case')
    expect(html).toContain('Terminate case')
  })

  test('TOOTH: an object the snapshot did not offer lifecycle actions on renders none, and a fabricated dispatch is refused', async () => {
    const { store, intents, disconnect, history } = setup()
    // ns-release offers approval actions only — no REOPEN_CASE, no TERMINATE_CASE.
    const releaseActions = headObjects(history)['ns-release']!.actions!.map((a) => a.intent)
    expect(releaseActions).not.toContain('REOPEN_CASE')
    expect(releaseActions).not.toContain('TERMINATE_CASE')
    const html = panelHtml(headObjects(store.getState().history), 'ns-release')
    expect(html).not.toContain('Reopen case')
    expect(html).not.toContain('Terminate case')

    // Even a fabricated intent for an unoffered action is refused by the authority — the UI can
    // only ever dispatch what ActionDescriptors offered, and the kernel is the second gate.
    const fabricated = { id: 'fabricated', label: 'Terminate case', intent: 'TERMINATE_CASE' as const, consequential: true, variant: 'DANGER' as const, requiresJustification: true }
    const rec = await intents.submit(ACTOR, 'ns-release', fabricated, { judgment: true, justification: 'because' })
    expect(rec.state).toBe('refused')
    expect(rec.code).toBe('UNAVAILABLE')
    expect(rec.note).toContain('does not offer this action')
    disconnect()
  })

  test('terminate with a reason lands; reopen brings the case back', async () => {
    // Two independent cases, mirroring L7: reopen on one, terminate on another (each transition
    // is offered only while the snapshot still offers it — a terminated case offers nothing).
    const terminating = setup()
    const terminate = findAction(headObjects(terminating.history), 'northstar', 'TERMINATE_CASE')
    const rec = await terminating.intents.submit(ACTOR, 'northstar', terminate, { judgment: true, justification: 'Superseded by a new investigation.' })
    expect(rec.state).toBe('accepted')
    await tick()
    const objs = terminating.store.getState().history.snapshots[nowRevision(terminating.store.getState().history)]!.objects
    expect(objs['northstar']?.status?.label).toBe('Terminated')
    terminating.disconnect()

    const reopening = setup()
    const reopen = findAction(headObjects(reopening.history), 'northstar', 'REOPEN_CASE')
    const rec2 = await reopening.intents.submit(ACTOR, 'northstar', reopen, { judgment: true, justification: 'New evidence arrived.' })
    expect(rec2.state).toBe('accepted')
    await tick()
    const objs2 = reopening.store.getState().history.snapshots[nowRevision(reopening.store.getState().history)]!.objects
    expect(objs2['northstar']?.status?.label).toBe('Reopened')
    reopening.disconnect()
  })
})

// ---------------------------------------------------------------------------
// Artifact dock (CJ08/CJ09): adapter resolution, digest verification, honest error card.

describe('journey: artifact dock', () => {
  const descriptor = { ref: 'evi-release-log', kind: 'document' as const, title: 'Release staging log', boundObject: 'ns-release', currentLevel: 'minimal' as const, mediaType: 'text/plain', sourceProvenance: 'sxr:cas://evi-release-log' }

  function dock(state: ReturnType<typeof createArtifactService> extends infer S ? S : never, ref = 'evi-release-log') {
    return render(
      <ArtifactDock
        descriptor={descriptor}
        state={state.get(ref)}
        siblings={[descriptor]}
        breadcrumb={['Northstar', 'Release']}
        pinned={false}
        onSelectSibling={() => {}}
        onClose={() => {}}
        onPin={() => {}}
        onFocusObject={() => {}}
        onSetTime={() => {}}
      />,
    )
  }

  test('a resolution failure renders the localized error card with the ref', async () => {
    const adapter = new LocalContractAdapter({ latency: 0, failArtifacts: ['evi-release-log'] })
    const service = createArtifactService(adapter)
    service.ensure('evi-release-log')
    await tick()
    const st = service.get('evi-release-log')
    expect(st.status).toBe('error')
    const html = dock(service)
    expect(html).toContain('data-testid="artifact-error"')
    expect(html).toContain('Could not load this artifact')
    expect(html).toContain('evi-release-log')
  })

  test('a digest mismatch is refused, never rendered as the requested artifact', async () => {
    const wrongPort = {
      resolveArtifact: () =>
        Promise.resolve({ evidence_id: 'sha256:' + 'a'.repeat(64), name: 'X', digest: 'sha256:' + 'b'.repeat(64), content_type: 'text/plain', content: 'tampered', provenance: { case_id: '', plan_item_id: '', invocation_id: '', run_id: '' } }),
    } as unknown as CaseworkPort
    const service = createArtifactService(wrongPort)
    const ref = 'sha256:' + 'a'.repeat(64)
    service.ensure(ref)
    await tick()
    const st = service.get(ref)
    expect(st.status).toBe('error')
    expect(st.status === 'error' && st.error).toContain('does not match the requested digest')
    const adapter = makeJourneyAdapter()
    const ok = createArtifactService(adapter)
    ok.ensure('evi-release-log')
    await tick()
    expect(ok.get('evi-release-log').status).toBe('ready')
  })
})

// ---------------------------------------------------------------------------
// Lazy renderers (VAR-006) and CJ10-CJ12 honest unavailability.

describe('journey: lazy renderers (VAR-006)', () => {
  test('all nine source renderers are registered as separate lazy chunks, none loaded eagerly', () => {
    // Exactly the nine source renderers, each behind its own dynamic import loader (registry.tsx
    // wraps every kind in React.lazy + import()). Importing the registry module itself loads no
    // renderer chunk: the tracking set the E2E ladder reads is still empty here.
    expect([...RENDERER_KINDS].sort()).toEqual(['chart', 'diff', 'graph', 'json', 'markdown', 'table', 'text', 'timeline', 'trace'])
    expect(loadedRenderers.size).toBe(0)
    expect(typeof SourceRenderer).toBe('function')
    expect(typeof RendererBoundary).toBe('function')
  })

  test('the tracking set stays empty until a renderer actually loads (ladder J2 asserts fetches)', () => {
    expect(loadedRenderers.size).toBe(0)
  })
})

describe('journey: CJ10-CJ12 unavailable in this deployment', () => {
  test('rendered as static descriptors — never buttons, never dispatchable', () => {
    const html = render(<UnavailableActions visible />)
    expect(html).toContain('data-testid="unavailable-actions"')
    expect(html.match(/data-testid="unavailable-action"/g)?.length).toBe(3)
    expect(html).toContain('Federation export / import')
    expect(html).toContain('Case memory promotion')
    expect(html).toContain('Gauntlet execution feed')
    expect(html).toContain('not wired in this deployment')
    // No interactive element: they cannot be dispatched.
    expect(html).not.toContain('<button')
    expect(html).not.toContain('onclick')
  })
})
