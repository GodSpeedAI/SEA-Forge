// Session/authentication glue for the HTTP surface (plan T07).
//
// Posture:
//   - /api/healthz and /api/readyz are public (health is cheap; readiness includes the kernel's
//     readiness.get through the SFWP client).
//   - The auth endpoints (login/logout/callback, /api/session) are the only way in.
//   - EVERY other /api route is protected: unauthenticated reads 401, unauthenticated or
//     session-authenticated state-changing POSTs without a valid CSRF token 403 BEFORE any
//     kernel call.
//   - Identity is fixed by the session: the pre-T07 ?actor=&role= override is refused for
//     authenticated requests, and an intent's client-asserted actor is OVERWRITTEN with the
//     session's kernel standing. A user can never act as another actor.
//   - auth mode dev additionally accepts Authorization: Bearer <static token> (resolved from the
//     config's env/file indirection). Bearer credentials are explicit per-request, not ambient,
//     so they are immune to CSRF and skip the CSRF check; every other protection applies. The
//     production posture refuses this mode at configuration validation.
//
// The session store is in-memory (single-process gateway, documented in internal/auth): a
// stateless-replica deployment moves sessions to shared storage (the plan's redesign_trigger).
package server

import (
	"context"
	"crypto/subtle"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// Login bounds (T11): field length and the number of concurrent argon2id verifications (each
// costs ~19 MiB).
const (
	maxLoginFieldBytes  = 1024
	maxConcurrentLogins = 8
)

// Cookie names. The session cookie is HttpOnly (never readable by JS); the CSRF cookie is
// deliberately readable (the synchronizer token's mirror) and SameSite=Strict like the session.
const (
	sessionCookieName = "casework_session"
	csrfCookieName    = "casework_csrf"
	csrfHeader        = "X-CSRF-Token"
)

// AuthOptions wire the authentication layer into the server. The zero value fails closed: every
// protected endpoint 401s and the login endpoints report "authentication is not configured".
type AuthOptions struct {
	// Authenticator is the pluggable login front door (auth.LocalUserStore for local/dev,
	// auth.OIDC for oidc). Nil disables the login endpoints.
	Authenticator auth.Authenticator
	// Sessions is the server-side session store. Nil disables session issuance (401s everywhere).
	Sessions *auth.SessionStore
	// CookieSecure sets the Secure attribute on both cookies. Production posture MUST set it
	// true (the config validation refuses insecure cookies in production); loopback plain-HTTP
	// dev may set it false.
	CookieSecure bool
	// StaticToken enables the DEV-ONLY bearer surface: when non-empty, "Authorization: Bearer
	// <value>" authenticates the request as BearerIdentity. Non-dev modes refuse the token at
	// configuration validation.
	StaticToken string
	// BearerIdentity is the identity a valid static token acts as (resolved at assembly from the
	// configured user).
	BearerIdentity *auth.Identity
}

// requestIdentity is the per-request authenticated standing: either a server-side session or a
// dev-mode bearer token.
type requestIdentity struct {
	Identity auth.Identity
	Session  *auth.Session // nil for bearer-authenticated requests
	Source   string        // "session" | "bearer"
}

// csrfRequired reports whether a state-changing request from this identity must present the
// session's synchronizer token. Bearer credentials are not ambient (the browser never attaches
// them automatically), so there is nothing for CSRF to defend against.
func (ri *requestIdentity) csrfRequired() bool { return ri != nil && ri.Session != nil }

type ctxKey struct{}

// identityFrom returns the authenticated identity attached to the request, if any.
func identityFrom(r *http.Request) *requestIdentity {
	ri, _ := r.Context().Value(ctxKey{}).(*requestIdentity)
	return ri
}

// requireSession resolves the request's identity and only then runs next. Unauthenticated
// requests get a typed 401 and NEVER reach the handler.
func (s *Server) requireSession(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ri, ok := s.resolveIdentity(w, r)
		if !ok {
			return // the typed 401 was already written
		}
		next(w, r.WithContext(context.WithValue(r.Context(), ctxKey{}, ri)))
	}
}

