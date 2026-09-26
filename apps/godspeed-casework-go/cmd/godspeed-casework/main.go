// Command godspeed-casework is the casework front end's entrypoint.
//
// Without -serve it does one thing end to end: load its single configuration authority, resolve
// secret indirections, run preflight, and report each configured capability's state with a typed
// error. It deliberately does NOT start any transport or accept work in that mode.
//
// With -serve it runs the same preflight first, then serves the LIVE cognitive projection over
// HTTP+SSE (T06): canonical spec-04 snapshots built from the governed kernel through the T05 SFWP
// client, intents translated one-to-one onto governed kernel verbs with the effective actor sent
// as on_behalf_of (T02), and kernel event frames relayed as SSE revisions keyed by the kernel's
// own event cursor. The served provenance is "go:live:sfwp" - nothing here is fixture-backed.
// A capability that selects adapter=sfwp is probed by preflight through the real SFWP client
// (system.hello, readiness.get, identity.get), and the fixture provider never stands in for it.
// The dev/demo fixture stack exists only behind -tags casework_fixture (see main_fixture.go).
package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/config"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/intents"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/preflight"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server"
)

// exitBlocked is used when a REQUIRED capability is unusable. It is a distinct code so an operator (or
// a calling script) never has to read the message to know the application refused to proceed.
const exitBlocked = 2

// fixtureProvider is the stand-in health prober registered in serve mode for capabilities that
// select the (build-tag-gated) in-process fixture stack. It exists ONLY in the tagged build; the
// production build refuses fixture selections at configuration validation.
type fixtureProvider struct{}

// Health implements ports.Health: the fixture provider lives in-process and cannot be unreachable.
func (fixtureProvider) Health(ctx context.Context) error { return nil }

