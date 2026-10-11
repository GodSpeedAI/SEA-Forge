package sfwp

import (
	"context"
	"encoding/json"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

func rawAskRequest() *Request {
	req := newRequest("ask")
	req.body["kind"] = "ask_capability"
	req.body["subject"] = "build"
	req.body["purpose"] = "planning"
	// Ask has no durable request_id. actor_id is the verified effective user; actor and
	// on_behalf_of carry the configured service and that same user as the protected principal pair.
	req.body["actor_id"] = "operator_local"
	Governance{
		Actor:      Actor{ActorID: "casework_gateway", Role: "gateway"},
		OnBehalfOf: &Actor{ActorID: "operator_local", Role: "operator"},
	}.apply(req.body)
	return req
}

func assertAskWire(t *testing.T, line string) {
	t.Helper()
	var got map[string]json.RawMessage
	if err := json.Unmarshal([]byte(line), &got); err != nil {
		t.Fatalf("decode Ask request line: %v: %s", err, line)
	}
	var verb string
	if err := json.Unmarshal(got["verb"], &verb); err != nil || verb != "ask" {
		t.Fatalf("wire verb=%q, want ask: %s", verb, line)
	}
	if _, exists := got["request_id"]; exists {
		t.Fatalf("uncorrelated Ask must not invent request_id: %s", line)
	}
	if _, exists := got["case"]; exists {
		t.Fatalf("an omitted optional case must remain absent: %s", line)
	}
	var actorID string
	if err := json.Unmarshal(got["actor_id"], &actorID); err != nil || actorID != "operator_local" {
		t.Fatalf("wire actor_id=%q, want verified effective session actor: %s", actorID, line)
	}
	var actor struct {
		ActorID string `json:"actor_id"`
		Role    string `json:"role"`
	}
	if err := json.Unmarshal(got["actor"], &actor); err != nil || actor.ActorID != "casework_gateway" || actor.Role != "gateway" {
		t.Fatalf("wire service actor=%+v, want configured gateway: %s", actor, line)
	}
	var onBehalfOf struct {
		ActorID string `json:"actor_id"`
		Role    string `json:"role"`
	}
	if err := json.Unmarshal(got["on_behalf_of"], &onBehalfOf); err != nil || onBehalfOf.ActorID != "operator_local" || onBehalfOf.Role != "operator" {
		t.Fatalf("wire on_behalf_of=%+v, want verified effective actor: %s", onBehalfOf, line)
	}
}

func TestAskWireHasProtectedActorPairAndNoCorrelationID(t *testing.T) {
	line, err := EncodeRequest(rawAskRequest())
	if err != nil {
		t.Fatal(err)
	}
	assertAskWire(t, string(line))
}

func TestUncertainAskEOFIsUnavailableWithoutResendOrStatusLookup(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) {
		return `{"answer_id":"would-be-answer"}`, true
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	_, err = client.Do(context.Background(), rawAskRequest())
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("ambiguous Ask EOF must return typed unavailable, got %v", err)
	}
	if got := fs.requestCount("ask"); got != 1 {
		t.Fatalf("uncertain Ask sent %d times, want exactly once", got)
	}
	if got := fs.requestCount("request_get_status"); got != 0 {
		t.Fatalf("uncorrelated Ask performed %d status lookups, want zero", got)
	}
	fs.mu.Lock()
	var requestLine string
	if len(fs.requests) == 1 {
		requestLine = fs.requests[0]
	}
	fs.mu.Unlock()
	if requestLine != "" {
		assertAskWire(t, requestLine)
	}
}

func TestUncertainAskTimeoutIsUnavailableWithoutResendOrStatusLookup(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) {
		time.Sleep(100 * time.Millisecond)
		return `{"answer_id":"late-answer"}`, false
	})
	cfg := testConfig(fs.socket)
	cfg.RequestTimeout = 20 * time.Millisecond
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	_, err = client.Do(context.Background(), rawAskRequest())
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("ambiguous Ask timeout must return typed unavailable, got %v", err)
	}
	if got := fs.requestCount("ask"); got != 1 {
		t.Fatalf("uncertain Ask timeout sent %d times, want exactly once", got)
	}
	if got := fs.requestCount("request_get_status"); got != 0 {
		t.Fatalf("uncorrelated Ask performed %d status lookups, want zero", got)
	}
}

func TestUncertainAskResponseOverflowIsUnavailableWithoutResendOrStatusLookup(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) {
		return `{"answer":"` + strings.Repeat("x", 200) + `"}`, false
	})
	cfg := testConfig(fs.socket)
	cfg.MaxResponseLineBytes = 64
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	_, err = client.Do(context.Background(), rawAskRequest())
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("ambiguous Ask overflow must return typed unavailable, got %v", err)
	}
	if got := fs.requestCount("ask"); got != 1 {
		t.Fatalf("uncertain Ask overflow sent %d times, want exactly once", got)
	}
	if got := fs.requestCount("request_get_status"); got != 0 {
		t.Fatalf("uncorrelated Ask performed %d status lookups, want zero", got)
	}
}

func TestExplicitServerBusyAskGetsOneSeparateSafeRetry(t *testing.T) {
	var calls int
	fs := newFakeServer(t, func(string) (string, bool) {
		calls++
		if calls == 1 {
			return `{"error":"the bounded dispatcher is full","error_class":"server_busy","no_side_effect":true}`, false
		}
		return `{"answer_id":"answer-after-safe-retry"}`, false
	})
	cfg := testConfig(fs.socket)
	cfg.BackoffBase = time.Millisecond
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	if _, err := client.Do(context.Background(), rawAskRequest()); err != nil {
		t.Fatalf("explicit pre-admission server_busy may be safely retried once: %v", err)
	}
	if got := fs.requestCount("ask"); got != 2 {
		t.Fatalf("server_busy Ask attempts=%d, want initial plus one bounded safe retry", got)
	}
	if got := fs.requestCount("request_get_status"); got != 0 {
		t.Fatalf("server_busy Ask performed %d status lookups, want zero", got)
	}
}
