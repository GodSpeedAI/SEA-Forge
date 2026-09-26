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
// - queryTemporalTrajectory is not served by this deployment yet (history is
//   reachable through getSnapshotAt with cursors observed on the event
//   stream); the call throws a typed error instead of fabricating points.
// - resolveArtifact goes through the OPEN_ARTIFACT intent (the kernel's
//   content-addressed artifact.get); the ref it accepts is therefore an
//   artifact digest.

import type {
  ActorRole,
  ArtifactPayload,
  CaseworkPort,
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

/** A session the adapter learned from GET /api/session. */
export interface SessionIdentity {
  user: string
  actor_id: string
  role: string
  roles: readonly string[]
}

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

export class HttpCaseworkAdapter implements CaseworkPort, TemplateSourcePort {
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
    // Artifacts are content-addressed in the kernel; the served surface is the
    // OPEN_ARTIFACT intent (artifact.get). The response's resulting payload is
    // re-hashed by the gateway; the adapter surfaces the payload verbatim.
    const identity = await this.requireSession()
    const response = await this.dispatchIntent({
      intent_id: `art-${crypto.randomUUID()}`,
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'OPEN_ARTIFACT',
      target_object_id: digest,
      case_id: '',
      client_cursor: '',
      actor: { actor_id: identity.actor_id, role: identity.role as ActorRole },
      parameters: { digest },
    } as unknown as InteractionIntent)
    if (!response.success) {
      const refusal = response.refusal
      throw new HttpRefusalError(refusal?.refusal_kind ?? 'UNAVAILABLE', refusal?.message ?? 'artifact refused')
    }
    const payload = response.resulting_object as unknown as
      | { content_base64?: string; media_type?: string; name?: string }
      | undefined
    if (!payload?.content_base64) {
      throw new HttpRefusalError('UNAVAILABLE', 'the kernel returned no artifact content for this digest')
    }
    const content = atob(payload.content_base64)
    return {
      evidence_id: digest,
      name: payload.name ?? digest,
      digest,
      content_type: (payload.media_type ?? 'application/octet-stream') as ArtifactPayload['content_type'],
      content,
      provenance: { case_id: '', plan_item_id: '', invocation_id: '', run_id: '' },
    }
  }

  // --- temporal ------------------------------------------------------------------

  async queryTemporalTrajectory(_caseId: string): Promise<TemporalTrajectoryResponse> {
    // Honest limit: this deployment does not serve a trajectory route yet.
    // History navigation happens through cursors observed on the event stream
    // (getSnapshotAt). Throwing typed beats fabricating points.
    throw new HttpRefusalError(
      'UNAVAILABLE',
      'Temporal trajectory is not served by this deployment yet; history is reachable through revision cursors.',
    )
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
    const url = new URL(`${this.base}/api/events`, this.base)
    if (sinceCursor) url.searchParams.set('last', sinceCursor)
    const es = new EventSource(url.toString(), { withCredentials: this.credentials === 'include' })
    const route = (ev: MessageEvent) => {
      const event = this.parseStreamEvent(ev, caseId)
      if (event) onEvent(event)
    }
    for (const kind of ['snapshot', 'execution_progress', 'settlement_recorded', 'resync_required']) {
      es.addEventListener(kind, route as EventListener)
    }
    es.onerror = () => {
      // EventSource retries on its own; the error callback carries connection state.
      onError(new Error(`event stream interrupted (reconnecting; backoff cap ${this.reconnectMaxMs}ms)`))
    }
    return () => es.close()
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
      let backoff = this.reconnectBaseMs ?? 500
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
                if (frame.id) lastCursor = frame.id
                const event = this.parseStreamEvent(ev, caseId)
                if (event) onEvent(event)
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
          backoff = Math.min(backoff * 2, this.reconnectMaxMs ?? 10_000)
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

  async getTemplates(): Promise<
    readonly {
      template_ref: string
      title: string
      description?: string
      parameters: readonly { name: string; param_type: string; required: boolean; default?: string }[]
    }[]
  > {
    const res = await this.fetchJSON('/api/templates')
    if (res.status === 401) throw new HttpRefusalError('UNAUTHORIZED_ROLE', 'Your session has ended. Sign in again.')
    if (!res.ok) throw await this.refusalFrom(res)
    return (res.body as { templates: { template_ref: string; title: string; description?: string; parameters: { name: string; param_type: string; required: boolean; default?: string }[] }[] })
      .templates
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
    const body = (await res.json().catch(() => ({}))) as { error?: string; error_class?: string }
    return new HttpRefusalError(body.error_class ?? 'UNAVAILABLE', body.error ?? `request failed (${res.status})`)
  }

  private parseStreamEvent(ev: MessageEvent, caseId: string): StreamEvent | null {
    let data: Record<string, unknown>
    try {
      data = JSON.parse(ev.data as string) as Record<string, unknown>
    } catch {
      return null
    }
    const payload = data.payload as Record<string, unknown> | undefined
    const snapCase = (payload?.case_id as string | undefined) ?? caseId
    if (snapCase && caseId && snapCase !== caseId) return null
    return {
      event_type: (data.event_type as string) ?? ev.type,
      cursor: (data.cursor as string) ?? ev.lastEventId,
      timestamp: (data.timestamp as string) ?? new Date().toISOString(),
      payload: data.payload,
    } as StreamEvent
  }

  /** Widen a contract snapshot to the UI's XSnapshot (x extensions optional). */
  private widen(snapshot: Record<string, unknown>): XSnapshot {
    const out = snapshot as unknown as XSnapshot
    if (!out.visible_objects) return out
    return { ...out, visible_objects: out.visible_objects.map((o) => ({ ...o, x: o.x })) }
  }
}
