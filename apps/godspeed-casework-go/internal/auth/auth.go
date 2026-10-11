// Package auth is the gateway's browser-authentication layer (plan T07). It owns:
//
//   - the pluggable Authenticator front door (LocalUserStore for mode local/dev; OIDC for the
//     authorization-code flow),
//   - argon2id password hashing per OWASP's recommended parameters,
//   - the in-memory server-side session store (bounded TTL + absolute expiry; single-process by
//     design - a stateless-replica deployment moves sessions to shared storage per the plan's
//     redesign_trigger),
//   - the browser-user -> kernel-actor mapping: every authenticated identity names the kernel
//     actor and role it acts as through the T02 gateway delegation. The mapping comes from
//     configuration (config.AuthSection); the KERNEL re-verifies every delegation against its
//     own allowlist at request time, so a mis-mapped user is refused by the kernel, never
//     trusted by the gateway.
//
// Identity is never accepted from the client: a session's kernel standing is fixed at login and
// the gateway overwrites any client-asserted actor with the session's own (T07 contract; the
// pre-T07 ?actor=&role= dev surface is refused for authenticated requests).
package auth

import (
	"context"
	"errors"
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// Authentication modes, mirroring config.AuthSection (kept as literals here so the auth package
// does not depend on the config package's schema; main.go wires them together).
const (
	ModeLocal = "local"
	ModeOIDC  = "oidc"
	ModeDev   = "dev"
)

// Sentinel errors the server maps onto typed 401/403 responses.
var (
	// ErrInvalidCredentials: the username/password pair did not verify (401).
	ErrInvalidCredentials = errors.New("invalid credentials")
	// ErrNoActorMapping: the verified identity maps to no configured kernel standing (403).
	ErrNoActorMapping = errors.New("the verified identity maps to no kernel actor")
	// ErrAuthUnavailable: the authenticator cannot run (misconfiguration, provider unreachable).
	ErrAuthUnavailable = errors.New("the authenticator is unavailable")
)

// Identity is an authenticated browser user mapped to its kernel standing.
type Identity struct {
	Username    string `json:"username"`
	DisplayName string `json:"display_name,omitempty"`
	ActorID     string `json:"actor_id"` // kernel actor (T02 gateway-delegable allowlist member)
	Role        string `json:"role"`     // kernel role held by that actor
}

// Claim returns the identity's kernel actor claim: what the gateway sends as on_behalf_of and
// what /api/world renders its perspective for.
func (i Identity) Claim() ports.ActorClaim {
	return ports.ActorClaim{ActorID: i.ActorID, Role: i.Role}
}

// Authenticator is the pluggable login front door. Concrete implementations:
// LocalUserStore (mode local and mode dev) and OIDC (mode oidc).
type Authenticator interface {
	// Mode reports the configured authentication mode (local | oidc | dev).
	Mode() string
}

// PasswordLogin verifies a username/password pair (LocalUserStore in both local and dev mode).
type PasswordLogin interface {
	Login(ctx context.Context, username, password string) (Identity, error)
}

// OIDCFlow drives the authorization-code flow (OIDC only).
type OIDCFlow interface {
	// LoginURL mints a single-use state and returns the provider redirect URL for it.
	LoginURL(ctx context.Context) (state, url string, err error)
	// Callback completes the flow: it validates the returned state (single use, bounded TTL),
	// exchanges the code at the token endpoint, verifies the ID token and maps the verified
	// claims onto kernel standing.
	Callback(ctx context.Context, state, code string) (Identity, error)
}

// normalizeUsername trims and case-folds usernames so the store is case-insensitive without
// accepting padding whitespace.
func normalizeUsername(s string) string {
	return strings.ToLower(strings.TrimSpace(s))
}
