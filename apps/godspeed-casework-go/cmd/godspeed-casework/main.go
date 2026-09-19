// Command godspeed-casework is the casework front end's entrypoint.
//
// At T01 it does one thing end to end: load its single configuration authority, resolve secret
// indirections, run preflight, and report each configured capability's state with a typed error. It
// deliberately does NOT start any transport or accept work: transport and the real adapters land in
// T04/T05/T11, and until then "no adapter registered" is the honest answer rather than a stub that
// pretends to be connected.
package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/config"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/preflight"
)

// exitBlocked is used when a REQUIRED capability is unusable. It is a distinct code so an operator (or
// a calling script) never has to read the message to know the application refused to proceed.
const exitBlocked = 2

func main() {
	configPath := flag.String("config", os.Getenv("GODSPEED_CONFIG"), "path to the configuration file")
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

	// No adapter is registered yet: T01 owns the ports and the preflight contract, not the wiring.
	probers := map[string]ports.Health{}

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
