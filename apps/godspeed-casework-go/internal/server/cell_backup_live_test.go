//go:build live

// T11 backup/restore, proven against the REAL kernel: a cell with a committed case, an executed
// item (request record, delegation-audit ledger) is stopped gracefully, backed up with
// scripts/casework-cell-backup.sh, wiped, restored with scripts/casework-cell-restore.sh into a
// new location, and a fresh kernel started on the restored cell must serve the identical durable
// records. Teeth: backup refuses a live cell; restore refuses a tampered archive and a non-empty
// target.
package server_test

import (
	"context"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livestack"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
)

func runScript(t *testing.T, name string, args ...string) (string, error) {
	t.Helper()
	cmd := exec.Command(filepath.Join(livetest.RepoRoot(t), "scripts", name), args...)
	out, err := cmd.CombinedOutput()
	return string(out), err
}

func TestLiveCellBackupWipeRestoreServesIdenticalRecords(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	ts := stack.HTTPServer(t)
	op := stack.Login(t, ts.URL, "operator")

	caseID := stack.CommitSentryChain(t, "gw-t11-bk-commit")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)
	const intentID = "gw-t11-bk-exec"
	resp, err := op.PostJSON("/api/intents", contract.InteractionIntent{
		IntentID: intentID, Kind: "CONSEQUENTIAL_CASE", ActionName: "EXECUTE_ITEM",
		TargetObjectID: "task_prepare", CaseID: caseID, ClientCursor: cursor,
		Actor:      contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters: map[string]any{"item_id": "task_prepare"},
	})
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	livetest.WaitUntil(t, 15*time.Second, "durable item_activated trace", func() bool {
		return cell.CountTraceKinds(caseID, "item_activated") >= 1
	})
	livetest.WaitUntil(t, 15*time.Second, "delegation-audit entry", func() bool {
		return cell.FindDelegationRecord(intentID) != nil
	})
	wantEvents := cell.CaseEvents(caseID)
	wantRequest := cell.RequestRecord(intentID)
	wantAudit := cell.FindDelegationRecord(intentID)
	if len(wantEvents) == 0 || wantRequest == nil || wantAudit == nil {
		t.Fatalf("pre-backup state incomplete: events=%d request=%v audit=%v", len(wantEvents), wantRequest, wantAudit)
	}

	work := t.TempDir()
	archive := filepath.Join(work, "cell.tar.gz")

	// Tooth 1: a live cell cannot be backed up (the kernel holds the cell lock).
	if out, err := runScript(t, "casework-cell-backup.sh", cell.Root(), archive); err == nil {
		t.Fatalf("backup of a LIVE cell must be refused, got success:\n%s", out)
	} else if !strings.Contains(out, "cell is live") {
		t.Fatalf("refusal must say why:\n%s", out)
	}
	if _, err := os.Stat(archive); err == nil {
		t.Fatal("a refused backup must leave no archive")
	}

	// Quiesce (graceful SIGTERM), then back up.
	stack.Cancel()
	cell.StopGraceful()
	if out, err := runScript(t, "casework-cell-backup.sh", cell.Root(), archive); err != nil {
		t.Fatalf("backup failed: %v\n%s", err, out)
	}
	for _, sidecar := range []string{archive + ".sha256", archive + ".manifest"} {
		if _, err := os.Stat(sidecar); err != nil {
			t.Fatalf("missing %s: %v", sidecar, err)
		}
	}
	if info, _ := os.Stat(archive); info.Mode().Perm() != 0o600 {
		t.Fatalf("archive holds keys and approvals: want mode 0600, got %v", info.Mode().Perm())
	}

	// Wipe the original cell completely.
	if err := os.RemoveAll(cell.Root()); err != nil {
		t.Fatal(err)
	}

	// Tooth 2: a tampered archive is refused before anything is extracted.
	tampered := filepath.Join(work, "tampered.tar.gz")
	raw, _ := os.ReadFile(archive)
	raw[len(raw)/2] ^= 0xff
	if err := os.WriteFile(tampered, raw, 0o600); err != nil {
		t.Fatal(err)
	}
	for _, ext := range []string{".sha256", ".manifest"} {
		side, _ := os.ReadFile(archive + ext)
		_ = os.WriteFile(tampered+ext, []byte(strings.ReplaceAll(string(side), "cell.tar.gz", "tampered.tar.gz")), 0o600)
	}
	refused := filepath.Join(work, "refused-cell")
	if out, err := runScript(t, "casework-cell-restore.sh", tampered, refused); err == nil {
		t.Fatalf("a tampered archive must be refused:\n%s", out)
	}
	if _, err := os.Stat(filepath.Join(refused, "server.yaml")); err == nil {
		t.Fatal("a refused restore must extract nothing")
	}

	// Tooth 3: restore never writes into a non-empty directory.
	occupied := filepath.Join(work, "occupied")
	if err := os.MkdirAll(occupied, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(occupied, "keep.txt"), []byte("x"), 0o600); err != nil {
		t.Fatal(err)
	}
	if out, err := runScript(t, "casework-cell-restore.sh", archive, occupied); err == nil {
		t.Fatalf("restore into a non-empty directory must be refused:\n%s", out)
	}

	// Restore, start a fresh kernel on it, compare the durable records.
	restored := filepath.Join(work, "restored")
	if out, err := runScript(t, "casework-cell-restore.sh", archive, restored); err != nil {
		t.Fatalf("restore failed: %v\n%s", err, out)
	}
	back := livetest.NewCellAt(t, restored)
	if got := back.CaseEvents(caseID); !reflect.DeepEqual(got, wantEvents) {
		t.Fatalf("restored case ledger differs:\n got %d events\nwant %d events", len(got), len(wantEvents))
	}
	if got := back.RequestRecord(intentID); !reflect.DeepEqual(got, wantRequest) {
		t.Fatalf("restored request record differs: %v vs %v", got, wantRequest)
	}
	if got := back.FindDelegationRecord(intentID); !reflect.DeepEqual(got, wantAudit) {
		t.Fatalf("restored delegation-audit differs: %v vs %v", got, wantAudit)
	}
	// The restored kernel serves the case through the real stack, and still enforces idempotency:
	// replaying the executed intent's request id must not run it a second time.
	restoredStack := livestack.AssembleStack(t, back)
	newest, err := restoredStack.Source.NewestCaseID(context.Background())
	if err != nil || newest != caseID {
		t.Fatalf("restored kernel newest case = %q (%v), want %q", newest, err, caseID)
	}
	before := back.CountTraceKinds(caseID, "item_activated")
	ts2 := restoredStack.HTTPServer(t)
	op2 := restoredStack.Login(t, ts2.URL, "operator")
	cursor2, _ := restoredStack.Relay.CursorForCase(caseID)
	replay, err := op2.PostJSON("/api/intents", contract.InteractionIntent{
		IntentID: intentID, Kind: "CONSEQUENTIAL_CASE", ActionName: "EXECUTE_ITEM",
		TargetObjectID: "task_prepare", CaseID: caseID, ClientCursor: cursor2,
		Actor:      contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters: map[string]any{"item_id": "task_prepare"},
	})
	if err != nil {
		t.Fatal(err)
	}
	replay.Body.Close()
	time.Sleep(500 * time.Millisecond)
	if after := back.CountTraceKinds(caseID, "item_activated"); after != before {
		t.Fatalf("replaying a pre-backup request id on the restored cell re-executed it (%d -> %d)", before, after)
	}
}