func main() {
	configPath := flag.String("config", os.Getenv("GODSPEED_CONFIG"), "path to the configuration file")
	serve := flag.Bool("serve", false, "after preflight, serve the live cognitive projection API over HTTP+SSE")
	addr := flag.String("addr", "127.0.0.1:4179", "listen address in -serve mode (loopback by default)")
	flag.Parse()

	ctx := context.Background()
	resolved, secrets, problems := config.Load(*configPath, config.Options{})
	if fatal := config.Fatal(problems); len(fatal) > 0 {
		for _, p := range fatal {
			report(p)
		}
		os.Exit(exitBlocked)
	}
	// Secret values are held here and passed only to the adapters that need them; they are never part
	// of `resolved`, so they cannot be logged or serialised with the configuration by accident.
	fmt.Printf("godspeed-casework: configuration loaded (version %s, cell_root %q, evidence_root %q, %d capability/ies, %d resolved secret(s))\n",
		resolved.Version, resolved.CellRoot, resolved.EvidenceRoot, len(resolved.Capabilities), len(secrets))

	caps := make([]preflight.Capability, 0, len(resolved.Capabilities))
	for _, c := range resolved.Capabilities {
		caps = append(caps, preflight.Capability{Name: c.Name, Kind: c.Kind, Required: c.Required})
	}

	// Live adapters are built for every capability that selects adapter=sfwp,
	// in BOTH modes: preflight must probe the real governed authority (Health
	// negotiates system.hello, reads readiness.get, and resolves identity.get)
	// whenever the configuration selects it - never a fixture standing in for
	// it. The endpoint is the kernel's Unix socket (a "unix://" prefix is
	// accepted and stripped); construction can only fail on an empty socket
	// path, which validation has already reported as that capability's own
	// problem, so the prober is then simply not registered and preflight
	// carries the typed config fault.
	liveClients := map[string]*sfwp.Client{}
	liveAuthorities := map[string]*sfwp.Authority{}
	probers := map[string]ports.Health{}
	for _, c := range resolved.Capabilities {
		if c.Adapter != config.AdapterLive {
			continue
		}
		client, err := sfwp.New(sfwp.Config{SocketPath: strings.TrimPrefix(c.Endpoint, "unix://")})
		if err != nil {
			continue
		}
		liveClients[c.Name] = client
		liveAuthorities[c.Name] = sfwp.NewAuthority(client)
		probers[c.Name] = liveAuthorities[c.Name]
	}
	defer func() {
		for _, client := range liveClients {
			client.Close()
		}
	}()
	if *serve {
		// Serve mode registers the build-tag-gated in-process fixture provider
		// for the remaining capabilities, so preflight sees them ready rather
		// than "no adapter registered". It compiles only with
		// -tags casework_fixture; the production build reports such capabilities
		// as unconfigured instead (and validation refuses the selection outright).
		if fixtureStackAvailable {
			for _, c := range resolved.Capabilities {
				if _, live := liveClients[c.Name]; !live {
					probers[c.Name] = fixtureProvider{}
				}
			}
		}
	}

	// Capability-scoped problems travel into preflight, so a fault disables only the capability it
	// belongs to and every other capability still reports its own honest state.
	results := preflight.Check(ctx, caps, probers, problems)
	for _, r := range results {
		switch r.State {
		case preflight.StateReady:
			fmt.Printf("  ready     %-14s kind=%s required=%t adapter=%s\n", r.Capability.Name, r.Capability.Kind, r.Capability.Required, adapterOf(resolved, r.Capability.Name))
		default:
			fmt.Printf("  %-9s %-14s kind=%s required=%t err=%v\n",
				r.State, r.Capability.Name, r.Capability.Kind, r.Capability.Required, r.Err)
		}
	}
	summary := preflight.Summarise(results)
	fmt.Printf("godspeed-casework: ready=%v degraded=%v blocking=%v\n", summary.Ready, summary.Degraded, summary.Blocking)
	if !summary.Proceed {
		fmt.Printf("godspeed-casework: refusing to proceed - %d required capability/ies blocking (%s)\n",
			len(summary.Blocking), strings.Join(summary.Blocking, ", "))
		os.Exit(exitBlocked)
	}
	fmt.Println("godspeed-casework: preflight clear (no required capability blocking)")

	if *serve {
		authorityName, authority, err := liveAuthorityForServe(resolved, results, liveAuthorities)
		if err != nil {
			report(err)
			os.Exit(exitBlocked)
		}
		if authority == nil {
			// No live authority: the only legal way here is the tagged fixture stack.
			if !fixtureStackAvailable {
				report(apperr.New(apperr.KindConfig, "", "serve",
					"live serve requires an authority capability selecting adapter=sfwp (the fixture stack builds with -tags casework_fixture)"))
				os.Exit(exitBlocked)
			}
			if err := serveFixtureStack(ctx, *addr, resolved); err != nil {
				report(apperr.Wrap(apperr.KindInternal, authorityName, "serve", "fixture stack stopped", err))
				os.Exit(1)
			}
			return
		}
		secretValues := map[string]string{}
		for _, s := range secrets {
			secretValues[s.Capability] = s.Value
		}
		if err := serveLive(ctx, *addr, resolved, authority, liveClients[authorityName], secretValues); err != nil {
			report(apperr.Wrap(apperr.KindInternal, authorityName, "serve", "server stopped", err))
			os.Exit(1)
		}
	}
}

// liveAuthorityForServe resolves the capability that backs live serve: an authority-kind
// capability selecting adapter=sfwp whose preflight state is ready. Anything else is a typed
// refusal - the gateway never serves a fixture world as if it were governed truth.
func liveAuthorityForServe(resolved config.Resolved, results []preflight.Result, liveAuthorities map[string]*sfwp.Authority) (string, *sfwp.Authority, error) {
	for _, c := range resolved.Capabilities {
		if c.Kind != config.KindAuthority || c.Adapter != config.AdapterLive {
			continue
		}
		for _, r := range results {
			if r.Capability.Name != c.Name {
				continue
			}
			if r.State != preflight.StateReady {
				return c.Name, nil, apperr.New(apperr.KindUnavailable, c.Name, "serve",
					"the live gateway refuses to serve: the governed authority capability did not pass preflight")
			}
			return c.Name, liveAuthorities[r.Capability.Name], nil
		}
	}
	return "", nil, nil
}

func adapterOf(resolved config.Resolved, name string) string {
	for _, c := range resolved.Capabilities {
		if c.Name == name && c.Adapter != "" {
			return c.Adapter
		}
	}
	return "in-process"
}

