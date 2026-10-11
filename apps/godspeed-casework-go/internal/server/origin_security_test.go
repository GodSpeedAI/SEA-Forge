package server

import (
	"bytes"
	"context"
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
)

func TestConfiguredOriginCORSStillRequiresSessionAndCSRF(t *testing.T) {
	const trusted = "https://app.example.test"
	h := newLiveHarness(t)
	h.api.opts.TrustedOrigins = []string{trusted}
	h.ts.Close()
	h.ts = httptest.NewServer(h.api.Handler())
	t.Cleanup(h.ts.Close)

	// Preflight grants only the exact configured origin and carries credentials for the browser
	// session cookie. It returns no projection data and never invokes an API handler.
	preflight, err := http.NewRequest(http.MethodOptions, h.ts.URL+"/api/world", nil)
	if err != nil {
		t.Fatal(err)
	}
	preflight.Header.Set("Origin", trusted)
	preflight.Header.Set("Access-Control-Request-Method", "GET")
	resp, err := http.DefaultClient.Do(preflight)
	if err != nil {
		t.Fatal(err)
	}
	body, _ := io.ReadAll(resp.Body)
	resp.Body.Close()
	if resp.StatusCode != http.StatusNoContent || len(body) != 0 {
		t.Fatalf("preflight must be an empty 204, got status=%d body=%q", resp.StatusCode, body)
	}
	if got := resp.Header.Get("Access-Control-Allow-Origin"); got != trusted {
		t.Fatalf("configured origin must receive exact CORS grant, got %q", got)
	}
	if got := resp.Header.Get("Access-Control-Allow-Credentials"); got != "true" {
		t.Fatalf("credentialed session requests need explicit CORS credentials, got %q", got)
	}

	op := newSessionFromServer(t, h.ts.URL)
	bodyJSON := `{"intent_id":"trusted-origin-intent","kind":"CONSEQUENTIAL_CASE","action_name":"EXECUTE_ITEM","case_id":"case_1","client_cursor":"01AAA","parameters":{"item_id":"i"}}`
	post := func(origin, csrf string) (*http.Response, error) {
		t.Helper()
		req, err := http.NewRequest(http.MethodPost, h.ts.URL+"/api/intents", strings.NewReader(bodyJSON))
		if err != nil {
			return nil, err
		}
		req.Header.Set("Content-Type", "application/json")
		if csrf != "" {
			req.Header.Set(csrfHeader, csrf)
		}
		if origin != "" {
			req.Header.Set("Origin", origin)
		}
		response, err := op.client.Do(req)
		return response, err
	}

	// An exact configured Origin does not bypass the existing synchronizer token.
	missingToken, err := post(trusted, "")
	if err != nil {
		t.Fatal(err)
	}
	missingToken.Body.Close()
	if missingToken.StatusCode != http.StatusForbidden || h.ints.called() {
		t.Fatalf("trusted origin without CSRF token must be refused before dispatch: status=%d called=%t", missingToken.StatusCode, h.ints.called())
	}

	forgedOrigin, err := post("https://evil.example.test", op.csrf)
	if err != nil {
		t.Fatal(err)
	}
	forgedOrigin.Body.Close()
	if forgedOrigin.StatusCode != http.StatusForbidden || h.ints.called() {
		t.Fatalf("untrusted origin with valid CSRF must be refused before dispatch: status=%d called=%t", forgedOrigin.StatusCode, h.ints.called())
	}

	// The configured same-site deployment origin passes CORS and Origin validation, while
	// authentication and the valid per-session synchronizer token remain mandatory.
	accepted, err := post(trusted, op.csrf)
	if err != nil {
		t.Fatal(err)
	}
	defer accepted.Body.Close()
	if accepted.StatusCode != http.StatusOK || !h.ints.called() {
		t.Fatalf("trusted origin with session and CSRF token should dispatch: status=%d called=%t", accepted.StatusCode, h.ints.called())
	}
}

func TestConfiguredOriginDoesNotExposeUnauthenticatedProjection(t *testing.T) {
	const trusted = "https://app.example.test"
	h := newLiveHarness(t)
	h.pushRevision(t, "01AAA", "case_1")
	h.api.opts.TrustedOrigins = []string{trusted}
	h.ts.Close()
	h.ts = httptest.NewServer(h.api.Handler())
	t.Cleanup(h.ts.Close)

	req, err := http.NewRequest(http.MethodGet, h.ts.URL+"/api/world", nil)
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Origin", trusted)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("CORS allowlisting must not authenticate an unauthenticated request, got %d", resp.StatusCode)
	}
	for _, privateField := range []string{"visible_objects", "world_id", "case_id"} {
		if bytes.Contains(body, []byte(privateField)) {
			t.Fatalf("unauthenticated response leaked %q: %s", privateField, body)
		}
	}
}

func TestOriginMiddlewareKeepsLoopbackDefaultAndRejectsDuplicateOrigin(t *testing.T) {
	if !originAllowed("http://127.0.0.1:4178", nil) || originAllowed("https://evil.example.test", nil) {
		t.Fatal("empty origin configuration must retain only the historical loopback allowlist")
	}
	called := false
	handler := withOriginCORS(http.HandlerFunc(func(http.ResponseWriter, *http.Request) { called = true }), []string{"https://app.example.test"})
	req := httptest.NewRequest(http.MethodPost, "http://gateway.test/api/intents", nil)
	req.Header.Add("Origin", "https://app.example.test")
	req.Header.Add("Origin", "https://evil.example.test")
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)
	if rec.Code != http.StatusForbidden || called || rec.Header().Get("Access-Control-Allow-Origin") != "" {
		t.Fatalf("duplicate Origin fields must fail closed before the route: status=%d called=%t headers=%v", rec.Code, called, rec.Header())
	}
}

func TestRequestLogEscapesClientControlledFields(t *testing.T) {
	r := httptest.NewRequest(http.MethodGet, "http://gateway.test/api/ok", nil)
	r.URL.Path = "/api/item\nstatus=200"
	r = r.WithContext(context.WithValue(r.Context(), ctxKey{}, &requestIdentity{
		Identity: auth.Identity{ActorID: "operator\nauth=admin"},
		Source:   "session",
	}))
	var logOutput bytes.Buffer
	logger := log.New(&logOutput, "", 0)
	logLine(logger, "intent-1\nstatus=200", r, http.StatusBadRequest, time.Millisecond)

	line := logOutput.String()
	if strings.Count(line, "\n") != 1 {
		t.Fatalf("one request must produce exactly one physical log line, got %q", line)
	}
	if !strings.Contains(line, `correlation_id="intent-1\nstatus=200"`) {
		t.Fatalf("correlation value must be escaped as one logfmt field, got %q", line)
	}
	if !strings.Contains(line, `path="/api/item\nstatus=200"`) {
		t.Fatalf("request path must be escaped as one logfmt field, got %q", line)
	}
	if !strings.Contains(line, `actor="operator\nauth=admin"`) {
		t.Fatalf("identity fields must be escaped as one logfmt field, got %q", line)
	}
}
