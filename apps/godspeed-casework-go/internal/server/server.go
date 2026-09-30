// Package server is the casework boundary's LIVE HTTP+SSE surface: it serves canonical spec-04
// CognitiveWorldSnapshots built from the governed kernel through the T05 SFWP client (operator
// decision D-1), translates the T01 consequential intents onto governed SFWP verbs, and relays
// kernel event frames as SSE revision events keyed by the kernel's own event cursor.
//
// The wire contract is the T01 ADR (.agents/reports/casework-live-wiring/adr-wire-contract.md):
// /api/world, /api/intents, /api/templates, /api/templates/preflight and /api/events serve the
// internal/contract shapes; refusals are typed outcomes, never transport errors. The server holds
// no case state of its own: the world source, intent dispatcher, template source, and relay are
// injected, and there are no globals. The FIXTURE-LABELED surface of the same package lives in
// fixture_server.go behind the casework_fixture build tag (tests and dev/demo only).
//
// Authentication (T07): every route except /api/healthz, /api/readyz and the auth entry points
// requires a session. Identity is session-bound and maps to the kernel actor through the T02
// delegation allowlist; the pre-auth ?actor=&role= dev surface is refused. Sessions, CSRF and
// the dev static-token mode live in session.go; static UI serving in static.go; the intent rate
// limit in ratelimit.go; correlation logging in logging.go.
package server

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// WorldSource is the live snapshot builder (projection.LiveSource in production, fakes in tests).
type WorldSource interface {
	Snapshot(ctx context.Context, caseID string, actor ports.ActorClaim, cursor string) (contract.CognitiveWorldSnapshot, error)
	NewestCaseID(ctx context.Context) (string, error)
}

// PerspectiveVerifier verifies a session actor against the kernel's identity and delegation rules
// before serving session-bound world, stream, trajectory, or artifact data. Actor and role query
// overrides are rejected rather than treated as identities.
type PerspectiveVerifier interface {
	VerifyPerspective(ctx context.Context, actor ports.ActorClaim) error
}

// TemplateSource serves the template discovery/preflight endpoints.
type TemplateSource interface {
	EntryOptions(ctx context.Context) ([]contract.TemplateEntryOption, error)
	Preflight(ctx context.Context, templateRef string, params map[string]any) (contract.TemplatePreflightResult, error)
}

// IntentDispatcher translates one intent (internal/intents.Handler in production).
type IntentDispatcher interface {
	Handle(ctx context.Context, in contract.InteractionIntent) contract.IntentResponse
}

// RevisionHistory is the relay's served history (projection.Store).
type RevisionHistory interface {
	At(cursor string) (projection.Revision, error)
	Trajectory(caseID string) []projection.Revision
	Oldest() string
	Head() string
	Live() (projection.Revision, bool)
	Subscribe(after string) (<-chan projection.Revision, func())
}

// RelayCursors is the per-case kernel-cursor view (Relay; fakes in tests).
type RelayCursors interface {
	CursorForCase(caseID string) (string, bool)
	Head() string
}

// Server wires the injected live stack into the HTTP surface.
type Server struct {
	world          WorldSource
	intents        IntentDispatcher
	tpl            TemplateSource
	artifacts      ArtifactGetter
	store          RevisionHistory
	relay          RelayCursors
	opts           Options
	intentsLimiter *RateLimiter
}

// New builds the live server. Every dependency is injected.
func New(world WorldSource, dispatcher IntentDispatcher, tpl TemplateSource, store RevisionHistory, relay RelayCursors, opts Options) *Server {
	return NewWithArtifacts(world, dispatcher, tpl, store, relay, nil, opts)
}

// NewWithArtifacts builds the live server with its governed artifact reader.
func NewWithArtifacts(world WorldSource, dispatcher IntentDispatcher, tpl TemplateSource, store RevisionHistory, relay RelayCursors, artifacts ArtifactGetter, opts Options) *Server {
	return &Server{
		world:          world,
		intents:        dispatcher,
		tpl:            tpl,
		artifacts:      artifacts,
		store:          store,
		relay:          relay,
		opts:           opts,
		intentsLimiter: NewRateLimiter(opts.RateLimit.PerMinute, opts.RateLimit.Burst, 0, nil),
	}
}

