//go:build casework_fixture

package artifactstore

import (
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

func fixtureSeed(t *testing.T) []projection.Artifact {
	t.Helper()
	ds, err := projection.Fixture()
	if err != nil {
		t.Fatalf("decode embedded fixture: %v", err)
	}
	return ds.Artifacts
}

func TestServesFixtureArtifactsWithLevels(t *testing.T) {
	s := New(fixtureSeed(t))
	art, ok := s.Get("art-interview-quote")
	if !ok {
		t.Fatal("fixture artifact art-interview-quote must be served")
	}
	if art.BoundObject != "ns-interview" || len(art.Levels) != 3 {
		t.Fatalf("fixture artifact shape: %+v", art)
	}
	if got := s.Count(); got != 9 {
		t.Fatalf("seeded count: got %d, want 9", got)
	}
	if _, ok := s.Get("art-nope"); ok {
		t.Fatal("an unknown ref must not resolve")
	}
}

func TestLevelBytesAreLevelAware(t *testing.T) {
	s := New(fixtureSeed(t))
	art, _ := s.Get("art-interview-quote")
	minimal, mt, ok := art.Content("minimal")
	if !ok || mt != "text/plain" || string(minimal) != "Secondary coverage is not optional." {
		t.Fatalf("minimal level: %q %q %v", minimal, mt, ok)
	}
	source, mt, ok := art.Content("source")
	if !ok || mt != "text/plain" || len(source) == 0 || string(source) == string(minimal) {
		t.Fatalf("source level must differ from minimal: %q", source)
	}
	if _, _, ok := art.Content("verbatim"); ok {
		t.Fatal("an unknown level must not resolve")
	}
}

func TestBinaryArtifactLevelsDecode(t *testing.T) {
	s := New(fixtureSeed(t))
	art, ok := s.Get("art-pilot-report")
	if !ok {
		t.Fatal("fixture artifact art-pilot-report must be served")
	}
	raw, mt, ok := art.Content("source")
	if !ok || mt != "application/pdf" {
		t.Fatalf("pdf source level: %q %v", mt, ok)
	}
	if len(raw) == 0 || raw[0] != '%' || raw[1] != 'P' {
		t.Fatalf("pdf bytes do not start with %%PDF: %q", raw[:min(5, len(raw))])
	}
}

// Durability is process-scoped by design: a persisted artifact must remain served for the store's
// lifetime (the stand-in for the process), with honest placeholder content.
func TestPersistIsDurableForProcessLifetime(t *testing.T) {
	s := New(fixtureSeed(t))
	s.Persist("art-new-note", "ns-workflow", "Operator note")
	s.Persist("art-new-note", "ns-workflow", "Operator note") // idempotent re-persist

	art, ok := s.Get("art-new-note")
	if !ok {
		t.Fatal("persisted artifact must be served afterwards")
	}
	if art.Title != "Operator note" || art.BoundObject != "ns-workflow" {
		t.Fatalf("persisted binding: %+v", art)
	}
	raw, mt, ok := art.Content("source")
	if !ok || mt != "text/plain" {
		t.Fatalf("persisted source level: %q %v", mt, ok)
	}
	if want := "not governed persistence"; !strings.Contains(string(raw), want) {
		t.Fatalf("persisted placeholder must state its limitation honestly: %q", raw)
	}
	if got := s.Count(); got != 10 {
		t.Fatalf("count after persist: got %d, want 10", got)
	}
}

func TestPersistFixtureRefKeepsFixtureContent(t *testing.T) {
	s := New(fixtureSeed(t))
	s.Persist("art-diff-491", "ns-pr-491", "PR #491 - coverage guard")
	art, ok := s.Get("art-diff-491")
	if !ok {
		t.Fatal("fixture ref must still resolve after persist")
	}
	raw, mt, ok := art.Content("source")
	if !ok || mt != "text/x-diff" {
		t.Fatalf("fixture content must stay authoritative for a fixture ref: %q %v", mt, ok)
	}
	if len(raw) == 0 || raw[0] != '-' {
		t.Fatalf("expected the fixture diff bytes, got %q", raw[:min(10, len(raw))])
	}
}
