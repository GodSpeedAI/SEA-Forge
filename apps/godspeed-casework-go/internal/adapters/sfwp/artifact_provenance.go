package sfwp

import (
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// artifactProvenanceOf joins artifact.get to the owning run.get records without inferring missing
// case, item, or capture-event links.
func artifactProvenanceOf(artifact ArtifactView, run RunArtifactProvenanceView) (string, string, error) {
	if artifact.RunID == "" || run.RunID != artifact.RunID {
		return "", "", ownershipUnavailable("the artifact's owning run record did not match")
	}
	if run.CaseID == nil || *run.CaseID == "" || run.PlanItemID == nil || *run.PlanItemID == "" {
		return "", "", ownershipUnavailable("the artifact's owning run has no case or plan item")
	}
	if artifact.EvidenceID == "" || artifact.URI == "" || artifact.Digest == "" {
		return "", "", ownershipUnavailable("the artifact response has incomplete evidence identity")
	}

	var matched *RunArtifactEvidenceView
	for i := range run.Evidence {
		row := &run.Evidence[i]
		if row.EvidenceID != artifact.EvidenceID {
			continue
		}
		if matched != nil {
			return "", "", ownershipUnavailable("the run contains ambiguous records for this evidence id")
		}
		matched = row
	}
	if matched == nil || matched.Kind != "artifact" || matched.URI != artifact.URI ||
		matched.SHA256 == nil || !sameArtifactDigest(*matched.SHA256, artifact.Digest) || matched.SourceEventID == "" {
		return "", "", ownershipUnavailable("the artifact evidence record did not match its owning run")
	}

	var source *RunArtifactTraceView
	for i := range run.Trace {
		row := &run.Trace[i]
		if row.EventID != matched.SourceEventID {
			continue
		}
		if source != nil {
			return "", "", ownershipUnavailable("the evidence source event is ambiguous in its owning run")
		}
		source = row
	}
	if source == nil || source.Kind != "artifact_captured" || source.PlanItemID == nil || *source.PlanItemID != *run.PlanItemID {
		return "", "", ownershipUnavailable("the artifact source event did not belong to the owning plan item")
	}

	return *run.CaseID, *run.PlanItemID, nil
}

func sameArtifactDigest(left, right string) bool {
	return strings.TrimPrefix(left, "sha256:") == strings.TrimPrefix(right, "sha256:")
}

func artifactOwnershipError(err error) error {
	if apperr.KindOf(err) == apperr.KindAuthorityDenied {
		return err
	}
	return apperr.Wrap(apperr.KindUnavailable, "artifact", "artifact_provenance",
		"the artifact's owning run records are unavailable", err)
}

func ownershipUnavailable(message string) error {
	return apperr.New(apperr.KindUnavailable, "artifact", "artifact_provenance", message)
}