// Handler returns the routed HTTP surface. The posture (T07):
//
//	public:      GET /api/healthz, GET /api/readyz
//	auth entry:  GET|POST /api/auth/login, GET /api/auth/callback, POST /api/auth/logout,
//	             GET /api/session (always answers; unauthenticated = authenticated:false)
//	protected:   GET /api/world, /api/events, /api/templates  -> session required (401 otherwise)
//	             GET /api/artifacts/{digest} -> session and kernel delegation verification
//	             POST /api/intents, /api/templates/preflight -> session + CSRF (401/403 otherwise)
//	             POST /api/intents is additionally rate limited per session and per IP (429)
//	static:      everything else serves the configured UI root with cache headers + CSP, or a
//	             typed 404 when no static root is configured
//
// Identity comes from the session cookie (or the dev-only static bearer token); the pre-T07
// ?actor=&role= override is refused for authenticated requests and every intent's actor is
// overwritten with the session's kernel standing. The chain is:
//
//	correlation logging -> security headers -> loopback CORS -> mux
func (s *Server) Handler() http.Handler {
	mux := http.NewServeMux()
	// Public surface.
	mux.HandleFunc("GET /api/healthz", s.handleHealthz)
	mux.HandleFunc("GET /api/readyz", s.handleReadyz)
	// Auth entry points (the guards are inside the handlers: login/callback are intentionally
	// pre-session, logout re-checks session + CSRF).
	mux.HandleFunc("GET /api/auth/login", s.handleLoginStart)
	mux.HandleFunc("POST /api/auth/login", s.handleLogin)
	mux.HandleFunc("GET /api/auth/callback", s.handleLoginCallback)
	mux.HandleFunc("POST /api/auth/logout", s.requireSession(s.requireCSRF(s.handleLogout)))
	mux.HandleFunc("GET /api/session", s.withIdentity(s.handleSession))
	// Protected reads: session required.
	mux.HandleFunc("GET /api/world", s.requireSession(s.handleWorld))
	mux.HandleFunc("GET /api/events", s.requireSession(s.handleEvents))
	mux.HandleFunc("GET /api/templates", s.requireSession(s.handleTemplates))
	mux.HandleFunc("GET /api/artifacts/{digest}", s.requireSession(s.handleArtifactGet))
	mux.HandleFunc("GET /api/trajectory", s.requireSession(s.handleTrajectory))
	// State-changing POSTs: session + CSRF; intents are rate limited (before the handler, so a
	// refused request never reaches the dispatcher).
	mux.HandleFunc("POST /api/intents", s.requireSession(s.requireCSRF(s.rateLimitIntent(s.handleIntent))))
	mux.HandleFunc("POST /api/templates/preflight", s.requireSession(s.requireCSRF(s.handlePreflight)))
	// Unknown /api paths are typed 404s, never the SPA fallback. (Method-scoped so the GET
	// catch-all below does not conflict; a POST to an unknown /api path is the mux's 405.)
	mux.HandleFunc("GET /api/", func(w http.ResponseWriter, r *http.Request) {
		writeTypedError(w, http.StatusNotFound, "invalid", "no such API route")
	})
	// Static UI (or a plain 404 when no root is configured).
	if s.opts.StaticRoot != "" {
		mux.Handle("GET /", s.staticHandler())
	}

	var h http.Handler = http.Handler(mux)
	h = withOriginCORS(h, s.opts.TrustedOrigins)
	h = withSecurityHeaders(h)
	h = s.withCorrelation(h)
	return h
}

func (s *Server) handleHealthz(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, healthResponse{
		Status:       "ok",
		Provenance:   projection.ProvenanceLabelLive,
		KernelCursor: s.relay.Head(),
	})
}

