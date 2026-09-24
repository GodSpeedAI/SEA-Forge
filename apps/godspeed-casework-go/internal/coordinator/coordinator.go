// Package coordinator owns intent handling for the casework boundary: validation, idempotency,
// the FIXTURE authority decision, lease lifecycle, and the staged world revisions an accepted
// consequential intent produces.
//
// FIXTURE-LABELED: the authority here is an allowlist over the Northstar fixture, not a governed
// authority. It refuses everything it does not explicitly permit (decide-approval is refused
// outright because the fixture has no review capability), and a refusal has ZERO effects: no
// lease, no revision. Real governed authority wiring lands in a later milestone and must replace
// this decision point without changing the refusal contract.
package coordinator

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/artifactstore"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// Intent is the interaction-intent body. Parameters is the wire contract's free-form object
// (deliberately a map: its keys belong to the intent kinds, not to a provider payload).
type Intent struct {
	ID         string         `json:"id"`
	Kind       string         `json:"kind"`
	Target     string         `json:"target"`
	Parameters map[string]any `json:"parameters,omitempty"`
	Cursor     *int64         `json:"cursor,omitempty"`
}

// Refusal reasons, exactly the wire contract's vocabulary.
const (
	ReasonInvalid         = "invalid"
	ReasonStaleProjection = "stale_projection"
	ReasonAuthorityDenied = "authority_denied"
	// ReasonUnavailable is part of the wire vocabulary but no fixture path emits it: the fixture
	// provider is in-process and cannot be unreachable. It exists so the reason set stays honest.
	ReasonUnavailable = "unavailable"
)

// Accepted/refused status values.
const (
	StatusAccepted = "accepted"
	StatusRefused  = "refused"
)

// Lease states.
const (
	LeaseClaimed  = "claimed"
	LeaseActive   = "active"
	LeaseReleased = "released"
)

// intentKinds is the allowlist: an intent whose kind is not listed is malformed, not unknown
// behaviour to guess at.
var intentKinds = map[string]bool{
	"focus-object":        true,
	"resolve-object":      true,
	"inspect-artifact":    true,
	"request-explanation": true,
	"propose-consequence": true,
	"decide-approval":     true,
	"persist-artifact":    true,
}

// implementTargets is the fixture authority's entire allowlist for a consequential implement
// action. Anything else is refused with zero effects.
var implementTargets = map[string]bool{
	"ns-migration": true,
	"ns-secondary": true,
}

// LeaseView is the lease as it appears on the wire.
type LeaseView struct {
	ID      string `json:"id"`
	State   string `json:"state"`
	Summary string `json:"summary"`
}

// Outcome is the intent's answer: an acceptance or a NORMAL refusal (both are HTTP 200 on the
// wire; a refusal is an outcome, not a transport error).
type Outcome struct {
	Status string     `json:"status"`
	Reason string     `json:"reason,omitempty"`
	Note   string     `json:"note,omitempty"`
	Lease  *LeaseView `json:"lease,omitempty"`
}

// Options tunes the staged revision timing. The zero value is the honest production default
// (real sleeps, 4s and 8s stages); tests inject a no-op sleep and zero stages so the whole flow
// runs instantly and deterministically.
type Options struct {
	// Sleep waits between staged revisions. Nil means time.Sleep.
	Sleep func(time.Duration)
	// Stage1 is the wait between acceptance and the candidate-checks revision. Default 4s.
	Stage1 time.Duration
	// Stage2 is the additional wait before the world quiets and the lease releases. Default 8s.
	Stage2 time.Duration
}

func (o Options) sleep() func(time.Duration) {
	if o.Sleep != nil {
		return o.Sleep
	}
	return time.Sleep
}

func (o Options) stage1() time.Duration {
	if o.Stage1 > 0 {
		return o.Stage1
	}
	return 4 * time.Second
}

func (o Options) stage2() time.Duration {
	if o.Stage2 > 0 {
		return o.Stage2
	}
	return 8 * time.Second
}

// leaseBuffer is the per-subscriber lease channel capacity; a stalled subscriber is evicted
// (channel closed) rather than allowed to stall lease transitions.
const leaseBuffer = 64

// Coordinator validates intents, decides them against the fixture authority, and drives the
// lease lifecycle and staged revisions of accepted consequential work. All methods are safe for
// concurrent use.
//
// Process-scoped state (documented limitation, pinned by tests): idempotency records and leases
// live in the Coordinator. A restart - or a second Coordinator over the same projection - starts
// with none of the first one's leases or intent records.
type Coordinator struct {
	proj *projection.Store
	arts *artifactstore.Store
	opts Options

	mu        sync.Mutex
	outcomes  map[string]idempotencyRecord
	leases    map[string]LeaseView
	leaseSubs map[int]chan LeaseView
	subSeq    int
}

