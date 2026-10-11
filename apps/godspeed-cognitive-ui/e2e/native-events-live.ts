import { HttpCaseworkAdapter } from '../src/adapters/http/httpCaseworkAdapter'
import type { ActorRole, InteractionIntent, StreamEvent, XSnapshot } from '../src/ports/contract'

type EventRequest = { last_query: string; last_event_id: string; request_count: number }

type PhaseState = {
  phase: string
  case_id: string
  initial_cursor: string
  retention: number
  head: string
  oldest: string
  store_len: number
  subscribers: number
  active_streams: number
  event_requests: EventRequest[]
  item_activated: number
  settlement_recorded: number
  item_completed: number
  plan_mutated: number
  head_summary: string
  head_case_id: string
}

type BrowserProof = {
  ok: boolean
  phase: string
  case_id: string
  cursor: string
  initial_cursor: string
  interrupted_cursor: string
  events: Array<{ type: string; cursor: string }>
  errors: string[]
  requests_before: number
  requests_after: number
  durable_delta: number
  reconnect_cursor: string
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message)
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function phaseState(phase: string): Promise<PhaseState> {
  const response = await fetch(`/__native-test/state/${phase}`, { cache: 'no-store' })
  assert(response.ok, `test state request failed (${response.status})`)
  return (await response.json()) as PhaseState
}

async function settledPhaseState(phase: string): Promise<PhaseState> {
  const response = await fetch(`/__native-test/stable-state/${phase}`, { cache: 'no-store' })
  assert(response.ok, `stable test state request failed (${response.status})`)
  return (await response.json()) as PhaseState
}

async function waitFor<T>(
  read: () => Promise<T>,
  predicate: (value: T) => boolean,
  label: string,
  timeoutMs = 15_000,
): Promise<T> {
  const deadline = Date.now() + timeoutMs
  let last: T | undefined
  while (Date.now() < deadline) {
    last = await read()
    if (predicate(last)) return last
    await sleep(25)
  }
  throw new Error(`timed out waiting for ${label}: ${JSON.stringify(last)}`)
}

function objectStanding(snapshot: XSnapshot, options: { includeTimestamp?: boolean } = {}): string {
  // Compare the complete actual wire snapshot. XObject has no top-level settlement
  // property; any optional extension is carried in x and remains part of the object.
  const objects = [...snapshot.visible_objects]
    .sort((left, right) => left.id.localeCompare(right.id))
  return JSON.stringify({
    world_id: snapshot.world_id,
    case_id: snapshot.case_id,
    cursor: snapshot.cursor,
    ...(options.includeTimestamp === false ? {} : { timestamp: snapshot.timestamp }),
    perspective: snapshot.perspective,
    summary: snapshot.summary,
    visible_objects: objects,
    available_actions: snapshot.available_actions,
    attention_focus: snapshot.attention_focus,
    x: snapshot.x,
  })
}

function freshCurrentStanding(snapshot: XSnapshot): string {
  // /api/world rebuilds live facts on each read, so its RFC3339 render timestamp can change
  // while the cursor-addressed projection remains identical.
  return objectStanding(snapshot, { includeTimestamp: false })
}

function assertValidSnapshotTimestamp(snapshot: XSnapshot, label: string): void {
  assert(Number.isFinite(Date.parse(snapshot.timestamp)), `${label} timestamp is not a valid date: ${snapshot.timestamp}`)
}

function eventCount(events: readonly StreamEvent[]): number {
  return events.length
}

function nativeEventSourceAvailable(): boolean {
  return typeof EventSource === 'function' &&
    /\[native code\]/.test(Function.prototype.toString.call(EventSource))
}

async function login(adapter: HttpCaseworkAdapter): Promise<{ actorId: string; role: ActorRole }> {
  assert(nativeEventSourceAvailable(), 'the browser does not expose its native EventSource')
  await adapter.login('operator', 'livetest')
  const identity = await adapter.session()
  assert(identity?.actor_id && identity.role, 'authenticated operator session did not resolve')
  return { actorId: identity.actor_id, role: identity.role as ActorRole }
}

