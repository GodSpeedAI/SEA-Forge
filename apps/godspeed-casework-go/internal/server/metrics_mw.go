package server

import (
	"net/http"
	"time"
)

// withMetrics times every request by its mux route pattern (a closed set, so label cardinality is
// bounded) and status class. The SSE route is excluded from the latency histogram - a stream's
// duration is its lifetime - and is accounted by the SSE gauges instead. It wraps the mux
// directly so r.Pattern, which the mux sets on the request it routes, is readable afterwards.
func (s *Server) withMetrics(next http.Handler) http.Handler {
	if s.opts.Metrics == nil {
		return next
	}
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		sw := &statusWriter{ResponseWriter: w, status: http.StatusOK}
		start := time.Now()
		next.ServeHTTP(sw, r)
		if r.Pattern == "GET /api/events" {
			return
		}
		s.opts.Metrics.ObserveRequest(r.Pattern, sw.status, time.Now().Sub(start))
	})
}

type statusWriter struct {
	http.ResponseWriter
	status int
	wrote  bool
}

func (w *statusWriter) WriteHeader(status int) {
	if !w.wrote {
		w.status, w.wrote = status, true
	}
	w.ResponseWriter.WriteHeader(status)
}

func (w *statusWriter) Flush() {
	if f, ok := w.ResponseWriter.(http.Flusher); ok {
		f.Flush()
	}
}

// Unwrap lets http.ResponseController reach the underlying writer.
func (w *statusWriter) Unwrap() http.ResponseWriter { return w.ResponseWriter }

// setCorrelation forwards to the logging writer beneath: the intent handler renames the request's
// correlation id through this interface, and a wrapper that hid it would silently break the
// browser -> gateway -> SFWP request_id join.
func (w *statusWriter) setCorrelation(id string) {
	if cs, ok := w.ResponseWriter.(correlationSetter); ok {
		cs.setCorrelation(id)
	}
}
