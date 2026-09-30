// The live HTTP adapter: the CaseworkPort served by the Go casework gateway
// (apps/godspeed-casework-go) over same-origin fetch + EventSource. This is
// the production path — the local contract adapter stays behind a dynamic
// import so production bundles cannot ship it (see main.tsx and the bundle
// check in the T08 evidence).
//
// Identity comes from the gateway session (T07): the adapter never sends a
// client-asserted actor. The actorId/role arguments the port historically
// carries are accepted for interface compatibility and deliberately IGNORED
// for authority — the snapshot's perspective is what the kernel verified.
//
// Refusals are the T01 typed envelope and are surfaced verbatim; the adapter
// never retries a mutation (STALE_PROJECTION means the UI refetches and shows
// the refusal — deciding again is a human act).
//
// Honest limits (documented, not hidden):
// - queryTemporalTrajectory reads only the gateway's bounded retained
//   revisions; unavailable cold or evicted history is returned as a typed error.
// - resolveArtifact uses the authenticated GET /api/artifacts/{digest} route;
//   the gateway verifies the session perspective and reads only through the
//   kernel's content-addressed artifact.get operation.

import type {
  ActorRole,
  ArtifactPayload,
  CaseworkPort,
  SessionIdentity,
  SessionPort,
  TemporalCheckpoint,
  TemplateEntryOption,
  TemplateSourcePort,
  IntentResponse,
  InteractionIntent,
  StreamEvent,
  TemporalTrajectoryResponse,
  XSnapshot,
} from '../../ports/contract'

export interface HttpAdapterOptions {
  /** Same-origin base (default: location.origin; tests pass the gateway URL). */
  base?: string
  /** Credentials policy for fetches (same-origin cookies are the session). */
  credentials?: RequestCredentials
  /** Reconnect backoff base for the fetch-based stream, ms. */
  reconnectBaseMs?: number
  /** Reconnect backoff cap, ms. */
  reconnectMaxMs?: number
}

/** The session identity this adapter resolves (the contract SessionIdentity, T07/T09). */
export type { SessionIdentity }

/** Typed refusal from the gateway (T01 envelope), thrown for pre-dispatch rejections. */
export class HttpRefusalError extends Error {
  readonly refusalKind: string
  readonly intentId?: string
  constructor(refusalKind: string, message: string, intentId?: string) {
    super(message)
    this.name = 'HttpRefusalError'
    this.refusalKind = refusalKind
    this.intentId = intentId
  }
}

const csrfCookie = (): string => {
  if (typeof document === 'undefined') return ''
  const match = document.cookie.match(/(?:^|;\s*)casework_csrf=([^;]+)/)
  return match ? decodeURIComponent(match[1]!) : ''
}

const ARTIFACT_CONTENT_TYPES = new Set<ArtifactPayload['content_type']>([
  'text/markdown',
  'application/json',
  'text/x-diff',
  'text/plain',
])

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function isArtifactPayload(value: unknown): value is ArtifactPayload {
  if (!isRecord(value) || !isRecord(value.provenance)) return false
  const p = value.provenance
  return (
    typeof value.evidence_id === 'string' &&
    typeof value.name === 'string' &&
    typeof value.digest === 'string' &&
    typeof value.content === 'string' &&
    ARTIFACT_CONTENT_TYPES.has(value.content_type as ArtifactPayload['content_type']) &&
    typeof p.case_id === 'string' &&
    typeof p.plan_item_id === 'string' &&
    typeof p.invocation_id === 'string' &&
    typeof p.run_id === 'string' &&
    (p.question_id === undefined || typeof p.question_id === 'string') &&
    (p.claim_id === undefined || typeof p.claim_id === 'string') &&
    (p.commit_sha === undefined || typeof p.commit_sha === 'string') &&
    (p.pr_number === undefined || typeof p.pr_number === 'number')
  )
}

