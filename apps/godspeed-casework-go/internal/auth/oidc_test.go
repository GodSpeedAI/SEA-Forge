// OIDC tests: the claim->actor mapping rules and the REAL flow construction against an in-test
// OIDC provider (discovery, code exchange, ID-token signature/nonce verification). What this
// proves: the wire behaviour of our OAuth2/OIDC client code end to end against a conformant
// provider. What it does NOT prove: a real browser redirect chain, a real identity provider,
// TLS, or the operator's issuer choice - those are deployment concerns and are documented as
// such in the T07 report.
package auth

import (
	"context"
	"crypto"
	"crypto/rand"
	"crypto/rsa"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"math/big"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
	"time"
)

// testOIDCProvider is a minimal but CONFORMANT OIDC provider: discovery, JWKS and a token
// endpoint that exchanges any code for an RS256-signed ID token. Only tests import it.
type testOIDCProvider struct {
	srv            *httptest.Server
	key            *rsa.PrivateKey
	kid            string
	pathCode       string // the code the test "browser" presents
	currentIDToken string // the token the /token endpoint hands out; each test re-signs it
}

func newTestOIDCProvider(t *testing.T) *testOIDCProvider {
	t.Helper()
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		t.Fatal(err)
	}
	p := &testOIDCProvider{key: key, kid: "test-key", pathCode: "the-code"}
	mux := http.NewServeMux()
	mux.HandleFunc("/.well-known/openid-configuration", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(t, w, map[string]any{
			"issuer":                                p.srv.URL,
			"authorization_endpoint":                p.srv.URL + "/authorize",
			"token_endpoint":                        p.srv.URL + "/token",
			"jwks_uri":                              p.srv.URL + "/jwks",
			"id_token_signing_alg_values_supported": []string{"RS256"},
		})
	})
	mux.HandleFunc("/jwks", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(t, w, map[string]any{"keys": []map[string]any{{
			"kty": "RSA", "use": "sig", "alg": "RS256", "kid": p.kid,
			"n": base64.RawURLEncoding.EncodeToString(key.PublicKey.N.Bytes()),
			"e": base64.RawURLEncoding.EncodeToString(big.NewInt(int64(key.PublicKey.E)).Bytes()),
		}}})
	})
	mux.HandleFunc("/token", func(w http.ResponseWriter, r *http.Request) {
		if err := r.ParseForm(); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		if got := r.FormValue("grant_type"); got != "authorization_code" {
			http.Error(w, "bad grant_type "+got, http.StatusBadRequest)
			return
		}
		if r.FormValue("client_id") != "casework-gateway" || r.FormValue("client_secret") != "the-secret" {
			http.Error(w, "bad client credentials", http.StatusUnauthorized)
			return
		}
		if r.FormValue("code") != p.pathCode {
			http.Error(w, "bad code", http.StatusBadRequest)
			return
		}
		if r.FormValue("redirect_uri") == "" {
			http.Error(w, "missing redirect_uri", http.StatusBadRequest)
			return
		}
		// The id_token rides on a test-scoped response the test pre-loads.
		writeJSON(t, w, map[string]any{
			"access_token": "at-" + r.FormValue("code"),
			"token_type":   "Bearer",
			"id_token":     p.currentIDToken,
		})
	})
	p.srv = httptest.NewServer(mux)
	t.Cleanup(p.srv.Close)
	return p
}

func writeJSON(t *testing.T, w http.ResponseWriter, v any) {
	t.Helper()
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(v)
}

// currentIDToken is the (single) token the /token endpoint hands out; each flow test re-signs it
// with the claims it wants the "provider" to assert (see the struct field).

func (p *testOIDCProvider) signIDToken(t *testing.T, nonce string, extra map[string]any) string {
	t.Helper()
	header := b64json(map[string]any{"alg": "RS256", "typ": "JWT", "kid": p.kid})
	now := time.Now()
	payload := map[string]any{
		"iss": p.srv.URL,
		"aud": "casework-gateway",
		"sub": "user-sub-42",
		"iat": now.Unix(),
		"exp": now.Add(5 * time.Minute).Unix(),
	}
	for k, v := range extra {
		payload[k] = v
	}
	if nonce != "" {
		payload["nonce"] = nonce
	}
	signingInput := header + "." + b64json(payload)
	sum := sha256.Sum256([]byte(signingInput))
	sig, err := rsa.SignPKCS1v15(rand.Reader, p.key, crypto.SHA256, sum[:])
	if err != nil {
		t.Fatal(err)
	}
	return signingInput + "." + base64.RawURLEncoding.EncodeToString(sig)
}

