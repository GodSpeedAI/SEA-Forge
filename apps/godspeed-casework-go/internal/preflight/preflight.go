// Package preflight validates configured capabilities BEFORE the application accepts consequential
// work, and reports each capability's state so a failure disables only what it must.
//
// REQ-CONFIG-010: required ports/adapters and configuration are validated before consequential work.
// REQ-CONFIG-012: a preflight failure disables only the capability whose dependency is unavailable,
// unless that capability is required for application startup - in which case the whole application is
// blocked, and the unrelated capabilities are still reported with their own honest states.
package preflight

import (
	"context"
	"sort"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// State is the outcome for one capability.
type State string

const (
	// StateReady means the capability answered and is usable.
	StateReady State = "ready"
	// StateDegraded means an OPTIONAL capability is unavailable: the application proceeds without it.
	StateDegraded State = "degraded"
	// StateBlocking means a REQUIRED capability is unusable: consequential work must not start.
	StateBlocking State = "blocking"
)

// Capability is the preflight view of one configured capability.
type Capability struct {
	Name     string
	Kind     string
	Required bool
}

// Result pairs a capability with its state and, when not ready, a typed error carrying the
// capability name so callers never string-match a message to work out what failed.
type Result struct {
	Capability Capability
	State      State
	Err        *apperr.Error
}

// Summary is the aggregate view.
type Summary struct {
	Blocking []string
	Degraded []string
	Ready    []string
	// Proceed is false when any required capability is blocking.
	Proceed bool
}

// Check evaluates every capability. probers maps capability name to its health prober; a capability
// with no prober is treated as unavailable, which is exactly what "no adapter is wired for this
// capability yet" means.
func Check(ctx context.Context, caps []Capability, probers map[string]ports.Health, configProblems []*apperr.Error) []Result {
	byCap := map[string][]*apperr.Error{}
	for _, p := range configProblems {
		if p == nil {
			continue
		}
		byCap[p.Capability] = append(byCap[p.Capability], p)
	}

	results := make([]Result, 0, len(caps))
	for _, c := range caps {
		if probs := byCap[c.Name]; len(probs) > 0 {
			results = append(results, Result{Capability: c, State: stateFor(c.Required), Err: probs[0]})
			continue
		}
		prober, ok := probers[c.Name]
		if !ok || prober == nil {
			results = append(results, Result{Capability: c, State: stateFor(c.Required), Err: apperr.New(
				apperr.KindUnavailable, c.Name, "preflight",
				"no adapter is registered for capability kind "+c.Kind)})
			continue
		}
		if err := prober.Health(ctx); err != nil {
			results = append(results, Result{Capability: c, State: stateFor(c.Required), Err: apperr.Wrap(
				apperr.KindUnavailable, c.Name, "preflight", "capability health check failed", err)})
			continue
		}
		results = append(results, Result{Capability: c, State: StateReady})
	}
	sort.SliceStable(results, func(i, j int) bool { return results[i].Capability.Name < results[j].Capability.Name })
	return results
}

func stateFor(required bool) State {
	if required {
		return StateBlocking
	}
	return StateDegraded
}

// Summarise aggregates results without hiding any: a blocking required capability sets Proceed=false
// while unrelated ready capabilities keep their names in Ready.
func Summarise(results []Result) Summary {
	s := Summary{Proceed: true}
	for _, r := range results {
		switch r.State {
		case StateReady:
			s.Ready = append(s.Ready, r.Capability.Name)
		case StateDegraded:
			s.Degraded = append(s.Degraded, r.Capability.Name)
		case StateBlocking:
			s.Blocking = append(s.Blocking, r.Capability.Name)
			s.Proceed = false
		}
	}
	return s
}

// Failed reports the typed error for a capability, or nil when the capability is ready.
func Failed(results []Result, capability string) *apperr.Error {
	for _, r := range results {
		if r.Capability.Name == capability {
			return r.Err
		}
	}
	return nil
}