function isTemporalTrajectoryResponse(value: unknown, caseId: string): value is TemporalTrajectoryResponse {
  if (!isRecord(value) || value.case_id !== caseId) return false
  if (typeof value.base_cursor !== 'string' || value.base_cursor.length === 0) return false
  if (typeof value.head_cursor !== 'string' || value.head_cursor.length === 0) return false
  if (!Array.isArray(value.points) || value.points.length === 0 || !value.points.every(isTemporalCheckpoint)) return false
  return value.base_cursor === value.points[0].cursor && value.head_cursor === value.points.at(-1)!.cursor
}

function isTemporalCheckpoint(value: unknown): value is TemporalCheckpoint {
  if (!isRecord(value)) return false
  const completed = value.completed_plan_items_count
  const total = value.total_plan_items_count
  return (
    typeof value.cursor === 'string' && value.cursor.length > 0 &&
    typeof value.timestamp === 'string' &&
    typeof value.event_type === 'string' &&
    typeof value.summary === 'string' &&
    typeof value.actor_id === 'string' &&
    typeof value.actor_role === 'string' &&
    typeof value.consequential === 'boolean' &&
    (value.active_stage_id === undefined || typeof value.active_stage_id === 'string') &&
    typeof completed === 'number' && Number.isInteger(completed) && completed >= 0 &&
    typeof total === 'number' && Number.isInteger(total) && total >= 0 &&
    completed <= total
  )
}

function sha256Hex(digest: string): string | null {
  const hex = digest.startsWith('sha256:') ? digest.slice('sha256:'.length) : digest
  return /^[0-9a-f]{64}$/.test(hex) ? hex : null
}

export class HttpCaseworkAdapter implements CaseworkPort, SessionPort, TemplateSourcePort {
  private readonly base: string
  private readonly credentials: RequestCredentials
  private readonly reconnectBaseMs: number
  private readonly reconnectMaxMs: number
  private identity: SessionIdentity | null = null
  /**
   * Minimal cookie jar for non-browser contexts (bun/tests): browsers send
   * same-origin cookies automatically; fetch elsewhere does not, and the
   * gateway's double-submit CSRF and HttpOnly session cookie both must ride
   * along. Populated from every response's Set-Cookie.
   */
  private jar = new Map<string, string>()
  private readonly inBrowser = typeof document !== 'undefined'

  constructor(opts: HttpAdapterOptions = {}) {
    this.base = (opts.base ?? (typeof location !== 'undefined' ? location.origin : '')).replace(/\/$/, '')
    this.credentials = opts.credentials ?? 'include'
    this.reconnectBaseMs = opts.reconnectBaseMs ?? 500
    this.reconnectMaxMs = opts.reconnectMaxMs ?? 10_000
  }

  // --- session ----------------------------------------------------------------

  /** The session identity, or null while unauthenticated. Cached after first learn. */
  async session(): Promise<SessionIdentity | null> {
    if (this.identity) return this.identity
    const res = await this.fetchJSON('/api/session')
    if (res.status === 401) return null
    if (!res.ok) throw new HttpRefusalError('UNAVAILABLE', `session lookup failed (${res.status})`)
    const body = res.body as { user?: string; actor_id?: string; role?: string; roles?: string[] }
    if (!body.actor_id) return null
    this.identity = {
      user: body.user ?? body.actor_id,
      actor_id: body.actor_id,
      role: body.role ?? body.roles?.[0] ?? '',
      roles: body.roles ?? [],
      display_name: body.user,
      kind: 'human',
    }
    return this.identity
  }

  /**
   * The CSRF token: the readable cookie mirror in a browser, or the Set-Cookie
   * header of a warm-up GET elsewhere (bun/tests keep no cookie jar).
   */
  private async ensureCsrf(): Promise<string> {
    const fromCookie = csrfCookie()
    if (fromCookie) return fromCookie
    let token = this.jar.get('casework_csrf') ?? ''
    if (!token) {
      // The CSRF cookie is issued by the identity gate on unauthenticated requests to
      // PROTECTED endpoints (session.go withIdentity), not by /api/healthz.
      const res = await fetch(`${this.base}/api/session`, { credentials: this.credentials, headers: this.extraHeaders() })
      this.harvestCookies(res)
      token = this.jar.get('casework_csrf') ?? ''
    }
    return token
  }

