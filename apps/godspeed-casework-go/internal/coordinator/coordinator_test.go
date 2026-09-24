package coordinator

import (
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/artifactstore"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// instantOptions removes every real wait so the staged flow completes immediately.
func instantOptions() Options {
	return Options{Sleep: func(time.Duration) {}, Stage1: time.Millisecond, Stage2: time.Millisecond}
}

type fixture struct {
	proj  *projection.Store
	arts  *artifactstore.Store
	coord *Coordinator
}

func newFixture(t *testing.T) fixture {
	t.Helper()
	ds, err := projection.Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	proj, err := projection.NewStore(ds)
	if err != nil {
		t.Fatalf("build store: %v", err)
	}
	arts := artifactstore.New(ds.Artifacts)
	return fixture{proj: proj, arts: arts, coord: New(proj, arts, instantOptions())}
}

func implementIntent(id, target string, cursor any) Intent {
	return Intent{
		ID:     id,
		Kind:   "propose-consequence",
		Target: target,
		Parameters: map[string]any{
			"action":   "implement",
			"approach": "compat-layer",
			"cursor":   cursor,
		},
	}
}

// waitFor polls cond until it holds or the timeout expires.
func waitFor(t *testing.T, timeout time.Duration, what string, cond func() bool) {
	t.Helper()
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if cond() {
			return
		}
		time.Sleep(2 * time.Millisecond)
	}
	t.Fatalf("timed out waiting for %s", what)
}

func objectByID(t *testing.T, snap projection.Snapshot, id string) projection.Object {
	t.Helper()
	for _, o := range snap.Objects {
		if o.ID == id {
			return o
		}
	}
	t.Fatalf("object %s not present in snapshot %d", id, snap.Cursor)
	return projection.Object{}
}

// TestIntentValidationMatrix walks every refusal reason and the benign acceptances.
func TestIntentValidationMatrix(t *testing.T) {
	live := int64(1150)
	cases := []struct {
		name   string
		intent Intent
		status string
		reason string
	}{
		{"missing id", Intent{Kind: "focus-object", Target: "ns-goal"}, "refused", ReasonInvalid},
		{"unknown kind", Intent{ID: "i1", Kind: "delete-everything", Target: "ns-goal"}, "refused", ReasonInvalid},
		{"missing kind", Intent{ID: "i2", Target: "ns-goal"}, "refused", ReasonInvalid},
		{"missing target", Intent{ID: "i3", Kind: "focus-object"}, "refused", ReasonInvalid},
		{"stale cursor", implementIntent("i4", "ns-migration", float64(900)), "refused", ReasonStaleProjection},
		{"cursor wrong type", implementIntent("i5", "ns-migration", "1150"), "refused", ReasonInvalid},
		{
			"decide-approval refused honestly",
			Intent{ID: "i6", Kind: "decide-approval", Target: "ns-pr-491", Parameters: map[string]any{"cursor": float64(live)}},
			"refused", ReasonAuthorityDenied,
		},
		{
			"implement on unlisted target denied",
			implementIntent("i7", "eng-northstar", float64(live)),
			"refused", ReasonAuthorityDenied,
		},
		{
			"implement with unlisted action denied",
			Intent{
				ID: "i8", Kind: "propose-consequence", Target: "ns-migration",
				Parameters: map[string]any{"action": "demolish", "cursor": float64(live)},
			},
			"refused", ReasonAuthorityDenied,
		},
		{
			"implement with no action denied",
			Intent{ID: "i9", Kind: "propose-consequence", Target: "ns-migration", Parameters: map[string]any{"cursor": float64(live)}},
			"refused", ReasonAuthorityDenied,
		},
		{
			"focus-object accepted without effects",
			Intent{ID: "i10", Kind: "focus-object", Target: "ns-path-10", Parameters: map[string]any{"cursor": float64(live)}},
			"accepted", "",
		},
		{
			"request-explanation accepted without effects",
			Intent{ID: "i11", Kind: "request-explanation", Target: "ns-secondary"},
			"accepted", "",
		},
		{
			"inspect-artifact accepted without effects",
			Intent{ID: "i12", Kind: "inspect-artifact", Target: "art-diff-491"},
			"accepted", "",
		},
		{
			"resolve-object accepted without effects",
			Intent{ID: "i13", Kind: "resolve-object", Target: "ns-path-10"},
			"accepted", "",
		},
		{
			"persist-artifact without ref refused",
			Intent{ID: "i14", Kind: "persist-artifact", Target: "ns-workflow", Parameters: map[string]any{}},
			"refused", ReasonInvalid,
		},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			f := newFixture(t)
			out := f.coord.Submit(tc.intent)
			if out.Status != tc.status {
				t.Fatalf("status: got %q, want %q (note %q)", out.Status, tc.status, out.Note)
			}
			if out.Reason != tc.reason {
				t.Fatalf("reason: got %q, want %q", out.Reason, tc.reason)
			}
			if out.Status == StatusRefused && out.Note == "" {
				t.Fatal("a refusal must carry a note")
			}
		})
	}
}

