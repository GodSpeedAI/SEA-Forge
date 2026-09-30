// Package livestack assembles the full T06/T07 live gateway over one livetest cell: the same
// wiring main.go's serveLive performs (T05 client -> authority -> live source -> relay -> intents
// -> HTTP surface, with the T07 session layer: dev-mode authenticator, sessions, CSRF, rate
// limits). It exists so the -tags live tests in other packages exercise the gateway exactly the
// way production wires it, without import cycles into the packages under test.
package livestack

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/http/cookiejar"
	"net/http/httptest"
	"net/url"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/intents"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server"
)

// NewCell boots a fresh seeded cell (re-exported for livestack callers' convenience).
var NewCell = livetest.NewCell

// Stack is the assembled live gateway over one cell.
type Stack struct {
	Cell       *livetest.Cell
	Client     *sfwp.Client
	Authority  *sfwp.Authority
	Source     *projection.LiveSource
	Store      *projection.Store
	Relay      *server.Relay
	Dispatcher *intents.Handler
	API        *server.Server
	Cancel     func()
}

// AssembleStack wires the full live stack (T06 projection + T07 sessions) over the cell's kernel.
func AssembleStack(t *testing.T, cell *livetest.Cell) *Stack {
	return AssembleStackWithRetention(t, cell, 0)
}

// AssembleStackWithRetention wires the live stack with an explicit projection retention limit.
// Live integration tests use small limits to exercise the real HTTP resync path.
func AssembleStackWithRetention(t *testing.T, cell *livetest.Cell, retention int) *Stack {
	t.Helper()
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)

	client := cell.Client()
	authority := sfwp.NewAuthority(client)
	source := projection.NewLiveSource(authority, ports.ActorClaim{ActorID: "gateway", Role: "service"})
	store := projection.NewStoreWithRetention(retention)

	sub, err := client.Subscribe(ctx)
	if err != nil {
		t.Fatalf("subscribe: %v", err)
	}
	relay := server.NewRelay(server.NewSFWPFeed(sub), source, store, server.RelayOptions{
		DefaultActor: ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
		Logger:       log.New(os.Stderr, "livetest-relay: ", 0),
	})
	go relay.Run(ctx)

	dispatcher := intents.NewHandler(authority, relay, authority, intents.Options{
		Gateway:             ports.ActorClaim{ActorID: "gateway", Role: "service"},
		PolicyRef:           "authority/active-policy.json",
		ExecutionTimeoutSec: 60,
	})
	api := server.NewWithArtifacts(source, dispatcher, source, store, relay, authority, server.Options{
		Perspective: ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
		Auth:        TestAuthOptions(t),
		Ready:       authority,
		RateLimit:   server.RateLimitOptions{PerMinute: 600, Burst: 100},
	})

	return &Stack{
		Cell:       cell,
		Client:     client,
		Authority:  authority,
		Source:     source,
		Store:      store,
		Relay:      relay,
		Dispatcher: dispatcher,
		API:        api,
		Cancel:     cancel,
	}
}

// TestAuthOptions is the T07 dev-mode session wiring the live tests share: two users mapping to
// the cell's two delegable end-user actors (the shape T10's two-user L5 ladder needs), short
// session TTLs, and no cookie Secure flag (plain-HTTP httptest).
func TestAuthOptions(t *testing.T) server.AuthOptions {
	t.Helper()
	store, err := auth.NewLocalUserStore(auth.ModeDev, []auth.User{
		{Username: "operator", DisplayName: "Operator", ActorID: "operator_local", Role: "operator"},
		{Username: "rso", DisplayName: "R-SO", ActorID: "rso_local", Role: "R-SO"},
	})
	if err != nil {
		t.Fatal(err)
	}
	return server.AuthOptions{
		Authenticator: store,
		Sessions:      auth.NewSessionStore(30*time.Minute, time.Hour, 0, nil),
		CookieSecure:  false,
	}
}

// Session is one authenticated browser user against the stack's HTTP surface: a cookie-jarred
// client bound to its server-side session, plus the CSRF token every state-changing POST must
// carry. The kernel standing is fixed by the session (operator_local/operator or rso_local/R-SO).
type Session struct {
	t    *testing.T
	base string
	// Client is the cookie-jarred HTTP client bound to this server-side session.
	Client   *http.Client
	CSRF     string
	Identity auth.Identity
}

