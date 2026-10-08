// Shared HTTP plumbing for both server surfaces of this package: the live server (server.go,
// production) and the fixture server (fixture_server.go, build tag casework_fixture for tests and
// the dev/demo stack). Helpers here are wire-neutral: strict JSON decoding, typed transport
// errors, trusted-origin CORS, and the SSE framing.
package server

import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
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
	// Ask is the optional governed self-disclosure service used by POST /api/ask.
	Ask ports.AskPort
	// TrustedOrigins is the exact browser Origin allowlist used by CORS and POST Origin checks.
	// Empty preserves the built-in loopback-only development default.
	TrustedOrigins []string
	// Heartbeat is the SSE comment interval. Default 15s.
	Heartbeat time.Duration
	// Perspective is the configured perspective attached to relay revisions. Authenticated reads
	// re-render retained captured facts for the verified session actor; a legacy revision without
	// captured facts is served only when its stored snapshot already matches that actor.
	Perspective ports.ActorClaim
	// Auth wires the session/authentication layer (T07). Zero value fails closed: every
	// protected endpoint 401s.
	Auth AuthOptions
	// StaticRoot serves the built UI from this directory with cache headers, CSP and SPA
	// fallback. Empty disables static serving.
	StaticRoot string
	// Ready is the kernel readiness probe behind /api/readyz (nil -> readyz reports
	// "no probe wired" as 503).
	Ready ReadinessProbe
	// RateLimit bounds POST /api/intents (per session + per IP token buckets). Zero fields take
	// the documented defaults.
	RateLimit RateLimitOptions
	// Logger receives logfmt request lines (nil disables request logging).
	Logger *log.Logger
}

func (o Options) heartbeat() time.Duration {
	if o.Heartbeat > 0 {
		return o.Heartbeat
	}
	return 15 * time.Second
}

var localOriginPattern = regexp.MustCompile(`^http://(127\.0\.0\.1|localhost|\[::1\])(:\d+)?$`)

// withLocalCORS preserves the built-in loopback-only browser origin policy used by the fixture
// surface. Live servers may supply an explicit trusted-origin allowlist through withOriginCORS.
func withLocalCORS(next http.Handler) http.Handler {
	return withOriginCORS(next, nil)
}

// originAllowed matches an exact configured Origin. An empty configured list keeps the
// historical loopback-only development default.
func originAllowed(origin string, trusted []string) bool {
	if origin == "" {
		return false
	}
	if len(trusted) == 0 {
		return localOriginPattern.MatchString(origin)
	}
	for _, allowed := range trusted {
		if origin == allowed {
			return true
		}
	}
	return false
}

// withOriginCORS configures exact credentialed browser origins. It also checks Origin on every
// POST before the route handler, so disallowed cross-origin state changes fail before session,
// CSRF, or kernel work. Allowed origins still require the route's existing session and CSRF
// guards. Preflight remains an empty response and never reaches a projection handler.
func withOriginCORS(next http.Handler, trusted []string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		origin := r.Header.Get("Origin")
		allowed := len(r.Header.Values("Origin")) == 1 && originAllowed(origin, trusted)
		if origin != "" && allowed {
			h := w.Header()
			h.Set("Access-Control-Allow-Origin", origin)
			h.Set("Vary", "Origin")
			if len(trusted) > 0 {
				h.Set("Access-Control-Allow-Credentials", "true")
			}
			h.Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
			h.Set("Access-Control-Allow-Headers", "Content-Type, X-CSRF-Token")
			h.Set("Access-Control-Max-Age", "600")
		}
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		if r.Method == http.MethodPost && (len(r.Header.Values("Origin")) > 1 || origin != "" && !allowed) {
			writeTypedError(w, http.StatusForbidden, "csrf_refused", "request Origin is not in the trusted origin allowlist")
			return
		}
		next.ServeHTTP(w, r)
	})
}

// withSecurityHeaders applies the API surface's minimal hardening (the CSP for documents lives
// with the static handler).
func withSecurityHeaders(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("X-Content-Type-Options", "nosniff")
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
