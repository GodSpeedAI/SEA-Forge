// OIDC: the authorization-code flow authenticator (auth.mode=oidc), built on
// github.com/coreos/go-oidc/v3 + golang.org/x/oauth2 (the two operator-approved OIDC
// dependencies). Construction performs real issuer discovery; the callback exchanges the code at
// the token endpoint, verifies the ID token's signature/audience/expiry/nonce, and maps the
// VERIFIED claims onto kernel standing through configured rules (first match wins, no match
// refused - an unmapped identity must not fall back to a default standing).
//
// What this code path proves in tests (httptest provider): discovery, code exchange, ID-token
// signature/nonce verification and the claim->actor mapping. What it delegates to deployment: a
// real identity provider, real TLS, real browser redirects, and the operator's trust in the
// issuer.
package auth

import (
	"context"
	"crypto/subtle"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/coreos/go-oidc/v3/oidc"
	"golang.org/x/oauth2"
)

// OIDCMapping is one verified-claim -> kernel-standing rule (config auth.oidc.mappings).
type OIDCMapping struct {
	Claim   string
	Equals  string
	ActorID string
	Role    string
}

// OIDCOptions configure the flow. They mirror the config OIDCSection with values already
// resolved (secrets resolved by the config layer).
type OIDCOptions struct {
	Issuer        string
	ClientID      string
	ClientSecret  string
	RedirectURL   string
	Scopes        []string
	UsernameClaim string // default "preferred_username"
	Mappings      []OIDCMapping

	// HTTPClient, when set, is used for discovery, token exchange and JWKS fetches (tests point
	// it at an httptest provider; production leaves it nil for the default transport).
	HTTPClient *http.Client
	// Now and StateTTL tune the single-use state store (tests freeze the clock).
	Now      func() time.Time
	StateTTL time.Duration
}

// DefaultUsernameClaim is the ID-token claim providing the display username.
const DefaultUsernameClaim = "preferred_username"

// DefaultStateTTL bounds how long a minted login state may sit un-completed.
const DefaultStateTTL = 10 * time.Minute

// MaxPendingOIDCStates caps un-completed login states held in memory.
const MaxPendingOIDCStates = 4096

// OIDC implements Authenticator and OIDCFlow.
type OIDC struct {
	verifier      *oidc.IDTokenVerifier
	oauth         oauth2.Config
	usernameClaim string
	mappings      []OIDCMapping
	now           func() time.Time
	stateTTL      time.Duration

	mu         sync.Mutex
	states     map[string]pendingLogin
	httpClient *http.Client
}

type pendingLogin struct {
	nonce   string
	expires time.Time
}

// NewOIDC builds the authenticator: it runs issuer discovery immediately (a mis-declared issuer
// must fail at startup, not at first login - fail closed).
func NewOIDC(ctx context.Context, opts OIDCOptions) (*OIDC, error) {
	if strings.TrimSpace(opts.Issuer) == "" || opts.ClientID == "" || opts.ClientSecret == "" {
		return nil, fmt.Errorf("%w: oidc authenticator requires issuer, client_id and client_secret", ErrAuthUnavailable)
	}
	if len(opts.Mappings) == 0 {
		return nil, fmt.Errorf("%w: the oidc authenticator requires at least one claim->actor mapping", ErrAuthUnavailable)
	}
	callCtx := ctx
	if opts.HTTPClient != nil {
		callCtx = oidc.ClientContext(ctx, opts.HTTPClient)
	}
	provider, err := oidc.NewProvider(callCtx, opts.Issuer)
	if err != nil {
		return nil, fmt.Errorf("%w: issuer discovery for %q failed: %v", ErrAuthUnavailable, opts.Issuer, err)
	}
	scopes := append([]string{oidc.ScopeOpenID}, opts.Scopes...)
	o := &OIDC{
		verifier: provider.Verifier(&oidc.Config{ClientID: opts.ClientID}),
		oauth: oauth2.Config{
			ClientID:     opts.ClientID,
			ClientSecret: opts.ClientSecret,
			Endpoint:     provider.Endpoint(),
			RedirectURL:  opts.RedirectURL,
			Scopes:       dedupeScopes(scopes),
		},
		usernameClaim: opts.UsernameClaim,
		mappings:      opts.Mappings,
		now:           opts.Now,
		stateTTL:      opts.StateTTL,
		states:        map[string]pendingLogin{},
		httpClient:    opts.HTTPClient,
	}
	if o.usernameClaim == "" {
		o.usernameClaim = DefaultUsernameClaim
	}
	if o.now == nil {
		o.now = time.Now
	}
	if o.stateTTL <= 0 {
		o.stateTTL = DefaultStateTTL
	}
	return o, nil
}

// Mode implements Authenticator.
func (o *OIDC) Mode() string { return ModeOIDC }

