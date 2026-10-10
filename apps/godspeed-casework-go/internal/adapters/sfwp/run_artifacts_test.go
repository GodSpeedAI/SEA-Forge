package sfwp

import "testing"

func str(s string) *string { return &s }

func TestRunArtifactRefsKeepsOnlyDigestedArtifactEvidence(t *testing.T) {
	hex := "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
	run := RunArtifactProvenanceView{
		RunID: "run_1",
		Evidence: []RunArtifactEvidenceView{
			{EvidenceID: "ev_1", Kind: "artifact", URI: "work/a.md", SHA256: str("sha256:" + hex), SourceEventID: "e1"},
			{EvidenceID: "ev_2", Kind: "artifact", URI: "work/b.md", SHA256: str(hex), SourceEventID: "e2"},
			{EvidenceID: "ev_3", Kind: "authority_decision", URI: "x", SHA256: str(hex), SourceEventID: "e3"},
			{EvidenceID: "ev_4", Kind: "artifact", URI: "work/c.md", SHA256: nil, SourceEventID: "e4"},
			{EvidenceID: "ev_5", Kind: "artifact", URI: "work/d.md", SHA256: str("sha256:nothex"), SourceEventID: "e5"},
			{EvidenceID: "ev_6", Kind: "artifact", URI: "work/e.md", SHA256: str(hex), SourceEventID: ""},
		},
	}
	got, err := runArtifactRefs("run_1", run)
	if err != nil {
		t.Fatal(err)
	}
	if len(got) != 2 || got[0].EvidenceID != "ev_1" || got[1].EvidenceID != "ev_2" {
		t.Fatalf("refs = %+v", got)
	}
	for _, r := range got {
		if r.Digest != "sha256:"+hex {
			t.Fatalf("digest not normalized: %q", r.Digest)
		}
	}
}

func TestRunArtifactRefsRefusesAMismatchedRun(t *testing.T) {
	if _, err := runArtifactRefs("run_1", RunArtifactProvenanceView{RunID: "run_2"}); err == nil {
		t.Fatal("a run record for a different run must be refused")
	}
}