  /** Store every Set-Cookie name=value this response carries (non-browser only). */
  private harvestCookies(res: Response): void {
    if (this.inBrowser) return
    for (const cookie of res.headers.getSetCookie?.() ?? []) {
      const [pair] = cookie.split(';')
      const eq = pair.indexOf('=')
      if (eq > 0) this.jar.set(pair.slice(0, eq).trim()!, pair.slice(eq + 1).trim()!)
    }
  }

  /** Ambient cookies in a browser; the jar's contents elsewhere. */
  private extraHeaders(): Record<string, string> {
    if (this.inBrowser || this.jar.size === 0) return {}
    const header = [...this.jar.entries()].map(([name, value]) => `${name}=${value}`).join('; ')
    return { Cookie: header }
  }

  /** Login (the CSRF token is minted by any prior GET; fetch it first if missing). */
  async login(username: string, password: string): Promise<SessionIdentity> {
    await this.ensureCsrf()
    const res = await this.rawPOST('/api/auth/login', { username, password })
    if (!res.ok) {
      const body = (await res.json().catch(() => ({}))) as { error?: string }
      throw new HttpRefusalError(body.error ?? 'INVALID', `login failed (${res.status})`)
    }
    this.identity = null
    const session = await this.session()
    if (!session) throw new HttpRefusalError('UNAVAILABLE', 'login succeeded but the session did not resolve')
    return session
  }

  async logout(): Promise<void> {
    await this.rawPOST('/api/auth/logout', {})
    this.identity = null
  }

  // --- reads --------------------------------------------------------------------

  async getSnapshot(_caseId: string, _actorId: string, _role: ActorRole): Promise<XSnapshot> {
    // The session is the identity; the historical actorId/role arguments are
    // not sent (the gateway refuses client-asserted actors, T07).
    const res = await this.fetchJSON(`/api/world?case_id=${encodeURIComponent(_caseId)}`)
    if (res.status === 401) throw new HttpRefusalError('UNAUTHORIZED_ROLE', 'Your session has ended. Sign in again.')
    if (!res.ok) throw await this.refusalFrom(res)
    return this.widen((res.body as { snapshot: Record<string, unknown> }).snapshot)
  }

  async getSnapshotAt(_caseId: string, cursor: string, _actorId: string, _role: ActorRole): Promise<XSnapshot> {
    const res = await this.fetchJSON(`/api/world?case_id=${encodeURIComponent(_caseId)}&cursor=${encodeURIComponent(cursor)}`)
    if (res.status === 404) throw new Error(`No snapshot at ${cursor} (resync required)`)
    if (res.status === 401) throw new HttpRefusalError('UNAUTHORIZED_ROLE', 'Your session has ended. Sign in again.')
    if (!res.ok) throw await this.refusalFrom(res)
    return this.widen((res.body as { snapshot: Record<string, unknown> }).snapshot)
  }

  // --- authority ------------------------------------------------------------------

  async dispatchIntent(intent: InteractionIntent): Promise<IntentResponse> {
    // The gateway overwrites actor fields with the session's verified standing
    // and re-verifies every delegation against the kernel (T02/T07).
    const res = await this.rawPOST('/api/intents', intent)
    const body = (await res.json().catch(() => ({}))) as IntentResponse & { error?: string; error_class?: string }
    if (!res.ok && !body.intent_id) {
      // Pre-dispatch gateway refusals (rate limit, CSRF, auth) arrive as HTTP
      // errors, not intent outcomes: surface them typed, never retry.
      throw new HttpRefusalError(body.error_class ?? 'UNAVAILABLE', body.error ?? `intent dispatch failed (${res.status})`, intent.intent_id)
    }
    return body as IntentResponse
  }

  // --- artifacts --------------------------------------------------------------------

