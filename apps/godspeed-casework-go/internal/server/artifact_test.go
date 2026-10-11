package server

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"net/http"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

type fakeArtifactGetter struct {
	content ports.ArtifactContent
	err     error
	calls   int
	digest  string
}

func (f *fakeArtifactGetter) GetArtifact(_ context.Context, digest string) (ports.ArtifactContent, error) {
	f.calls++
	f.digest = digest
	return f.content, f.err
}

func artifactTestDigest(data []byte) string {
	sum := sha256.Sum256(data)
	return hex.EncodeToString(sum[:])
}

func TestArtifactGetReturnsVerifiedCanonicalPayloadAndSessionPerspective(t *testing.T) {
	data := []byte("# Résumé ☃\n")
	digest := artifactTestDigest(data)
	getter := &fakeArtifactGetter{content: ports.ArtifactContent{
		Digest: digest, EvidenceID: "evi-17", RunID: "run-17", CaseID: "case-17", PlanItemID: "item-17", URI: "artifacts/work/résumé.md",
		Size: int64(len(data)), Data: data,
	}}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	var verified ports.ActorClaim
	h.world.verify = func(actor ports.ActorClaim) error { verified = actor; return nil }
	operator := h.login(t, "operator", "ignored-in-dev")

	resp, err := operator.get("/api/artifacts/sha256:" + digest)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("artifact get: got %d, want 200", resp.StatusCode)
	}
	var got artifactPayload
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if got.EvidenceID != "evi-17" || got.Name != "résumé.md" || got.Digest != "sha256:"+digest || got.ContentType != "text/markdown" {
		t.Fatalf("artifact metadata was not preserved: %+v", got)
	}
	if got.Content != string(data) || got.Provenance.RunID != "run-17" {
		t.Fatalf("artifact text or run provenance was lost: %+v", got)
	}
	if got.Provenance.CaseID != "case-17" || got.Provenance.PlanItemID != "item-17" || got.Provenance.InvocationID != "" {
		t.Fatalf("source provenance was lost or an invocation was fabricated: %+v", got.Provenance)
	}
	if getter.calls != 1 || getter.digest != digest {
		t.Fatalf("kernel must receive normalized digest once, calls=%d digest=%q", getter.calls, getter.digest)
	}
	if verified != (ports.ActorClaim{ActorID: "operator_local", Role: "operator"}) {
		t.Fatalf("delegation check must use the session actor, got %+v", verified)
	}
}

func TestArtifactGetPreservesEmptyText(t *testing.T) {
	digest := artifactTestDigest(nil)
	getter := &fakeArtifactGetter{content: ports.ArtifactContent{
		Digest: digest, EvidenceID: "evi-empty", RunID: "run-empty", CaseID: "case-empty", PlanItemID: "item-empty", URI: "work/empty.txt", Size: 0, Data: []byte{},
	}}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	operator := h.login(t, "operator", "ignored-in-dev")
	resp, err := operator.get("/api/artifacts/" + digest)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var got artifactPayload
	if err := json.NewDecoder(resp.Body).Decode(&got); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusOK || got.Content != "" || got.ContentType != "text/plain" {
		t.Fatalf("empty artifact must be a successful empty text payload: status=%d payload=%+v", resp.StatusCode, got)
	}
}

func TestArtifactGetMapsKernelNotFoundToTyped404(t *testing.T) {
	getter := &fakeArtifactGetter{err: apperr.New(apperr.KindInvalid, "", "artifact_get", "no artifact")}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	operator := h.login(t, "operator", "ignored-in-dev")
	digest := artifactTestDigest([]byte("missing"))
	resp, err := operator.get("/api/artifacts/" + digest)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var body errorBody
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusNotFound || body.Error.Kind != "not_found" {
		t.Fatalf("missing artifact must be a typed 404, got %d %+v", resp.StatusCode, body.Error)
	}
}

func TestArtifactGetMapsKernelIntegrityFailureToTypedIntegrityMismatch(t *testing.T) {
	getter := &fakeArtifactGetter{err: apperr.Wrap(apperr.KindInternal, "", "artifact.get", "the stored artifact no longer matches its digest", ports.ErrArtifactIntegrity)}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	operator := h.login(t, "operator", "ignored-in-dev")
	resp, err := operator.get("/api/artifacts/" + artifactTestDigest([]byte("corrupted")))
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var body errorBody
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusBadGateway || body.Error.Kind != "integrity_mismatch" {
		t.Fatalf("a corrupted stored artifact must be a typed integrity_mismatch, got %d %+v", resp.StatusCode, body.Error)
	}
}