type idempotencyRecord struct {
	bodyHash [sha256.Size]byte
	outcome  Outcome
}

// New builds a coordinator over the projection and artifact stores.
func New(proj *projection.Store, arts *artifactstore.Store, opts Options) *Coordinator {
	return &Coordinator{
		proj:      proj,
		arts:      arts,
		opts:      opts,
		outcomes:  map[string]idempotencyRecord{},
		leases:    map[string]LeaseView{},
		leaseSubs: map[int]chan LeaseView{},
	}
}

// Submit validates and decides one intent. It returns the outcome to answer with; refusals are
// outcomes, not errors. The same intent id with a byte-identical canonical body replays the
// recorded outcome; the same id with a different body is refused invalid.
func (c *Coordinator) Submit(in Intent) Outcome {
	if note, ok := validateStructure(in); !ok {
		return refused(ReasonInvalid, note)
	}
	hash := bodyHash(in)

	c.mu.Lock()
	if rec, seen := c.outcomes[in.ID]; seen {
		c.mu.Unlock()
		if rec.bodyHash != hash {
			return refused(ReasonInvalid, "intent id was already used with a different body")
		}
		return rec.outcome // idempotent replay: the recorded outcome stands
	}
	c.mu.Unlock()

	if refusal, refuseNow := freshnessRefusal(in, c.proj.LiveCursor()); refuseNow {
		return c.record(in.ID, hash, refusal)
	}

	switch in.Kind {
	case "decide-approval":
		return c.record(in.ID, hash, refused(ReasonAuthorityDenied,
			"the fixture authority has no review capability; approvals are refused rather than simulated"))
	case "propose-consequence":
		action, _ := in.Parameters["action"].(string)
		if action != "implement" || !implementTargets[in.Target] {
			return c.record(in.ID, hash, refused(ReasonAuthorityDenied,
				"fixture authority allowlist permits implement only on ns-migration or ns-secondary; everything else is refused with no effects"))
		}
		return c.beginImplement(in, hash)
	case "persist-artifact":
		ref, _ := in.Parameters["ref"].(string)
		bound, _ := in.Parameters["boundObject"].(string)
		title, _ := in.Parameters["title"].(string)
		art := c.arts.Persist(ref, bound, title)
		return c.record(in.ID, hash, Outcome{
			Status: StatusAccepted,
			Note:   fmt.Sprintf("artifact %s persisted in the fixture-scoped durable store (process lifetime; not governed persistence)", art.Ref),
		})
	default:
		// focus-object, resolve-object, inspect-artifact, request-explanation: cognitive reads
		// with no consequential work, so there is nothing to accept beyond the acknowledgement.
		return c.record(in.ID, hash, Outcome{
			Status: StatusAccepted,
			Note:   "accepted (fixture): no consequential work is attached to this intent kind",
		})
	}
}

// Leases returns the current lease registry (for inspection and tests).
func (c *Coordinator) Leases() []LeaseView {
	c.mu.Lock()
	defer c.mu.Unlock()
	out := make([]LeaseView, 0, len(c.leases))
	for _, lv := range c.leases {
		out = append(out, lv)
	}
	sortByLeaseID(out)
	return out
}

// SubscribeLeases streams lease state transitions from this moment on (live only, no replay).
// The returned cancel removes the subscription and is safe to call more than once.
func (c *Coordinator) SubscribeLeases() (<-chan LeaseView, func()) {
	c.mu.Lock()
	ch := make(chan LeaseView, leaseBuffer)
	c.subSeq++
	id := c.subSeq
	c.leaseSubs[id] = ch
	c.mu.Unlock()

	var once sync.Once
	cancel := func() {
		once.Do(func() {
			c.mu.Lock()
			if sub, ok := c.leaseSubs[id]; ok {
				delete(c.leaseSubs, id)
				close(sub)
			}
			c.mu.Unlock()
		})
	}
	return ch, cancel
}

func sortByLeaseID(in []LeaseView) {
	for i := 1; i < len(in); i++ {
		for j := i; j > 0 && in[j].ID < in[j-1].ID; j-- {
			in[j], in[j-1] = in[j-1], in[j]
		}
	}
}

