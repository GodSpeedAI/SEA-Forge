// Package projection owns the cognitive projection model the casework boundary serves: world
// snapshots, their revision history, and the fixture dataset the projections are built from in
// this milestone.
//
// FIXTURE-LABELED: the dataset embedded here is the canonical Northstar fixture. It stands in for
// a governed authority projection until the real wiring lands in a later milestone, and nothing
// built on this package may be presented as governed integration.
//
// The JSON tags are the wire contract (apps/godspeed-cognitive-ui WIRE.md); the golden and
// round-trip tests pin them so a rename cannot drift silently.
package projection

// ProvenanceLabel names the projection source on every served snapshot and on the health answer.
// It states the fixture provenance honestly: this is the Go casework boundary over the Northstar
// fixture, not a governed authority view.
const ProvenanceLabel = "go:fixture:northstar"

// Position is an object's placement in the cognitive space.
type Position struct {
	X     float64 `json:"x"`
	Y     float64 `json:"y"`
	Depth float64 `json:"depth"`
}

// Object is one cognitive object in the world projection.
type Object struct {
	ID        string   `json:"id"`
	Kind      string   `json:"kind"`
	Label     string   `json:"label"`
	Position  Position `json:"position"`
	Salience  float64  `json:"salience"`
	ParentID  *string  `json:"parentId,omitempty"` // omitted when top-level
	Note      string   `json:"note"`
	Attention string   `json:"attention,omitempty"` // "notable" | "requires-judgment" | omitted (= quiet)
}

// Surface groups object ids for orientation.
type Surface struct {
	ID        string   `json:"id"`
	Label     string   `json:"label"`
	ObjectIDs []string `json:"objectIds"`
}

// Relationship is a directed edge between two objects.
type Relationship struct {
	From string `json:"from"`
	To   string `json:"to"`
	Kind string `json:"kind"`
}

// Snapshot is one complete world projection as served over the wire.
type Snapshot struct {
	Cursor        int64          `json:"cursor"`
	Surfaces      []Surface      `json:"surfaces"`
	Objects       []Object       `json:"objects"`
	Relationships []Relationship `json:"relationships"`
	Provenance    string         `json:"provenance"`
}

// Revision is one position of the world's history: a snapshot's content plus when it was recorded
// and a one-line summary of what changed.
type Revision struct {
	Cursor        int64          `json:"cursor"`
	At            string         `json:"at"` // RFC 3339
	Summary       string         `json:"summary"`
	Surfaces      []Surface      `json:"surfaces"`
	Objects       []Object       `json:"objects"`
	Relationships []Relationship `json:"relationships"`
}

// WindowEntry is a revision reduced to its position in the history window.
type WindowEntry struct {
	Cursor  int64  `json:"cursor"`
	At      string `json:"at"`
	Summary string `json:"summary"`
}

// ArtifactLevel is one verbosity level of an artifact's content.
type ArtifactLevel struct {
	Level       string `json:"level"`
	MediaType   string `json:"mediaType"`
	Text        string `json:"text,omitempty"`
	BytesBase64 string `json:"bytesBase64,omitempty"`
}

// Artifact is a piece of case evidence bound to a world object, with leveled content.
type Artifact struct {
	Ref         string          `json:"ref"`
	Kind        string          `json:"kind"`
	Title       string          `json:"title"`
	BoundObject string          `json:"boundObject"`
	Levels      []ArtifactLevel `json:"levels"`
}

// Dataset is the fixture document shape: the revision history plus the artifact set.
type Dataset struct {
	Provenance string     `json:"provenance"`
	LiveCursor int64      `json:"liveCursor"`
	Revisions  []Revision `json:"revisions"`
	Artifacts  []Artifact `json:"artifacts"`
}

// Content returns the bytes and media type for one verbosity level of the artifact. Base64-encoded
// levels (binary artifacts) are decoded here so callers never handle encodings.
func (a Artifact) Content(level string) ([]byte, string, bool) {
	for _, lv := range a.Levels {
		if lv.Level != level {
			continue
		}
		switch {
		case lv.Text != "":
			return []byte(lv.Text), lv.MediaType, true
		case lv.BytesBase64 != "":
			raw, err := decodeBase64(lv.BytesBase64)
			if err != nil {
				return nil, "", false
			}
			return raw, lv.MediaType, true
		}
		return nil, "", false
	}
	return nil, "", false
}

// SnapshotOf projects a stored revision into the served snapshot view, labelling the provenance.
func SnapshotOf(rev Revision) Snapshot {
	return Snapshot{
		Cursor:        rev.Cursor,
		Surfaces:      cloneSurfaces(rev.Surfaces),
		Objects:       cloneObjects(rev.Objects),
		Relationships: cloneRelationships(rev.Relationships),
		Provenance:    ProvenanceLabel,
	}
}

// cloneRevision deep-copies a revision so a stored one can never be mutated through a handed-out
// reference.
func cloneRevision(rev Revision) Revision {
	return Revision{
		Cursor:        rev.Cursor,
		At:            rev.At,
		Summary:       rev.Summary,
		Surfaces:      cloneSurfaces(rev.Surfaces),
		Objects:       cloneObjects(rev.Objects),
		Relationships: cloneRelationships(rev.Relationships),
	}
}

func cloneSurfaces(in []Surface) []Surface {
	if in == nil {
		return nil
	}
	out := make([]Surface, len(in))
	for i, s := range in {
		out[i] = Surface{ID: s.ID, Label: s.Label, ObjectIDs: append([]string(nil), s.ObjectIDs...)}
	}
	return out
}

func cloneObjects(in []Object) []Object {
	if in == nil {
		return nil
	}
	out := make([]Object, len(in))
	for i, o := range in {
		o.ParentID = cloneStringPtr(o.ParentID)
		out[i] = o
	}
	return out
}

func cloneRelationships(in []Relationship) []Relationship {
	if in == nil {
		return nil
	}
	return append([]Relationship(nil), in...)
}

func cloneStringPtr(p *string) *string {
	if p == nil {
		return nil
	}
	v := *p
	return &v
}