  async resolveArtifact(digest: string): Promise<ArtifactPayload> {
    const expected = sha256Hex(digest)
    if (!expected) throw new HttpRefusalError('INVALID', 'artifact digest must be a SHA-256 hex digest')
    await this.requireSession()
    const res = await this.fetchJSON(`/api/artifacts/${encodeURIComponent(digest)}`)
    if (!res.ok) throw await this.refusalFrom(res)
    if (!isArtifactPayload(res.body)) {
      throw new HttpRefusalError('UNAVAILABLE', 'the gateway returned a malformed artifact payload')
    }
    if (sha256Hex(res.body.digest) !== expected) {
      throw new HttpRefusalError('INTEGRITY_MISMATCH', 'the gateway returned a different artifact digest')
    }
    return res.body
  }

  // --- temporal ------------------------------------------------------------------

  async queryTemporalTrajectory(caseId: string): Promise<TemporalTrajectoryResponse> {
    await this.requireSession()
    const res = await this.fetchJSON(`/api/trajectory?case_id=${encodeURIComponent(caseId)}`)
    if (res.status === 401) throw new HttpRefusalError('UNAUTHORIZED_ROLE', 'Your session has ended. Sign in again.')
    if (!res.ok) throw await this.refusalFrom(res)
    if (!isTemporalTrajectoryResponse(res.body, caseId)) {
      throw new HttpRefusalError('INVALID', 'the gateway returned a malformed or foreign case trajectory')
    }
    return res.body
  }

  // --- event stream -------------------------------------------------------------

  subscribeEvents(
    caseId: string,
    sinceCursor: string | undefined,
    onEvent: (event: StreamEvent) => void,
    onError: (err: Error) => void,
  ): () => void {
    if (typeof EventSource === 'undefined') {
      // Non-browser runtimes (bun/tests) ship no EventSource, and a fetch-based
      // stream is the deployment-flexible transport anyway (it carries the
      // session cookie explicitly). Parse SSE lines off the response body.
      return this.subscribeEventsFetch(caseId, sinceCursor, onEvent, onError)
    }
    let closed = false
    let lastCursor = sinceCursor
    let backoff = this.reconnectBaseMs
    let source: EventSource | undefined
    let retryTimer: ReturnType<typeof setTimeout> | undefined
    const maxBackoff = Math.max(this.reconnectBaseMs, this.reconnectMaxMs)

    const scheduleReconnect = (error: Error) => {
      if (closed) return
      onError(error)
      if (retryTimer !== undefined) return
      const delay = backoff
      backoff = Math.min(backoff * 2, maxBackoff)
      retryTimer = setTimeout(() => {
        retryTimer = undefined
        connect()
      }, delay)
    }

    const connect = () => {
      if (closed) return
      const url = new URL(`${this.base}/api/events`, this.base)
      if (lastCursor) url.searchParams.set('last', lastCursor)
      let current: EventSource
      try {
        current = new EventSource(url.toString(), { withCredentials: this.credentials === 'include' })
      } catch (err) {
        scheduleReconnect(err instanceof Error ? err : new Error(String(err)))
        return
      }
      source = current
      const route = (ev: MessageEvent) => {
        if (closed || source !== current) return
        const event = this.parseStreamEvent(ev, caseId)
        if (!event) return
        const cursor = event.cursor
        // The gateway cursor is monotonic. Ignore a replayed frame, including the
        // last frame at a reconnect boundary, before delivering it twice.
        if (cursor && lastCursor && cursor <= lastCursor) return
        if (event.event_type === 'resync_required') {
          // A control frame can carry the oldest retained cursor. The snapshot
          // for that same cursor still has to be delivered as authoritative state.
          onEvent(event)
        } else {
          lastCursor = cursor
          backoff = this.reconnectBaseMs
          onEvent(event)
        }
      }
      for (const kind of ['snapshot', 'execution_progress', 'settlement_recorded', 'resync_required']) {
        current.addEventListener(kind, route as EventListener)
      }
      current.onerror = () => {
        if (closed || source !== current) return
        current.close() // disable EventSource's unbounded native retry loop
        source = undefined
        scheduleReconnect(new Error(`event stream interrupted (reconnecting; backoff cap ${this.reconnectMaxMs}ms)`))
      }
    }

    connect()
    return () => {
      closed = true
      if (retryTimer !== undefined) clearTimeout(retryTimer)
      retryTimer = undefined
      source?.close()
      source = undefined
    }
  }

