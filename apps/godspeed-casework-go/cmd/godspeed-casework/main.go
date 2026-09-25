// Command godspeed-casework is the casework front end's entrypoint.
//
// Without -serve it does one thing end to end: load its single configuration authority, resolve
// secret indirections, run preflight, and report each configured capability's state with a typed
// error. It deliberately does NOT start any transport or accept work in that mode.
//
// With -serve it runs the same preflight first, then serves the cognitive
// projection API over HTTP+SSE from the Northstar fixture. FIXTURE-LABELED:
// the served projections and the intent decisions still come from the fixture
// allowlist, never from a governed authority (T06 wires the live projection).
// What IS live since T05: a capability that selects adapter=sfwp is probed by
// preflight through the real SFWP client (system.hello, readiness.get,
// identity.get), and the fixture provider never stands in for it.
package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/artifactstore"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/config"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/coordinator"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/preflight"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server"
)

// exitBlocked is used when a REQUIRED capability is unusable. It is a distinct code so an operator (or
// a calling script) never has to read the message to know the application refused to proceed.
const exitBlocked = 2

// fixtureProvider is the FIXTURE-LABELED in-process provider registered in serve mode: the
// projection, artifacts, and intent decisions are all served from the embedded fixture inside
// this process, so every configured capability answers health from here. It is an honest
// stand-in, not a governed authority.
type fixtureProvider struct{}

// Health implements ports.Health: the fixture provider lives in-process and cannot be unreachable.
func (fixtureProvider) Health(ctx context.Context) error { return nil }

func main() {
	configPath := flag.String("config", os.Getenv("GODSPEED_CONFIG"), "path to the configuration file")
	serve := flag.Bool("serve", false, "after preflight, serve the fixture cognitive projection API over HTTP+SSE")
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
	// it. Construction can only fail on an empty socket path, which validation
	// has already reported as that capability's own problem; the prober is
	// then simply not registered, and preflight carries the typed config fault.
	liveClients := map[string]*sfwp.Client{}
	probers := map[string]ports.Health{}
	for _, c := range resolved.Capabilities {
		if c.Adapter != config.AdapterLive {
			continue
		}
		client, err := sfwp.New(sfwp.Config{SocketPath: c.Endpoint})
		if err != nil {
			continue
		}
		liveClients[c.Name] = client
		probers[c.Name] = sfwp.NewAuthority(client)
	}
	defer func() {
		for _, client := range liveClients {
			client.Close()
		}
	}()
	if *serve {
		// Serve mode registers the FIXTURE-LABELED in-process provider for the
		// remaining capabilities, so preflight sees them ready rather than "no
		// adapter registered". The projection/intent wiring onto the live
		// authority is T06's work and must not be pretended here.
		for _, c := range resolved.Capabilities {
			if _, live := liveClients[c.Name]; !live {
				probers[c.Name] = fixtureProvider{}
			}
		}
	}

	// Capability-scoped problems travel into preflight, so a fault disables only the capability it
	// belongs to and every other capability still reports its own honest state.
	results := preflight.Check(ctx, caps, probers, problems)
	for _, r := range results {
		switch r.State {
		case preflight.StateReady:
			fmt.Printf("  ready     %-14s kind=%s required=%t\n", r.Capability.Name, r.Capability.Kind, r.Capability.Required)
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
		if err := serveForever(ctx, *addr); err != nil {
			report(apperr.Wrap(apperr.KindInternal, "", "serve", "server stopped", err))
			os.Exit(1)
		}
	}
}

// serveForever builds the fixture-labeled stack and blocks until SIGINT or SIGTERM, then shuts
// down gracefully.
func serveForever(ctx context.Context, addr string) error {
	dataset, err := projection.Fixture()
	if err != nil {
		return err
	}
	proj, err := projection.NewStore(dataset)
	if err != nil {
		return err
	}
	arts := artifactstore.New(dataset.Artifacts)
	coord := coordinator.New(proj, arts, coordinator.Options{})
	api := server.New(proj, coord, arts, server.Options{})

	httpServer := &http.Server{
		Addr:              addr,
		Handler:           api.Handler(),
		ReadHeaderTimeout: 10 * time.Second,
	}

	serveErr := make(chan error, 1)
	go func() { serveErr <- httpServer.ListenAndServe() }()
	fmt.Printf("godspeed-casework: serving cognitive projection on http://%s (FIXTURE-LABELED: %s)\n", addr, projection.ProvenanceLabel)

	signals := make(chan os.Signal, 1)
	signal.Notify(signals, syscall.SIGINT, syscall.SIGTERM)
	select {
	case err := <-serveErr:
		return err
	case sig := <-signals:
		fmt.Printf("godspeed-casework: %s received - shutting down\n", sig)
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return httpServer.Shutdown(shutdownCtx)
	}
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