// Login runs the credential flow against an httptest server wrapping the stack's API and
// returns the authenticated session. The dev posture does not check passwords; the point of the
// exercise is the session/CSRF/actor path.
func (s *Stack) Login(t *testing.T, base, username string) *Session {
	t.Helper()
	jar, err := cookiejar.New(nil)
	if err != nil {
		t.Fatal(err)
	}
	sess := &Session{t: t, base: base, Client: &http.Client{Jar: jar}}
	// Any GET mints the pre-login CSRF cookie (double submit).
	resp, err := sess.Client.Get(base + "/api/session")
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	// POST the credentials with the cookie's value echoed in the header.
	body := fmt.Sprintf(`{"username":%q,"password":"livetest"}`, username)
	req, err := http.NewRequest(http.MethodPost, base+"/api/auth/login", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-CSRF-Token", sess.cookieValue(base, "casework_csrf"))
	resp, err = sess.Client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		raw, _ := io.ReadAll(resp.Body)
		t.Fatalf("login as %s: status %d body %s", username, resp.StatusCode, raw)
	}
	var wire struct {
		Authenticated bool   `json:"authenticated"`
		Username      string `json:"username"`
		ActorID       string `json:"actor_id"`
		Role          string `json:"role"`
		CSRFToken     string `json:"csrf_token"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&wire); err != nil {
		t.Fatal(err)
	}
	if !wire.Authenticated || wire.CSRFToken == "" {
		t.Fatalf("login as %s did not start a session: %+v", username, wire)
	}
	sess.CSRF = wire.CSRFToken
	sess.Identity = auth.Identity{Username: wire.Username, ActorID: wire.ActorID, Role: wire.Role}
	return sess
}

// HTTPServer wraps the stack's API in an httptest server (each test runs its own instance).
func (s *Stack) HTTPServer(t *testing.T) *httptest.Server {
	t.Helper()
	ts := httptest.NewServer(s.API.Handler())
	t.Cleanup(ts.Close)
	return ts
}

func (s *Session) cookieValue(base, name string) string {
	s.t.Helper()
	u, err := url.Parse(base)
	if err != nil {
		s.t.Fatal(err)
	}
	for _, c := range s.Client.Jar.Cookies(u) {
		if c.Name == name {
			return c.Value
		}
	}
	return ""
}

// Get performs an authenticated GET and returns the response (caller closes the body).
func (s *Session) Get(path string) (*http.Response, error) {
	req, err := http.NewRequest(http.MethodGet, s.base+path, nil)
	if err != nil {
		s.t.Fatal(err)
	}
	return s.Client.Do(req)
}

// PostJSON performs an authenticated state-changing POST with the session's CSRF token.
func (s *Session) PostJSON(path string, body any) (*http.Response, error) {
	raw, err := json.Marshal(body)
	if err != nil {
		s.t.Fatal(err)
	}
	req, err := http.NewRequest(http.MethodPost, s.base+path, bytes.NewReader(raw))
	if err != nil {
		s.t.Fatal(err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("X-CSRF-Token", s.CSRF)
	return s.Client.Do(req)
}

// CommitSentryChain commits the T03 sentry-chain template as the delegated operator and returns
// the case id. The preflight digest is fetched live so the commit passes the kernel's own
// precondition check.
func (s *Stack) CommitSentryChain(t *testing.T, requestID string) string {
	t.Helper()
	return s.CommitTemplate(t, "e2e-sentry-chain@0.1.0", map[string]string{
		"dataset_name":  "orders-q3",
		"dataset_label": "t06",
		"max_rows":      "25",
		"out_dir":       "work",
	}, requestID, "")
}

// CommitTemplate commits one template as the delegated operator; when pin is non-nil its digest
// rides as the commit precondition (the kernel verifies it).
func (s *Stack) CommitTemplate(t *testing.T, templateRef string, params map[string]string, requestID string, pinDigest string) string {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	draft := ports.CaseDraft{TemplateRef: templateRef, Params: params}
	pin := ports.PreconditionDigest{}
	if pinDigest != "" {
		pin = ports.PreconditionDigest{Present: true, TemplateRef: "template:" + templateRef, ExpectedDigest: pinDigest}
	}
	receipt, err := s.Authority.CommitCase(ctx, draft, pin, ports.GovernedOptions{
		Governance: livetest.DelegatedPortsGovernance("operator_local", "operator"),
		Policy:     "authority/active-policy.json",
		RequestID:  requestID,
	})
	if err != nil {
		t.Fatalf("commit %s: %v", templateRef, err)
	}
	if receipt.CaseID == "" {
		t.Fatalf("commit returned no case id: %+v", receipt)
	}
	return receipt.CaseID
}

// WaitRevision waits until the relay records a revision at or after the given minimum cursor
// (or any revision for the case when minCursor is empty) and returns the case's cursor.
func (s *Stack) WaitRevision(t *testing.T, caseID, minCursor string) string {
	t.Helper()
	livetest.WaitUntil(t, 10*time.Second, "relay revision for "+caseID, func() bool {
		cursor, ok := s.Relay.CursorForCase(caseID)
		return ok && (minCursor == "" || cursor > minCursor)
	})
	cursor, _ := s.Relay.CursorForCase(caseID)
	return cursor
}