  /** Fetch-streaming subscribe: parse SSE frames off a ReadableStream, reconnect with backoff. */
  private subscribeEventsFetch(
    caseId: string,
    sinceCursor: string | undefined,
    onEvent: (event: StreamEvent) => void,
    onError: (err: Error) => void,
  ): () => void {
    let closed = false
    let lastCursor = sinceCursor
    const controller = new AbortController()
    const run = async () => {
      let backoff = this.reconnectBaseMs
      const maxBackoff = Math.max(this.reconnectBaseMs, this.reconnectMaxMs)
      while (!closed) {
        try {
          const url = new URL(`${this.base}/api/events`, this.base)
          if (lastCursor) url.searchParams.set('last', lastCursor)
          const res = await fetch(url, { credentials: this.credentials, headers: this.extraHeaders(), signal: controller.signal })
          if (!res.ok || !res.body) {
            const detail = await res.text().catch(() => '')
            throw new Error(`event stream status ${res.status}: ${detail.slice(0, 200)}`)
          }
          const reader = res.body.getReader()
          const decoder = new TextDecoder()
          let buffer = ''
          let frame: { id?: string; event?: string; data?: string } = {}
          for (;;) {
            const { done, value } = await reader.read()
            if (closed || done) break
            buffer += decoder.decode(value, { stream: true })
            let newline: number
            while ((newline = buffer.indexOf('\n')) >= 0) {
              const line = buffer.slice(0, newline).replace(/\r$/, '')
              buffer = buffer.slice(newline + 1)
              if (line.startsWith('id: ')) frame.id = line.slice(4)
              else if (line.startsWith('event: ')) frame.event = line.slice(7)
              else if (line.startsWith('data: ')) frame.data = (frame.data ?? '') + line.slice(6)
              else if (line === '' && (frame.event || frame.data)) {
                const ev = new MessageEvent(frame.event ?? 'message', { data: frame.data, lastEventId: frame.id })
                const event = this.parseStreamEvent(ev, caseId)
                if (event && (!lastCursor || event.cursor > lastCursor)) {
                  if (event.event_type === 'resync_required') {
                    onEvent(event)
                  } else {
                    lastCursor = event.cursor
                    backoff = this.reconnectBaseMs
                    onEvent(event)
                  }
                }
                frame = {}
              }
            }
          }
          if (closed) break
          throw new Error('event stream ended')
        } catch (err) {
          if (closed) return
          onError(err instanceof Error ? err : new Error(String(err)))
          await new Promise((r) => setTimeout(r, backoff))
          backoff = Math.min(backoff * 2, maxBackoff)
        }
      }
    }
    void run()
    return () => {
      closed = true
      controller.abort()
    }
  }

  // --- templates (additive port surface; optional on CaseworkPort) -----------------

  async getTemplates(): Promise<readonly TemplateEntryOption[]> {
    const res = await this.fetchJSON('/api/templates')
    if (res.status === 401) throw new HttpRefusalError('UNAUTHORIZED_ROLE', 'Your session has ended. Sign in again.')
    if (!res.ok) throw await this.refusalFrom(res)
    const wire = (res.body as { templates?: Record<string, unknown>[] }).templates ?? []
    // Normalize the wire parameter shape (golden templates-entry-options.json: `type`,
    // `default_value`) onto the port's naming (`param_type`, `default`).
    return wire.map((t) => ({
      template_ref: String(t.template_ref),
      title: String(t.title),
      ...(t.description !== undefined && t.description !== null ? { description: String(t.description) } : {}),
      parameters: ((t.parameters ?? []) as Record<string, unknown>[]).map((p) => ({
        name: String(p.name),
        ...(p.title !== undefined && p.title !== null ? { title: String(p.title) } : {}),
        ...(p.description !== undefined && p.description !== null ? { description: String(p.description) } : {}),
        param_type: String(p.type ?? p.param_type ?? 'string'),
        required: p.required === undefined ? false : Boolean(p.required),
        ...(p.default_value !== undefined || p.default !== undefined
          ? { default: String(p.default_value ?? p.default) }
          : {}),
        ...(Array.isArray(p.options) ? { options: (p.options as unknown[]).map(String) } : {}),
      })),
    }))
  }

