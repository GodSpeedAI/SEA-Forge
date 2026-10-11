//go:build casework_fixture

// The FIXTURE-LABELED server surface (build tag casework_fixture): it serves the cognitive
// projections built from the Northstar fixture and accepts the fixture-stage interaction
// intents. This file exists for tests and the dev/demo fixture stack (just casework-go-up);
// the production build (no tags) compiles only the live server in server.go. Superseded as the
// wire contract by the live server per the T01 ADR.
//
// The wire contract is apps/godspeed-cognitive-ui/src/adapters/go/WIRE.md; the shapes here must
// not drift from it. The server holds no state of its own: the projection, coordinator, and
// artifact stores are injected, and there are no globals. Shared HTTP plumbing lives in http.go.
package server

import (
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/artifactstore"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/coordinator"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// FixtureServer wires the injected fixture stores into the fixture HTTP surface.
type FixtureServer struct {
	proj  *projection.FixtureStore
	coord *coordinator.Coordinator
	arts  *artifactstore.Store
	opts  Options
}

// NewFixtureServer builds the FIXTURE-LABELED server. Every dependency is injected; the server
// owns none of the state itself.
func NewFixtureServer(proj *projection.FixtureStore, coord *coordinator.Coordinator, arts *artifactstore.Store, opts Options) *FixtureServer {
	return &FixtureServer{proj: proj, coord: coord, arts: arts, opts: opts}
}

// Handler returns the routed HTTP handler, wrapped for colocated browser access by withLocalCORS.
func (s *FixtureServer) Handler() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /api/healthz", s.handleHealthz)
	mux.HandleFunc("GET /api/world", s.handleWorld)
	mux.HandleFunc("GET /api/time", s.handleTime)
	mux.HandleFunc("GET /api/events", s.handleEvents)
	mux.HandleFunc("POST /api/intents", s.handleIntent)
	mux.HandleFunc("GET /api/artifacts", s.handleArtifactList)
	mux.HandleFunc("POST /api/artifacts", s.handleArtifactPersist)
	mux.HandleFunc("GET /api/artifacts/{ref}", s.handleArtifactGet)
	return withLocalCORS(mux)
}

func (s *FixtureServer) handleHealthz(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, fixtureHealthBody{
		Status:     "ok",
		Provenance: projection.ProvenanceLabel,
		LiveCursor: s.proj.LiveCursor(),
	})
}

func (s *FixtureServer) handleWorld(w http.ResponseWriter, r *http.Request) {
	raw := r.URL.Query().Get("cursor")
	if raw == "" {
		writeJSON(w, http.StatusOK, fixtureWorldBody{Snapshot: s.proj.Live()})
		return
	}
	cursor, err := strconv.ParseInt(raw, 10, 64)
	if err != nil {
		writeTypedError(w, http.StatusNotFound, "invalid", "cursor must be an integer")
		return
	}
	snap, err := s.proj.At(cursor)
	if err != nil {
		writeTypedError(w, http.StatusNotFound, "invalid", "unknown cursor")
		return
	}
	writeJSON(w, http.StatusOK, fixtureWorldBody{Snapshot: snap})
}

func (s *FixtureServer) handleTime(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, timeBody{Positions: s.proj.Window(), Truncated: false})
}

func (s *FixtureServer) handleIntent(w http.ResponseWriter, r *http.Request) {
	if !requireJSON(w, r) {
		return
	}
	var in coordinator.Intent
	if !decodeStrict(w, r, &in) {
		return
	}
	writeJSON(w, http.StatusOK, s.coord.Submit(in))
}

func (s *FixtureServer) handleArtifactPersist(w http.ResponseWriter, r *http.Request) {
	if !requireJSON(w, r) {
		return
	}
	var body artifactPersistBody
	if !decodeStrict(w, r, &body) {
		return
	}
	if body.Ref == "" {
		writeJSON(w, http.StatusOK, coordinator.Outcome{
			Status: "refused", Reason: "invalid", Note: "ref is required",
		})
		return
	}
	s.arts.Persist(body.Ref, body.BoundObject, body.Title)
	writeJSON(w, http.StatusOK, artifactPersistResponse{Status: "accepted"})
}

