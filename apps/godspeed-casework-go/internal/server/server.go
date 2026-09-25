// Package server is the casework boundary's LIVE HTTP+SSE surface: it serves canonical spec-04
// CognitiveWorldSnapshots built from the governed kernel through the T05 SFWP client (operator
// decision D-1), translates the T01 consequential intents onto governed SFWP verbs, and relays
// kernel event frames as SSE revision events keyed by the kernel's own event cursor.
//
// The wire contract is the T01 ADR (.agents/reports/casework-live-wiring/adr-wire-contract.md):
// /api/world, /api/intents, /api/templates, /api/templates/preflight and /api/events serve the
// internal/contract shapes; refusals are typed outcomes, never transport errors. The server holds
// no state of its own: the world source, intent dispatcher, template source, and relay are
// injected, and there are no globals. The FIXTURE-LABELED surface of the same package lives in
// fixture_server.go behind the casework_fixture build tag (tests and dev/demo only).
//
// Authentication (sessions, cookies, the browser-user to kernel-actor mapping) is T07: until it
// lands, /api/world accepts an explicit ?actor=&role= perspective (verified against the kernel's
// identity.get delegation rules), and intents carry the acting user in their envelope.
package server

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// WorldSource is the live snapshot builder (projection.LiveSource in production, fakes in tests).
type WorldSource interface {
	Snapshot(ctx context.Context, caseID string, actor ports.ActorClaim, cursor string) (contract.CognitiveWorldSnapshot, error)
	NewestCaseID(ctx context.Context) (string, error)
}

// PerspectiveVerifier optionally verifies an explicit ?actor=&role= perspective against the
// kernel's identity rules before serving it.
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
	world   WorldSource
	intents IntentDispatcher
	tpl     TemplateSource
	store   RevisionHistory
	relay   RelayCursors
	opts    Options
}

// New builds the live server. Every dependency is injected.
func New(world WorldSource, dispatcher IntentDispatcher, tpl TemplateSource, store RevisionHistory, relay RelayCursors, opts Options) *Server {
	return &Server{world: world, intents: dispatcher, tpl: tpl, store: store, relay: relay, opts: opts}
}

// Handler returns the routed HTTP handler, wrapped for colocated browser access (the same
// loopback-only CORS posture the boundary has always had).
func (s *Server) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/healthz", s.handleHealthz)
	mux.HandleFunc("GET /api/world", s.handleWorld)
	mux.HandleFunc("GET /api/events", s.handleEvents)
	mux.HandleFunc("POST /api/intents", s.handleIntent)
	mux.HandleFunc("GET /api/templates", s.handleTemplates)
	mux.HandleFunc("POST /api/templates/preflight", s.handlePreflight)
	return withLocalCORS(mux)
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
// refetches the live world either way).
func (s *Server) handleWorld(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	if raw := r.URL.Query().Get("cursor"); raw != "" {
		rev, err := s.store.At(raw)
		if err != nil {
			writeTypedError(w, http.StatusNotFound, "invalid",
				"unknown or evicted kernel cursor "+raw+"; refetch the live world")
			return
		}
		writeJSON(w, http.StatusOK, worldResponse{Snapshot: rev.Snapshot})
		return
	}

	actor, ok := s.perspective(w, r)
	if !ok {
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

// perspective resolves the snapshot's viewpoint. Default: the gateway's own configured
// perspective (also the relay's revision perspective). Explicit ?actor=&role= overrides are
// kernel-verified when the world source supports it, so a snapshot can never render a standing
// the kernel would not grant this connection; both fields must be given together or not at all.
func (s *Server) perspective(w http.ResponseWriter, r *http.Request) (ports.ActorClaim, bool) {
	actor := ports.ActorClaim{ActorID: r.URL.Query().Get("actor"), Role: r.URL.Query().Get("role")}
	if actor.ActorID == "" && actor.Role == "" {
		return s.opts.Perspective, true
	}
	if actor.ActorID == "" || actor.Role == "" {
		writeTypedError(w, http.StatusBadRequest, "invalid", "actor and role must be given together")
		return ports.ActorClaim{}, false
	}
	if v, ok := s.world.(PerspectiveVerifier); ok {
		if err := v.VerifyPerspective(r.Context(), actor); err != nil {
			writeTypedError(w, http.StatusForbidden, "authority_denied",
				"the kernel does not allow this connection to act as "+actor.ActorID+": "+err.Error())
			return ports.ActorClaim{}, false
		}
	}
	return actor, true
}

// handleIntent accepts one consequential intent; refusals are typed outcomes (HTTP 200).
func (s *Server) handleIntent(w http.ResponseWriter, r *http.Request) {
	if !requireJSON(w, r) {
		return
	}
	var in contract.InteractionIntent
	if !decodeStrict(w, r, &in) {
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
			writeSSE(w, rev.Cursor, "snapshot", contract.StreamEvent{
				EventType: "snapshot",
				Cursor:    rev.Cursor,
				Timestamp: rev.At.UTC().Format(time.RFC3339),
				Payload:   rev.Snapshot,
			})
			flusher.Flush()
		}
	}
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