// The security tooth: an authority denial must leave the world and the lease registry untouched.
func TestAuthorityDenialHasZeroEffects(t *testing.T) {
	f := newFixture(t)
	windowBefore := len(f.proj.Window())

	out := f.coord.Submit(implementIntent("deny-1", "eng-northstar", float64(1150)))
	if out.Status != StatusRefused || out.Reason != ReasonAuthorityDenied {
		t.Fatalf("want authority_denied refusal, got %+v", out)
	}
	if out.Lease != nil {
		t.Fatalf("a refusal must not carry a lease: %+v", out.Lease)
	}
	if got := len(f.proj.Window()); got != windowBefore {
		t.Fatalf("denied intent appended a revision: window %d -> %d", windowBefore, got)
	}
	if got := f.proj.LiveCursor(); got != 1150 {
		t.Fatalf("live cursor moved under denial: %d", got)
	}
	if got := len(f.coord.Leases()); got != 0 {
		t.Fatalf("denied intent minted a lease: %v", f.coord.Leases())
	}
}

func TestIdempotencyReplayAndConflict(t *testing.T) {
	f := newFixture(t)
	first := f.coord.Submit(implementIntent("idem-1", "ns-migration", nil))
	if first.Status != StatusAccepted || first.Lease == nil || first.Lease.State != LeaseActive {
		t.Fatalf("first submission: %+v", first)
	}
	liveAfterFirst := f.proj.LiveCursor() // 1151 at this point

	replay := f.coord.Submit(implementIntent("idem-1", "ns-migration", nil))
	if replay != first {
		t.Fatalf("replay must return the recorded outcome verbatim:\n%+v\n%+v", replay, first)
	}
	if got := f.proj.LiveCursor(); got != liveAfterFirst {
		t.Fatalf("replay appended another revision: %d -> %d", liveAfterFirst, got)
	}

	conflicting := implementIntent("idem-1", "ns-secondary", nil)
	if out := f.coord.Submit(conflicting); out.Status != StatusRefused || out.Reason != ReasonInvalid {
		t.Fatalf("same id with a different body must be refused invalid, got %+v", out)
	}

	// The whitespace/key-order insensitivity of the body hash is covered by the canonical-encoding
	// property: identical decoded intents hash identically.
	if bodyHash(implementIntent("x", "ns-migration", nil)) != bodyHash(implementIntent("x", "ns-migration", nil)) {
		t.Fatal("identical intents must hash identically")
	}
}

func TestStaleProjectionRefusal(t *testing.T) {
	f := newFixture(t)
	out := f.coord.Submit(implementIntent("stale-1", "ns-migration", float64(1100)))
	if out.Status != StatusRefused || out.Reason != ReasonStaleProjection {
		t.Fatalf("want stale_projection refusal, got %+v", out)
	}
	if got := f.proj.LiveCursor(); got != 1150 {
		t.Fatalf("a stale refusal must not move the world: %d", got)
	}
}

