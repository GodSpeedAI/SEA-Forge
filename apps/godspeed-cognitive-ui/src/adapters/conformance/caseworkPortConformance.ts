import type {
  ActorRole,
  ArtifactPayload,
  CaseworkPort,
  InteractionIntent,
  StreamEvent,
  TemplateSourcePort,
} from '../../ports/contract'

export interface CaseworkPortConformanceScenario {
  port: CaseworkPort & TemplateSourcePort
  caseId: string
  actor: { actor_id: string; role: ActorRole }
  template: { ref: string; params: Record<string, unknown> }
  artifact: {
    refAfterDispatch: (response: Awaited<ReturnType<CaseworkPort['dispatchIntent']>>) => string | Promise<string>
    expectedProvenance: Partial<ArtifactPayload['provenance']>
    /** Local authored fixtures use a synthetic marker; live artifacts must hash their actual bytes. */
    digestMatchesContent: boolean
  }
  initialHistoryMinimum: number
  /** Local adapters are future-only; the HTTP adapter replays retained events after this cursor. */
  resume: 'replay-retained' | 'future-only'
  resumeWaitMs: number
  waitForMs: number
  acceptedIntent: (cursor: string) => InteractionIntent
  captureConsequentialState: () => Promise<string>
  stateBoundaryDescription: string
}

function assertConforms(condition: unknown, behavior: string, detail = ''): asserts condition {
  if (!condition) throw new Error(`CaseworkPort conformance failed: ${behavior}${detail ? ` — ${detail}` : ''}`)
}

function digestHex(digest: string): string | null {
  const hex = digest.startsWith('sha256:') ? digest.slice('sha256:'.length) : digest
  return /^[0-9a-f]{64}$/.test(hex) ? hex : null
}

async function sha256(content: string): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(content)))
  return [...digest].map((byte) => byte.toString(16).padStart(2, '0')).join('')
}

async function waitForEvent(
  events: StreamEvent[],
  predicate: (event: StreamEvent) => boolean,
  timeoutMs: number,
  behavior: string,
): Promise<StreamEvent> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const event = events.find(predicate)
    if (event) return event
    await new Promise((resolve) => setTimeout(resolve, 25))
  }
  throw new Error(`CaseworkPort conformance failed: timed out waiting for ${behavior}`)
}