// validateStructure applies the shape rules every intent must pass before anything else looks at
// it.
func validateStructure(in Intent) (string, bool) {
	if in.ID == "" {
		return "intent id is required", false
	}
	if !intentKinds[in.Kind] {
		return fmt.Sprintf("intent kind %q is not one of the allowed kinds", in.Kind), false
	}
	if in.Target == "" {
		return "intent target is required", false
	}
	if in.Kind == "persist-artifact" {
		if ref, _ := in.Parameters["ref"].(string); ref == "" {
			return "persist-artifact requires parameters.ref", false
		}
	}
	return "", true
}

// freshnessRefusal applies the wire rule: parameters.cursor, when supplied, must equal the
// provider's live cursor. A non-numeric cursor is malformed (invalid); a numeric cursor that does
// not match live is stale. The second return is true exactly when a refusal applies.
func freshnessRefusal(in Intent, live int64) (Outcome, bool) {
	raw, present := in.Parameters["cursor"]
	if !present || raw == nil {
		return Outcome{}, false
	}
	num, ok := raw.(float64)
	if !ok {
		return refused(ReasonInvalid, "parameters.cursor must be a number"), true
	}
	if num != float64(live) {
		note := fmt.Sprintf("parameters.cursor %v does not match the live cursor %d; reload the world and retry", num, live)
		return refused(ReasonStaleProjection, note), true
	}
	return Outcome{}, false
}

// bodyHash hashes the canonical encoding of the decoded intent, so semantically identical bodies
// (same fields, regardless of whitespace or key order in the raw request) hash identically.
func bodyHash(in Intent) [sha256.Size]byte {
	canonical, err := json.Marshal(in)
	if err != nil {
		// Intent holds only JSON-safe types; a failure here cannot be produced by decoded input.
		canonical = []byte(in.ID + "\x00" + in.Kind + "\x00" + in.Target)
	}
	return sha256.Sum256(canonical)
}

func refused(reason, note string) Outcome {
	return Outcome{Status: StatusRefused, Reason: reason, Note: note}
}

func (c *Coordinator) record(id string, hash [sha256.Size]byte, outcome Outcome) Outcome {
	c.mu.Lock()
	c.outcomes[id] = idempotencyRecord{bodyHash: hash, outcome: outcome}
	c.mu.Unlock()
	return outcome
}

// beginImplement runs the acceptance path for an allowed implement intent: mint the lease,
// transition it claimed -> active, append the first revision (the compatibility layer appears),
// then hand the remaining stages to a background goroutine.
func (c *Coordinator) beginImplement(in Intent, hash [sha256.Size]byte) Outcome {
	leaseID := mintLeaseID()
	c.mu.Lock()
	c.leases[leaseID] = LeaseView{ID: leaseID, State: LeaseClaimed, Summary: "Claimed: implement " + in.Target + " (compatibility layer)"}
	c.mu.Unlock()
	c.emitLease(LeaseView{ID: leaseID, State: LeaseClaimed, Summary: "Claimed: implement " + in.Target + " (compatibility layer)"})

	activeSummary := "Active: compatibility layer under way on " + in.Target
	c.mu.Lock()
	c.leases[leaseID] = LeaseView{ID: leaseID, State: LeaseActive, Summary: activeSummary}
	c.mu.Unlock()
	c.emitLease(LeaseView{ID: leaseID, State: LeaseActive, Summary: activeSummary})

	c.appendStage("Agent claimed the work - compatibility layer added", func(objs []projection.Object) []projection.Object {
		return addObject(objs, fixLayerObject("Agent claimed the work", 0.9, "notable"))
	})

	outcome := Outcome{
		Status: StatusAccepted,
		Note:   "implement accepted (fixture authority allowlist); lease active, revisions follow",
		Lease:  &LeaseView{ID: leaseID, State: LeaseActive, Summary: activeSummary},
	}
	c.record(in.ID, hash, outcome)
	go c.runStages(leaseID, in.Target)
	return outcome
}

