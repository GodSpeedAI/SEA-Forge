// Package server is the casework boundary's HTTP+SSE surface (FIXTURE-LABELED): it serves the
// cognitive projections built from the Northstar fixture and accepts interaction intents.
//
// The wire contract is apps/godspeed-cognitive-ui/src/adapters/go/WIRE.md; the shapes here must
// not drift from it. The server holds no state of its own: the projection, coordinator, and
// artifact stores are injected, and there are no globals.
package server

import (
	"encoding/json"
	"errors"
	"fmt"
	"mime"
	"net/http"
	"regexp"
	"strconv"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/artifactstore"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/coordinator"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// maxBodyBytes bounds request bodies on the POST endpoints.
const maxBodyBytes = 1 << 20 // 1 MiB

// Options tunes server behaviour; the zero value is the production default.
type Options struct {
	// Heartbeat is the SSE comment interval. Default 15s.
	Heartbeat time.Duration
}

func (o Options) heartbeat() time.Duration {
	if o.Heartbeat > 0 {
		return o.Heartbeat
	}
	return 15 * time.Second
}

// Server wires the injected stores into the HTTP surface.
type Server struct {
	proj  *projection.Store
	coord *coordinator.Coordinator
	arts  *artifactstore.Store
	opts  Options
}

// New builds the server. Every dependency is injected; the server owns none of the state itself.
func New(proj *projection.Store, coord *coordinator.Coordinator, arts *artifactstore.Store, opts Options) *Server {
	return &Server{proj: proj, coord: coord, arts: arts, opts: opts}
}

// Handler returns the routed HTTP handler, wrapped for colocated browser access: the React host
// runs on a different loopback port, so same-machine origins (127.0.0.1 / localhost, any port)
// are allowed cross-origin callers. Nothing else: the server binds loopback by default and this
// never widens to non-local Origins.
func (s *Server) Handler() http.Handler {
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

var localOriginPattern = regexp.MustCompile(`^http://(127\.0\.0\.1|localhost|\[::1\])(:\d+)?$`)

func withLocalCORS(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		origin := r.Header.Get("Origin")
		if origin != "" && localOriginPattern.MatchString(origin) {
			h := w.Header()
			h.Set("Access-Control-Allow-Origin", origin)
			h.Set("Vary", "Origin")
			h.Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
			h.Set("Access-Control-Allow-Headers", "Content-Type")
			h.Set("Access-Control-Max-Age", "600")
		}
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func (s *Server) handleHealthz(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, healthBody{
		Status:     "ok",
		Provenance: projection.ProvenanceLabel,
		LiveCursor: s.proj.LiveCursor(),
	})
}

func (s *Server) handleWorld(w http.ResponseWriter, r *http.Request) {
	raw := r.URL.Query().Get("cursor")
	if raw == "" {
		writeJSON(w, http.StatusOK, worldBody{Snapshot: s.proj.Live()})
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
	writeJSON(w, http.StatusOK, worldBody{Snapshot: snap})
}

func (s *Server) handleTime(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, timeBody{Positions: s.proj.Window(), Truncated: false})
}

func (s *Server) handleIntent(w http.ResponseWriter, r *http.Request) {
	if !requireJSON(w, r) {
		return
	}
	var in coordinator.Intent
	if !decodeStrict(w, r, &in) {
		return
	}
	writeJSON(w, http.StatusOK, s.coord.Submit(in))
}

func (s *Server) handleArtifactPersist(w http.ResponseWriter, r *http.Request) {
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

func (s *Server) handleArtifactGet(w http.ResponseWriter, r *http.Request) {
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

func (s *Server) handleArtifactList(w http.ResponseWriter, r *http.Request) {
	all := s.arts.List()
	out := make([]artifactDescriptor, 0, len(all))
	for _, a := range all {
		out = append(out, artifactDescriptor{Ref: a.Ref, Kind: a.Kind, Title: a.Title, BoundObject: a.BoundObject})
	}
	writeJSON(w, http.StatusOK, artifactListBody{Descriptors: out})
}

// handleEvents streams the SSE feed: hello, the replay of revisions newer than ?last=, then live
// revisions and lease transitions, with a heartbeat comment and a flush per event. When the client
// disconnects, every subscription is cancelled so no goroutine or channel leaks.
func (s *Server) handleEvents(w http.ResponseWriter, r *http.Request) {
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
	revCh, cancelRevisions := s.proj.Subscribe(last)
	leaseCh, cancelLeases := s.coord.SubscribeLeases()
	defer cancelRevisions()
	defer cancelLeases()

	w.Header().Set("Content-Type", "text/event-stream; charset=utf-8")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("X-Accel-Buffering", "no")
	w.WriteHeader(http.StatusOK)
	writeSSE(w, "", "hello", helloBody{Provenance: projection.ProvenanceLabel, LiveCursor: s.proj.LiveCursor()})
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
			writeSSE(w, strconv.FormatInt(snap.Cursor, 10), "revision", worldBody{Snapshot: snap})
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

// wire response shapes

type healthBody struct {
	Status     string `json:"status"`
	Provenance string `json:"provenance"`
	LiveCursor int64  `json:"liveCursor"`
}

type worldBody struct {
	Snapshot projection.Snapshot `json:"snapshot"`
}

type timeBody struct {
	Positions []projection.WindowEntry `json:"positions"`
	Truncated bool                     `json:"truncated"`
}

type helloBody struct {
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

type wireError struct {
	Kind string `json:"kind"`
	Note string `json:"note"`
}

type errorBody struct {
	Error wireError `json:"error"`
}

// helpers

func writeJSON(w http.ResponseWriter, status int, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	enc := json.NewEncoder(w)
	enc.SetEscapeHTML(false)
	// An encode failure after the status is sent cannot be reported; the short write surfaces to
	// the client as a broken body, which is the honest outcome.
	_ = enc.Encode(v)
}

func writeTypedError(w http.ResponseWriter, status int, kind, note string) {
	writeJSON(w, status, errorBody{Error: wireError{Kind: kind, Note: note}})
}

// requireJSON enforces the strict POST content type.
func requireJSON(w http.ResponseWriter, r *http.Request) bool {
	mt, _, err := mime.ParseMediaType(r.Header.Get("Content-Type"))
	if err != nil || mt != "application/json" {
		writeTypedError(w, http.StatusUnsupportedMediaType, "invalid", "content-type must be application/json")
		return false
	}
	return true
}

// decodeStrict reads one JSON value with unknown fields rejected and the body size bounded.
func decodeStrict(w http.ResponseWriter, r *http.Request, v any) bool {
	dec := json.NewDecoder(http.MaxBytesReader(w, r.Body, maxBodyBytes))
	dec.DisallowUnknownFields()
	if err := dec.Decode(v); err != nil {
		var tooBig *http.MaxBytesError
		if errors.As(err, &tooBig) {
			writeTypedError(w, http.StatusRequestEntityTooLarge, "invalid", "request body exceeds 1 MiB")
			return false
		}
		writeTypedError(w, http.StatusBadRequest, "invalid", "request body is not valid JSON for this endpoint: "+err.Error())
		return false
	}
	var extra json.RawMessage
	if err := dec.Decode(&extra); err == nil {
		writeTypedError(w, http.StatusBadRequest, "invalid", "request body must contain a single JSON object")
		return false
	}
	return true
}

// writeSSE writes one event. Data is marshalled onto a single line (JSON string escaping never
// emits a raw newline), matching the text/event-stream framing.
func writeSSE(w interface{ Write([]byte) (int, error) }, id, event string, data any) {
	if id != "" {
		fmt.Fprintf(w, "id: %s\n", id)
	}
	if event != "" {
		fmt.Fprintf(w, "event: %s\n", event)
	}
	payload, err := json.Marshal(data)
	if err != nil {
		payload = []byte(`{}`)
	}
	fmt.Fprintf(w, "data: %s\n\n", payload)
}