func TestArtifactGetChecksBytesAgainstRequestedDigest(t *testing.T) {
	wanted := []byte("expected")
	digest := artifactTestDigest(wanted)
	tampered := []byte("tampered")
	getter := &fakeArtifactGetter{content: ports.ArtifactContent{
		// Metadata repeats the requested digest, so only hashing the returned bytes catches this.
		Digest: digest, EvidenceID: "evi-17", RunID: "run-17", CaseID: "case-17", PlanItemID: "item-17", URI: "work/file.txt",
		Size: int64(len(tampered)), Data: tampered,
	}}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	operator := h.login(t, "operator", "ignored-in-dev")
	resp, err := operator.get("/api/artifacts/" + digest)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var body errorBody
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusBadGateway || body.Error.Kind != "integrity_mismatch" {
		t.Fatalf("tampered bytes must be rejected as an integrity mismatch, got %d %+v", resp.StatusCode, body.Error)
	}
}

func TestArtifactGetRejectsNonUTF8Bytes(t *testing.T) {
	data := []byte{0xff, 0xfe}
	digest := artifactTestDigest(data)
	getter := &fakeArtifactGetter{content: ports.ArtifactContent{
		Digest: digest, EvidenceID: "evi-binary", RunID: "run-binary", CaseID: "case-binary", PlanItemID: "item-binary", URI: "work/file.txt",
		Size: int64(len(data)), Data: data,
	}}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	operator := h.login(t, "operator", "ignored-in-dev")
	resp, err := operator.get("/api/artifacts/" + digest)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var body errorBody
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusUnsupportedMediaType || body.Error.Kind != "unsupported_artifact" {
		t.Fatalf("binary bytes must not be replacement-decoded as text: status=%d error=%+v", resp.StatusCode, body.Error)
	}
}

func TestArtifactGetRefusesUnresolvedCaseOrPlanItem(t *testing.T) {
	data := []byte("artifact")
	digest := artifactTestDigest(data)
	getter := &fakeArtifactGetter{content: ports.ArtifactContent{
		Digest: digest, EvidenceID: "evi-ownerless", RunID: "run-ownerless", URI: "work/file.txt",
		Size: int64(len(data)), Data: data,
	}}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	operator := h.login(t, "operator", "ignored-in-dev")
	resp, err := operator.get("/api/artifacts/" + digest)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var body errorBody
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusBadGateway || body.Error.Kind != "unavailable" {
		t.Fatalf("missing artifact ownership must be a typed unavailable refusal, got %d %+v", resp.StatusCode, body.Error)
	}
}

func TestArtifactContentTypeUsesSupportedTextKinds(t *testing.T) {
	for _, tc := range []struct {
		uri, body, want string
	}{
		{"report.md", "# report", "text/markdown"},
		{"report.json", `{"ok":true}`, "application/json"},
		{"report.json", "not json", "text/plain"},
		{"change.diff", "--- a/a", "text/x-diff"},
		{"report.bin", "plain text", "text/plain"},
	} {
		if got := artifactContentType(tc.uri, []byte(tc.body)); got != tc.want {
			t.Errorf("artifactContentType(%q) = %q, want %q", tc.uri, got, tc.want)
		}
	}
}

func TestArtifactGetRefusesUndelegatedSessionBeforeKernelRead(t *testing.T) {
	getter := &fakeArtifactGetter{content: ports.ArtifactContent{}}
	h := newLiveHarness(t)
	h.api.artifacts = getter
	h.world.verify = func(ports.ActorClaim) error {
		return apperr.New(apperr.KindAuthorityDenied, "", "identity", "delegation denied")
	}
	operator := h.login(t, "operator", "ignored-in-dev")
	resp, err := operator.get("/api/artifacts/" + artifactTestDigest(nil))
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusForbidden || getter.calls != 0 {
		t.Fatalf("delegation refusal must stop before artifact lookup: status=%d calls=%d", resp.StatusCode, getter.calls)
	}
}

func TestArtifactGetValidatesDigestAndRequiresSession(t *testing.T) {
	getter := &fakeArtifactGetter{}
	h := newLiveHarness(t)
	h.api.artifacts = getter

	resp, err := http.Get(h.ts.URL + "/api/artifacts/not-a-digest")
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("unauthenticated artifact read must fail before route validation, got %d", resp.StatusCode)
	}
	operator := h.login(t, "operator", "ignored-in-dev")
	resp, err = operator.get("/api/artifacts/not-a-digest")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusBadRequest || getter.calls != 0 {
		t.Fatalf("invalid digest must be rejected before authority call: status=%d calls=%d", resp.StatusCode, getter.calls)
	}
}