// withIdentity attaches whatever identity the request carries WITHOUT refusing anonymous
// callers. GET /api/session uses it: it must answer honestly for unauthenticated browsers too
// (the login page's bootstrap).
func (s *Server) withIdentity(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if ri, ok := s.tryResolveIdentity(r); ok {
			r = r.WithContext(context.WithValue(r.Context(), ctxKey{}, ri))
		} else {
			// Keep the request anonymous; the handler reports authenticated:false.
		}
		next(w, r)
	}
}

// resolveIdentity authenticates the request from the session cookie (or, in dev mode, the static
// bearer token). It also ensures an unauthenticated response carries a fresh pre-login CSRF
// cookie so the login POST can double-submit.
func (s *Server) resolveIdentity(w http.ResponseWriter, r *http.Request) (*requestIdentity, bool) {
	if ri, ok := s.tryResolveIdentity(r); ok {
		return ri, true
	}
	// Unauthenticated: typed 401, no snapshot, no kernel call. The pre-login CSRF cookie is
	// (re)issued so the login POST's double-submit has something to match.
	s.ensureCSRFCookie(w, r)
	writeTypedError(w, http.StatusUnauthorized, "unauthorized",
		"authentication is required: log in via POST /api/auth/login (or GET /api/auth/login in oidc mode)")
	return nil, false
}

// tryResolveIdentity is resolveIdentity without the refusal side effects.
func (s *Server) tryResolveIdentity(r *http.Request) (*requestIdentity, bool) {
	if c, err := r.Cookie(sessionCookieName); err == nil && s.opts.Auth.Sessions != nil {
		if sess, ok := s.opts.Auth.Sessions.Resolve(c.Value); ok {
			return &requestIdentity{Identity: sess.Identity, Session: sess, Source: "session"}, true
		}
	}
	if s.opts.Auth.StaticToken != "" && s.opts.Auth.BearerIdentity != nil {
		if h := r.Header.Get("Authorization"); strings.HasPrefix(h, "Bearer ") {
			token := strings.TrimSpace(strings.TrimPrefix(h, "Bearer "))
			if subtle.ConstantTimeCompare([]byte(token), []byte(s.opts.Auth.StaticToken)) == 1 {
				return &requestIdentity{Identity: *s.opts.Auth.BearerIdentity, Source: "bearer"}, true
			}
		}
	}
	return nil, false
}

// requireCSRF enforces the synchronizer token on state-changing POSTs from session identities.
// It runs BEFORE the handler: a valid cookie with a missing/invalid token is a 403 and the
// dispatcher (hence the kernel) is never called.
func (s *Server) requireCSRF(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ri := identityFrom(r)
		if ri.csrfRequired() {
			presented := r.Header.Get(csrfHeader)
			if presented == "" {
				writeTypedError(w, http.StatusForbidden, "csrf_refused",
					"a state-changing POST must carry the "+csrfHeader+" header with the session's CSRF token (GET /api/session issues it)")
				return
			}
			if subtle.ConstantTimeCompare([]byte(presented), []byte(ri.Session.CSRFToken)) != 1 {
				writeTypedError(w, http.StatusForbidden, "csrf_refused",
					"the CSRF token does not match this session; refetch /api/session")
				return
			}
		}
		next(w, r)
	}
}

// ensureCSRFCookie mints the pre-login double-submit cookie when the request carries none. The
// cookie is readable (the login page's JS echoes it back in the X-CSRF-Token header).
func (s *Server) ensureCSRFCookie(w http.ResponseWriter, r *http.Request) {
	if _, err := r.Cookie(csrfCookieName); err == nil {
		return
	}
	token, ok := newCSRFToken()
	if !ok {
		return // a broken CSPRNG fails the next login POST closed instead of panicking here
	}
	http.SetCookie(w, &http.Cookie{
		Name:     csrfCookieName,
		Value:    token,
		Path:     "/",
		HttpOnly: false,
		Secure:   s.opts.Auth.CookieSecure,
		SameSite: http.SameSiteStrictMode,
	})
}

