package server

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"strings"
	"sync"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// cachedIntentDispatcher models the behavior that makes the HTTP authorization check
// security-sensitive: an exact successful replay returns its saved receipt without repeating
// the logical mutation. The HTTP test verifies that a revoked session cannot reach this cache.
type cachedIntentDispatcher struct {
	mu        sync.Mutex
	receipts  map[string]contract.IntentResponse
	last      contract.InteractionIntent
	handles   int
	mutations int
}

func newCachedIntentDispatcher() *cachedIntentDispatcher {
	return &cachedIntentDispatcher{receipts: map[string]contract.IntentResponse{}}
}

func (d *cachedIntentDispatcher) Handle(_ context.Context, in contract.InteractionIntent) contract.IntentResponse {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.handles++
	d.last = in
	if receipt, exists := d.receipts[in.IntentID]; exists {
		return receipt
	}
	d.mutations++
	receipt := contract.IntentResponse{IntentID: in.IntentID, Success: true}
	d.receipts[in.IntentID] = receipt
	return receipt
}

func (d *cachedIntentDispatcher) counts() (handles, mutations int) {
	d.mu.Lock()
	defer d.mu.Unlock()
	return d.handles, d.mutations
}

func (d *cachedIntentDispatcher) lastActor() contract.IntentActor {
	d.mu.Lock()
	defer d.mu.Unlock()
	return d.last.Actor
}

func TestIntentAuthorizationPrecedesCachedReplay(t *testing.T) {
	h := newLiveHarness(t)
	dispatcher := newCachedIntentDispatcher()
	h.api.intents = dispatcher

	verified := 0
	authorized := true
	h.world.verify = func(actor ports.ActorClaim) error {
		verified++
		if actor != (ports.ActorClaim{ActorID: "operator_local", Role: "operator"}) {
			t.Errorf("intent authorization must verify the session actor, got %+v", actor)
		}
		if !authorized {
			return apperr.New(apperr.KindAuthorityDenied, "", "identity", "delegation revoked")
		}
		return nil
	}
	op := h.login(t, "operator", "ignored-in-dev")
	body := `{"intent_id":"cached-revoked-replay","kind":"CONSEQUENTIAL_CASE","action_name":"ADD_DISCRETIONARY_WORK","target_object_id":"stage-build","case_id":"case_1","client_cursor":"01AAA","actor":{"actor_id":"rso_local","role":"R-SO"},"parameters":{"case_id":"case_1","stage_id":"stage-build","kind":"sandboxed_task","title":"one governed addition"}}`

	// First authorized dispatch writes the cached receipt and the fake's single logical mutation.
	first, err := op.post("/api/intents", "application/json", body)
	if err != nil {
		t.Fatal(err)
	}
	firstBody, err := io.ReadAll(first.Body)
	first.Body.Close()
	if err != nil {
		t.Fatal(err)
	}
	if first.StatusCode != http.StatusOK || !bytes.Contains(firstBody, []byte(`"success":true`)) {
		t.Fatalf("initial authorized intent was not accepted: status=%d body=%s", first.StatusCode, firstBody)
	}
	if actor := dispatcher.lastActor(); actor != (contract.IntentActor{ActorID: "operator_local", Role: "operator"}) {
		t.Fatalf("request actor must be overwritten by the authenticated session: %+v", actor)
	}
	if handles, mutations := dispatcher.counts(); handles != 1 || mutations != 1 {
		t.Fatalf("initial dispatch counts = handles:%d mutations:%d, want 1/1", handles, mutations)
	}

	// Revocation must be checked before the dispatcher can return the cached successful receipt.
	authorized = false
	revoked, err := op.post("/api/intents", "application/json", body)
	if err != nil {
		t.Fatal(err)
	}
	var denied errorBody
	if err := json.NewDecoder(revoked.Body).Decode(&denied); err != nil {
		revoked.Body.Close()
		t.Fatal(err)
	}
	revoked.Body.Close()
	if revoked.StatusCode != http.StatusForbidden || denied.Error.Kind != "authority_denied" {
		t.Fatalf("revoked cached replay must fail closed before returning its receipt: status=%d error=%+v", revoked.StatusCode, denied.Error)
	}
	if handles, mutations := dispatcher.counts(); handles != 1 || mutations != 1 {
		t.Fatalf("revoked replay reached dispatcher/cache: handles:%d mutations:%d, want unchanged 1/1", handles, mutations)
	}

	// Restoring the same session perspective returns the exact cached response, with no new
	// logical mutation. This confirms the guard does not invalidate or rewrite the old receipt.
	authorized = true
	replayed, err := op.post("/api/intents", "application/json", body)
	if err != nil {
		t.Fatal(err)
	}
	replayBody, err := io.ReadAll(replayed.Body)
	replayed.Body.Close()
	if err != nil {
		t.Fatal(err)
	}
	if replayed.StatusCode != http.StatusOK || !bytes.Equal(replayBody, firstBody) {
		t.Fatalf("authorized exact replay must return the original cached receipt: status=%d first=%s replay=%s", replayed.StatusCode, firstBody, replayBody)
	}
	if handles, mutations := dispatcher.counts(); handles != 2 || mutations != 1 {
		t.Fatalf("authorized replay counts = handles:%d mutations:%d, want cache lookup without another mutation", handles, mutations)
	}
	if verified != 3 {
		t.Fatalf("each valid intent POST must verify the session perspective before dispatch; calls=%d", verified)
	}
}