// handleWorld serves the canonical snapshot: live-built by default, or the stored revision at
// ?cursor= (true kernel history; 404 for a cursor that is evicted or never existed - the client
// refetches the live world either way). The perspective is the SESSION's kernel actor (T07):
// identity is session-bound, so the pre-auth ?actor=&role= override is refused - a client that
// asks for someone else's standing gets a typed 400, never a snapshot.
func (s *Server) handleWorld(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	if hasIdentityOverride(r) {
		writeTypedError(w, http.StatusBadRequest, "invalid",
			"identity is session-bound: the ?actor=&role= override was removed in T07; log in as the user you need (the kernel verifies the delegation)")
		return
	}
	actor := sessionIdentityOf(r).Claim()
	if !s.verifySessionPerspective(w, r, actor) {
		return
	}
	if raw := r.URL.Query().Get("cursor"); raw != "" {
		rev, err := s.store.At(raw)
		if err != nil {
			writeTypedError(w, http.StatusNotFound, "invalid",
				"unknown or evicted kernel cursor "+raw+"; refetch the live world")
			return
		}
		if requestedCases, supplied := r.URL.Query()["case_id"]; supplied {
			if len(requestedCases) != 1 || requestedCases[0] != rev.CaseID {
				writeTypedError(w, http.StatusBadRequest, "invalid", "the requested cursor belongs to a different case")
				return
			}
		}
		snap, ok := renderRevision(rev, actor)
		if !ok {
			writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "retained authority facts are unavailable for this session perspective")
			return
		}
		writeJSON(w, http.StatusOK, worldResponse{Snapshot: snap})
		return
	}
	caseID := r.URL.Query().Get("case_id")
	if caseID == "" {
		newest, err := s.world.NewestCaseID(ctx)
		if err != nil {
			writeTypedError(w, http.StatusBadGateway, "unavailable", "the kernel view is unreachable: "+err.Error())
			return
		}
		caseID = newest
	}
	if caseID == "" {
		writeJSON(w, http.StatusOK, worldResponse{Snapshot: projection.EmptyWorld(actor, s.relay.Head(), time.Now())})
		return
	}
	// The snapshot exists at the case's newest observed kernel cursor: the same address space the
	// intents' staleness guard compares against.
	cursor, _ := s.relay.CursorForCase(caseID)
	snap, err := s.world.Snapshot(ctx, caseID, actor, cursor)
	if err != nil {
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the kernel view could not be built: "+err.Error())
		return
	}
	writeJSON(w, http.StatusOK, worldResponse{Snapshot: snap})
}

// handleIntent accepts one consequential intent; dispatcher refusals are typed outcomes (HTTP
// 200), while request and identity-verification failures retain their typed HTTP statuses.
// Middleware has already enforced session + CSRF + rate limit before this runs. The intent's
// client-asserted actor is OVERWRITTEN with the session's kernel standing: identity is never
// trusted from the client (T07). The current session perspective is verified against the kernel
// before the dispatcher can read its idempotency cache or perform an action; the kernel still
// authorizes the action itself. The correlation id becomes the intent id (the SFWP request_id the
// kernel correlates on), joining the gateway's request log to the kernel's durable request record.
func (s *Server) handleIntent(w http.ResponseWriter, r *http.Request) {
	if !requireJSON(w, r) {
		return
	}
	var in contract.InteractionIntent
	if !decodeStrict(w, r, &in) {
		return
	}
	in.Actor = intentActorFromSession(r)
	if rw, ok := w.(correlationSetter); ok && in.IntentID != "" {
		rw.setCorrelation(in.IntentID)
	}
	if !s.verifySessionPerspective(w, r, sessionIdentityOf(r).Claim()) {
		return
	}
	writeJSON(w, http.StatusOK, s.intents.Handle(r.Context(), in))
}

func (s *Server) handleTemplates(w http.ResponseWriter, r *http.Request) {
	opts, err := s.tpl.EntryOptions(r.Context())
	if err != nil {
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the kernel template view could not be read: "+err.Error())
		return
	}
	writeJSON(w, http.StatusOK, templatesResponse{Templates: opts})
}

func (s *Server) handlePreflight(w http.ResponseWriter, r *http.Request) {
	if !requireJSON(w, r) {
		return
	}
	var body preflightRequest
	if !decodeStrict(w, r, &body) {
		return
	}
	if body.TemplateRef == "" {
		writeTypedError(w, http.StatusBadRequest, "invalid", "template_ref is required")
		return
	}
	result, err := s.tpl.Preflight(r.Context(), body.TemplateRef, body.Params)
	if err != nil {
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the kernel preflight could not be run: "+err.Error())
		return
	}
	writeJSON(w, http.StatusOK, result)
}

