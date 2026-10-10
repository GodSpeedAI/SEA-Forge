package projection

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

const evDigest = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

func factsWithCapturedArtifact() CaseFacts {
	facts := baseFacts()
	facts.Horizon.Items = []ports.HorizonItem{{ItemID: "item_build", Name: "Build", Kind: "sandboxed_task", Execution: "completed", Settlement: "accepted"}}
	facts.Runs = []ports.RunSummary{{RunID: "run_1", CaseID: "case_test", PlanItemID: "item_build", Execution: "completed", Settlement: "accepted", EvidenceCount: 2}}
	facts.RunArtifacts = map[string][]ports.RunArtifactRef{"run_1": {{EvidenceID: "ev_1", URI: "work/dataset.md", Digest: evDigest}}}
	return facts
}

func TestBuildBindsCapturedArtifactsToTheItemAsEvidenceRecords(t *testing.T) {
	snap := Build(factsWithCapturedArtifact())
	ev := objectByID(t, snap, "ev_1")
	if ev.Kind != "evidence_record" || ev.ParentID == nil || *ev.ParentID != "item_build" {
		t.Fatalf("evidence object = kind %q parent %v; want evidence_record under item_build", ev.Kind, ev.ParentID)
	}
	if ev.X == nil || len(ev.X.Artifacts) != 1 {
		t.Fatalf("evidence object carries no artifact descriptor: %+v", ev.X)
	}
	a := ev.X.Artifacts[0]
	if a.Ref != evDigest || a.BoundObject != "ev_1" || a.Title != "dataset.md" || a.MediaType != "text/markdown" || a.CurrentLevel != "source" {
		t.Fatalf("artifact descriptor = %+v", a)
	}
	if !hasAction(ev, "OPEN_ARTIFACT") {
		t.Fatalf("evidence is offered no OPEN_ARTIFACT action: %+v", ev.Actions)
	}
	for _, act := range ev.Actions {
		if act.Consequential {
			t.Fatalf("opening evidence must not be consequential: %+v", act)
		}
	}
}

func TestBuildBindsNoArtifactWithoutAListedOne(t *testing.T) {
	facts := factsWithCapturedArtifact()
	facts.RunArtifacts = nil
	snap := Build(facts)
	for _, o := range snap.VisibleObjects {
		if o.Kind == "evidence_record" || o.X != nil {
			t.Fatalf("an artifact binding was invented for %q without a listed artifact", o.ID)
		}
	}
	// And the wire shape of every other object is unchanged: no x key at all.
	raw, _ := json.Marshal(snap)
	if strings.Contains(string(raw), `"x":`) {
		t.Fatalf("snapshot carries an x extension without artifacts: %s", raw)
	}
}

func TestBuildDoesNotBindArtifactsOfARunWithoutAHorizonParent(t *testing.T) {
	facts := factsWithCapturedArtifact()
	facts.Runs[0].PlanItemID = "item_missing"
	for _, o := range Build(facts).VisibleObjects {
		if o.Kind == "evidence_record" {
			t.Fatalf("evidence bound to a run with no horizon parent: %q", o.ID)
		}
	}
}

func TestBuildNeverReusesAnExistingObjectIDForEvidence(t *testing.T) {
	facts := factsWithCapturedArtifact()
	facts.RunArtifacts["run_1"][0].EvidenceID = "item_build"
	snap := Build(facts)
	n := 0
	for _, o := range snap.VisibleObjects {
		if o.ID == "item_build" {
			n++
		}
	}
	if n != 1 {
		t.Fatalf("object id item_build appears %d times", n)
	}
}

type countingLister struct {
	calls int
	refs  []ports.RunArtifactRef
	err   error
}

func (c *countingLister) RunArtifacts(_ context.Context, _ string) ([]ports.RunArtifactRef, error) {
	c.calls++
	return c.refs, c.err
}

func TestRunArtifactsAreCachedPerEvidenceCountAndFailuresBindNothing(t *testing.T) {
	lister := &countingLister{refs: []ports.RunArtifactRef{{EvidenceID: "ev_1", URI: "a.md", Digest: evDigest}}}
	src := &LiveSource{artifacts: lister, artifactCache: map[string]cachedRunArtifacts{}}
	runs := []ports.RunSummary{{RunID: "run_1", EvidenceCount: 1}, {RunID: "run_2", EvidenceCount: 0}}
	for i := 0; i < 3; i++ {
		got := src.runArtifacts(context.Background(), runs)
		if len(got["run_1"]) != 1 || len(got) != 1 {
			t.Fatalf("pass %d: artifacts = %+v", i, got)
		}
	}
	if lister.calls != 1 {
		t.Fatalf("the same evidence count must be read once, got %d reads", lister.calls)
	}
	runs[0].EvidenceCount = 2
	src.runArtifacts(context.Background(), runs)
	if lister.calls != 2 {
		t.Fatalf("a grown evidence journal must be re-read, got %d reads", lister.calls)
	}

	failing := &LiveSource{artifacts: &countingLister{err: context.DeadlineExceeded}, artifactCache: map[string]cachedRunArtifacts{}}
	if got := failing.runArtifacts(context.Background(), runs); len(got) != 0 {
		t.Fatalf("an unreadable run must bind nothing, got %+v", got)
	}
	if got := (&LiveSource{}).runArtifacts(context.Background(), runs); got != nil {
		t.Fatalf("an authority without the capability must bind nothing, got %+v", got)
	}
}