func b64json(v any) string {
	raw, _ := json.Marshal(v)
	return base64.RawURLEncoding.EncodeToString(raw)
}

func newOIDCFor(t *testing.T, p *testOIDCProvider, mappings []OIDCMapping) *OIDC {
	t.Helper()
	a, err := NewOIDC(context.Background(), OIDCOptions{
		Issuer:        p.srv.URL,
		ClientID:      "casework-gateway",
		ClientSecret:  "the-secret",
		RedirectURL:   "http://127.0.0.1:4179/api/auth/callback",
		UsernameClaim: "preferred_username",
		Mappings:      mappings,
		HTTPClient:    p.srv.Client(),
	})
	if err != nil {
		t.Fatal(err)
	}
	return a
}

func TestOIDCDiscoveryAndFlowConstruction(t *testing.T) {
	p := newTestOIDCProvider(t)
	a := newOIDCFor(t, p, []OIDCMapping{{Claim: "preferred_username", Equals: "alice", ActorID: "operator_local", Role: "operator"}})
	if a.Mode() != ModeOIDC {
		t.Fatalf("mode: %q", a.Mode())
	}
	state, loginURL, err := a.LoginURL(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	u, err := url.Parse(loginURL)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(loginURL, p.srv.URL+"/authorize") {
		t.Fatalf("the login URL must target the provider's authorization endpoint: %q", loginURL)
	}
	q := u.Query()
	if q.Get("client_id") != "casework-gateway" || q.Get("response_type") != "code" {
		t.Fatalf("authorization request: %v", q)
	}
	if q.Get("state") != state || q.Get("nonce") == "" {
		t.Fatalf("state/nonce: %v", q)
	}
	if !containsScope(q.Get("scope"), "openid") {
		t.Fatalf("openid scope missing: %v", q)
	}
	if q.Get("redirect_uri") != "http://127.0.0.1:4179/api/auth/callback" {
		t.Fatalf("redirect_uri: %v", q)
	}
}

func containsScope(scope, want string) bool {
	for _, s := range strings.Fields(scope) {
		if s == want {
			return true
		}
	}
	return false
}

func TestOIDCMappingRules(t *testing.T) {
	mappings := []OIDCMapping{
		{Claim: "groups", Equals: "operators", ActorID: "operator_local", Role: "operator"},
		{Claim: "groups", Equals: "rso", ActorID: "rso_local", Role: "R-SO"},
		{Claim: "preferred_username", Equals: "root", ActorID: "operator_local", Role: "operator"},
	}
	id, err := MapClaimsToIdentity(map[string]any{
		"preferred_username": "alice", "name": "Alice Example", "sub": "sub-1",
		"groups": []any{"rso", "operators"},
	}, "preferred_username", mappings)
	if err != nil {
		t.Fatal(err)
	}
	// First matching rule wins (groups membership "rso" precedes "operators" in claim order but
	// the RULE order decides: operators is rule 0).
	if id.ActorID != "operator_local" || id.Role != "operator" {
		t.Fatalf("first matching rule must win: %+v", id)
	}
	if id.Username != "alice" || id.DisplayName != "Alice Example" {
		t.Fatalf("identity claims: %+v", id)
	}

	id, err = MapClaimsToIdentity(map[string]any{"preferred_username": "root", "sub": "sub-9"}, "preferred_username", mappings)
	if err != nil || id.ActorID != "operator_local" {
		t.Fatalf("scalar claim rule: %+v err=%v", id, err)
	}

	if _, err := MapClaimsToIdentity(map[string]any{"preferred_username": "mallory"}, "preferred_username", mappings); !errors.Is(err, ErrNoActorMapping) {
		t.Fatalf("an unmapped identity must be refused: %v", err)
	}
	if _, err := MapClaimsToIdentity(map[string]any{"sub": "sub-2"}, "", mappings); !errors.Is(err, ErrNoActorMapping) {
		t.Fatalf("missing claim: %v", err)
	}
}

func TestOIDCStateIsSingleUseAndBounded(t *testing.T) {
	p := newTestOIDCProvider(t)
	a := newOIDCFor(t, p, []OIDCMapping{{Claim: "preferred_username", Equals: "alice", ActorID: "operator_local", Role: "operator"}})
	ctx := context.Background()

	state, _, err := a.LoginURL(ctx)
	if err != nil {
		t.Fatal(err)
	}
	// An unknown state is refused BEFORE any network call.
	if _, err := a.Callback(ctx, "forged", p.pathCode); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("forged state: %v", err)
	}
	// The real state exchanges and maps.
	p.currentIDToken = p.signIDToken(t, "", map[string]any{"preferred_username": "alice"})
	id, err := a.Callback(ctx, state, p.pathCode)
	if err != nil {
		t.Fatal(err)
	}
	if id.ActorID != "operator_local" || id.Username != "alice" {
		t.Fatalf("flow identity: %+v", id)
	}
	// The state was consumed: replaying it must fail.
	if _, err := a.Callback(ctx, state, p.pathCode); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("a state must be single use: %v", err)
	}
	// A bad code is refused.
	state2, _, _ := a.LoginURL(ctx)
	if _, err := a.Callback(ctx, state2, "wrong-code"); !errors.Is(err, ErrInvalidCredentials) {
		t.Fatalf("bad code: %v", err)
	}
}