func (s *FixtureServer) handleArtifactGet(w http.ResponseWriter, r *http.Request) {
	ref := r.PathValue("ref")
	art, ok := s.arts.Get(ref)
	if !ok {
		writeTypedError(w, http.StatusNotFound, "invalid", "unknown artifact ref")
		return
	}
	level := r.URL.Query().Get("level")
	if level == "" {
		level = "minimal"
	}
	switch level {
	case "minimal", "summary", "source":
	default:
		writeTypedError(w, http.StatusBadRequest, "invalid", "level must be minimal, summary, or source")
		return
	}
	content, mediaType, ok := art.Content(level)
	if !ok {
		writeTypedError(w, http.StatusNotFound, "invalid", "artifact has no "+level+" level")
		return
	}
	w.Header().Set("Content-Type", mediaType)
	w.Header().Set("X-Content-Type-Options", "nosniff")
	w.Header().Set("Content-Length", strconv.Itoa(len(content)))
	w.WriteHeader(http.StatusOK)
	if r.Method == http.MethodGet {
		w.Write(content)
	}
}

// artifactDescriptors is the catalog join the UI performs by boundObject.
type artifactDescriptor struct {
	Ref         string `json:"ref"`
	Kind        string `json:"kind"`
	Title       string `json:"title"`
	BoundObject string `json:"boundObject"`
}

func (s *FixtureServer) handleArtifactList(w http.ResponseWriter, r *http.Request) {
	all := s.arts.List()
	out := make([]artifactDescriptor, 0, len(all))
	for _, a := range all {
		out = append(out, artifactDescriptor{Ref: a.Ref, Kind: a.Kind, Title: a.Title, BoundObject: a.BoundObject})
	}
	writeJSON(w, http.StatusOK, artifactListBody{Descriptors: out})
}

// handleEvents streams the fixture SSE feed: hello, the replay of revisions newer than ?last=,
// then live revisions and lease transitions, with a heartbeat comment and a flush per event. When
// the client disconnects, every subscription is cancelled so no goroutine or channel leaks.
func (s *FixtureServer) handleEvents(w http.ResponseWriter, r *http.Request) {
	flusher, ok := w.(http.Flusher)
	if !ok {
		writeTypedError(w, http.StatusInternalServerError, "internal", "streaming is not supported by this connection")
		return
	}

	last := int64(0)
	if raw := r.URL.Query().Get("last"); raw != "" {
		if parsed, err := strconv.ParseInt(raw, 10, 64); err == nil {
			last = parsed
		}
		// An unparsable last is treated as "from the beginning": the client asked for a position
		// that does not exist, and a full replay is the recoverable answer.
	}

	// Subscriptions are registered before the hello is written, so anything the world does while
	// the hello is in flight is queued rather than missed.
	revCh, cancelFixtureRevisions := s.proj.Subscribe(last)
	leaseCh, cancelLeases := s.coord.SubscribeLeases()
	defer cancelFixtureRevisions()
	defer cancelLeases()

	w.Header().Set("Content-Type", "text/event-stream; charset=utf-8")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("X-Accel-Buffering", "no")
	w.WriteHeader(http.StatusOK)
	writeSSE(w, "", "hello", fixtureHelloBody{Provenance: projection.ProvenanceLabel, LiveCursor: s.proj.LiveCursor()})
	flusher.Flush()

	heartbeat := time.NewTicker(s.opts.heartbeat())
	defer heartbeat.Stop()

	for {
		select {
		case <-r.Context().Done():
			return
		case <-heartbeat.C:
			fmt.Fprint(w, ": heartbeat\n\n")
			flusher.Flush()
		case snap, ok := <-revCh:
			if !ok {
				return // evicted: the client replays from ?last= on reconnect
			}
			writeSSE(w, strconv.FormatInt(snap.Cursor, 10), "revision", fixtureWorldBody{Snapshot: snap})
			flusher.Flush()
		case lease, ok := <-leaseCh:
			if !ok {
				return
			}
			writeSSE(w, "", "lease", lease)
			flusher.Flush()
		}
	}
}

// fixture wire shapes (this surface only; the live server's shapes live in server.go)

type fixtureHealthBody struct {
	Status     string `json:"status"`
	Provenance string `json:"provenance"`
	LiveCursor int64  `json:"liveCursor"`
}

type fixtureWorldBody struct {
	Snapshot projection.Snapshot `json:"snapshot"`
}

type timeBody struct {
	Positions []projection.WindowEntry `json:"positions"`
	Truncated bool                     `json:"truncated"`
}

type fixtureHelloBody struct {
	Provenance string `json:"provenance"`
	LiveCursor int64  `json:"liveCursor"`
}

type artifactPersistBody struct {
	Ref         string `json:"ref"`
	BoundObject string `json:"boundObject"`
	Title       string `json:"title"`
}

type artifactPersistResponse struct {
	Status string `json:"status"`
}

type artifactListBody struct {
	Descriptors []artifactDescriptor `json:"descriptors"`
}