  async preflightTemplate(
    templateRef: string,
    params: Record<string, unknown>,
  ): Promise<{ passed: boolean; digest?: string; reasons: readonly string[] }> {
    const res = await this.rawPOST('/api/templates/preflight', { template_ref: templateRef, params })
    const body = (await res.json().catch(() => ({}))) as Record<string, unknown>
    if (!res.ok && body.error) throw new HttpRefusalError(String(body.error_class ?? 'INVALID'), String(body.error))
    return body as { passed: boolean; digest?: string; reasons: readonly string[] }
  }

  // --- internals -----------------------------------------------------------------

  private async requireSession(): Promise<SessionIdentity> {
    const session = await this.session()
    if (!session) throw new HttpRefusalError('UNAUTHORIZED_ROLE', 'Sign in before working with the case.')
    return session
  }

  private async fetchJSON(path: string): Promise<{ ok: boolean; status: number; body: any; json: () => Promise<unknown> }> {
    const res = await fetch(`${this.base}${path}`, { credentials: this.credentials, headers: this.extraHeaders() })
    this.harvestCookies(res)
    const body = await res.json().catch(() => ({}))
    return { ok: res.ok, status: res.status, body, json: async () => body }
  }

  private async rawPOST(path: string, body: unknown): Promise<Response> {
    const token = await this.ensureCsrf()
    const res = await fetch(`${this.base}${path}`, {
      method: 'POST',
      credentials: this.credentials,
      headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': token, ...this.extraHeaders() },
      body: JSON.stringify(body),
    })
    this.harvestCookies(res)
    return res
  }

  private async refusalFrom(res: { status: number; json: () => Promise<unknown> }): Promise<Error> {
    const body = await res.json().catch(() => ({}))
    const record = isRecord(body) ? body : {}
    const detail = isRecord(record.error) ? record.error : {}
    const kind = typeof record.error_class === 'string'
      ? record.error_class
      : typeof detail.kind === 'string'
        ? detail.kind
        : 'UNAVAILABLE'
    const message = typeof detail.note === 'string'
      ? detail.note
      : typeof record.error === 'string'
        ? record.error
        : `request failed (${res.status})`
    return new HttpRefusalError(kind, message)
  }

  private parseStreamEvent(ev: MessageEvent, caseId: string): StreamEvent | null {
    let data: unknown
    try {
      data = JSON.parse(ev.data as string)
    } catch {
      return null
    }
    if (!isRecord(data) || !isRecord(data.payload)) return null
    const eventType = typeof data.event_type === 'string' ? data.event_type : ev.type
    const cursor = typeof data.cursor === 'string' ? data.cursor : ev.lastEventId
    if (typeof eventType !== 'string' || typeof cursor !== 'string' || cursor.length === 0) return null
    const payload = data.payload
    if (payload.case_id !== undefined && (typeof payload.case_id !== 'string' || payload.case_id !== caseId)) return null
    return {
      event_type: eventType as StreamEvent['event_type'],
      cursor,
      timestamp: (data.timestamp as string) ?? new Date().toISOString(),
      payload,
    } as StreamEvent
  }

  /** Widen a contract snapshot to the UI's XSnapshot (x extensions optional). */
  private widen(snapshot: Record<string, unknown>): XSnapshot {
    const out = snapshot as unknown as XSnapshot
    if (!out.visible_objects) return out
    return { ...out, visible_objects: out.visible_objects.map((o) => ({ ...o, x: o.x })) }
  }
}
