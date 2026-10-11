// Package fakeauthority is a PROVIDER-SIDE adapter used by boundary and preflight tests.
//
// It exists to make the boundary real rather than asserted: the wire shape below is deliberately
// provider-flavoured (a foreign envelope with foreign field names), and the adapter is the only
// place it may appear. Everything the adapter returns is translated into internal/ports types, and
// internal/boundary fails the build if a core package imports this package or mentions its
// vocabulary.
package fakeauthority

import (
	"context"
	"errors"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// wireEnvelope is a provider-native payload. Its field names are intentionally unlike the
// application's, so a leak into the core would be visible rather than cosmetic.
type wireEnvelope struct {
	WireCaseID    string   `json:"case_ulid"`
	WireState     string   `json:"standing_code"`
	WireBasisRefs []string `json:"basis_entry_refs"`
	WireSettledAt string   `json:"settled_at_rfc3339"`
	WireReview    bool     `json:"human_review_required"`
}

// standing maps the provider's standing codes to the application's vocabulary. An unknown code is an
// error rather than a guess: REQ-ARCH-002 requires translation, and guessing would invent meaning.
func standing(code string) (ports.Standing, error) {
	switch code {
	case "ACCEPTED":
		return ports.StandingAccepted, nil
	case "REVIEW":
		return ports.StandingReviewRequired, nil
	case "PENDING":
		return ports.StandingPending, nil
	default:
		return "", errors.New("unknown standing code from provider: " + code)
	}
}

// Authority adapts a provider transport to ports.AuthorityPort.
type Authority struct {
	// Fetch returns the provider envelope for a run. Tests inject faults here.
	Fetch func(ctx context.Context, run string) (wireEnvelope, error)
	// Probe reports provider health; nil means healthy.
	Probe func(ctx context.Context) error
}

// Health implements ports.Health.
func (a *Authority) Health(ctx context.Context) error {
	if a.Probe == nil {
		return nil
	}
	return a.Probe(ctx)
}

// Settlement implements ports.AuthorityPort, translating the provider envelope into the application
// model. No provider field escapes this method.
func (a *Authority) Settlement(ctx context.Context, run ports.RunRef) (ports.SettlementObservation, error) {
	if a.Fetch == nil {
		return ports.SettlementObservation{}, errors.New("fakeauthority: no fetch function configured")
	}
	env, err := a.Fetch(ctx, string(run))
	if err != nil {
		return ports.SettlementObservation{}, err
	}
	st, err := standing(env.WireState)
	if err != nil {
		return ports.SettlementObservation{}, err
	}
	settled := time.Time{}
	if env.WireSettledAt != "" {
		settled, err = time.Parse(time.RFC3339, env.WireSettledAt)
		if err != nil {
			return ports.SettlementObservation{}, err
		}
	}
	return ports.SettlementObservation{
		Run:            ports.RunRef(env.WireCaseID),
		Standing:       st,
		Basis:          append([]string(nil), env.WireBasisRefs...),
		ReviewRequired: env.WireReview,
		SettledAt:      settled,
	}, nil
}

// ListCases implements ports.AuthorityPort for the tests that need a case view.
func (a *Authority) ListCases(ctx context.Context) ([]ports.CaseSummary, error) {
	return nil, errors.New("fakeauthority: ListCases is not part of the T01 scenario")
}

// History implements ports.AuthorityPort for the tests that need a history page.
func (a *Authority) History(ctx context.Context, ref ports.CaseRef, since time.Time, limit int) (ports.HistoryWindow, error) {
	return ports.HistoryWindow{}, errors.New("fakeauthority: History is not part of the T01 scenario")
}

// ArtifactProber is an OPTIONAL capability adapter that can be made unavailable on demand, so the
// preflight blast-radius scenario can run without a real artifact store.
type ArtifactProber struct {
	Available bool
}

// Health implements ports.Health.
func (p ArtifactProber) Health(ctx context.Context) error {
	if !p.Available {
		return errors.New("artifact adapter unavailable")
	}
	return nil
}

// Put implements ports.ArtifactStore.
func (p ArtifactProber) Put(ctx context.Context, name string, content []byte) (ports.ArtifactRef, error) {
	return "", errors.New("fakeauthority: Put is not part of the T01 scenario")
}

// Get implements ports.ArtifactStore.
func (p ArtifactProber) Get(ctx context.Context, ref ports.ArtifactRef) ([]byte, error) {
	return nil, errors.New("fakeauthority: Get is not part of the T01 scenario")
}

// Probe is a ports.Health implementation whose fault can be injected, so preflight tests can exercise
// a failing required capability without a real transport.
type Probe struct {
	Err error
}

// Health implements ports.Health.
func (p Probe) Health(ctx context.Context) error { return p.Err }
