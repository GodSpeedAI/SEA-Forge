// Auth and production-posture configuration (plan T07). This file owns the `auth` section
// (browser authentication: mode, local users, dev static token, OIDC), the `serve` section's
// production posture, and the fail-closed validation matrix that refuses:
//
//   - production posture with auth.mode=dev (the dev static-token/unchecked-password surface
//     must never exist on a production gateway),
//   - production posture with auth.insecure_cookie (cookies must carry Secure in production),
//   - a static bearer token in any non-dev mode (a static token is a dev backdoor by definition),
//   - a serve posture with no explicit, internally consistent auth section.
//
// Secrets (auth.static_token, auth.oidc.client_secret) follow REQ-CONFIG-002: only `env:NAME` /
// `file:/path` indirections are accepted, and only resolved VALUES reach the caller (via Load's
// secret list), never the configuration itself.
package config

import (
	"fmt"
	"net/url"
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// Authentication modes (auth.mode). The empty mode is NOT valid in a serve posture: a gateway
// that cannot say how its users log in must refuse to start (fail closed).
const (
	AuthModeLocal = "local" // users from config, argon2id password hashes
	AuthModeOIDC  = "oidc"  // authorization-code flow against the configured issuer
	AuthModeDev   = "dev"   // test/dev only: passwords unchecked + optional static bearer token
)

// AuthMode describes the configured authentication surface.
type AuthMode string

// AuthUser is one local user: browser credentials plus the kernel standing the session maps to
// (the kernel actor + role from the T02 gateway-delegable allowlist; the kernel re-verifies every
// delegation, so a user mapped outside the allowlist simply cannot act).
type AuthUser struct {
	Username     string `json:"username"`
	DisplayName  string `json:"display_name,omitempty"`
	PasswordHash string `json:"password_hash,omitempty"` // argon2id PHC string; required in local mode, ignored in dev mode
	ActorID      string `json:"actor_id"`                // kernel actor (T02 allowlist member)
	Role         string `json:"role"`                    // kernel role held by that actor's binding
}

// OIDCMappingRule maps a verified ID-token claim onto kernel standing. Rules are evaluated in
// order; the first matching rule wins; no match refuses the login (fail closed - an unmapped
// user must not fall back to a default standing).
type OIDCMappingRule struct {
	Claim   string `json:"claim"`  // e.g. "preferred_username", "email", "groups"
	Equals  string `json:"equals"` // the claim value that triggers this rule
	ActorID string `json:"actor_id"`
	Role    string `json:"role"`
}

// OIDCSection configures the authorization-code flow. ClientSecret is an indirection, never a
// value. RedirectURL is the gateway's own callback (e.g. https://host/api/auth/callback).
type OIDCSection struct {
	Issuer        string            `json:"issuer"`
	ClientID      string            `json:"client_id"`
	ClientSecret  string            `json:"client_secret"` // indirection (env:NAME / file:/path)
	RedirectURL   string            `json:"redirect_url"`
	Scopes        []string          `json:"scopes,omitempty"`         // default: openid profile (openid is always added)
	UsernameClaim string            `json:"username_claim,omitempty"` // default "preferred_username"
	Mappings      []OIDCMappingRule `json:"mappings"`
}

// AuthSection is the `auth` configuration block.
type AuthSection struct {
	Mode  string     `json:"mode"`
	Users []AuthUser `json:"users,omitempty"`
	// StaticToken is the dev-mode bearer token (indirection). Any non-dev mode with a static
	// token is refused at validation: it is a dev convenience, not a production factor.
	StaticToken string `json:"static_token,omitempty"`
	// StaticTokenUser names the configured user a valid static token acts as. Default: the first
	// configured user. Dev mode only.
	StaticTokenUser string `json:"static_token_user,omitempty"`
	// InsecureCookie drops the Secure flag from the session/CSRF cookies for plain-HTTP loopback
	// development. Refused in production posture. (Browsers treat loopback http origins as
	// trustworthy, so Secure cookies also work there; this switch exists for non-loopback
	// plain-HTTP dev cells.)
	InsecureCookie bool `json:"insecure_cookie,omitempty"`
	// Session lifetimes. IdleTTLMinutes is the sliding idle timeout (default 30); AbsoluteTTLHours
	// is the hard upper bound on a session's life (default 12). The store is in-memory and
	// single-process: a stateless-replica deployment moves sessions to shared storage (the plan's
	// redesign_trigger).
	IdleTTLMinutes   int          `json:"session_idle_minutes,omitempty"`
	AbsoluteTTLHours int          `json:"session_absolute_hours,omitempty"`
	OIDC             *OIDCSection `json:"oidc,omitempty"`
}

// RateLimitSection bounds /api/intents: a token bucket per session AND per remote IP. Defaults:
// 60 intents/minute, burst 20 - generous for a single operator, tight enough to blunt a runaway
// client before it becomes kernel load.
type RateLimitSection struct {
	IntentsPerMinute int `json:"intents_per_minute,omitempty"`
	Burst            int `json:"burst,omitempty"`
}

// ServeProduction is the production-posture switch (serve.production=true): it refuses the dev
// auth surface and insecure cookies, and documents that the gateway expects TLS termination and
// a trusted reverse proxy in front of it (see configs/README or the config reference).
const ServeProduction = "production"

// authDefaults fills the documented defaults of an auth section. It does NOT invent a mode: a
// serve posture without an explicit mode is a validation fault (fail closed), while non-serve
// postures (preflight, the fixture stack) simply never read this section.
func authDefaults(section *AuthSection) AuthSection {
	out := AuthSection{}
	if section != nil {
		out = *section
	}
	if out.IdleTTLMinutes <= 0 {
		out.IdleTTLMinutes = 30
	}
	if out.AbsoluteTTLHours <= 0 {
		out.AbsoluteTTLHours = 12
	}
	return out
}

// rateLimitDefaults fills the documented rate-limit defaults.
func rateLimitDefaults(section *RateLimitSection) RateLimitSection {
	out := RateLimitSection{}
	if section != nil {
		out = *section
	}
	if out.IntentsPerMinute <= 0 {
		out.IntentsPerMinute = 60
	}
	if out.Burst <= 0 {
		out.Burst = 20
	}
	return out
}

// AuthOrDefaults returns the document's auth section with the documented defaults applied.
// Callers never read Document.Auth directly (a nil section has no meaning of its own).
func (d Document) AuthOrDefaults() AuthSection {
	return authDefaults(d.Auth)
}

// validateAuth enforces the auth matrix for a serve posture (doc.Serve != nil). Every problem it
// returns is document-level (no capability attribution) and therefore fatal: an authentication
// configuration that does not hold together must stop the process, not degrade one capability.
func validateAuth(doc Document, serve ServeSection) []*apperr.Error {
	var problems []*apperr.Error
	auth := authDefaults(doc.Auth)
	fail := func(format string, args ...any) {
		problems = append(problems, apperr.New(apperr.KindConfig, "", "auth",
			fmt.Sprintf(format, args...)))
	}

	if strings.TrimSpace(auth.Mode) == "" {
		fail("serve posture requires an explicit auth.mode (local, oidc, or dev); a gateway that " +
			"cannot say how its users log in refuses to start")
		return problems
	}
	if len(auth.Users) == 0 {
		fail("auth.mode %q requires at least one configured user (the browser user -> kernel actor mapping)", auth.Mode)
		return problems
	}

	seen := map[string]bool{}
	for _, u := range auth.Users {
		if strings.TrimSpace(u.Username) == "" {
			fail("an auth user has no username")
			continue
		}
		if seen[u.Username] {
			fail("duplicate auth user %q", u.Username)
		}
		seen[u.Username] = true
		if strings.TrimSpace(u.ActorID) == "" || strings.TrimSpace(u.Role) == "" {
			fail("auth user %q has no kernel standing (actor_id and role are required)", u.Username)
		}
		if auth.Mode == AuthModeLocal && strings.TrimSpace(u.PasswordHash) == "" {
			fail("auth user %q has no password_hash (local mode verifies argon2id hashes; dev mode is the only mode that skips them)", u.Username)
		}
	}

	switch auth.Mode {
	case AuthModeLocal:
		// Hash formats are verified by the auth package at assembly; here only presence and
		// basic spelling are checked so the failure is a config fault, not a runtime panic.
		for _, u := range auth.Users {
			if u.PasswordHash != "" && !strings.HasPrefix(u.PasswordHash, "$argon2id$") {
				fail("auth user %q password_hash is not an argon2id PHC string ($argon2id$...)", u.Username)
			}
		}
	case AuthModeOIDC:
		if auth.OIDC == nil {
			fail("auth.mode oidc requires the auth.oidc section (issuer, client_id, client_secret, redirect_url, mappings)")
			return problems
		}
		o := auth.OIDC
		if strings.TrimSpace(o.Issuer) == "" || strings.TrimSpace(o.ClientID) == "" ||
			strings.TrimSpace(o.ClientSecret) == "" || strings.TrimSpace(o.RedirectURL) == "" {
			fail("auth.oidc requires issuer, client_id, client_secret and redirect_url")
		}
		if !isIndirection(o.ClientSecret) {
			fail("auth.oidc.client_secret must be an indirection (env:NAME or file:/path); a bare value is not accepted")
		}
		if len(o.Mappings) == 0 {
			fail("auth.oidc.mappings is empty: a verified identity with no mapping to a kernel actor must be refused, so at least one rule is required")
		}
		for i, m := range o.Mappings {
			if strings.TrimSpace(m.Claim) == "" || strings.TrimSpace(m.Equals) == "" ||
				strings.TrimSpace(m.ActorID) == "" || strings.TrimSpace(m.Role) == "" {
				fail("auth.oidc.mappings[%d] is incomplete (claim, equals, actor_id, role are all required)", i)
			}
		}
	case AuthModeDev:
		if serve.Production {
			fail("auth.mode dev is REFUSED in the production posture: the dev surface skips password " +
				"verification and may carry a static bearer token; configure auth.mode local or oidc for production")
		}
	default:
		fail("unknown auth.mode %q (known: local, oidc, dev)", auth.Mode)
	}

	if auth.StaticToken != "" {
		if auth.Mode != AuthModeDev {
			fail("auth.static_token is only valid in auth.mode dev (a static bearer token in mode %q is refused)", auth.Mode)
		}
		if !isIndirection(auth.StaticToken) {
			fail("auth.static_token must be an indirection (env:NAME or file:/path); a bare value is not accepted")
		}
	}
	if auth.StaticTokenUser != "" && !seen[auth.StaticTokenUser] {
		fail("auth.static_token_user %q is not a configured user", auth.StaticTokenUser)
	}
	if auth.InsecureCookie && serve.Production {
		fail("auth.insecure_cookie is REFUSED in the production posture: session cookies must carry Secure " +
			"(terminate TLS at the reverse proxy; loopback plain-HTTP dev is the only consumer of this switch)")
	}
	return problems
}

// isIndirection reports whether the value follows the REQ-CONFIG-002 indirection grammar.
func isIndirection(ref string) bool {
	scheme, rest, ok := strings.Cut(ref, ":")
	if !ok || rest == "" {
		return false
	}
	return scheme == "env" || scheme == "file"
}

// validateServe covers the serve section's production posture and the rate limits.
func validateServe(doc Document) []*apperr.Error {
	var problems []*apperr.Error
	if doc.Serve == nil {
		return nil
	}
	serve := ServeDefaults(doc.Serve)
	rl := rateLimitDefaults(serve.RateLimit)
	if rl.IntentsPerMinute <= 0 || rl.Burst <= 0 {
		problems = append(problems, apperr.New(apperr.KindConfig, "", "serve",
			"serve.rate_limit values must be positive"))
	}
	if serve.ExecutionTimeoutSec < 1 || serve.ExecutionTimeoutSec > MaxExecutionTimeoutSec {
		problems = append(problems, apperr.New(apperr.KindConfig, "", "serve",
			fmt.Sprintf("serve.execution_timeout_sec must be between 1 and %d", MaxExecutionTimeoutSec)))
	}
	seenOrigins := make(map[string]bool, len(serve.TrustedOrigins))
	for _, origin := range serve.TrustedOrigins {
		if err := validateTrustedOrigin(origin, serve.Production); err != nil {
			problems = append(problems, apperr.New(apperr.KindConfig, "", "serve",
				fmt.Sprintf("serve.trusted_origins contains invalid origin %q: %v", origin, err)))
			continue
		}
		if seenOrigins[origin] {
			problems = append(problems, apperr.New(apperr.KindConfig, "", "serve",
				fmt.Sprintf("serve.trusted_origins contains duplicate origin %q", origin)))
		}
		seenOrigins[origin] = true
	}
	return problems
}

// validateTrustedOrigin accepts one scheme-and-authority Origin. A non-loopback HTTP origin is
// permitted only for local development; production origins must use HTTPS.
func validateTrustedOrigin(origin string, production bool) error {
	if origin == "" || strings.TrimSpace(origin) != origin || strings.ContainsAny(origin, "\\\r\n\t ") {
		return fmt.Errorf("must be a non-empty origin without whitespace")
	}
	u, err := url.Parse(origin)
	if err != nil {
		return fmt.Errorf("must be a valid absolute URL")
	}
	if u.Opaque != "" || u.User != nil || u.Host == "" || u.Hostname() == "" || u.Path != "" || u.RawPath != "" || u.RawQuery != "" || u.ForceQuery || u.Fragment != "" {
		return fmt.Errorf("must contain only a scheme and host, without userinfo, path, query, or fragment")
	}
	if strings.ContainsAny(u.Host, "%\\*") || u.Scheme != strings.ToLower(u.Scheme) || u.Host != strings.ToLower(u.Host) {
		return fmt.Errorf("must use a plain canonical scheme and host")
	}
	// Parse validates port syntax as part of the URL authority.
	_ = u.Port()
	if u.Scheme != "http" && u.Scheme != "https" {
		return fmt.Errorf("scheme must be http or https")
	}
	host := strings.ToLower(u.Hostname())
	loopback := host == "localhost" || host == "127.0.0.1" || host == "::1"
	if u.Scheme == "http" && (production || !loopback) {
		return fmt.Errorf("HTTP is allowed only for loopback origins in development; production requires HTTPS")
	}
	return nil
}

// resolveAuthSecrets resolves the auth section's indirections into the caller-held secret list.
// Problems are document-level (fatal): a serve posture whose configured secret cannot be resolved
// must not start.
func resolveAuthSecrets(doc Document, env func(string) (string, bool)) ([]Secret, []*apperr.Error) {
	auth := authDefaults(doc.Auth)
	var secrets []Secret
	var problems []*apperr.Error
	add := func(name, ref string) {
		if ref == "" {
			return
		}
		value, err := resolveSecret(name, ref, env)
		if err != nil {
			// Re-attribute to the document level: auth secrets are fatal, and the note names the
			// exact key so the operator can fix the right one.
			note := err.Message
			problems = append(problems, apperr.New(apperr.KindConfig, "", "auth", note))
			return
		}
		secrets = append(secrets, Secret{Capability: name, Value: value})
	}
	add("auth.static_token", auth.StaticToken)
	if auth.OIDC != nil {
		add("auth.oidc.client_secret", auth.OIDC.ClientSecret)
	}
	return secrets, problems
}