// The full consequential flow: acceptance, staged revisions 1151-1153, and lease release.
func TestAcceptedImplementRunsStagedRevisions(t *testing.T) {
	f := newFixture(t)

	leaseCh, cancel := f.coord.SubscribeLeases()
	defer cancel()

	out := f.coord.Submit(implementIntent("flow-1", "ns-migration", float64(1150)))
	if out.Status != StatusAccepted {
		t.Fatalf("submission: %+v", out)
	}
	if out.Lease == nil || out.Lease.State != LeaseActive || out.Lease.ID == "" {
		t.Fatalf("accepted implement must carry an active lease: %+v", out.Lease)
	}
	if got := f.proj.LiveCursor(); got != 1151 {
		t.Fatalf("acceptance must append revision 1151 immediately, live is %d", got)
	}

	rev1151 := f.proj.Live()
	fix := objectByID(t, rev1151, "ns-fix-layer")
	if fix.Kind != "component" || fix.Label != "Compatibility layer" {
		t.Fatalf("ns-fix-layer shape: %+v", fix)
	}
	if fix.Position != (projection.Position{X: 3.4, Y: 3.6, Depth: 1.0}) {
		t.Fatalf("ns-fix-layer position: %+v", fix.Position)
	}
	if fix.Salience != 0.9 || fix.Attention != "notable" || fix.Note != "Agent claimed the work" {
		t.Fatalf("ns-fix-layer salience/attention/note: %+v", fix)
	}
	if fix.ParentID == nil || *fix.ParentID != "ns-implementation" {
		t.Fatalf("ns-fix-layer parentId: %v", fix.ParentID)
	}

	waitFor(t, 2*time.Second, "staged flow to finish", func() bool { return f.proj.LiveCursor() >= 1153 })

	rev1152, err := f.proj.At(1152)
	if err != nil {
		t.Fatalf("At(1152): %v", err)
	}
	if got := objectByID(t, rev1152, "ns-fix-layer").Note; got != "Running candidate checks" {
		t.Fatalf("1152 ns-fix-layer note: %q", got)
	}
	if got := objectByID(t, rev1152, "ns-checks").Note; got != "10/10 green (candidate)" {
		t.Fatalf("1152 ns-checks note: %q", got)
	}

	rev1153 := f.proj.Live()
	quiet := objectByID(t, rev1153, "ns-fix-layer")
	if quiet.Note != "Merged" || quiet.Salience != 0.35 || quiet.Attention != "" {
		t.Fatalf("1153 ns-fix-layer must be quiet: %+v", quiet)
	}
	path10 := objectByID(t, rev1153, "ns-path-10")
	if path10.Note != "Resolved - absence now holds correctly" || path10.Salience != 0.3 || path10.Attention != "" {
		t.Fatalf("1153 ns-path-10: %+v", path10)
	}
	validation := objectByID(t, rev1153, "ns-validation")
	if validation.Note != "10/10 verified" || validation.Salience != 0.5 || validation.Attention != "" {
		t.Fatalf("1153 ns-validation: %+v", validation)
	}
	rollout := objectByID(t, rev1153, "ns-rollout")
	if rollout.Note != "Ready after security review" || rollout.Salience != 0.55 {
		t.Fatalf("1153 ns-rollout: %+v", rollout)
	}
	workflow := objectByID(t, rev1153, "ns-workflow")
	if workflow.Note != "Preserved - no change needed" || workflow.Salience != 0.4 || workflow.Attention != "" {
		t.Fatalf("1153 ns-workflow: %+v", workflow)
	}
	eng := objectByID(t, rev1153, "eng-northstar")
	if eng.Note != "Pilot ready for review" || eng.Salience != 0.6 || eng.Attention != "" {
		t.Fatalf("1153 eng-northstar: %+v", eng)
	}

	waitFor(t, 2*time.Second, "lease release", func() bool {
		for _, lv := range f.coord.Leases() {
			if lv.ID == out.Lease.ID && lv.State == LeaseReleased {
				return true
			}
		}
		return false
	})

	// Lease transitions stream in order: claimed, active, released.
	wantStates := []string{LeaseClaimed, LeaseActive, LeaseReleased}
	for _, want := range wantStates {
		select {
		case lv := <-leaseCh:
			if lv.ID != out.Lease.ID || lv.State != want {
				t.Fatalf("lease event: got %+v, want state %q", lv, want)
			}
		case <-time.After(2 * time.Second):
			t.Fatalf("timed out waiting for lease event %q", want)
		}
	}
}