func TestOIDCNonceIsVerifiedAndWrongAudienceRefused(t *testing.T) {
	p := newTestOIDCProvider(t)
	a := newOIDCFor(t, p, []OIDCMapping{{Claim: "preferred_username", Equals: "alice", ActorID: "operator_local", Role: "operator"}})
	ctx := context.Background()
	state, loginURL, _ := a.LoginURL(ctx)
	nonce := loginURLNonce(t, loginURL)

	// A token asserting a DIFFERENT nonce must be refused (and consumes the state).
	p.currentIDToken = p.signIDToken(t, "attacker-nonce", map[string]any{"preferred_username": "alice"})
	if _, err := a.Callback(ctx, state, p.pathCode); err == nil || !strings.Contains(err.Error(), "nonce") {
		t.Fatalf("a mismatched nonce must be refused, got %v", err)
	}
	// A fresh login whose token carries the request's nonce passes.
	state, loginURL, _ = a.LoginURL(ctx)
	nonce = loginURLNonce(t, loginURL)
	p.currentIDToken = p.signIDToken(t, nonce, map[string]any{"preferred_username": "alice"})
	if _, err := a.Callback(ctx, state, p.pathCode); err != nil {
		t.Fatalf("matching nonce must pass: %v", err)
	}
	// A token for another audience (client) must fail signature/audience verification.
	state, loginURL, _ = a.LoginURL(ctx)
	nonce = loginURLNonce(t, loginURL)
	wrongAud := p.signIDToken(t, nonce, map[string]any{"preferred_username": "alice", "aud": "someone-else"})
	p.currentIDToken = wrongAud
	if _, err := a.Callback(ctx, state, p.pathCode); err == nil {
		t.Fatal("a foreign audience must not verify")
	}
}

func loginURLNonce(t *testing.T, loginURL string) string {
	t.Helper()
	u, err := url.Parse(loginURL)
	if err != nil {
		t.Fatal(err)
	}
	return u.Query().Get("nonce")
}

func TestOIDCConstructionRefusesIncompleteConfig(t *testing.T) {
	if _, err := NewOIDC(context.Background(), OIDCOptions{Issuer: "", ClientID: "x", ClientSecret: "y",
		Mappings: []OIDCMapping{{Claim: "a", Equals: "b", ActorID: "c", Role: "d"}}}); !errors.Is(err, ErrAuthUnavailable) {
		t.Fatalf("missing issuer: %v", err)
	}
	if _, err := NewOIDC(context.Background(), OIDCOptions{Issuer: "https://i", ClientID: "x", ClientSecret: "y"}); !errors.Is(err, ErrAuthUnavailable) {
		t.Fatalf("missing mappings: %v", err)
	}
	// An unreachable issuer fails construction (fail closed at startup, not at first login).
	if _, err := NewOIDC(context.Background(), OIDCOptions{Issuer: "http://127.0.0.1:1/dne", ClientID: "x", ClientSecret: "y",
		Mappings: []OIDCMapping{{Claim: "a", Equals: "b", ActorID: "c", Role: "d"}}}); !errors.Is(err, ErrAuthUnavailable) {
		t.Fatalf("unreachable issuer must fail construction: %v", err)
	}
}
