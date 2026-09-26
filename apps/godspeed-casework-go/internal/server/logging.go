// Structured request logging with a correlation id (plan T07). The middleware mints a
// correlation id for every request; for POST /api/intents the intent handler REPLACES it with
// the intent's id - which is exactly the request_id the gateway sends to the kernel - so a
// gateway log line joins to the kernel's durable request record on the same identifier. All
// other requests keep their minted id (the kernel never sees it).
//
// The log format is logfmt (key=value pairs, one line per request): structured enough to grep
// and ship, without adding a logging dependency.
package server

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"log"
	"net/http"
	"time"
)

type correlationKey struct{}

// correlationSetter is implemented by the recording response writer so a handler can rename the
// request's correlation id (the intent handler sets it to the intent id / SFWP request_id).
type correlationSetter interface {
	setCorrelation(id string)
}

// correlationID returns the request's correlation id ("" before the middleware ran).
func correlationID(r *http.Request) string {
	if id, ok := r.Context().Value(correlationKey{}).(string); ok {
		return id
	}
	return ""
}

// mintCorrelationID returns 128 bits of CSPRNG output as hex; a broken CSPRNG falls back to a
// nanosecond timestamp (still unique enough for a log line; never security-relevant).
func mintCorrelationID() string {
	var buf [16]byte
	if _, err := rand.Read(buf[:]); err != nil {
		return "ts-" + hex.EncodeToString([]byte(time.Now().Format(time.RFC3339Nano)))
	}
	return hex.EncodeToString(buf[:])
}

// recordingWriter remembers the status so the request line is honest, and lets a handler rename
// the correlation id.
type recordingWriter struct {
	http.ResponseWriter
	status      int
	correlation string
}

func (w *recordingWriter) WriteHeader(status int) {
	w.status = status
	w.ResponseWriter.WriteHeader(status)
}

func (w *recordingWriter) setCorrelation(id string) { w.correlation = id }

// withCorrelation wraps the whole surface: security headers, then correlation logging.
func (s *Server) withCorrelation(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if s.opts.Logger == nil {
			next.ServeHTTP(w, r)
			return
		}
		rw := &recordingWriter{ResponseWriter: w, status: http.StatusOK, correlation: mintCorrelationID()}
		r = r.WithContext(context.WithValue(r.Context(), correlationKey{}, rw.correlation))
		start := time.Now()
		next.ServeHTTP(rw, r)
		logLine(s.opts.Logger, rw.correlation, r, rw.status, time.Since(start))
	})
}

// logLine emits one logfmt request line.
func logLine(l *log.Logger, correlation string, r *http.Request, status int, took time.Duration) {
	ri := identityFrom(r)
	actor := "-"
	session := "-"
	if ri != nil {
		actor = ri.Identity.ActorID
		session = ri.Source
		if ri.Session != nil {
			session = "session:" + ri.Session.ID[:12]
		}
	}
	l.Printf("ts=%s correlation_id=%s method=%s path=%s status=%d duration_ms=%.1f remote=%s auth=%s actor=%s",
		time.Now().UTC().Format(time.RFC3339), correlation, r.Method, r.URL.Path, status,
		float64(took.Microseconds())/1000.0, remoteIP(r), session, actor)
}
