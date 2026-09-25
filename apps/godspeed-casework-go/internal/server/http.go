// Shared HTTP plumbing for both server surfaces of this package: the live server (server.go,
// production) and the fixture server (fixture_server.go, build tag casework_fixture for tests and
// the dev/demo stack). Helpers here are wire-neutral: strict JSON decoding, typed transport
// errors, local-only CORS, and the SSE framing.
package server

import (
	"encoding/json"
	"errors"
	"fmt"
	"mime"
	"net/http"
	"regexp"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// maxBodyBytes bounds request bodies on the POST endpoints.
const maxBodyBytes = 1 << 20 // 1 MiB

// Options tunes server behaviour; the zero value is the production default.
type Options struct {
	// Heartbeat is the SSE comment interval. Default 15s.
	Heartbeat time.Duration
	// Perspective is the default viewpoint for /api/world snapshots built without an explicit
	// ?actor=&role= override (the configured serve perspective; T07 sessions replace this).
	Perspective ports.ActorClaim
}

func (o Options) heartbeat() time.Duration {
	if o.Heartbeat > 0 {
		return o.Heartbeat
	}
	return 15 * time.Second
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

type wireError struct {
	Kind string `json:"kind"`
	Note string `json:"note"`
}

type errorBody struct {
	Error wireError `json:"error"`
}

// helloBody opens an SSE stream with the serving surface's provenance and the newest cursor.
type helloBody struct {
	Provenance   string `json:"provenance"`
	KernelCursor string `json:"kernel_cursor"`
}

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
