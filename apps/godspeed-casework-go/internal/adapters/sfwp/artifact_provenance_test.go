package sfwp

import (
	"bufio"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"net"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestGetArtifactResolvesCasePlanItemAndEvidenceThroughRunGet(t *testing.T) {
	data := "# report\n"
	sum := sha256.Sum256([]byte(data))
	digest := hex.EncodeToString(sum[:])
	clientSide, authoritySide := net.Pipe()
	defer authoritySide.Close()
	requests := make(chan map[string]any, 2)
	serverErrors := make(chan error, 1)
	go func() {
		defer authoritySide.Close()
		reader := bufio.NewReader(authoritySide)
		responses := []map[string]any{
			{"digest": digest, "run_id": "run-17", "evidence_id": "evi-17", "uri": "artifacts/report.md", "size_bytes": len(data), "content": data},
			{
				"run_id": "run-17", "case_id": "case-17", "plan_item_id": "item-17",
				"evidence": []map[string]any{{
					"evidence_id": "evi-17", "kind": "artifact", "uri": "artifacts/report.md",
					"sha256": digest, "source_event_id": "evt-capture-17",
				}},
				"trace": []map[string]any{{
					"event_id": "evt-capture-17", "kind": "artifact_captured", "plan_item_id": "item-17",
				}},
			},
		}
		for _, response := range responses {
			line, err := reader.ReadBytes('\n')
			if err != nil {
				serverErrors <- err
				return
			}
			var request map[string]any
			if err := json.Unmarshal(line, &request); err != nil {
				serverErrors <- err
				return
			}
			requests <- request
			if err := json.NewEncoder(authoritySide).Encode(response); err != nil {
				serverErrors <- err
				return
			}
		}
		serverErrors <- nil
	}()

	client, err := New(Config{
		SocketPath: "test-authority",
		MaxConns:   1,
		Dial: func(context.Context, string) (net.Conn, error) {
			return clientSide, nil
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	content, err := NewAuthority(client).GetArtifact(context.Background(), digest)
	if err != nil {
		t.Fatal(err)
	}
	if content.CaseID != "case-17" || content.PlanItemID != "item-17" || content.RunID != "run-17" || content.EvidenceID != "evi-17" {
		t.Fatalf("artifact ownership did not come from its run and evidence records: %+v", content)
	}
	if content.Digest != digest || content.Size != int64(len(data)) || string(content.Data) != data {
		t.Fatalf("artifact bytes or integrity metadata changed during provenance resolution: %+v", content)
	}
	first, second := <-requests, <-requests
	if first["verb"] != "artifact_get" || second["verb"] != "run_get" || second["run_id"] != "run-17" {
		t.Fatalf("expected artifact.get followed by run.get for its returned run id, got %#v then %#v", first, second)
	}
	if err := <-serverErrors; err != nil {
		t.Fatal(err)
	}
}

func TestArtifactProvenanceRefusesMismatchedOrMissingOwnedRun(t *testing.T) {
	caseID, itemID := "case-17", "item-17"
	digest, uri := "sha256:abc", "artifacts/report.md"
	artifact := ArtifactView{RunID: "run-17", EvidenceID: "evi-17", Digest: digest, URI: uri}
	ownedRun := RunArtifactProvenanceView{
		RunID: "run-17", CaseID: &caseID, PlanItemID: &itemID,
		Evidence: []RunArtifactEvidenceView{{
			EvidenceID: "evi-17", Kind: "artifact", URI: uri, SHA256: &digest, SourceEventID: "evt-17",
		}},
		Trace: []RunArtifactTraceView{{EventID: "evt-17", Kind: "artifact_captured", PlanItemID: &itemID}},
	}
	if gotCase, gotItem, err := artifactProvenanceOf(artifact, ownedRun); err != nil || gotCase != caseID || gotItem != itemID {
		t.Fatalf("actual run/item/evidence association should resolve, got case=%q item=%q err=%v", gotCase, gotItem, err)
	}

	for name, run := range map[string]RunArtifactProvenanceView{
		"different run": {RunID: "run-other", CaseID: &caseID, PlanItemID: &itemID},
		"missing run":   {},
		"missing case":  {RunID: "run-17", PlanItemID: &itemID},
		"missing item":  {RunID: "run-17", CaseID: &caseID},
	} {
		t.Run(name, func(t *testing.T) {
			if _, _, err := artifactProvenanceOf(artifact, run); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
				t.Fatalf("unavailable or mismatched ownership must be refused as unavailable, got %v", err)
			}
		})
	}
	if err := artifactOwnershipError(apperr.New(apperr.KindInvalid, "", "run_get", "run not found")); apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("missing owning run from run.get must remain a typed unavailable refusal, got %v", err)
	}
}

func TestArtifactProvenanceRefusesEvidenceFromAnotherItem(t *testing.T) {
	caseID, itemID, otherItemID := "case-17", "item-17", "item-other"
	digest, uri := "abc", "artifacts/report.md"
	artifact := ArtifactView{RunID: "run-17", EvidenceID: "evi-17", Digest: digest, URI: uri}
	run := RunArtifactProvenanceView{
		RunID: "run-17", CaseID: &caseID, PlanItemID: &itemID,
		Evidence: []RunArtifactEvidenceView{{
			EvidenceID: "evi-17", Kind: "artifact", URI: uri, SHA256: &digest, SourceEventID: "evt-17",
		}},
		Trace: []RunArtifactTraceView{{EventID: "evt-17", Kind: "artifact_captured", PlanItemID: &otherItemID}},
	}
	if _, _, err := artifactProvenanceOf(artifact, run); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("evidence whose capture event names another plan item must be refused, got %v", err)
	}
}

func TestArtifactOwnershipErrorPreservesAuthorityDenial(t *testing.T) {
	denied := apperr.New(apperr.KindAuthorityDenied, "", "run_get", "denied")
	if got := artifactOwnershipError(denied); !errors.Is(got, denied) || apperr.KindOf(got) != apperr.KindAuthorityDenied {
		t.Fatalf("run.get authority refusal must remain attributable, got %v", got)
	}
}

// A stored artifact whose bytes no longer hash to its digest is refused by the kernel with an
// artifact_integrity_error. The adapter must type that as an integrity failure (never an outage,
// never content) so the gateway can answer integrity_mismatch.
func TestGetArtifactTypesKernelIntegrityErrorAsIntegrityFailure(t *testing.T) {
	digest := "90999fcee4382f5d72e4ba9034d9427ad861c3202e483516003afa20aa7faa56"
	clientSide, authoritySide := net.Pipe()
	defer authoritySide.Close()
	go func() {
		reader := bufio.NewReader(authoritySide)
		if _, err := reader.ReadBytes('\n'); err != nil {
			return
		}
		_ = json.NewEncoder(authoritySide).Encode(map[string]any{
			"error":       "internal error: artifact_integrity_error: content at artifacts/x.md hashes to abc, not " + digest,
			"error_class": "internal",
		})
	}()
	client, err := New(Config{SocketPath: "test-authority", MaxConns: 1, Dial: func(context.Context, string) (net.Conn, error) { return clientSide, nil }})
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	content, err := NewAuthority(client).GetArtifact(context.Background(), digest)
	if !errors.Is(err, ports.ErrArtifactIntegrity) {
		t.Fatalf("kernel artifact_integrity_error must map to ports.ErrArtifactIntegrity, got %v", err)
	}
	if len(content.Data) != 0 {
		t.Fatalf("no content may accompany an integrity failure: %+v", content)
	}
}