// serveLive assembles the T06 live stack with the T07 session layer and blocks until SIGINT or
// SIGTERM, then shuts down gracefully:
//
//	sfwp.Client (Unix-socket NDJSON, T05)
//	  -> sfwp.Authority (ports.CaseAuthorityPort)
//	     -> projection.LiveSource (fetches case views, builds spec-04 snapshots)
//	     -> server.Relay (subscribes to kernel frames; per-case cursors; revision store)
//	     -> intents.Handler (T01 intent table; on_behalf_of; typed refusals)
//	  -> server.Server (HTTP+SSE surface; T07: sessions, CSRF, static UI, rate limits, readyz)
//
// The authenticator is built from the config's auth section (mode local/dev/oidc) with its
// secrets resolved from the Load-time indirections. Any assembly failure here is a typed refusal:
// the gateway never serves unauthenticated because its authentication half-configured.
func serveLive(ctx context.Context, addr string, resolved config.Resolved, authority *sfwp.Authority, client *sfwp.Client, secrets map[string]string) error {
	serve := config.ServeDefaults(resolved.Serve)
	authCfg := resolved.AuthOrDefaults()
	logger := log.New(os.Stderr, "", 0)

	authOpts, err := buildAuthenticator(ctx, authCfg, secrets)
	if err != nil {
		return err
	}

	source := projection.NewLiveSource(authority, ports.ActorClaim{
		ActorID: serve.GatewayActorID,
		Role:    serve.GatewayRole,
	})
	store := projection.NewStore()

	// The subscription and relay run for the process lifetime: they are the gateway's window on
	// kernel truth (per-case cursors and revision history).
	sub, err := client.Subscribe(ctx)
	if err != nil {
		return err
	}
	relay := server.NewRelay(server.NewSFWPFeed(sub), source, store, server.RelayOptions{
		DefaultActor: ports.ActorClaim{ActorID: serve.PerspectiveActorID, Role: serve.PerspectiveRole},
		Logger:       log.New(os.Stderr, "godspeed-casework: ", 0),
	})
	relayCtx, cancelRelay := context.WithCancel(ctx)
	defer cancelRelay()
	go relay.Run(relayCtx)

	dispatcher := intents.NewHandler(authority, relay, authority, intents.Options{
		Gateway:             ports.ActorClaim{ActorID: serve.GatewayActorID, Role: serve.GatewayRole},
		PolicyRef:           serve.PolicyRef,
		ExecutionTimeoutSec: 60,
	})

	// Session sweeps keep the bounded store clean without depending on request traffic.
	sweepCtx, stopSweeps := context.WithCancel(ctx)
	defer stopSweeps()
	go sweepSessions(sweepCtx, authOpts.Sessions, time.Minute)

	api := server.New(source, dispatcher, source, store, relay, server.Options{
		Perspective: ports.ActorClaim{ActorID: serve.PerspectiveActorID, Role: serve.PerspectiveRole},
		Auth:        authOpts,
		StaticRoot:  serve.StaticRoot,
		Ready:       authority,
		RateLimit: server.RateLimitOptions{
			PerMinute: serve.RateLimitOrDefaults().IntentsPerMinute,
			Burst:     serve.RateLimitOrDefaults().Burst,
		},
		Logger: logger,
	})
	httpServer := &http.Server{
		Addr:              addr,
		Handler:           api.Handler(),
		ReadHeaderTimeout: 10 * time.Second,
	}

	serveErr := make(chan error, 1)
	go func() { serveErr <- httpServer.ListenAndServe() }()
	staticNote := "no static UI"
	if serve.StaticRoot != "" {
		staticNote = "static UI " + serve.StaticRoot
	}
	fmt.Printf("godspeed-casework: serving LIVE cognitive projection on http://%s (provenance %s; kernel socket %s; auth mode %s; %s)\n",
		addr, projection.ProvenanceLabelLive, client.SocketPath(), authCfg.Mode, staticNote)

	signals := make(chan os.Signal, 1)
	signal.Notify(signals, syscall.SIGINT, syscall.SIGTERM)
	select {
	case err := <-serveErr:
		return err
	case sig := <-signals:
		fmt.Printf("godspeed-casework: %s received - shutting down\n", sig)
		stopSweeps()
		cancelRelay()
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return httpServer.Shutdown(shutdownCtx)
	}
}