// csrfCookieValue returns the double-submit cookie for the login POST.
func csrfCookieValue(r *http.Request) string {
	if c, err := r.Cookie(csrfCookieName); err == nil {
		return c.Value
	}
	return ""
}

// checkLoginCSRF enforces the pre-login double submit: the header must equal the cookie. A
// cross-site form post cannot read the cookie, so it cannot forge this pair.
func (s *Server) checkLoginCSRF(w http.ResponseWriter, r *http.Request) bool {
	cookie := csrfCookieValue(r)
	header := r.Header.Get(csrfHeader)
	if cookie == "" || header == "" || subtle.ConstantTimeCompare([]byte(cookie), []byte(header)) != 1 {
		writeTypedError(w, http.StatusForbidden, "csrf_refused",
			"login requires the "+csrfHeader+" header to match the "+csrfCookieName+" cookie issued by any GET (e.g. GET /api/session)")
		return false
	}
	return true
}

// setAuthCookies issues the session cookie (HttpOnly) and the matching CSRF cookie (readable)
// carrying the session's synchronizer token.
func (s *Server) setAuthCookies(w http.ResponseWriter, sess *auth.Session) {
	http.SetCookie(w, &http.Cookie{
		Name:     sessionCookieName,
		Value:    sess.ID,
		Path:     "/",
		HttpOnly: true,
		Secure:   s.opts.Auth.CookieSecure,
		SameSite: http.SameSiteStrictMode,
	})
	http.SetCookie(w, &http.Cookie{
		Name:     csrfCookieName,
		Value:    sess.CSRFToken,
		Path:     "/",
		HttpOnly: false,
		Secure:   s.opts.Auth.CookieSecure,
		SameSite: http.SameSiteStrictMode,
	})
}

// clearAuthCookies expires both cookies (logout).
func (s *Server) clearAuthCookies(w http.ResponseWriter) {
	for _, name := range []string{sessionCookieName, csrfCookieName} {
		http.SetCookie(w, &http.Cookie{
			Name:     name,
			Value:    "",
			Path:     "/",
			MaxAge:   -1,
			HttpOnly: name == sessionCookieName,
			Secure:   s.opts.Auth.CookieSecure,
			SameSite: http.SameSiteStrictMode,
		})
	}
}

// --- auth endpoints ---

// sessionBody is GET /api/session's answer: who the browser is and what it can act as. For an
// unauthenticated browser it reports authenticated=false and (re)issues the pre-login CSRF
// cookie, so the login flow can bootstrap.
type sessionBody struct {
	Authenticated bool   `json:"authenticated"`
	Username      string `json:"username,omitempty"`
	DisplayName   string `json:"display_name,omitempty"`
	ActorID       string `json:"actor_id,omitempty"`
	Role          string `json:"role,omitempty"`
	CSRFToken     string `json:"csrf_token,omitempty"`
	Mode          string `json:"auth_mode,omitempty"`
}

func (s *Server) handleSession(w http.ResponseWriter, r *http.Request) {
	if ri := identityFrom(r); ri != nil {
		body := sessionBody{
			Authenticated: true,
			Username:      ri.Identity.Username,
			DisplayName:   ri.Identity.DisplayName,
			ActorID:       ri.Identity.ActorID,
			Role:          ri.Identity.Role,
			Mode:          s.authMode(),
		}
		if ri.Session != nil {
			body.CSRFToken = ri.Session.CSRFToken
		}
		writeJSON(w, http.StatusOK, body)
		return
	}
	s.ensureCSRFCookie(w, r)
	writeJSON(w, http.StatusOK, sessionBody{Authenticated: false, Mode: s.authMode()})
}

func (s *Server) authMode() string {
	if s.opts.Auth.Authenticator == nil {
		return ""
	}
	return s.opts.Auth.Authenticator.Mode()
}

