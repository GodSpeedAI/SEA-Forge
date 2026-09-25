//go:build casework_fixture

// Package artifactstore is the FIXTURE-SCOPED artifact persistence for this milestone.
//
// Limitation, stated rather than hidden: durability here is the running process's memory. A
// persisted artifact survives for the process lifetime and is served with leveled content, but
// this is never governed persistence - a restart starts from the fixture seed again, and nothing
// in this package may be presented as a governed artifact store.
package artifactstore

import (
	"fmt"
	"sort"
	"sync"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// Store serves the fixture's artifacts plus artifacts persisted at runtime, keyed by ref.
type Store struct {
	mu    sync.Mutex
	byRef map[string]projection.Artifact
}

// New seeds the store with the fixture artifact set (copied, so the caller's slice is never
// shared with stored state).
func New(seed []projection.Artifact) *Store {
	s := &Store{byRef: make(map[string]projection.Artifact, len(seed))}
	for _, a := range seed {
		stored := a
		stored.Levels = append([]projection.ArtifactLevel(nil), a.Levels...)
		s.byRef[a.Ref] = stored
	}
	return s
}

// Get returns the artifact recorded under ref, if any.
func (s *Store) Get(ref string) (projection.Artifact, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	a, ok := s.byRef[ref]
	if !ok {
		return projection.Artifact{}, false
	}
	out := a
	out.Levels = append([]projection.ArtifactLevel(nil), a.Levels...)
	return out, true
}

// Persist records an artifact binding in the fixture-scoped durable store.
//
// When the ref already names a fixture artifact, its leveled content is kept and only the binding
// (title, bound object) is refreshed: the fixture remains the content authority for its own
// artifacts. When the ref is new, the store records the binding and synthesizes honest
// placeholder levels - the wire contract carries no content bytes on persist, and pretending
// otherwise would be fabrication.
func (s *Store) Persist(ref, boundObject, title string) projection.Artifact {
	s.mu.Lock()
	defer s.mu.Unlock()
	if existing, ok := s.byRef[ref]; ok {
		if title != "" {
			existing.Title = title
		}
		if boundObject != "" {
			existing.BoundObject = boundObject
		}
		s.byRef[ref] = existing
		return existing
	}
	if title == "" {
		title = ref
	}
	fresh := projection.Artifact{
		Ref:         ref,
		Kind:        "document",
		Title:       title,
		BoundObject: boundObject,
		Levels: []projection.ArtifactLevel{
			{
				Level:     "minimal",
				MediaType: "text/plain",
				Text:      fmt.Sprintf("%s (persisted, fixture store)", title),
			},
			{
				Level:     "summary",
				MediaType: "text/plain",
				Text:      fmt.Sprintf("Persisted from the cognitive environment into the fixture-scoped store: %s, bound to %s.", title, boundObject),
			},
			{
				Level:     "source",
				MediaType: "text/plain",
				Text: fmt.Sprintf("PERSISTED ARTIFACT (fixture store, process-lifetime durability)\nRef: %s\nTitle: %s\nBound object: %s\n\n"+
					"The wire contract carries no content bytes on persist, so this record holds the binding only.\n"+
					"FIXTURE-LABELED: this is not governed persistence.", ref, title, boundObject),
			},
		},
	}
	s.byRef[ref] = fresh
	return fresh
}

// Count reports how many artifacts the store currently holds (fixture seed plus persisted).
func (s *Store) Count() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.byRef)
}

// List returns every artifact the store holds, copied and sorted by ref: the fixture seed plus
// anything persisted at runtime.
func (s *Store) List() []projection.Artifact {
	s.mu.Lock()
	defer s.mu.Unlock()
	out := make([]projection.Artifact, 0, len(s.byRef))
	for _, a := range s.byRef {
		stored := a
		stored.Levels = append([]projection.ArtifactLevel(nil), a.Levels...)
		out = append(out, stored)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Ref < out[j].Ref })
	return out
}