async function phaseOne(): Promise<BrowserProof> {
  const state = await phaseState('phase1')
  assert(state.retention === 1 && state.store_len === 1, `retention-one setup is not exact: ${JSON.stringify(state)}`)
  assert(state.head === state.initial_cursor && state.oldest === state.initial_cursor, 'phase-one C is not the sole retained cursor')

  const adapter = new HttpCaseworkAdapter({
    base: `${location.origin}/phase1`,
    reconnectBaseMs: 1_500,
    reconnectMaxMs: 1_500,
  })
  const identity = await login(adapter)
  const before = await adapter.getSnapshot(state.case_id, identity.actorId, identity.role)
  assert(before.cursor === state.initial_cursor, `fresh browser snapshot cursor ${before.cursor} differs from C=${state.initial_cursor}`)
  const taskBefore = before.visible_objects.find((object) => object.id === 'task_prepare')
  assert(taskBefore, 'the governed task_prepare item is absent at C')
  assert(taskBefore.actions?.some((descriptor) => descriptor.intent === 'EXECUTE_ITEM'),
    'task_prepare does not offer EXECUTE_ITEM in its own actions')

  const intent: InteractionIntent = {
    intent_id: `native-f14-phase1-${crypto.randomUUID()}`,
    kind: 'CONSEQUENTIAL_CASE',
    action_name: 'EXECUTE_ITEM',
    target_object_id: 'task_prepare',
    case_id: state.case_id,
    client_cursor: state.initial_cursor,
    actor: { actor_id: identity.actorId, role: identity.role },
    parameters: { item_id: 'task_prepare' },
  }
  const outcome = await adapter.dispatchIntent(intent)
  assert(outcome.success && outcome.new_cursor, `real EXECUTE_ITEM was refused: ${JSON.stringify(outcome.refusal)}`)
  const executionReceiptCursor = outcome.new_cursor
  // The harness resolves K by matching the actual last durable case trace event ID and kind to
  // its persisted SFWP frame, then waits until that exact cursor is queryable in the retained store.
  const afterMutation = await settledPhaseState('phase1')
  assert(afterMutation.head >= executionReceiptCursor, 'the retained terminal cursor precedes the accepted execution receipt')
  assert(afterMutation.head > state.initial_cursor, 'execution did not advance the kernel cursor')
  assert(afterMutation.item_activated === 1 && afterMutation.settlement_recorded >= 1 && afterMutation.item_completed >= 1,
    `durable kernel execution trace is incomplete: ${JSON.stringify(afterMutation)}`)
  assert(afterMutation.event_requests.length === 0, 'phase one must subscribe only after C has been evicted')
  const planMutationBefore = afterMutation.plan_mutated

  const events: StreamEvent[] = []
  const errors: string[] = []
  let firstErrorAt = 0
  const dispose = adapter.subscribeEvents(
    state.case_id,
    state.initial_cursor,
    (event) => events.push(event),
    (error) => {
      errors.push(error.message)
      if (firstErrorAt === 0) firstErrorAt = Date.now()
    },
  )
  try {
    await waitFor(() => Promise.resolve(events), (items) => items.length >= 2, 'resync control and same-cursor snapshot')
    assert(events.length === 2, `one retained revision must produce exactly two callbacks: ${JSON.stringify(events)}`)
    const [resync, snapshotEvent] = events
    assert(resync?.event_type === 'resync_required' && resync.cursor === afterMutation.head,
      `first callback must be resync_required K: ${JSON.stringify(resync)}`)
    const resyncPayload = resync.payload as { requested_cursor?: string; oldest_available_cursor?: string }
    assert(resyncPayload.requested_cursor === state.initial_cursor && resyncPayload.oldest_available_cursor === afterMutation.head,
      `resync control did not name C and K: ${JSON.stringify(resyncPayload)}`)
    assert(snapshotEvent?.event_type === 'snapshot' && snapshotEvent.cursor === afterMutation.head,
      `second callback must be the authoritative same-cursor snapshot K: ${JSON.stringify(snapshotEvent)}`)

    const streamed = snapshotEvent.payload as XSnapshot
    const atK = await adapter.getSnapshotAt(state.case_id, afterMutation.head, identity.actorId, identity.role)
    const current = await adapter.getSnapshot(state.case_id, identity.actorId, identity.role)
    assert(current.cursor === afterMutation.head,
      `fresh current snapshot cursor ${current.cursor} differs from K=${afterMutation.head}`)
    assertValidSnapshotTimestamp(current, 'fresh K snapshot')
    const taskAfter = streamed.visible_objects.find((object) => object.id === 'task_prepare')
    assert(taskAfter && taskAfter.status !== taskBefore.status,
      `kernel execution did not change task_prepare standing: before=${taskBefore.status} after=${taskAfter?.status}`)
    assert(objectStanding(streamed) === objectStanding(atK), 'the streamed K snapshot differs from authenticated Store.At(K)')
    assert(freshCurrentStanding(streamed) === freshCurrentStanding(current),
      'the streamed K snapshot differs from the fresh authenticated current projection')

    const connected = await waitFor(
      () => phaseState('phase1'),
      (currentState) => currentState.active_streams === 1 && currentState.subscribers === 1,
      'phase-one native EventSource subscription',
    )
    assert(connected.event_requests[0]?.last_query === state.initial_cursor,
      `initial native request did not use C=${state.initial_cursor}: ${JSON.stringify(connected.event_requests)}`)
    const dropped = await fetch('/__native-test/disconnect/phase1', { method: 'POST' })
    assert(dropped.ok, `phase-one stream drop failed (${dropped.status})`)
    const dropCount = (await dropped.json() as { closed: number }).closed
    assert(dropCount === 1, `phase-one drop closed ${dropCount} actual stream connections, want 1`)
    await waitFor(() => Promise.resolve(errors), (items) => items.length === 1, 'first native EventSource error callback')
    await waitFor(
      () => phaseState('phase1'),
      (currentState) => currentState.active_streams === 0 && currentState.subscribers === 0,
      'phase-one first stream disconnect',
    )

    // Keep the native subscription alive. The retry request is held by the test server until
    // this real authenticated mutation is durable and the retention-one head has settled.
    const currentAtK = await adapter.getSnapshot(state.case_id, identity.actorId, identity.role)
    assert(currentAtK.cursor === afterMutation.head, `fresh snapshot moved away from interrupted K=${afterMutation.head}`)
    assertValidSnapshotTimestamp(currentAtK, 'interrupted K snapshot')
    assert(!currentAtK.visible_objects.some((object) => object.name === 'native events recovery plan change'),
      'phase-one recovery plan item already exists before its mutation')
    // Dispatch through the protected T06 server intent path to create the feed update.
    // This tests the transport, not UI action availability; T09 projection wiring is separate.
    const recoveryIntent: InteractionIntent = {
      intent_id: `native-f14-phase1-recovery-${crypto.randomUUID()}`,
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'ADD_DISCRETIONARY_WORK',
      target_object_id: 'task_prepare',
      case_id: state.case_id,
      client_cursor: afterMutation.head,
      actor: { actor_id: identity.actorId, role: identity.role },
      parameters: {
        case_id: state.case_id,
        stage_id: 'task_prepare',
        kind: 'sandboxed_task',
        title: 'native events recovery plan change',
        summary: 'Add one governed plan item after the first native stream interruption.',
        justification: 'The retention-one reconnect proof requires a real authorized mutation while the stream is down.',
      },
    }
    const recovery = await adapter.dispatchIntent(recoveryIntent)
    assert(recovery.success && recovery.new_cursor, `offline governed recovery mutation was refused: ${JSON.stringify(recovery.refusal)}`)
    const offline = await waitFor(
      () => phaseState('phase1'),
      (currentState) => currentState.head > afterMutation.head && currentState.store_len === 1 &&
        currentState.oldest === currentState.head && currentState.plan_mutated === planMutationBefore + 1,
      'offline plan mutation to evict K and publish a newer retained revision',
    )
    assert(offline.head_summary === 'case.trace.plan_mutated' && offline.head_case_id === state.case_id,
      `offline mutation did not publish a plan_mutated revision: ${JSON.stringify(offline)}`)
    assert(offline.event_requests.length <= 2 && offline.subscribers === 0 &&
      (offline.event_requests.length === 1 ? offline.active_streams === 0 : offline.active_streams === 1),
      `reconnect must remain unsubscribed until the recovery mutation completes: ${JSON.stringify(offline)}`)

    const retryPending = await waitFor(
      () => phaseState('phase1'),
      (currentState) => currentState.event_requests.length === 2 && currentState.active_streams === 1 && currentState.subscribers === 0,
      'bounded native retry request waiting for the settled kernel head',
      10_000,
    )
    assert(retryPending.event_requests[1]?.last_query === afterMutation.head,
      `native retry did not resume from K=${afterMutation.head}: ${JSON.stringify(retryPending.event_requests)}`)
    const retryDelay = Date.now() - firstErrorAt
    assert(retryDelay >= 1_200 && retryDelay < 10_000,
      `configured 1,500 ms native retry was not bounded as expected: ${retryDelay} ms`)
    const released = await fetch('/__native-test/release/phase1', { method: 'POST' })
    assert(released.ok, `could not release the actual retry after relay settlement (${released.status})`)
    const retained = await released.json() as PhaseState
    assert(retained.head > afterMutation.head && retained.store_len === 1 && retained.oldest === retained.head &&
      retained.plan_mutated === planMutationBefore + 1 && retained.head_summary === 'case.trace.plan_mutated' &&
      retained.head_case_id === state.case_id,
      `stable retention-one head is not the one durable plan mutation at L: ${JSON.stringify(retained)}`)

    await waitFor(() => Promise.resolve(events), (items) => items.length >= 4,
      'native retry resync control and same-cursor snapshot L', 10_000)
    assert(eventCount(events) === 4, `retention-one reconnect must add exactly resync L and snapshot L: ${JSON.stringify(events)}`)
    const [resyncK, snapshotK, resyncL, snapshotL] = events
    assert(resyncL?.event_type === 'resync_required' && resyncL.cursor === retained.head,
      `retry must first report resync_required L: ${JSON.stringify(resyncL)}`)
    const retryResyncPayload = resyncL.payload as { requested_cursor?: string; oldest_available_cursor?: string }
    assert(retryResyncPayload.requested_cursor === afterMutation.head && retryResyncPayload.oldest_available_cursor === retained.head,
      `retry resync must name interrupted K and retained L: ${JSON.stringify(retryResyncPayload)}`)
    assert(snapshotL?.event_type === 'snapshot' && snapshotL.cursor === retained.head,
      `retry must next deliver authoritative same-cursor snapshot L: ${JSON.stringify(snapshotL)}`)
    assert(resyncK?.event_type === 'resync_required' && resyncK.cursor === afterMutation.head &&
      snapshotK?.event_type === 'snapshot' && snapshotK.cursor === afterMutation.head,
      `initial retained-window recovery must remain resync K then snapshot K: ${JSON.stringify(events.slice(0, 2))}`)

    const streamedL = snapshotL.payload as XSnapshot
    const atL = await adapter.getSnapshotAt(state.case_id, retained.head, identity.actorId, identity.role)
    const freshL = await adapter.getSnapshot(state.case_id, identity.actorId, identity.role)
    assert(freshL.cursor === retained.head, `fresh current snapshot cursor ${freshL.cursor} differs from L=${retained.head}`)
    assertValidSnapshotTimestamp(freshL, 'fresh L snapshot')
    assert(objectStanding(streamedL) === objectStanding(atL), 'reconnected L differs from authenticated Store.At(L)')
    assert(freshCurrentStanding(streamedL) === freshCurrentStanding(freshL), 'reconnected L differs from fresh authenticated current projection')
    assert(streamedL.visible_objects.some((object) => object.name === 'native events recovery plan change'),
      'reconnected L does not show the newly authorized recovery plan item')

    const reconnected = await waitFor(
      () => phaseState('phase1'),
      (currentState) => currentState.event_requests.length === 2 && currentState.active_streams === 1 && currentState.subscribers === 1,
      'one bounded native retry subscribed after the recovery mutation',
      10_000,
    )
    assert(reconnected.event_requests[1]?.last_query === afterMutation.head,
      `second native request did not use interrupted K: ${JSON.stringify(reconnected.event_requests)}`)

    const secondDrop = await fetch('/__native-test/disconnect/phase1', { method: 'POST' })
    assert(secondDrop.ok, `second phase-one stream drop failed (${secondDrop.status})`)
    const secondDropCount = (await secondDrop.json() as { closed: number }).closed
    assert(secondDropCount === 1, `second phase-one drop closed ${secondDropCount} actual stream connections, want 1`)
    await waitFor(() => Promise.resolve(errors), (items) => items.length === 2, 'second native EventSource error callback')
    dispose()
    const disposed = await waitFor(
      () => phaseState('phase1'),
      (currentState) => currentState.active_streams === 0 && currentState.subscribers === 0,
      'phase-one disposal during the second pending retry',
    )
    const eventCountBefore = events.length
    const errorCountBefore = errors.length
    await sleep(1_750) // greater than the configured 1,500 ms retry cap
    const afterCap = await phaseState('phase1')
    assert(afterCap.event_requests.length === disposed.event_requests.length && afterCap.event_requests.length === 2,
      `disposed second retry timer opened a third stream: ${JSON.stringify(afterCap.event_requests)}`)
    assert(afterCap.active_streams === 0 && afterCap.subscribers === 0, 'phase-one disposal left a subscriber active')
    assert(events.length === eventCountBefore && errors.length === errorCountBefore, 'callbacks fired after phase-one disposal')
    await adapter.logout()

    return {
      ok: true,
      phase: 'phase1',
      case_id: state.case_id,
      initial_cursor: state.initial_cursor,
      cursor: retained.head,
      interrupted_cursor: afterMutation.head,
      events: events.map(({ event_type, cursor }) => ({ type: event_type, cursor })),
      errors,
      requests_before: connected.event_requests.length,
      requests_after: afterCap.event_requests.length,
      durable_delta: retained.plan_mutated - planMutationBefore,
      reconnect_cursor: reconnected.event_requests[1]?.last_query ?? '',
    }
  } finally {
    dispose()
  }
}

