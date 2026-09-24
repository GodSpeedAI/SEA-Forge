package projection

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"testing"
)

// canonicalFixture is the sibling app's canonical dataset, relative to this package's directory.
// When the cognitive-ui app is absent (isolated checkout), the drift guard skips rather than
// fails: the embedded copy is still pinned by the round-trip golden test below.
const canonicalFixture = "../../../godspeed-cognitive-ui/src/adapters/fixture/northstar/northstar.world.json"

// TestEmbeddedFixtureMatchesCanonical is the drift guard: the embedded copy must be byte-equal to
// the canonical fixture dataset, so this server can never serve a projection the cognitive-ui
// fixture directory does not hold.
func TestEmbeddedFixtureMatchesCanonical(t *testing.T) {
	raw, err := os.ReadFile(filepath.Clean(canonicalFixture))
	if err != nil {
		if os.IsNotExist(err) {
			t.Skipf("canonical fixture absent in this checkout (%s); drift guard skipped", canonicalFixture)
		}
		t.Fatalf("read canonical fixture: %v", err)
	}
	if !bytes.Equal(raw, FixtureBytes()) {
		t.Fatal("embedded fixturedata/northstar.world.json differs from the canonical dataset; copy the canonical file again")
	}
}

// TestFixtureDecodes pins the dataset's own headline facts, so a malformed embed fails loudly here
// rather than somewhere deep in a server test.
func TestFixtureDecodes(t *testing.T) {
	ds, err := Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	if ds.Provenance != "fixture:northstar" {
		t.Fatalf("dataset provenance: got %q", ds.Provenance)
	}
	if ds.LiveCursor != 1150 {
		t.Fatalf("dataset liveCursor: got %d, want 1150", ds.LiveCursor)
	}
	if len(ds.Revisions) != 7 {
		t.Fatalf("dataset revisions: got %d, want 7", len(ds.Revisions))
	}
	if first, last := ds.Revisions[0].Cursor, ds.Revisions[6].Cursor; first != 900 || last != 1150 {
		t.Fatalf("dataset revision cursors run %d..%d, want 900..1150", first, last)
	}
	if len(ds.Artifacts) != 9 {
		t.Fatalf("dataset artifacts: got %d, want 9", len(ds.Artifacts))
	}
}

// TestFixtureRoundTripGolden proves the JSON tags capture every field of the canonical dataset:
// decode, re-encode, and compare semantically against the original document. A dropped or renamed
// field cannot survive this comparison.
func TestFixtureRoundTripGolden(t *testing.T) {
	ds, err := Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	reencoded, err := json.Marshal(ds)
	if err != nil {
		t.Fatalf("re-encode dataset: %v", err)
	}
	var want, got any
	if err := json.Unmarshal(FixtureBytes(), &want); err != nil {
		t.Fatalf("decode original: %v", err)
	}
	if err := json.Unmarshal(reencoded, &got); err != nil {
		t.Fatalf("decode re-encoded: %v", err)
	}
	if !reflect.DeepEqual(want, got) {
		t.Fatalf("dataset did not round-trip through the projection types;\noriginal:    %s\nre-encoded: %s", truncate(FixtureBytes()), truncate(reencoded))
	}
}

// TestSnapshotJSONTagsIsExact pins the served wire shapes (camelCase, optional parentId and
// attention) as a literal golden string.
func TestSnapshotJSONTagsIsExact(t *testing.T) {
	parent := "eng-northstar"
	snap := Snapshot{
		Cursor: 1150,
		Surfaces: []Surface{
			{ID: "surface-northstar", Label: "Northstar pilot", ObjectIDs: []string{"ns-goal", "ns-migration"}},
		},
		Objects: []Object{
			{
				ID: "eng-northstar", Kind: "engagement", Label: "Northstar Health",
				Position: Position{X: 5.2, Y: 1.4, Depth: 0.5},
				Salience: 1, Note: "Claims pilot - one path unresolved", Attention: "notable",
			},
			{
				ID: "ns-goal", Kind: "goal", Label: "Customer goal",
				Position: Position{X: -2.6, Y: 2.4, Depth: 0.4},
				Salience: 0.62, ParentID: &parent,
				Note: "Straight-through claims without losing coverage accuracy",
			},
		},
		Relationships: []Relationship{{From: "eng-northstar", To: "ns-goal", Kind: "intends"}},
		Provenance:    ProvenanceLabel,
	}
	got, err := json.Marshal(snap)
	if err != nil {
		t.Fatalf("marshal snapshot: %v", err)
	}
	want := `{"cursor":1150,"surfaces":[{"id":"surface-northstar","label":"Northstar pilot","objectIds":["ns-goal","ns-migration"]}],"objects":[{"id":"eng-northstar","kind":"engagement","label":"Northstar Health","position":{"x":5.2,"y":1.4,"depth":0.5},"salience":1,"note":"Claims pilot - one path unresolved","attention":"notable"},{"id":"ns-goal","kind":"goal","label":"Customer goal","position":{"x":-2.6,"y":2.4,"depth":0.4},"salience":0.62,"parentId":"eng-northstar","note":"Straight-through claims without losing coverage accuracy"}],"relationships":[{"from":"eng-northstar","to":"ns-goal","kind":"intends"}],"provenance":"go:fixture:northstar"}`
	if string(got) != want {
		t.Fatalf("snapshot wire shape drifted:\n got: %s\nwant: %s", got, want)
	}
}

func truncate(b []byte) string {
	const limit = 400
	if len(b) <= limit {
		return string(b)
	}
	return string(b[:limit]) + "..."
}
