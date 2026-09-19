package preflight

import (
	"context"
	"errors"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/fakeauthority"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// The plan's second tooth: one required adapter credential missing AND one optional artifact adapter
// unavailable. Expected: the required consequential capability blocks with a typed error, the
// unrelated capability stays available, and the optional one degrades instead of blocking.
//
// This is REQ-CONFIG-012 stated as a test: a preflight failure disables only the capability whose
// dependency is unavailable unless that capability is required for startup - in which case the
// application blocks, but the other capabilities keep their own honest states.
func TestPreflightBlastRadius(t *testing.T) {
	ctx := context.Background()

	caps := []Capability{
		{Name: "authority", Kind: "authority", Required: true},
		{Name: "repository", Kind: "repository", Required: true},
		{Name: "artifact", Kind: "artifact", Required: false},
	}

	// authority is healthy; artifact is present but unavailable; repository never got wired because
	// its credential indirection failed validation, which is reported as a config problem.
	probers := map[string]ports.Health{
		"authority": fakeauthority.Probe{},
		"artifact":  fakeauthority.ArtifactProber{Available: false},
	}
	configProblems := []*apperr.Error{
		apperr.New(apperr.KindConfig, "repository", "secret", "credential indirection env:REPO_TOKEN is not set"),
	}

	results := Check(ctx, caps, probers, configProblems)
	summary := Summarise(results)

	if got := stateOf(results, "authority"); got != StateReady {
		t.Errorf("authority should be ready, got %q (%v)", got, Failed(results, "authority"))
	}
	if got := stateOf(results, "repository"); got != StateBlocking {
		t.Errorf("a required capability with a config problem must block, got %q", got)
	}
	if got := stateOf(results, "artifact"); got != StateDegraded {
		t.Errorf("an unavailable OPTIONAL capability must degrade, got %q", got)
	}
	if summary.Proceed {
		t.Error("the application must not proceed while a required capability is blocking")
	}
	if !contains(summary.Ready, "authority") {
		t.Errorf("the unrelated ready capability must still be reported ready: %v", summary.Ready)
	}
	if !contains(summary.Degraded, "artifact") {
		t.Errorf("the optional capability must be reported degraded: %v", summary.Degraded)
	}

	// Typed errors, not prose: the blocking capability is a configuration fault; the degraded one is
	// an availability fault. A caller can act on that distinction.
	if got := apperr.KindOf(Failed(results, "repository")); got != apperr.KindConfig {
		t.Errorf("repository failure kind = %q, want %q", got, apperr.KindConfig)
	}
	if got := apperr.KindOf(Failed(results, "artifact")); got != apperr.KindUnavailable {
		t.Errorf("artifact failure kind = %q, want %q", got, apperr.KindUnavailable)
	}
	if got := apperr.CapabilityOf(Failed(results, "artifact")); got != "artifact" {
		t.Errorf("typed error must name its capability, got %q", got)
	}
}

// A capability with no registered adapter is unavailable, never silently ready.
func TestUnwiredCapabilityIsUnavailableNotReady(t *testing.T) {
	ctx := context.Background()
	caps := []Capability{
		{Name: "authority", Kind: "authority", Required: true},
		{Name: "artifact", Kind: "artifact", Required: false},
	}
	results := Check(ctx, caps, map[string]ports.Health{}, nil)
	if got := stateOf(results, "authority"); got != StateBlocking {
		t.Errorf("required unwired capability = %q, want blocking", got)
	}
	if got := stateOf(results, "artifact"); got != StateDegraded {
		t.Errorf("optional unwired capability = %q, want degraded", got)
	}
	if got := apperr.KindOf(Failed(results, "authority")); got != apperr.KindUnavailable {
		t.Errorf("unwired capability kind = %q, want %q", got, apperr.KindUnavailable)
	}
}

// A probe that reports a fault must be classified by requiredness, and the cause must survive for
// diagnosis even though the caller only switches on Kind.
func TestFailingProbeKeepsCauseAndClassification(t *testing.T) {
	ctx := context.Background()
	cause := errors.New("socket refused")
	caps := []Capability{{Name: "authority", Kind: "authority", Required: true}}
	results := Check(ctx, caps, map[string]ports.Health{
		"authority": fakeauthority.Probe{Err: cause},
	}, nil)
	err := Failed(results, "authority")
	if err == nil {
		t.Fatal("a failing probe must produce a typed error")
	}
	if !errors.Is(err, cause) {
		t.Errorf("the cause must be reachable with errors.Is: %v", err)
	}
	if err.Err == nil {
		t.Error("the typed error must wrap the cause rather than hide it")
	}
}

func TestSummariseProceedIsFalseOnlyForBlocking(t *testing.T) {
	s := Summarise([]Result{
		{Capability: Capability{Name: "a"}, State: StateReady},
		{Capability: Capability{Name: "b"}, State: StateDegraded},
	})
	if !s.Proceed {
		t.Error("degradation alone must not stop the application")
	}
	s = Summarise([]Result{{Capability: Capability{Name: "c"}, State: StateBlocking}})
	if s.Proceed {
		t.Error("a blocking capability must stop the application")
	}
}

func stateOf(results []Result, name string) State {
	for _, r := range results {
		if r.Capability.Name == name {
			return r.State
		}
	}
	return "<missing>"
}

func contains(list []string, want string) bool {
	for _, v := range list {
		if v == want {
			return true
		}
	}
	return false
}
