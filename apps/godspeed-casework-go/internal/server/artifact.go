package server

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"net/http"
	"path"
	"regexp"
	"strings"
	"unicode/utf8"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// ArtifactGetter reads content only through the configured governed authority.
type ArtifactGetter interface {
	GetArtifact(ctx context.Context, digest string) (ports.ArtifactContent, error)
}

var sha256DigestPattern = regexp.MustCompile(`^(?:sha256:)?[0-9a-f]{64}$`)

type artifactPayload struct {
	EvidenceID  string             `json:"evidence_id"`
	Name        string             `json:"name"`
	Digest      string             `json:"digest"`
	ContentType string             `json:"content_type"`
	Content     string             `json:"content"`
	Provenance  artifactProvenance `json:"provenance"`
}

type artifactProvenance struct {
	CaseID       string `json:"case_id"`
	PlanItemID   string `json:"plan_item_id"`
	InvocationID string `json:"invocation_id"`
	RunID        string `json:"run_id"`
}

func (s *Server) handleArtifactGet(w http.ResponseWriter, r *http.Request) {
	digest, ok := normalizeSHA256Digest(r.PathValue("digest"))
	if !ok {
		writeTypedError(w, http.StatusBadRequest, "invalid", "artifact digest must be a SHA-256 hex digest")
		return
	}
	if s.artifacts == nil {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "artifact reads are not configured")
		return
	}
	verifier, ok := s.world.(PerspectiveVerifier)
	if !ok {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "artifact identity verification is not configured")
		return
	}
	actor := sessionIdentityOf(r).Claim()
	if actor.ActorID == "" || actor.Role == "" {
		writeTypedError(w, http.StatusUnauthorized, "unauthorized", "a session identity is required")
		return
	}
	if err := verifier.VerifyPerspective(r.Context(), actor); err != nil {
		if apperr.KindOf(err) == apperr.KindAuthorityDenied {
			writeTypedError(w, http.StatusForbidden, "authority_denied", "the kernel refused this artifact read")
		} else {
			writeTypedError(w, http.StatusBadGateway, "unavailable", "the kernel could not verify the session identity")
		}
		return
	}

	content, err := s.artifacts.GetArtifact(r.Context(), digest)
	if err != nil {
		writeArtifactReadError(w, err)
		return
	}
	returnedDigest, ok := normalizeSHA256Digest(content.Digest)
	if !ok || returnedDigest != digest || content.Size != int64(len(content.Data)) {
		writeTypedError(w, http.StatusBadGateway, "integrity_mismatch", "the kernel artifact metadata did not match the requested digest")
		return
	}
	actual := sha256.Sum256(content.Data)
	if hex.EncodeToString(actual[:]) != digest {
		writeTypedError(w, http.StatusBadGateway, "integrity_mismatch", "the kernel artifact bytes did not match the requested digest")
		return
	}
	if !utf8.Valid(content.Data) {
		writeTypedError(w, http.StatusUnsupportedMediaType, "unsupported_artifact", "the artifact is not valid UTF-8 text")
		return
	}
	if content.RunID == "" || content.CaseID == "" || content.PlanItemID == "" {
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the artifact's owning run, case, and plan item could not be resolved")
		return
	}

	name := path.Base(content.URI)
	if name == "." || name == "/" || name == "" {
		name = content.EvidenceID
	}
	writeJSON(w, http.StatusOK, artifactPayload{
		EvidenceID:  content.EvidenceID,
		Name:        name,
		Digest:      "sha256:" + digest,
		ContentType: artifactContentType(content.URI, content.Data),
		Content:     string(content.Data),
		Provenance: artifactProvenance{
			CaseID: content.CaseID, PlanItemID: content.PlanItemID, RunID: content.RunID,
		},
	})
}

func normalizeSHA256Digest(digest string) (string, bool) {
	if !sha256DigestPattern.MatchString(digest) {
		return "", false
	}
	return strings.TrimPrefix(digest, "sha256:"), true
}

func writeArtifactReadError(w http.ResponseWriter, err error) {
	if errors.Is(err, ports.ErrArtifactIntegrity) {
		writeTypedError(w, http.StatusBadGateway, "integrity_mismatch", "the stored artifact no longer matches its digest; it was not served")
		return
	}
	switch apperr.KindOf(err) {
	case apperr.KindInvalid:
		writeTypedError(w, http.StatusNotFound, "not_found", "the requested artifact was not found")
	case apperr.KindAuthorityDenied:
		writeTypedError(w, http.StatusForbidden, "authority_denied", "the kernel refused this artifact read")
	case apperr.KindUnavailable:
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the governed artifact authority is unavailable")
	default:
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the governed artifact authority could not serve this artifact")
	}
}

func artifactContentType(uri string, data []byte) string {
	switch strings.ToLower(path.Ext(uri)) {
	case ".md", ".markdown", ".mdown":
		return "text/markdown"
	case ".json":
		if json.Valid(data) {
			return "application/json"
		}
	case ".diff", ".patch":
		return "text/x-diff"
	}
	return "text/plain"
}
