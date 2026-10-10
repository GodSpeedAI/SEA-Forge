package metrics

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

func TestNilRegistryIsSafe(t *testing.T) {
	var r *Registry
	r.ObserveRequest("GET /x", 200, time.Millisecond)
	r.ObserveSFWPError("verb", "unavailable")
	c := r.SSEOpen("", nil)
	c.Delivered("c", time.Now())
	c.Close()
	r.SetBehindSource(nil)
}

func TestHistogramIsCumulativeAndCountsErrorsByClass(t *testing.T) {
	r := New()
	r.ObserveRequest("GET /a", 200, 7*time.Millisecond) // lands in le=0.01
	r.ObserveRequest("GET /a", 200, 20*time.Second)     // +Inf only
	r.ObserveSFWPError("case_commit", "unavailable")
	r.ObserveSFWPError("case_commit", "unavailable")
	r.ObserveSFWPError("approval_decide", "authority_denied")
	var sb strings.Builder
	r.Write(&sb)
	out := sb.String()
	for _, want := range []string{
		`casework_http_request_duration_seconds_bucket{route="GET /a",status_class="2xx",le="0.005"} 0`,
		`casework_http_request_duration_seconds_bucket{route="GET /a",status_class="2xx",le="0.01"} 1`,
		`casework_http_request_duration_seconds_bucket{route="GET /a",status_class="2xx",le="10"} 1`,
		`casework_http_request_duration_seconds_bucket{route="GET /a",status_class="2xx",le="+Inf"} 2`,
		`casework_http_request_duration_seconds_count{route="GET /a",status_class="2xx"} 2`,
		`casework_sfwp_errors_total{class="unavailable",verb="case_commit"} 2`,
		`casework_sfwp_errors_total{class="authority_denied",verb="approval_decide"} 1`,
		`casework_sse_clients 0`,
	} {
		if !strings.Contains(out, want+"\n") {
			t.Errorf("missing %q in:\n%s", want, out)
		}
	}
}

func TestLagGaugesUseTheLastDeliveredCursor(t *testing.T) {
	r := New()
	r.SetBehindSource(func(cursor string) int {
		if cursor == "01A" {
			return 3
		}
		return 0
	})
	queued := 5
	c := r.SSEOpen("01A", func() int { return queued })
	var sb strings.Builder
	r.Write(&sb)
	for _, want := range []string{"casework_sse_clients 1", "casework_sse_queue_depth_max 5", "casework_sse_cursor_lag_max 3"} {
		if !strings.Contains(sb.String(), want+"\n") {
			t.Errorf("missing %q in:\n%s", want, sb.String())
		}
	}
	c.Delivered("01B", time.Now())
	sb.Reset()
	r.Write(&sb)
	if !strings.Contains(sb.String(), "casework_sse_cursor_lag_max 0\n") {
		t.Errorf("lag must drop once the client is caught up:\n%s", sb.String())
	}
	c.Close()
}

func TestHandlerMethodsAndMux(t *testing.T) {
	r := New()
	rec := httptest.NewRecorder()
	r.ServeMux().ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/metrics", nil))
	if rec.Code != 200 || !strings.HasPrefix(rec.Header().Get("Content-Type"), "text/plain; version=0.0.4") {
		t.Fatalf("scrape: %d %v", rec.Code, rec.Header())
	}
	rec = httptest.NewRecorder()
	r.ServeMux().ServeHTTP(rec, httptest.NewRequest(http.MethodPost, "/metrics", nil))
	if rec.Code != http.StatusMethodNotAllowed {
		t.Fatalf("POST: %d", rec.Code)
	}
	rec = httptest.NewRecorder()
	r.ServeMux().ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/other", nil))
	if rec.Code != http.StatusNotFound {
		t.Fatalf("other path: %d", rec.Code)
	}
}

func TestValidateListenAddrRefusesNonLoopback(t *testing.T) {
	for _, ok := range []string{"127.0.0.1:9100", "[::1]:9100", "localhost:9100"} {
		if err := ValidateListenAddr(ok); err != nil {
			t.Errorf("%s: %v", ok, err)
		}
	}
	for _, bad := range []string{"0.0.0.0:9100", ":9100", "10.1.2.3:9100", "example.com:9100", "nonsense"} {
		if err := ValidateListenAddr(bad); err == nil {
			t.Errorf("%s must be refused", bad)
		}
	}
}