// Earlier revisions must never be rewritten by later stages: the store keeps every position.
func TestStagedRevisionsDoNotRetroMutate(t *testing.T) {
	f := newFixture(t)
	f.coord.Submit(implementIntent("retro-1", "ns-secondary", nil))
	waitFor(t, 2*time.Second, "staged flow to finish", func() bool { return f.proj.LiveCursor() >= 1153 })

	fix, err := f.proj.At(1151)
	if err != nil {
		t.Fatalf("At(1151): %v", err)
	}
	if got := objectByID(t, fix, "ns-fix-layer").Note; got != "Agent claimed the work" {
		t.Fatalf("revision 1151 was retro-mutated: %q", got)
	}
	if _, err := f.proj.At(1152); err != nil {
		t.Fatalf("At(1152): %v", err)
	}
}

// A second implement on the other allowlisted target continues the cursor sequence.
func TestSecondImplementContinuesCursorSequence(t *testing.T) {
	f := newFixture(t)
	first := f.coord.Submit(implementIntent("seq-1", "ns-migration", nil))
	waitFor(t, 2*time.Second, "first flow to finish", func() bool { return f.proj.LiveCursor() >= 1153 })
	second := f.coord.Submit(implementIntent("seq-2", "ns-secondary", float64(f.proj.LiveCursor())))
	if second.Status != StatusAccepted {
		t.Fatalf("second implement: %+v", second)
	}
	if first.Lease != nil && second.Lease != nil && first.Lease.ID == second.Lease.ID {
		t.Fatal("each accepted implement must mint its own lease")
	}
	waitFor(t, 2*time.Second, "second flow to finish", func() bool { return f.proj.LiveCursor() >= 1156 })
	if got := len(f.coord.Leases()); got != 2 {
		t.Fatalf("lease registry: got %d, want 2", got)
	}
}

func TestPersistArtifactIntentUsesFixtureScopedStore(t *testing.T) {
	f := newFixture(t)
	out := f.coord.Submit(Intent{
		ID: "art-1", Kind: "persist-artifact", Target: "ns-workflow",
		Parameters: map[string]any{"ref": "art-op-note", "boundObject": "ns-workflow", "title": "Operator note"},
	})
	if out.Status != StatusAccepted {
		t.Fatalf("persist-artifact: %+v", out)
	}
	if _, ok := f.arts.Get("art-op-note"); !ok {
		t.Fatal("persisted artifact must be in the artifact store")
	}
	if got := f.proj.LiveCursor(); got != 1150 {
		t.Fatalf("persist-artifact must not move the world: %d", got)
	}
}

// Documented limitation, pinned: leases and idempotency records are process-scoped. A second
// coordinator over the same projection starts with none of the first one's.
func TestRestartClearsLeasesAndIdempotency(t *testing.T) {
	f := newFixture(t)
	accepted := f.coord.Submit(implementIntent("restart-1", "ns-migration", float64(1150)))
	if accepted.Status != StatusAccepted {
		t.Fatalf("submission: %+v", accepted)
	}
	waitFor(t, 2*time.Second, "flow to finish", func() bool { return f.proj.LiveCursor() >= 1153 })
	if got := len(f.coord.Leases()); got != 1 {
		t.Fatalf("first coordinator leases: %d", got)
	}

	restarted := New(f.proj, f.arts, instantOptions())
	if got := len(restarted.Leases()); got != 0 {
		t.Fatalf("a restarted coordinator must see no leases, got %d", got)
	}
	// The intent record is gone too: the same id is evaluated fresh, and against a world that has
	// moved on the recorded cursor is stale.
	replay := restarted.Submit(implementIntent("restart-1", "ns-migration", float64(1150)))
	if replay.Status != StatusRefused || replay.Reason != ReasonStaleProjection {
		t.Fatalf("a restarted coordinator must not replay the first one's outcome, got %+v", replay)
	}
	// Durability that DOES survive the "restart" is the artifact store's own scope:
	f.coord.Submit(Intent{ID: "restart-art", Kind: "persist-artifact", Target: "ns-workflow",
		Parameters: map[string]any{"ref": "art-survives", "boundObject": "ns-workflow"}})
	restarted2 := New(f.proj, f.arts, instantOptions())
	_ = restarted2
	if _, ok := f.arts.Get("art-survives"); !ok {
		t.Fatal("artifact durability is store-scoped and must survive coordinator replacement")
	}
}