// handleLogin is POST /api/auth/login for the credential modes (local, dev). In oidc mode it
// redirects the client to the GET flow with a typed note.
func (s *Server) handleLogin(w http.ResponseWriter, r *http.Request) {
	if s.opts.Auth.Authenticator == nil || s.opts.Auth.Sessions == nil {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "authentication is not configured on this gateway")
		return
	}
	if s.authMode() == auth.ModeOIDC {
		writeTypedError(w, http.StatusBadRequest, "invalid",
			"this gateway authenticates via oidc: start the flow at GET /api/auth/login")
		return
	}
	login, ok := s.opts.Auth.Authenticator.(auth.PasswordLogin)
	if !ok {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "the configured authenticator does not accept passwords")
		return
	}
	if !s.checkLoginCSRF(w, r) {
		return
	}
	if !requireJSON(w, r) {
		return
	}
	var creds struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}
	if !decodeStrict(w, r, &creds) {
		return
	}
	if len(creds.Username) > maxLoginFieldBytes || len(creds.Password) > maxLoginFieldBytes {
		writeTypedError(w, http.StatusBadRequest, "invalid", "username or password is too long")
		return
	}
	ipKey := "ip:" + remoteIP(r)
	userKey := "user:" + strings.ToLower(strings.TrimSpace(creds.Username))
	if s.loginFailIP.Blocked(ipKey) || s.loginFailUser.Blocked(userKey) {
		s.writeRateLimited(w, "too many failed logins; retry after the refill window")
		return
	}
	select {
	case s.loginSlots <- struct{}{}:
		defer func() { <-s.loginSlots }()
	default:
		s.writeRateLimited(w, "too many logins are being verified; retry shortly")
		return
	}
	identity, err := login.Login(r.Context(), creds.Username, creds.Password)
	if err != nil {
		if errors.Is(err, auth.ErrInvalidCredentials) {
			s.loginFailIP.Allow(ipKey)
			s.loginFailUser.Allow(userKey)
			writeTypedError(w, http.StatusUnauthorized, "invalid_credentials", "the username or password did not verify")
			return
		}
		s.logf("credential check failed: %v", err)
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the credential check failed")
		return
	}
	sess := s.opts.Auth.Sessions.Start(identity)
	s.setAuthCookies(w, sess)
	writeJSON(w, http.StatusOK, sessionBody{
		Authenticated: true,
		Username:      identity.Username,
		DisplayName:   identity.DisplayName,
		ActorID:       identity.ActorID,
		Role:          identity.Role,
		CSRFToken:     sess.CSRFToken,
		Mode:          s.authMode(),
	})
}

// handleLoginStart is GET /api/auth/login: the oidc redirect (a no-op for credential modes,
// which authenticate by POST).
func (s *Server) handleLoginStart(w http.ResponseWriter, r *http.Request) {
	flow, ok := s.opts.Auth.Authenticator.(auth.OIDCFlow)
	if s.opts.Auth.Authenticator == nil || !ok {
		writeTypedError(w, http.StatusMethodNotAllowed, "invalid",
			"this gateway authenticates with credentials: POST /api/auth/login {username, password}")
		return
	}
	if !s.loginStartIP.Allow("ip:" + remoteIP(r)) {
		s.writeRateLimited(w, "too many login starts; retry after the refill window")
		return
	}
	state, url, err := flow.LoginURL(r.Context())
	if err != nil {
		s.logf("oidc login start failed: %v", err)
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the identity provider could not start the login")
		return
	}
	http.Redirect(w, r, url, http.StatusFound)
	_ = state // the flow keeps the single-use state; the callback validates it
}