/** The common behavioral assertions used by local tests and the real-stack live run. */
export async function runCaseworkPortConformance(s: CaseworkPortConformanceScenario): Promise<void> {
  const { port } = s

  const snapshot = await port.getSnapshot(s.caseId, s.actor.actor_id, s.actor.role)
  assertConforms(snapshot.case_id === s.caseId, 'snapshot names the requested case')
  assertConforms(snapshot.cursor.length > 0, 'snapshot has a cursor')
  assertConforms(snapshot.visible_objects.length > 0, 'snapshot contains projected objects')

  const trajectoryBefore = await port.queryTemporalTrajectory(s.caseId)
  assertConforms(trajectoryBefore.case_id === s.caseId, 'trajectory is case-scoped')
  assertConforms(trajectoryBefore.points.length >= s.initialHistoryMinimum, 'trajectory exposes configured retained history',
    `got ${trajectoryBefore.points.length}, expected at least ${s.initialHistoryMinimum}`)
  assertConforms(trajectoryBefore.base_cursor === trajectoryBefore.points[0]?.cursor, 'trajectory base cursor matches its first point')
  assertConforms(trajectoryBefore.head_cursor === trajectoryBefore.points.at(-1)?.cursor, 'trajectory head cursor matches its last point')
  const historicalCursor = trajectoryBefore.points[0]!.cursor
  const historicalBefore = await port.getSnapshotAt(s.caseId, historicalCursor, s.actor.actor_id, s.actor.role)
  assertConforms(historicalBefore.case_id === s.caseId && historicalBefore.cursor === historicalCursor,
    'retained history resolves to its exact snapshot cursor')

  const templates = await port.getTemplates()
  assertConforms(templates.some((template) => template.template_ref === s.template.ref), 'template source lists the configured template', s.template.ref)
  const preflight = await port.preflightTemplate(s.template.ref, s.template.params)
  assertConforms(preflight.passed && digestHex(preflight.digest ?? '') !== null, 'valid template preflight returns a SHA-256 digest',
    JSON.stringify(preflight))
  assertConforms(preflight.reasons.length === 0, 'passing preflight has no refusal reasons', JSON.stringify(preflight.reasons))

  const events: StreamEvent[] = []
  const errors: Error[] = []
  const unsubscribe = port.subscribeEvents(s.caseId, snapshot.cursor, (event) => events.push(event), (error) => errors.push(error))
  const acceptedIntent = s.acceptedIntent(snapshot.cursor)
  const accepted = await port.dispatchIntent(acceptedIntent)
  assertConforms(accepted.success, 'valid intent is accepted', JSON.stringify(accepted.refusal ?? accepted.error_message ?? {}))
  assertConforms(typeof accepted.new_cursor === 'string' && accepted.new_cursor.length > 0 && accepted.new_cursor !== snapshot.cursor,
    'accepted intent advances the projection cursor', JSON.stringify(accepted))
  let mutationEvent: StreamEvent
  try {
    mutationEvent = await waitForEvent(
      events,
      (event) => event.cursor > snapshot.cursor && (event.event_type === 'snapshot' || event.event_type === 'settlement_recorded'),
      s.waitForMs,
      'a post-dispatch event with an advanced cursor',
    )
  } finally {
    unsubscribe()
  }
  assertConforms(errors.length === 0, 'event subscription stays healthy', errors.map((error) => error.message).join('; '))
  assertConforms(mutationEvent.cursor > snapshot.cursor, 'event cursor advances beyond the resume point')

  const headCursorBeforeResume = (await port.queryTemporalTrajectory(s.caseId)).head_cursor
  const resumedEvents: StreamEvent[] = []
  const unsubscribeResume = port.subscribeEvents(
    s.caseId,
    snapshot.cursor,
    (event) => resumedEvents.push(event),
    (error) => errors.push(error),
  )
  try {
    if (s.resume === 'replay-retained') {
      const replay = await waitForEvent(
        resumedEvents,
        (event) => event.cursor > snapshot.cursor,
        s.waitForMs,
        'retained events after the supplied resume cursor',
      )
      assertConforms(replay.cursor >= accepted.new_cursor!, 'resumed stream replays the accepted mutation cursor',
        `got ${replay.cursor}, expected at least ${accepted.new_cursor}`)
    } else {
      await new Promise((resolve) => setTimeout(resolve, s.resumeWaitMs))
      assertConforms(resumedEvents.every((event) => event.cursor > headCursorBeforeResume),
        'future-only fixture does not replay events already present at subscription time',
        `subscription head was ${headCursorBeforeResume}; received ${resumedEvents.map((event) => event.cursor).join(', ')}`)
    }
  } finally {
    unsubscribeResume()
  }

  const trajectoryAfterAccepted = await port.queryTemporalTrajectory(s.caseId)
  assertConforms(trajectoryAfterAccepted.points.some((point) => point.cursor === accepted.new_cursor),
    'accepted intent is retained in trajectory history', accepted.new_cursor)
  assertConforms(trajectoryAfterAccepted.points.length >= trajectoryBefore.points.length,
    'trajectory keeps its prior retained history')
  const historicalAfter = await port.getSnapshotAt(s.caseId, historicalCursor, s.actor.actor_id, s.actor.role)
  assertConforms(historicalAfter.cursor === historicalBefore.cursor && JSON.stringify(historicalAfter) === JSON.stringify(historicalBefore),
    'new activity does not rewrite a retained snapshot')

  const artifactRef = await s.artifact.refAfterDispatch(accepted)
  const artifact = await port.resolveArtifact(artifactRef)
  assertConforms(artifact.content.length > 0, 'artifact resolution returns source content')
  assertConforms(digestHex(artifact.digest) !== null, 'artifact carries a well-formed SHA-256 digest', artifact.digest)
  assertConforms(artifact.evidence_id.length > 0 && typeof artifact.provenance.case_id === 'string' &&
    typeof artifact.provenance.plan_item_id === 'string' && typeof artifact.provenance.invocation_id === 'string' &&
    artifact.provenance.run_id.length > 0,
  'artifact includes evidence and schema-shaped execution provenance with a source-backed run id', JSON.stringify(artifact.provenance))
  for (const [key, value] of Object.entries(s.artifact.expectedProvenance)) {
    assertConforms(artifact.provenance[key as keyof ArtifactPayload['provenance']] === value,
      `artifact provenance ${key} matches the configured source`, `got ${artifact.provenance[key as keyof ArtifactPayload['provenance']]}, expected ${value}`)
  }
  if (s.artifact.digestMatchesContent) {
    assertConforms(digestHex(artifact.digest) === await sha256(artifact.content),
      'artifact digest matches the actual returned content bytes', artifact.digest)
  }

  const stateBeforeRefusal = await s.captureConsequentialState()
  const staleIntent = { ...acceptedIntent, intent_id: `${acceptedIntent.intent_id}-stale`, client_cursor: snapshot.cursor }
  const refusal = await port.dispatchIntent(staleIntent)
  assertConforms(!refusal.success &&
    (refusal.refusal?.refusal_kind === 'STALE_PROJECTION' || refusal.error_code === 'STALE_PROJECTION'),
  'stale intent surfaces the typed STALE_PROJECTION refusal', JSON.stringify(refusal.refusal ?? refusal.error_code ?? {}))
  const stateAfterRefusal = await s.captureConsequentialState()
  assertConforms(stateBeforeRefusal === stateAfterRefusal, 'refused intent does not change the configured consequential-state boundary',
    s.stateBoundaryDescription)
  const trajectoryAfterRefusal = await port.queryTemporalTrajectory(s.caseId)
  assertConforms(trajectoryAfterRefusal.points.length === trajectoryAfterAccepted.points.length &&
    trajectoryAfterRefusal.head_cursor === trajectoryAfterAccepted.head_cursor,
  'refused intent does not append a trajectory revision')
}