// handleEvents streams the SSE feed: hello, resync_required when the requested position predates
// the retained history, the replay of stored revisions newer than Last-Event-ID / ?last=, then
// live revisions as the relay records them - each with id=kernel cursor. There are no
// command-level frames on the kernel bus (decision-log D-3-followups), so execution progress
// reaches clients through the snapshot revisions themselves.
func (s *Server) handleEvents(w http.ResponseWriter, r *http.Request) {
	if hasIdentityOverride(r) {
		writeTypedError(w, http.StatusBadRequest, "invalid", "identity is session-bound: actor and role query overrides are not accepted")
		return
	}
	actor := sessionIdentityOf(r).Claim()
	if !s.verifySessionPerspective(w, r, actor) {
		return
	}
	flusher, ok := w.(http.Flusher)
	if !ok {
		writeTypedError(w, http.StatusInternalServerError, "internal", "streaming is not supported by this connection")
		return
	}

	last := r.URL.Query().Get("last")
	if last == "" {
		// The EventSource resume contract: the browser sends the last event's id back on
		// reconnect. ?last= is honoured equally (fetch-based streams and tests).
		last = r.Header.Get("Last-Event-ID")
	}

	// The subscription is registered before the hello is written, so anything the relay records
	// while the hello is in flight is queued rather than missed.
	revCh, cancel := s.store.Subscribe(last)
	defer cancel()

	w.Header().Set("Content-Type", "text/event-stream; charset=utf-8")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("X-Accel-Buffering", "no")
	w.WriteHeader(http.StatusOK)
	writeSSE(w, "", "hello", helloBody{Provenance: projection.ProvenanceLabelLive, KernelCursor: s.relay.Head()})
	flusher.Flush()

	if last != "" {
		if oldest := s.store.Oldest(); oldest != "" && last < oldest {
			// The requested position predates the retained history: a gap-free replay is not
			// possible, and pretending otherwise would silently skip kernel truth.
			writeSSE(w, oldest, "resync_required", contract.StreamEvent{
				EventType: "resync_required",
				Cursor:    oldest,
				Timestamp: time.Now().UTC().Format(time.RFC3339),
				Payload: contract.ResyncRequiredPayload{
					RequestedCursor:       last,
					OldestAvailableCursor: oldest,
					Reason:                "retention window evicted the requested revision",
				},
			})
			flusher.Flush()
		}
	}

	heartbeat := time.NewTicker(s.opts.heartbeat())
	defer heartbeat.Stop()

	for {
		select {
		case <-r.Context().Done():
			return
		case <-heartbeat.C:
			fmt.Fprint(w, ": heartbeat\n\n")
			flusher.Flush()
		case rev, ok := <-revCh:
			if !ok {
				// Evicted for falling behind: the client reconnects with Last-Event-ID and replays.
				return
			}
			snap, renderable := renderRevision(rev, actor)
			if !renderable {
				// A legacy source without captured facts can only stream the exact perspective it
				// originally rendered. Ending the stream is fail-closed and forces an authenticated
				// current read instead of leaking another actor's offers.
				return
			}
			writeSSE(w, rev.Cursor, "snapshot", contract.StreamEvent{
				EventType: "snapshot",
				Cursor:    rev.Cursor,
				Timestamp: rev.At.UTC().Format(time.RFC3339),
				Payload:   snap,
			})
			flusher.Flush()
		}
	}
}

func hasIdentityOverride(r *http.Request) bool {
	query := r.URL.Query()
	_, actor := query["actor"]
	_, role := query["role"]
	return actor || role
}

func (s *Server) verifySessionPerspective(w http.ResponseWriter, r *http.Request, actor ports.ActorClaim) bool {
	verifier, ok := s.world.(PerspectiveVerifier)
	if !ok {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "kernel identity verification is not configured")
		return false
	}
	if err := verifier.VerifyPerspective(r.Context(), actor); err != nil {
		if apperr.KindOf(err) == apperr.KindAuthorityDenied {
			writeTypedError(w, http.StatusForbidden, "authority_denied", "the kernel refused this session perspective")
		} else {
			writeTypedError(w, http.StatusBadGateway, "unavailable", "the kernel could not verify the session perspective")
		}
		return false
	}
	return true
}

func renderRevision(rev projection.Revision, actor ports.ActorClaim) (contract.CognitiveWorldSnapshot, bool) {
	if rev.Facts != nil {
		facts := projection.CloneFacts(rev.Facts)
		facts.Actor = actor
		return projection.Build(*facts), true
	}
	if rev.Snapshot.Perspective.ActorID != actor.ActorID || rev.Snapshot.Perspective.Role != actor.Role {
		return contract.CognitiveWorldSnapshot{}, false
	}
	return rev.Snapshot, true
}

// wire response shapes

type healthResponse struct {
	Status       string `json:"status"`
	Provenance   string `json:"provenance"`
	KernelCursor string `json:"kernel_cursor"`
}

type worldResponse struct {
	Snapshot contract.CognitiveWorldSnapshot `json:"snapshot"`
}

type templatesResponse struct {
	Templates []contract.TemplateEntryOption `json:"templates"`
}

type preflightRequest struct {
	TemplateRef string         `json:"template_ref"`
	Params      map[string]any `json:"params,omitempty"`
}
