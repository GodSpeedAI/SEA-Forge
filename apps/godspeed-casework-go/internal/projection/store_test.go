package projection

import (
	"errors"
	"testing"
	"time"
)

func newFixtureStore(t *testing.T) *Store {
	t.Helper()
	ds, err := Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	store, err := NewStore(ds)
	if err != nil {
		t.Fatalf("build store: %v", err)
	}
	return store
}

func TestLiveIsLatestRevision(t *testing.T) {
	store := newFixtureStore(t)
	if got := store.LiveCursor(); got != 1150 {
		t.Fatalf("live cursor: got %d, want 1150", got)
	}
	snap := store.Live()
	if snap.Cursor != 1150 {
		t.Fatalf("live snapshot cursor: got %d", snap.Cursor)
	}
	if snap.Provenance != ProvenanceLabel {
		t.Fatalf("live snapshot provenance: got %q", snap.Provenance)
	}
}

func TestAtKnownAndUnknownCursor(t *testing.T) {
	store := newFixtureStore(t)
	snap, err := store.At(900)
	if err != nil {
		t.Fatalf("At(900): %v", err)
	}
	if snap.Cursor != 900 || snap.Provenance != ProvenanceLabel {
		t.Fatalf("At(900) returned cursor=%d provenance=%q", snap.Cursor, snap.Provenance)
	}
	if _, err := store.At(901); !errors.Is(err, ErrUnknownCursor) {
		t.Fatalf("At(901) must fail with ErrUnknownCursor, got %v", err)
	}
}

func TestWindowIsMonotonic(t *testing.T) {
	store := newFixtureStore(t)
	window := store.Window()
	if len(window) != 7 {
		t.Fatalf("window length: got %d, want 7", len(window))
	}
	for i := 1; i < len(window); i++ {
		if window[i].Cursor <= window[i-1].Cursor {
			t.Fatalf("window cursors not strictly monotonic at %d: %d then %d", i, window[i-1].Cursor, window[i].Cursor)
		}
	}
	if window[0].Cursor != 900 || window[len(window)-1].Cursor != 1150 {
		t.Fatalf("window bounds: %d..%d", window[0].Cursor, window[len(window)-1].Cursor)
	}
	if window[0].At == "" || window[0].Summary == "" {
		t.Fatalf("window entries must carry at and summary: %+v", window[0])
	}
}

func TestAppendEnforcesStrictMonotonicCursors(t *testing.T) {
	store := newFixtureStore(t)
	if err := store.Append(Revision{Cursor: 1150}); err == nil {
		t.Fatal("appending the live cursor again must be refused")
	}
	if err := store.Append(Revision{Cursor: 1149}); err == nil {
		t.Fatal("appending a cursor below live must be refused")
	}
	if err := store.Append(Revision{Cursor: 1151, Summary: "first appended", At: time.Now().UTC().Format(time.RFC3339)}); err != nil {
		t.Fatalf("append 1151: %v", err)
	}
	if got := store.LiveCursor(); got != 1151 {
		t.Fatalf("live cursor after append: got %d", got)
	}
	if _, err := store.At(1151); err != nil {
		t.Fatalf("At(1151) after append: %v", err)
	}
	if got := len(store.Window()); got != 8 {
		t.Fatalf("window length after append: got %d, want 8", got)
	}
}

func TestNewStoreRejectsInconsistentDataset(t *testing.T) {
	ds, err := Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	ds.LiveCursor = 9999
	if _, err := NewStore(ds); err == nil {
		t.Fatal("a dataset whose liveCursor disagrees with its last revision must be rejected")
	}
	ds2, err := Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	ds2.Revisions = append(ds2.Revisions, ds2.Revisions[len(ds2.Revisions)-1])
	if _, err := NewStore(ds2); err == nil {
		t.Fatal("a dataset with a repeated cursor must be rejected")
	}
	if _, err := NewStore(Dataset{}); err == nil {
		t.Fatal("an empty dataset must be rejected")
	}
}

// Handed-out snapshots must be deep copies: mutating one may never reach stored state or other
// callers.
func TestSnapshotsAreDeepCopies(t *testing.T) {
	store := newFixtureStore(t)
	a := store.Live()
	before := store.Live()
	if len(a.Objects) == 0 || len(a.Surfaces) == 0 {
		t.Fatal("fixture snapshot is unexpectedly empty")
	}
	a.Objects[0].Note = "mutated"
	a.Objects[0].Position.X = -99
	a.Surfaces[0].ObjectIDs[0] = "clobbered"
	after := store.Live()
	if after.Objects[0].Note != before.Objects[0].Note {
		t.Fatalf("object mutation escaped the snapshot copy: %q", after.Objects[0].Note)
	}
	if after.Objects[0].Position.X != before.Objects[0].Position.X {
		t.Fatalf("position mutation escaped the snapshot copy: %v", after.Objects[0].Position.X)
	}
	if after.Surfaces[0].ObjectIDs[0] != before.Surfaces[0].ObjectIDs[0] {
		t.Fatalf("surface mutation escaped the snapshot copy: %q", after.Surfaces[0].ObjectIDs[0])
	}
}

func TestSubscribeReplaysThenStreamsLive(t *testing.T) {
	store := newFixtureStore(t)
	ch, cancel := store.Subscribe(1070)
	defer cancel()

	wantReplay := []int64{1110, 1150}
	for _, want := range wantReplay {
		select {
		case snap := <-ch:
			if snap.Cursor != want {
				t.Fatalf("replay order: got cursor %d, want %d", snap.Cursor, want)
			}
		case <-time.After(2 * time.Second):
			t.Fatalf("timed out waiting for replay of %d", want)
		}
	}

	if err := store.Append(Revision{Cursor: 1151, Summary: "live"}); err != nil {
		t.Fatalf("append: %v", err)
	}
	select {
	case snap := <-ch:
		if snap.Cursor != 1151 {
			t.Fatalf("live event: got cursor %d, want 1151", snap.Cursor)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("timed out waiting for the live revision")
	}

	if got := store.SubscriberCount(); got != 1 {
		t.Fatalf("subscriber count: got %d, want 1", got)
	}
	cancel()
	if got := store.SubscriberCount(); got != 0 {
		t.Fatalf("subscriber count after cancel: got %d, want 0", got)
	}
	cancel() // must be safe to call twice
}

func TestSubscribeWithoutReplay(t *testing.T) {
	store := newFixtureStore(t)
	ch, cancel := store.Subscribe(1150)
	defer cancel()
	if err := store.Append(Revision{Cursor: 1151}); err != nil {
		t.Fatalf("append: %v", err)
	}
	select {
	case snap := <-ch:
		if snap.Cursor != 1151 {
			t.Fatalf("expected only the live revision, got cursor %d", snap.Cursor)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("timed out waiting for the live revision")
	}
}