type worldWithoutPerspectiveVerifier struct{ WorldSource }

func TestIntentAuthorizationFailsClosedBeforeDispatcher(t *testing.T) {
	for _, tc := range []struct {
		name       string
		world      func(*liveHarness)
		wantStatus int
		wantKind   string
	}{
		{
			name: "verifier unavailable",
			world: func(h *liveHarness) {
				h.api.world = worldWithoutPerspectiveVerifier{WorldSource: h.world}
			},
			wantStatus: http.StatusServiceUnavailable,
			wantKind:   "unavailable",
		},
		{
			name: "kernel identity check unavailable",
			world: func(h *liveHarness) {
				h.world.verify = func(ports.ActorClaim) error {
					return apperr.New(apperr.KindUnavailable, "", "identity", "kernel unavailable")
				}
			},
			wantStatus: http.StatusBadGateway,
			wantKind:   "unavailable",
		},
		{
			name: "session actor is not delegated",
			world: func(h *liveHarness) {
				h.world.verify = func(ports.ActorClaim) error {
					return apperr.New(apperr.KindAuthorityDenied, "", "identity", "session actor is not mapped")
				}
			},
			wantStatus: http.StatusForbidden,
			wantKind:   "authority_denied",
		},
	} {
		t.Run(tc.name, func(t *testing.T) {
			h := newLiveHarness(t)
			dispatcher := newCachedIntentDispatcher()
			h.api.intents = dispatcher
			tc.world(h)
			op := h.login(t, "operator", "ignored-in-dev")
			resp, err := op.post("/api/intents", "application/json", validIntentBody("fail-closed-"+strings.ReplaceAll(tc.name, " ", "-")))
			if err != nil {
				t.Fatal(err)
			}
			var body errorBody
			if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
				resp.Body.Close()
				t.Fatal(err)
			}
			resp.Body.Close()
			if resp.StatusCode != tc.wantStatus || body.Error.Kind != tc.wantKind {
				t.Fatalf("intent auth failure: status=%d kind=%q, want %d/%q", resp.StatusCode, body.Error.Kind, tc.wantStatus, tc.wantKind)
			}
			if handles, mutations := dispatcher.counts(); handles != 0 || mutations != 0 {
				t.Fatalf("failed intent authorization reached dispatcher/cache: handles:%d mutations:%d", handles, mutations)
			}
		})
	}
}

func TestIntentRequestGuardsPrecedePerspectiveVerification(t *testing.T) {
	h := newLiveHarness(t)
	dispatcher := newCachedIntentDispatcher()
	h.api.intents = dispatcher
	verified := 0
	h.world.verify = func(ports.ActorClaim) error {
		verified++
		return nil
	}
	op := h.login(t, "operator", "ignored-in-dev")

	requests := []struct {
		name       string
		body       string
		csrf       bool
		origin     string
		wantStatus int
	}{
		{name: "missing CSRF", body: validIntentBody("guard-no-csrf"), wantStatus: http.StatusForbidden},
		{name: "untrusted origin", body: validIntentBody("guard-origin"), csrf: true, origin: "https://attacker.example.test", wantStatus: http.StatusForbidden},
		{name: "malformed JSON", body: `{"intent_id":`, csrf: true, wantStatus: http.StatusBadRequest},
	}
	for _, tc := range requests {
		t.Run(tc.name, func(t *testing.T) {
			req, err := http.NewRequest(http.MethodPost, h.ts.URL+"/api/intents", strings.NewReader(tc.body))
			if err != nil {
				t.Fatal(err)
			}
			req.Header.Set("Content-Type", "application/json")
			if tc.csrf {
				req.Header.Set(csrfHeader, op.csrf)
			}
			if tc.origin != "" {
				req.Header.Set("Origin", tc.origin)
			}
			resp, err := op.client.Do(req)
			if err != nil {
				t.Fatal(err)
			}
			io.Copy(io.Discard, resp.Body)
			resp.Body.Close()
			if resp.StatusCode != tc.wantStatus {
				t.Fatalf("guard status = %d, want %d", resp.StatusCode, tc.wantStatus)
			}
		})
	}
	if verified != 0 {
		t.Fatalf("CSRF, Origin, and strict body guards must run before kernel perspective verification; calls=%d", verified)
	}
	if handles, mutations := dispatcher.counts(); handles != 0 || mutations != 0 {
		t.Fatalf("rejected request reached dispatcher/cache: handles:%d mutations:%d", handles, mutations)
	}
}

func validIntentBody(id string) string {
	return `{"intent_id":"` + id + `","kind":"CONSEQUENTIAL_CASE","action_name":"ADD_DISCRETIONARY_WORK","target_object_id":"stage-build","case_id":"case_1","client_cursor":"01AAA","actor":{"actor_id":"someone_else","role":"admin"},"parameters":{"case_id":"case_1","stage_id":"stage-build","kind":"sandboxed_task","title":"test addition"}}`
}