async function phaseTwo(): Promise<BrowserProof> {
  const state = await phaseState('phase2')
  assert(state.retention === 2 && state.store_len === 2, `retention-two setup is not exact: ${JSON.stringify(state)}`)
  assert(state.head === state.initial_cursor && state.oldest < state.initial_cursor, 'phase-two C must be the retained head above its predecessor')
  const planMutationBefore = state.plan_mutated

  const adapter = new HttpCaseworkAdapter({
    base: `${location.origin}/phase2`,
    reconnectBaseMs: 5_000,
    reconnectMaxMs: 5_000,
  })
  const identity = await login(adapter)
  const before = await adapter.getSnapshot(state.case_id, identity.actorId, identity.role)
  assert(before.cursor === state.initial_cursor, `fresh browser snapshot cursor ${before.cursor} differs from C=${state.initial_cursor}`)
  assert(!before.visible_objects.some((object) => object.name === 'native events offline plan change'), 'phase-two test item already exists before its mutation')

  const events: StreamEvent[] = []
  const errors: string[] = []
  let errorAt = 0
  const dispose = adapter.subscribeEvents(
    state.case_id,
    state.initial_cursor,
    (event) => events.push(event),
    (error) => {
      errors.push(error.message)
      if (errorAt === 0) errorAt = Date.now()
    },
  )
  try {
    const connected = await waitFor(
      () => phaseState('phase2'),
      (current) => current.active_streams === 1 && current.subscribers === 1 && current.event_requests.length === 1,
      'phase-two native EventSource connection at C',
    )
    assert(events.length === 0, `subscribing at current C unexpectedly delivered callbacks: ${JSON.stringify(events)}`)
    const dropped = await fetch('/__native-test/disconnect/phase2', { method: 'POST' })
    assert(dropped.ok, `phase-two stream drop failed (${dropped.status})`)
    const dropCount = (await dropped.json() as { closed: number }).closed
    assert(dropCount === 1, `phase-two drop closed ${dropCount} actual stream connections, want 1`)
    await waitFor(
      () => phaseState('phase2'),
      (current) => current.active_streams === 0 && current.subscribers === 0,
      'phase-two native disconnect',
    )
    await waitFor(() => Promise.resolve(errors), (items) => items.length === 1, 'phase-two adapter onError')

    // The authorized offline mutation has a single real plan_mutated frame. The gateway must
    // report success while the stream is down; the two-entry store must still contain C.
    // Dispatch through the protected T06 server intent path. This tests the transport, not UI
    // action availability; action-advertisement coverage belongs to projection wiring.
    const intent: InteractionIntent = {
      intent_id: `native-f14-phase2-${crypto.randomUUID()}`,
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'ADD_DISCRETIONARY_WORK',
      target_object_id: 'task_prepare',
      case_id: state.case_id,
      client_cursor: state.initial_cursor,
      actor: { actor_id: identity.actorId, role: identity.role },
      parameters: {
        case_id: state.case_id,
        stage_id: 'task_prepare',
        kind: 'sandboxed_task',
        title: 'native events offline plan change',
        summary: 'Add one governed plan item while the native event connection is interrupted.',
        justification: 'The replay proof requires one authorized mutation during the stream outage.',
      },
    }
    const outcome = await adapter.dispatchIntent(intent)
    assert(outcome.success && outcome.new_cursor, `offline governed plan mutation was refused: ${JSON.stringify(outcome.refusal)}`)
    const offline = await waitFor(
      () => phaseState('phase2'),
      (current) => current.head === outcome.new_cursor && current.store_len === 2 && current.oldest === state.initial_cursor,
      'one offline plan mutation to retain C and append L',
    )
    assert(offline.head > state.initial_cursor, 'offline mutation did not advance the cursor')
    assert(offline.plan_mutated === planMutationBefore + 1 && offline.head_summary === 'case.trace.plan_mutated',
      `offline mutation did not produce exactly one durable plan_mutated frame: ${JSON.stringify(offline)}`)
    assert(offline.event_requests.length === 1 && offline.active_streams === 0 && offline.subscribers === 0,
      `the stream reconnected before the offline mutation finished: ${JSON.stringify(offline)}`)

    await waitFor(() => Promise.resolve(events), (items) => items.length === 1, 'reconnected snapshot L without resync', 15_000)
    const snapshotEvent = events[0]
    assert(snapshotEvent?.event_type === 'snapshot' && snapshotEvent.cursor === offline.head,
      `phase-two callback must be snapshot L and contain no resync control: ${JSON.stringify(events)}`)
    const streamed = snapshotEvent.payload as XSnapshot
    const atL = await adapter.getSnapshotAt(state.case_id, offline.head, identity.actorId, identity.role)
    const current = await adapter.getSnapshot(state.case_id, identity.actorId, identity.role)
    assert(current.cursor === offline.head, `fresh current snapshot cursor ${current.cursor} differs from L=${offline.head}`)
    assertValidSnapshotTimestamp(current, 'fresh L snapshot')
    assert(objectStanding(streamed) === objectStanding(atL), 'replayed L differs from authenticated Store.At(L)')
    assert(freshCurrentStanding(streamed) === freshCurrentStanding(current), 'replayed L differs from fresh authenticated current projection')
    assert(streamed.visible_objects.some((object) => object.name === 'native events offline plan change'),
      'replayed L does not show the newly authorized plan item')

    const reconnected = await waitFor(
      () => phaseState('phase2'),
      (currentState) => currentState.event_requests.length === 2 && currentState.active_streams === 1 && currentState.subscribers === 1,
      'one bounded native EventSource retry',
      15_000,
    )
    const reconnectDelay = Date.now() - errorAt
    assert(reconnectDelay >= 4_500 && reconnectDelay < 10_000,
      `configured 5,000 ms retry delay was not bounded as expected: ${reconnectDelay} ms`)
    assert(reconnected.event_requests[1]?.last_query === state.initial_cursor,
      `the new EventSource request did not resume from C: ${JSON.stringify(reconnected.event_requests)}`)

    dispose()
    const disposed = await waitFor(
      () => phaseState('phase2'),
      (currentState) => currentState.active_streams === 0 && currentState.subscribers === 0,
      'phase-two final disposal',
    )
    assert(disposed.event_requests.length === 2, 'phase-two disposal opened another request')
    const requestsBefore = connected.event_requests.length
    const requestsAfter = disposed.event_requests.length
    await adapter.logout()
    return {
      ok: true,
      phase: 'phase2',
      case_id: state.case_id,
      initial_cursor: state.initial_cursor,
      cursor: offline.head,
      interrupted_cursor: '',
      events: events.map(({ event_type, cursor }) => ({ type: event_type, cursor })),
      errors,
      requests_before: requestsBefore,
      requests_after: requestsAfter,
      durable_delta: offline.plan_mutated - planMutationBefore,
      reconnect_cursor: reconnected.event_requests[1]?.last_query ?? '',
    }
  } finally {
    dispose()
  }
}

export async function runNativeEventPhase(phase: string): Promise<BrowserProof> {
  if (phase === 'phase1') return phaseOne()
  if (phase === 'phase2') return phaseTwo()
  throw new Error(`unknown native EventSource phase ${phase}`)
}
