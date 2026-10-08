package fakeauthority

import (
	"context"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// Compile-time proof that the adapter satisfies the port: the port is what the core depends on, and
// this file is where the provider shape stops.
var (
	_ ports.AuthorityPort = (*Authority)(nil)
	_ ports.ArtifactStore = ArtifactProber{}
)

// Translate, do not leak: the provider envelope's fields must arrive as application types.
func TestSettlementTranslatesProviderEnvelope(t *testing.T) {
	a := &Authority{Fetch: func(ctx context.Context, run string) (wireEnvelope, error) {
		return wireEnvelope{
			WireCaseID:    "case-1",
			WireState:     "ACCEPTED",
			WireBasisRefs: []string{"evidence-a", "evidence-b"},
			WireSettledAt: "2026-09-19T12:00:00Z",
			WireReview:    false,
		}, nil
	}}
	got, err := a.Settlement(context.Background(), ports.RunRef("run-1"))
	if err != nil {
		t.Fatalf("settlement: %v", err)
	}
	if got.Standing != ports.StandingAccepted {
		t.Errorf("standing = %q, want %q", got.Standing, ports.StandingAccepted)
	}
	if len(got.Basis) != 2 {
		t.Errorf("basis should carry the cited evidence refs, got %v", got.Basis)
	}
	if !got.SettledAt.Equal(time.Date(2026, 9, 19, 12, 0, 0, 0, time.UTC)) {
		t.Errorf("settled_at not parsed: %v", got.SettledAt)
	}
}

// An unknown provider code must fail rather than be guessed into an application meaning.
func TestUnknownStandingCodeIsRefused(t *testing.T) {
	a := &Authority{Fetch: func(ctx context.Context, run string) (wireEnvelope, error) {
		return wireEnvelope{WireState: "SOMETHING_NEW"}, nil
	}}
	if _, err := a.Settlement(context.Background(), ports.RunRef("run-1")); err == nil {
		t.Fatal("an unmapped provider code must be an error, not a silent default")
	}
}

// A malformed timestamp is a translation failure, not a zero-valued settlement.
func TestMalformedTimestampIsRefused(t *testing.T) {
	a := &Authority{Fetch: func(ctx context.Context, run string) (wireEnvelope, error) {
		return wireEnvelope{WireState: "PENDING", WireSettledAt: "yesterday"}, nil
	}}
	if _, err := a.Settlement(context.Background(), ports.RunRef("run-1")); err == nil {
		t.Fatal("an unparsable timestamp must fail loudly")
	}
}