// sweepSessions destroys expired sessions on an interval until ctx is done.
func sweepSessions(ctx context.Context, sessions *auth.SessionStore, interval time.Duration) {
	if sessions == nil {
		return
	}
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			sessions.Sweep()
		}
	}
}

// buildAuthenticator wires the configured authentication mode. Every failure is fatal: a gateway
// that cannot build its front door refuses to start (the config validation has already refused
// the inconsistent combinations, e.g. dev mode in the production posture).
func buildAuthenticator(ctx context.Context, authCfg config.AuthSection, secrets map[string]string) (server.AuthOptions, error) {
	opts := server.AuthOptions{CookieSecure: !authCfg.InsecureCookie}
	if len(authCfg.Users) == 0 {
		return opts, apperr.New(apperr.KindConfig, "", "auth", "the auth section carries no users")
	}
	users := make([]auth.User, 0, len(authCfg.Users))
	for _, u := range authCfg.Users {
		users = append(users, auth.User{
			Username: u.Username, DisplayName: u.DisplayName,
			PasswordHash: u.PasswordHash, ActorID: u.ActorID, Role: u.Role,
		})
	}
	switch authCfg.Mode {
	case config.AuthModeLocal, config.AuthModeDev:
		mode := auth.ModeLocal
		if authCfg.Mode == config.AuthModeDev {
			mode = auth.ModeDev
		}
		store, err := auth.NewLocalUserStore(mode, users)
		if err != nil {
			return opts, apperr.New(apperr.KindConfig, "", "auth", err.Error())
		}
		opts.Authenticator = store
		if authCfg.StaticToken != "" {
			token, ok := secrets["auth.static_token"]
			if !ok {
				return opts, apperr.New(apperr.KindConfig, "", "auth",
					"auth.static_token is configured but did not resolve")
			}
			identity, err := store.StaticTokenIdentity(authCfg.StaticTokenUser)
			if err != nil {
				return opts, apperr.New(apperr.KindConfig, "", "auth", err.Error())
			}
			opts.StaticToken = token
			opts.BearerIdentity = &identity
		}
	case config.AuthModeOIDC:
		secret, ok := secrets["auth.oidc.client_secret"]
		if !ok {
			return opts, apperr.New(apperr.KindConfig, "", "auth",
				"auth.oidc.client_secret is configured but did not resolve")
		}
		mappings := make([]auth.OIDCMapping, 0, len(authCfg.OIDC.Mappings))
		for _, m := range authCfg.OIDC.Mappings {
			mappings = append(mappings, auth.OIDCMapping{Claim: m.Claim, Equals: m.Equals, ActorID: m.ActorID, Role: m.Role})
		}
		oidcAuth, err := auth.NewOIDC(ctx, auth.OIDCOptions{
			Issuer:        authCfg.OIDC.Issuer,
			ClientID:      authCfg.OIDC.ClientID,
			ClientSecret:  secret,
			RedirectURL:   authCfg.OIDC.RedirectURL,
			Scopes:        authCfg.OIDC.Scopes,
			UsernameClaim: authCfg.OIDC.UsernameClaim,
			Mappings:      mappings,
		})
		if err != nil {
			return opts, apperr.New(apperr.KindConfig, "", "auth", err.Error())
		}
		opts.Authenticator = oidcAuth
	default:
		return opts, apperr.New(apperr.KindConfig, "", "auth", "unknown auth mode "+authCfg.Mode)
	}
	opts.Sessions = auth.NewSessionStore(
		time.Duration(authCfg.IdleTTLMinutes)*time.Minute,
		time.Duration(authCfg.AbsoluteTTLHours)*time.Hour,
		0, nil,
	)
	return opts, nil
}

// report prints a typed error with its kind and capability, so the exit is explainable without
// parsing prose.
func report(err error) {
	var typed *apperr.Error
	if errors.As(err, &typed) {
		if typed.Capability != "" {
			fmt.Fprintf(os.Stderr, "godspeed-casework: %s error (capability=%s): %s\n", typed.Kind, typed.Capability, typed.Message)
			return
		}
		fmt.Fprintf(os.Stderr, "godspeed-casework: %s error: %s\n", typed.Kind, typed.Message)
		return
	}
	fmt.Fprintf(os.Stderr, "godspeed-casework: internal error: %v\n", err)
}