// handleLoginCallback is GET /api/auth/callback: completes the oidc authorization-code flow and
// issues the session.
func (s *Server) handleLoginCallback(w http.ResponseWriter, r *http.Request) {
	flow, ok := s.opts.Auth.Authenticator.(auth.OIDCFlow)
	if s.opts.Auth.Authenticator == nil || !ok {
		writeTypedError(w, http.StatusNotFound, "invalid", "no oidc callback is configured on this gateway")
		return
	}
	identity, err := flow.Callback(r.Context(), r.URL.Query().Get("state"), r.URL.Query().Get("code"))
	if err != nil {
		s.logf("oidc callback refused: %v", err)
		writeTypedError(w, http.StatusUnauthorized, "invalid_credentials", "the login did not complete")
		return
	}
	sess := s.opts.Auth.Sessions.Start(identity)
	s.setAuthCookies(w, sess)
	http.Redirect(w, r, "/", http.StatusSeeOther)
}

// handleLogout is POST /api/auth/logout: session + CSRF required (it is state-changing), then
// the session is destroyed server-side and both cookies cleared.
func (s *Server) handleLogout(w http.ResponseWriter, r *http.Request) {
	ri := identityFrom(r)
	if ri == nil || ri.Session == nil {
		writeTypedError(w, http.StatusUnauthorized, "unauthorized", "no live session to log out")
		return
	}
	s.opts.Auth.Sessions.Destroy(ri.Session.ID)
	s.clearAuthCookies(w)
	writeJSON(w, http.StatusOK, map[string]bool{"logged_out": true})
}

// sessionIdentityOf returns the identity a protected handler must act as (already attached to
// the request context by requireSession).
func sessionIdentityOf(r *http.Request) auth.Identity {
	if ri := identityFrom(r); ri != nil {
		return ri.Identity
	}
	return auth.Identity{}
}

// intentActorFromSession overwrites the intent's client-asserted actor with the session's
// kernel standing: identity is session-bound, never client-settable (T07).
func intentActorFromSession(r *http.Request) contract.IntentActor {
	id := sessionIdentityOf(r)
	return contract.IntentActor{ActorID: id.ActorID, Role: id.Role}
}

// ReadinessProbe is the kernel readiness surface /api/readyz reports (ports.CaseAuthorityPort
// satisfies it: readiness.get through the SFWP client).
type ReadinessProbe interface {
	Readiness(ctx context.Context) (ports.ReadinessReport, error)
}

func (s *Server) handleReadyz(w http.ResponseWriter, r *http.Request) {
	if s.opts.Ready == nil {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "no kernel readiness probe is wired into this gateway")
		return
	}
	ctx, cancel := context.WithTimeout(r.Context(), 3*time.Second)
	defer cancel()
	report, err := s.opts.Ready.Readiness(ctx)
	if err != nil {
		// Unauthenticated endpoint: the raw error (socket paths, internals) stays in the log (T11 review F6).
		s.logf("readyz: kernel probe failed: %v", err)
		writeJSON(w, http.StatusServiceUnavailable, map[string]any{
			"status": "not_ready",
			"kernel": map[string]string{"error": "the kernel readiness probe failed"},
		})
		return
	}
	// The verdict semantics are the kernel's own (OverallReadiness): ready and
	// ready_with_limitations serve; "unknown" is a fresh cell with no committed source standing
	// yet (serviceable - the live ladder runs on one); blocked / integrity_halted / stale do not
	// serve. The verdict rides verbatim either way.
	ready := true
	switch strings.ToLower(report.Overall) {
	case "", "ready", "ready_with_limitations", "readywithlimitations", "unknown":
	default:
		ready = false
	}
	status := "ready"
	httpStatus := http.StatusOK
	if !ready {
		status = "not_ready"
		httpStatus = http.StatusServiceUnavailable
	}
	writeJSON(w, httpStatus, map[string]any{
		"status": status,
		"kernel": map[string]any{"overall": report.Overall, "foundations": len(report.Foundations), "capabilities": len(report.Capabilities)},
	})
}

// logf writes a diagnostic line to the request logger (no-op when logging is disabled). Callers
// must never pass secrets: it exists so error detail stays server-side instead of in responses.
func (s *Server) logf(format string, args ...any) {
	if s.opts.Logger != nil {
		s.opts.Logger.Printf(format, args...)
	}
}