// LoginURL implements OIDCFlow: it mints a single-use state (with a matching nonce) and returns
// the provider's authorization redirect for it.
func (o *OIDC) LoginURL(_ context.Context) (string, string, error) {
	state, err := randomToken()
	if err != nil {
		return "", "", fmt.Errorf("cannot mint oidc state: %w", err)
	}
	nonce, err := randomToken()
	if err != nil {
		return "", "", fmt.Errorf("cannot mint oidc nonce: %w", err)
	}
	o.mu.Lock()
	// Bounded: drop expired states while we hold the lock, and refuse (never evict, which would
	// let a flood cancel genuine pending logins) once the cap is reached. GET /api/auth/login is
	// unauthenticated, so without the cap it is an unbounded-memory primitive (T11 review F5).
	now := o.now()
	for s, p := range o.states {
		if now.After(p.expires) {
			delete(o.states, s)
		}
	}
	if len(o.states) >= MaxPendingOIDCStates {
		o.mu.Unlock()
		return "", "", fmt.Errorf("too many pending oidc logins; retry shortly")
	}
	o.states[state] = pendingLogin{nonce: nonce, expires: now.Add(o.stateTTL)}
	o.mu.Unlock()
	url := o.oauth.AuthCodeURL(state, oidcNonce(nonce))
	return state, url, nil
}

func oidcNonce(nonce string) oauth2.AuthCodeOption {
	return oauth2.SetAuthURLParam("nonce", nonce)
}

// Callback implements OIDCFlow: state validation (single use, TTL), code exchange, ID-token
// verification (signature, audience, expiry, nonce) and claim->standing mapping.
func (o *OIDC) Callback(ctx context.Context, state, code string) (Identity, error) {
	if state == "" || code == "" {
		return Identity{}, fmt.Errorf("%w: the callback requires state and code", ErrInvalidCredentials)
	}
	o.mu.Lock()
	pending, ok := o.states[state]
	if ok {
		delete(o.states, state) // single use: consumed whether or not the exchange succeeds
	}
	o.mu.Unlock()
	if !ok || o.now().After(pending.expires) {
		return Identity{}, fmt.Errorf("%w: unknown or expired login state", ErrInvalidCredentials)
	}

	// The http.Client (when configured) travels through the context for the token exchange too.
	exchangeCtx := ctx
	if o.httpClient != nil {
		exchangeCtx = oidc.ClientContext(ctx, o.httpClient)
	}
	token, err := o.oauth.Exchange(exchangeCtx, code)
	if err != nil {
		return Identity{}, fmt.Errorf("%w: the code exchange failed: %v", ErrInvalidCredentials, err)
	}
	rawID, ok := token.Extra("id_token").(string)
	if !ok || rawID == "" {
		return Identity{}, fmt.Errorf("%w: the token response carried no id_token", ErrInvalidCredentials)
	}
	idToken, err := o.verifier.Verify(ctx, rawID)
	if err != nil {
		return Identity{}, fmt.Errorf("%w: the id_token did not verify: %v", ErrInvalidCredentials, err)
	}
	// The nonce is REQUIRED (T11): an id_token without one could be replayed from another
	// login, so an absent nonce is refused exactly like a wrong one.
	if subtle.ConstantTimeCompare([]byte(idToken.Nonce), []byte(pending.nonce)) != 1 {
		return Identity{}, fmt.Errorf("%w: the id_token nonce does not match the login request", ErrInvalidCredentials)
	}
	var claims map[string]any
	if err := idToken.Claims(&claims); err != nil {
		return Identity{}, fmt.Errorf("%w: the id_token claims are unreadable: %v", ErrInvalidCredentials, err)
	}
	return MapClaimsToIdentity(claims, o.usernameClaim, o.mappings)
}

// MapClaimsToIdentity applies the mapping rules to verified claims: the first rule whose claim
// equals the configured value wins; string and []string claims are both honoured (groups-style
// arrays). No match is a refusal, never a default standing.
func MapClaimsToIdentity(claims map[string]any, usernameClaim string, mappings []OIDCMapping) (Identity, error) {
	if usernameClaim == "" {
		usernameClaim = DefaultUsernameClaim
	}
	for _, m := range mappings {
		if claimMatches(claims, m.Claim, m.Equals) {
			id := Identity{
				ActorID:  m.ActorID,
				Role:     m.Role,
				Username: stringClaim(claims, usernameClaim),
			}
			if id.Username == "" {
				id.Username = stringClaim(claims, "sub")
			}
			id.DisplayName = stringClaim(claims, "name")
			return id, nil
		}
	}
	return Identity{}, fmt.Errorf("%w: no mapping rule matched the verified claims", ErrNoActorMapping)
}

// claimMatches reports whether claims[claim] equals want, accepting scalar strings and arrays of
// strings (a groups claim must be able to trigger a rule on membership).
func claimMatches(claims map[string]any, claim, want string) bool {
	v, ok := claims[claim]
	if !ok {
		return false
	}
	switch t := v.(type) {
	case string:
		return t == want
	case []string:
		for _, s := range t {
			if s == want {
				return true
			}
		}
	case []any:
		for _, e := range t {
			if s, ok := e.(string); ok && s == want {
				return true
			}
		}
	case json.Number:
		return t.String() == want
	}
	return false
}

func stringClaim(claims map[string]any, name string) string {
	if s, ok := claims[name].(string); ok {
		return s
	}
	return ""
}

func dedupeScopes(scopes []string) []string {
	seen := map[string]bool{}
	out := make([]string, 0, len(scopes))
	for _, s := range scopes {
		if s == "" || seen[s] {
			continue
		}
		seen[s] = true
		out = append(out, s)
	}
	return out
}