// runStages performs the timed remainder of an accepted implement intent: candidate checks, then
// the world quiets and the lease releases. The sleeper is injectable so tests run instantly.
func (c *Coordinator) runStages(leaseID, target string) {
	sleep := c.opts.sleep()

	sleep(c.opts.stage1())
	c.appendStage("Candidate checks running against the compatibility layer", func(objs []projection.Object) []projection.Object {
		objs = withObject(objs, "ns-fix-layer", func(o *projection.Object) { o.Note = "Running candidate checks" })
		objs = withObject(objs, "ns-checks", func(o *projection.Object) { o.Note = "10/10 green (candidate)" })
		return objs
	})

	sleep(c.opts.stage2())
	c.appendStage("World quieted - compatibility layer merged, path 10 resolved", quietTheWorld)
	c.mu.Lock()
	summary := "Released: compatibility layer merged - world quieted"
	c.leases[leaseID] = LeaseView{ID: leaseID, State: LeaseReleased, Summary: summary}
	c.mu.Unlock()
	c.emitLease(LeaseView{ID: leaseID, State: LeaseReleased, Summary: summary})
}

// quietTheWorld applies the final stage: the claimed work settles and every attention marker it
// justified is removed.
func quietTheWorld(objs []projection.Object) []projection.Object {
	objs = withObject(objs, "ns-fix-layer", func(o *projection.Object) {
		o.Note = "Merged"
		o.Salience = 0.35
		o.Attention = ""
	})
	objs = withObject(objs, "ns-path-10", func(o *projection.Object) {
		o.Note = "Resolved - absence now holds correctly"
		o.Salience = 0.3
		o.Attention = ""
	})
	objs = withObject(objs, "ns-validation", func(o *projection.Object) {
		o.Note = "10/10 verified"
		o.Salience = 0.5
		o.Attention = ""
	})
	objs = withObject(objs, "ns-rollout", func(o *projection.Object) {
		o.Note = "Ready after security review"
		o.Salience = 0.55
	})
	objs = withObject(objs, "ns-workflow", func(o *projection.Object) {
		o.Note = "Preserved - no change needed"
		o.Salience = 0.4
		o.Attention = ""
	})
	objs = withObject(objs, "eng-northstar", func(o *projection.Object) {
		o.Note = "Pilot ready for review"
		o.Salience = 0.6
		o.Attention = ""
	})
	return objs
}

func fixLayerObject(note string, salience float64, attention string) projection.Object {
	parent := "ns-implementation"
	return projection.Object{
		ID:        "ns-fix-layer",
		Kind:      "component",
		Label:     "Compatibility layer",
		Position:  projection.Position{X: 3.4, Y: 3.6, Depth: 1.0},
		Salience:  salience,
		ParentID:  &parent,
		Note:      note,
		Attention: attention,
	}
}

// appendStage builds the next revision from a deep copy of the live snapshot, applies the stage's
// mutations, and appends it. The coordinator mutex serialises cursor assignment so staged
// revisions from concurrent leases never collide.
func (c *Coordinator) appendStage(summary string, mutate func([]projection.Object) []projection.Object) {
	c.mu.Lock()
	defer c.mu.Unlock()
	live := c.proj.Live() // a fresh deep copy every call
	rev := projection.Revision{
		Cursor:        live.Cursor + 1,
		At:            time.Now().UTC().Format(time.RFC3339),
		Summary:       summary,
		Surfaces:      live.Surfaces,
		Objects:       mutate(live.Objects),
		Relationships: live.Relationships,
	}
	if err := c.proj.Append(rev); err != nil {
		// Unreachable while the coordinator mutex serialises every append; logged rather than
		// panicked so a future caller cannot take the server down with a bad stage.
		log.Printf("coordinator: append stage %q failed: %v", summary, err)
	}
}

// emitLease hands a lease transition to every subscriber. Called without c.mu held.
func (c *Coordinator) emitLease(lv LeaseView) {
	c.mu.Lock()
	defer c.mu.Unlock()
	for id, ch := range c.leaseSubs {
		select {
		case ch <- lv:
		default:
			close(ch)
			delete(c.leaseSubs, id)
		}
	}
}

// mintLeaseID mints a lease id from a timestamp and random hex, with no dependencies beyond the
// standard library.
func mintLeaseID() string {
	var rnd [4]byte
	if _, err := rand.Read(rnd[:]); err != nil {
		return fmt.Sprintf("lease-%d", time.Now().UnixNano())
	}
	return fmt.Sprintf("lease-%d-%s", time.Now().UnixNano(), hex.EncodeToString(rnd[:]))
}

func addObject(objs []projection.Object, o projection.Object) []projection.Object {
	for i := range objs {
		if objs[i].ID == o.ID {
			objs[i] = o
			return objs
		}
	}
	return append(objs, o)
}

func withObject(objs []projection.Object, id string, fn func(*projection.Object)) []projection.Object {
	for i := range objs {
		if objs[i].ID == id {
			fn(&objs[i])
			return objs
		}
	}
	return objs
}
