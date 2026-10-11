package projection

import (
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestStoreDeepCopiesUnreadableRunIDs(t *testing.T) {
	store := NewStore()
	facts := &CaseFacts{
		Record:           ports.CaseRecord{Ref: "case_1"},
		UnreadableRunIDs: []string{"run_unreadable"},
	}
	if err := store.Append(Revision{Cursor: "01AAA", CaseID: "case_1", Snapshot: emptyTestSnapshot("01AAA", "case_1"), Facts: facts}); err != nil {
		t.Fatal(err)
	}
	facts.UnreadableRunIDs[0] = "mutated source"

	first, err := store.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	if len(first.Facts.UnreadableRunIDs) != 1 || first.Facts.UnreadableRunIDs[0] != "run_unreadable" {
		t.Fatalf("append retained mutable unreadable IDs: %v", first.Facts.UnreadableRunIDs)
	}
	first.Facts.UnreadableRunIDs[0] = "mutated returned read"

	second, err := store.At("01AAA")
	if err != nil {
		t.Fatal(err)
	}
	if len(second.Facts.UnreadableRunIDs) != 1 || second.Facts.UnreadableRunIDs[0] != "run_unreadable" {
		t.Fatalf("returned facts exposed retained unreadable IDs: %v", second.Facts.UnreadableRunIDs)
	}
}
