// The live projection builder: pure functions folding governed-authority view DTOs (ports.Case*)
// into the canonical spec-04 CognitiveWorldSnapshot (operator decision D-1, T01 contract).
//
// Purity contract: Build performs no I/O and consults no global state - every input it needs rides
// in CaseFacts. The network enters only through LiveSource (live.go), which fetches the facts via
// ports.CaseAuthorityPort and hands them here. This keeps the fold unit-testable against exact
// standing data and keeps "what the kernel said" separable from "how the gateway renders it".
//
// Two standing rules this builder exists to keep (plan operating contract):
//   - Execution standing and settlement standing stay DISJOINT: an item that executed cleanly with
//     no settlement record is rendered as completed-but-unsettled, never "done".
//   - Blocked/pending items show WHY in plain language, composed from the real standing data the
//     kernel's view verbs carry (execution standing, settlement standing, depends_on, the
//     approval inbox). The kernel's case.get_horizon does not expose template sentry predicates,
//     so a pending item with no dependencies is explained by what the standing factually says
//     (no events folded yet; entry sentries not fired) - never by an invented sentry name.
package projection

import (
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// ProvenanceLabelLive is the provenance the live stack serves: the Go casework boundary over the
// governed kernel via the SFWP client. It replaces the fixture label in every served snapshot and
// health answer so a served world can never masquerade as governed truth.
const ProvenanceLabelLive = "go:live:sfwp"

// CaseFacts is everything the builder folds: one case's bounded views plus the perspective the
// snapshot is built for.
type CaseFacts struct {
	Record    ports.CaseRecord   // from case.list (summary, state, timestamps)
	Overview  ports.CaseOverview // from case.get_overview (stages, template, settlements)
	Horizon   ports.CaseHorizon  // from case.get_horizon (per-item standing, depends_on)
	Approvals []ports.ApprovalRecord
	Runs      []ports.RunSummary
	// RunArtifacts are the artifacts each run captured, by run id (run.get evidence). Absent when
	// the authority cannot list them; the snapshot then binds no artifacts.
	RunArtifacts map[string][]ports.RunArtifactRef
	// UnreadableRunIDs are run IDs the scoped authority query reported but could not read.
	UnreadableRunIDs []string
	// Actor is the perspective the snapshot is rendered for (actor_id + kernel role spelling).
	Actor ports.ActorClaim
	// Cursor is the kernel event cursor this snapshot exists at ("" when unknown).
	Cursor string
	// Now is the render time; injectable so tests are deterministic.
	Now time.Time
}

// Build folds CaseFacts into the canonical snapshot. It never returns nil objects or actions
// slices: empty is the honest shape for "nothing offered".
func Build(facts CaseFacts) contract.CognitiveWorldSnapshot {
	if facts.Now.IsZero() {
		facts.Now = time.Now()
	}
	role := facts.Actor.Role
	offer := RoleOffersConsequentialWork(role)

	approvalsByItem := map[string][]ports.ApprovalRecord{}
	for _, ap := range facts.Approvals {
		if ap.Expired {
			continue
		}
		approvalsByItem[ap.ItemID] = append(approvalsByItem[ap.ItemID], ap)
	}

	byID := map[string]ports.HorizonItem{}
	for _, it := range facts.Horizon.Items {
		byID[it.ItemID] = it
	}
	stageChildren := map[string][]ports.HorizonItem{}
	for _, it := range facts.Horizon.Items {
		if it.ParentStage != "" {
			stageChildren[it.ParentStage] = append(stageChildren[it.ParentStage], it)
		}
	}

	var objects []contract.CognitiveObject
	for _, stageID := range facts.Overview.Stages {
		status := stageStatusFor(stageChildren[stageID])
		objects = append(objects, contract.CognitiveObject{
			ID:       stageID,
			Kind:     "stage",
			Name:     titleize(stageID),
			Status:   status,
			Badge:    stageBadgeFor(status, len(stageChildren[stageID])),
			Salience: 0.5,
			Actions:  []contract.ActionDescriptor{},
		})
	}
	for _, item := range facts.Horizon.Items {
		objects = append(objects, itemBuilder(item, byID, approvalsByItem, role, offer))
	}
	focus := attentionFor(objects, string(facts.Overview.Ref))
	objects = append(objects, runChildren(facts, objects)...)
	objects = append(objects, evidenceChildren(facts, objects)...)

	snapshot := contract.CognitiveWorldSnapshot{
		WorldID:          "world-" + string(facts.Overview.Ref),
		CaseID:           string(facts.Overview.Ref),
		Cursor:           facts.Cursor,
		Timestamp:        facts.Now.UTC().Format(time.RFC3339),
		Perspective:      contract.ActorPerspective{ActorID: facts.Actor.ActorID, Role: facts.Actor.Role},
		Summary:          summaryFor(facts),
		VisibleObjects:   objects,
		AvailableActions: []contract.ActionDescriptor{},
		AttentionFocus:   focus,
	}

	// available_actions: the role-filtered union of every per-object action, plus the case-level
	// lifecycle offers (reopen/terminate ride on the snapshot, targeting the case itself).
	seen := map[string]bool{}
	for _, obj := range objects {
		for _, a := range obj.Actions {
			if !seen[a.ID] {
				seen[a.ID] = true
				snapshot.AvailableActions = append(snapshot.AvailableActions, a)
			}
		}
	}
	for _, a := range lifecycleOffers(facts, role, offer) {
		if !seen[a.ID] {
			seen[a.ID] = true
			snapshot.AvailableActions = append(snapshot.AvailableActions, a)
		}
	}
	return snapshot
}

func runChildren(facts CaseFacts, parentObjects []contract.CognitiveObject) []contract.CognitiveObject {
	caseID := string(facts.Overview.Ref)
	parentByID := make(map[string]ports.HorizonItem, len(facts.Horizon.Items))
	parentCounts := make(map[string]int, len(facts.Horizon.Items))
	for _, item := range facts.Horizon.Items {
		parentCounts[item.ItemID]++
		parentByID[item.ItemID] = item
	}
	objectCounts := make(map[string]int, len(parentObjects))
	for _, object := range parentObjects {
		objectCounts[object.ID]++
	}
	runCounts := make(map[string]int, len(facts.Runs))
	for _, run := range facts.Runs {
		runCounts[run.RunID]++
	}
	children := make([]contract.CognitiveObject, 0, len(facts.Runs))
	for _, run := range facts.Runs {
		if strings.TrimSpace(run.RunID) == "" || runCounts[run.RunID] != 1 || objectCounts[run.RunID] != 0 ||
			run.CaseID != caseID || strings.TrimSpace(run.PlanItemID) == "" || parentCounts[run.PlanItemID] != 1 {
			continue
		}
		parent, ok := parentByID[run.PlanItemID]
		if !ok || !validCapturedExecution(run.Execution) || !validCapturedSettlement(run.Settlement) || run.EvidenceCount < 0 {
			continue
		}
		standing := parent
		standing.Execution = run.Execution
		standing.Settlement = run.Settlement
		status, badge := cognitiveStatus(standing)
		if status == "COMPLETED" && run.Settlement == "unsettled" {
			// The episode ran but its settlement is not recorded: the run is waiting on the
			// settlement authority. Contract status COMPLETED is reserved for an accepted
			// settlement, so a client reading the typed status never reads executed as settled.
			status = "WAITING_ON_OTHERS"
		}
		parentID := run.PlanItemID
		explanation := "Execution: " + run.Execution + "; settlement: " + run.Settlement + "."
		children = append(children, contract.CognitiveObject{
			ID:          run.RunID,
			Kind:        "execution_trace",
			Name:        run.RunID,
			Status:      status,
			Badge:       badge,
			Explanation: &explanation,
			Salience:    salienceFor(standing, false),
			ParentID:    &parentID,
			Actions:     []contract.ActionDescriptor{},
		})
	}
	return children
}

// evidenceChildren binds each artifact a run captured to the plan item that ran it, as an
// evidence_record object carrying the artifact descriptor. The ref is the content digest: the UI
// resolves it through GET /api/artifacts/{digest} and verifies what comes back against it. The
// descriptor says nothing about the bytes; it only says where they can be fetched from.
func evidenceChildren(facts CaseFacts, existing []contract.CognitiveObject) []contract.CognitiveObject {
	if len(facts.RunArtifacts) == 0 {
		return nil
	}
	taken := make(map[string]int, len(existing))
	for _, o := range existing {
		taken[o.ID]++
	}
	parents := make(map[string]bool, len(facts.Horizon.Items))
	for _, it := range facts.Horizon.Items {
		parents[it.ItemID] = true
	}
	var out []contract.CognitiveObject
	for _, run := range facts.Runs {
		if !parents[run.PlanItemID] {
			continue
		}
		for _, ref := range facts.RunArtifacts[run.RunID] {
			if taken[ref.EvidenceID] != 0 || ref.EvidenceID == "" || ref.Digest == "" {
				continue
			}
			taken[ref.EvidenceID]++
			name := ref.URI
			if i := strings.LastIndex(name, "/"); i >= 0 {
				name = name[i+1:]
			}
			if name == "" {
				name = ref.EvidenceID
			}
			parent := run.PlanItemID
			explanation := "Captured by " + run.RunID + "; content-addressed " + ref.Digest[:19] + "."
			out = append(out, contract.CognitiveObject{
				ID:          ref.EvidenceID,
				Kind:        "evidence_record",
				Name:        name,
				Status:      "COMPLETED",
				Badge:       "Captured artifact",
				Explanation: &explanation,
				Salience:    0.3,
				ParentID:    &parent,
				Actions:     []contract.ActionDescriptor{actionFor(ref.EvidenceID, "OPEN_ARTIFACT", "Open evidence", "SECONDARY", false, false)},
				X: &contract.ObjectExtensions{Artifacts: []contract.CognitiveArtifact{{
					Ref:              ref.Digest,
					Kind:             "document",
					Title:            name,
					BoundObject:      ref.EvidenceID,
					CurrentLevel:     "source",
					MediaType:        artifactMediaType(name),
					SourceProvenance: "kernel evidence " + ref.EvidenceID + " of run " + run.RunID,
				}}},
			})
		}
	}
	return out
}

func artifactMediaType(name string) string {
	switch strings.ToLower(name[strings.LastIndex(name, ".")+1:]) {
	case "md", "markdown":
		return "text/markdown"
	case "json":
		return "application/json"
	case "diff", "patch":
		return "text/x-diff"
	}
	return "text/plain"
}

// itemBuilder renders one horizon item as a cognitive object: status, badge, plain-language
// explanation (the sentry reason), and the role-filtered action descriptors.
func itemBuilder(item ports.HorizonItem, byID map[string]ports.HorizonItem, approvals map[string][]ports.ApprovalRecord, role string, offer bool) contract.CognitiveObject {
	obj := contract.CognitiveObject{
		ID:        item.ItemID,
		Name:      item.Name,
		Salience:  0.4,
		Actions:   []contract.ActionDescriptor{},
		DependsOn: append([]string(nil), item.DependsOn...),
	}
	if item.ParentStage != "" {
		parent := item.ParentStage
		obj.ParentID = &parent
	}
	if item.Kind == "milestone" {
		obj.Kind = "milestone"
	} else {
		obj.Kind = "work_item"
	}

	// Separation of duties on the wire: execution standing and settlement standing are carried
	// side by side and never collapsed into one word.
	obj.Status, obj.Badge = cognitiveStatus(item)
	obj.Explanation = explanationFor(item, byID, approvals)
	obj.Salience = salienceFor(item, len(approvals[item.ItemID]) > 0)

	if offer {
		obj.Actions = actionsFor(item, len(approvals[item.ItemID]) > 0, role)
	}
	return obj
}

// cognitiveStatus maps the kernel's DISJOINT standings onto the spec-04 cognitive status. The
// badge always names both facts when they disagree (executed but unsettled is the important case).
func cognitiveStatus(item ports.HorizonItem) (status, badge string) {
	switch item.Execution {
	case "failed":
		return "FAILED", "Failed"
	case "terminated":
		return "FAILED", "Terminated"
	case "active":
		if item.Kind == "human_task" {
			return "ACTION_REQUIRED", "Waiting on your decision"
		}
		return "IN_PROGRESS", "Running"
	case "enabled":
		return "READY_TO_BEGIN", "Ready"
	case "completed":
		switch item.Settlement {
		case "accepted":
			return "COMPLETED", "Settled accepted"
		case "rejected":
			return "REJECTED", "Settlement rejected"
		case "escalated":
			return "ACTION_REQUIRED", "Awaiting approval"
		default: // unsettled: executed, NOT settled - the standing the UI must never read as done
			return "COMPLETED", "Executed; settlement pending"
		}
	default: // pending: in the plan, nothing has happened yet
		return "WAITING", "Waiting"
	}
}

// salienceFor ranks what needs a human eye: actionable standing first, settled history last.
func salienceFor(item ports.HorizonItem, hasApproval bool) float64 {
	switch {
	case hasApproval:
		return 0.9
	case item.Execution == "enabled":
		return 0.85
	case item.Execution == "failed" || item.Execution == "terminated":
		return 0.8
	case item.Execution == "active":
		return 0.7
	case item.Execution == "completed":
		return 0.3
	default:
		return 0.4
	}
}

// actionsFor returns the role-filtered descriptors the item's standing supports. The same
// predicates back the intent translator's UNAUTHORIZED_ROLE guard, so the offer and the guard
// cannot drift apart.
func actionsFor(item ports.HorizonItem, hasPendingApproval bool, role string) []contract.ActionDescriptor {
	out := []contract.ActionDescriptor{}
	switch {
	case hasPendingApproval:
		// A governed approval record is open on this item: only approver roles see the decision.
		if MayDecideApproval(role) {
			out = append(out,
				actionFor(item.ItemID, "APPROVE_HUMAN_TASK", "Approve", "PRIMARY", true, true),
				actionFor(item.ItemID, "REJECT_HUMAN_TASK", "Reject", "DANGER", true, true),
			)
		}
	case item.Execution == "enabled" && item.Kind != "human_task" && item.Kind != "milestone":
		// Entry criteria are met and the item is eligible to dispatch (kernel Enabled standing).
		if MayExecute(role) {
			out = append(out, actionFor(item.ItemID, "EXECUTE_ITEM", "Execute", "PRIMARY", true, false))
		}
	case item.Execution == "active" && item.Kind == "human_task":
		// The case engine parked a human task: the assigned role completes it.
		if MayExecute(role) {
			out = append(out, actionFor(item.ItemID, "COMPLETE_HUMAN_TASK", "Complete", "PRIMARY", true, true))
		}
	}
	// Discretionary work anchors to live work: a proposer role may add optional work after any
	// item that is still in play (pending, enabled or active), never after finished or failed
	// work and never while an approval decision is open on the item. The proposal rides the same
	// roleWouldOffer guard as every other consequential intent (MayPropose).
	if !hasPendingApproval && item.Kind != "milestone" && MayPropose(role) &&
		(item.Execution == "pending" || item.Execution == "enabled" || item.Execution == "active") {
		out = append(out, actionFor(item.ItemID, "ADD_DISCRETIONARY_WORK", "Add discretionary work", "SECONDARY", true, true))
	}
	return out
}

// lifecycleOffers are the case-level actions riding in available_actions (the snapshot has no
// "case" object kind in the spec-04 vocabulary, so lifecycle targets the case_id).
func lifecycleOffers(facts CaseFacts, role string, offer bool) []contract.ActionDescriptor {
	if !offer || !MayDriveLifecycle(role) {
		return nil
	}
	caseID := string(facts.Overview.Ref)
	switch facts.Horizon.State {
	case "completed", "terminated":
		return []contract.ActionDescriptor{actionFor(caseID, "REOPEN_CASE", "Reopen case", "SECONDARY", true, true)}
	case "active", "awaiting_approval":
		return []contract.ActionDescriptor{actionFor(caseID, "TERMINATE_CASE", "Terminate case", "DANGER", true, true)}
	}
	return nil
}

// explanationFor composes the plain-language WHY an item is where it is, from the real standing
// data only. This is where the sentry reasons live: the kernel's view verbs do not expose the
// template's sentry predicates, so the reason names the observable facts - which dependency is
// still open and at what standing, or that no entry sentry has fired yet.
func explanationFor(item ports.HorizonItem, byID map[string]ports.HorizonItem, approvals map[string][]ports.ApprovalRecord) *string {
	if aps := approvals[item.ItemID]; len(aps) > 0 {
		s := "An approval decision is open on this item (" + aps[0].ApprovalID + "); an approver must resolve it before the work settles."
		return &s
	}
	var parts []string
	switch item.Execution {
	case "pending":
		if blocked := openDependencies(item, byID); len(blocked) > 0 {
			parts = append(parts, "Waiting on "+humanList(blocked)+".")
		} else {
			parts = append(parts, "No work has been recorded for this item yet: its entry sentries have not fired.")
		}
	case "enabled":
		parts = append(parts, "Entry criteria are met; this item is ready to execute.")
	case "active":
		if item.Kind == "human_task" {
			parts = append(parts, "Parked for a human decision; completing the task resolves it.")
		} else {
			parts = append(parts, "An execution episode is running.")
		}
	case "completed":
		switch item.Settlement {
		case "unsettled":
			parts = append(parts, "Execution finished, but no settlement has been recorded yet - this is not settled work.")
		case "escalated":
			parts = append(parts, "Execution finished and the settlement was escalated for human approval.")
		}
	case "failed":
		parts = append(parts, "Execution ended in error.")
	case "terminated":
		parts = append(parts, "Execution was deliberately stopped before completing.")
	}
	// Settlement standing is always stated separately from execution - never folded in.
	switch item.Settlement {
	case "accepted":
		parts = append(parts, "Settled as accepted.")
	case "rejected":
		parts = append(parts, "The settlement record rejected this work.")
	}
	if len(parts) == 0 {
		return nil
	}
	s := strings.Join(parts, " ")
	return &s
}

// openDependencies names the item's dependencies that have not finished settled work, as
// "name (standing)" strings - the WHY behind WAITING_ON_OTHERS.
func openDependencies(item ports.HorizonItem, byID map[string]ports.HorizonItem) []string {
	var out []string
	for _, depID := range item.DependsOn {
		dep, ok := byID[depID]
		if !ok {
			out = append(out, depID+" (not in plan)")
			continue
		}
		if dep.Execution == "completed" && dep.Settlement == "accepted" {
			continue
		}
		out = append(out, fmt.Sprintf("%s (%s, settlement %s)", dep.Name, dep.Execution, dep.Settlement))
	}
	return out
}

func stageStatusFor(children []ports.HorizonItem) string {
	if len(children) == 0 {
		return "WAITING"
	}
	allSettled := true
	started := false
	for _, c := range children {
		if c.Execution == "active" || c.Execution == "enabled" {
			return "IN_PROGRESS"
		}
		if c.Execution == "pending" {
			allSettled = false
		} else {
			started = true
		}
		if c.Settlement != "accepted" {
			allSettled = false
		}
	}
	switch {
	case allSettled:
		return "COMPLETED"
	case started:
		return "IN_PROGRESS"
	default:
		return "WAITING"
	}
}

func stageBadgeFor(status string, n int) string {
	switch status {
	case "COMPLETED":
		return "Stage complete"
	case "IN_PROGRESS":
		return "Stage in progress"
	default:
		return fmt.Sprintf("Stage (%d items)", n)
	}
}

// summaryFor builds the headline block. phase is the case state; the status phrase counts what
// needs a decision; progress folds BOTH standings (an item counts once its work is executed and
// its settlement recorded - execution alone is not progress toward done).
func summaryFor(facts CaseFacts) contract.WorldSummary {
	items := len(facts.Horizon.Items)
	ready := 0
	progressed := 0
	for _, it := range facts.Horizon.Items {
		if it.Execution == "enabled" {
			ready++
		}
		if it.Execution == "completed" && it.Settlement == "accepted" {
			progressed++
		}
	}
	state := facts.Horizon.State
	var headline, phrase string
	switch state {
	case "completed":
		headline = "Case completed"
		phrase = "All required work settled."
	case "terminated":
		headline = "Case terminated"
		phrase = "This case was stopped before completion."
		if facts.Record.CloseReason != "" {
			phrase += " Reason: " + facts.Record.CloseReason + "."
		}
	case "awaiting_approval":
		headline = "A decision is waiting on your approval"
		phrase = "The case is parked until the pending approval is resolved."
	default:
		headline = "Case in progress"
		switch {
		case len(facts.Approvals) > 0:
			phrase = "An approval is open; the case cannot settle past it."
		case ready > 0:
			phrase = fmt.Sprintf("%d item(s) ready to execute.", ready)
		default:
			phrase = "Waiting on entry criteria."
		}
	}
	var pct *float64
	if items > 0 {
		p := float64(progressed) / float64(items) * 100
		pct = &p
	}
	return contract.WorldSummary{
		Headline:        headline,
		Phase:           state,
		StatusPhrase:    phrase,
		ProgressPercent: pct,
	}
}

// attentionFor picks the object a human should look at: the highest-salience object that offers
// an action or demands attention, else the highest-salience object at all.
func attentionFor(objects []contract.CognitiveObject, caseID string) contract.AttentionFocus {
	ranked := append([]contract.CognitiveObject(nil), objects...)
	sort.SliceStable(ranked, func(i, j int) bool { return ranked[i].Salience > ranked[j].Salience })
	var rank []string
	primary := ""
	for _, obj := range ranked {
		rank = append(rank, obj.ID)
		if primary == "" && (len(obj.Actions) > 0 || obj.Status == "ACTION_REQUIRED" || obj.Status == "READY_TO_BEGIN") {
			primary = obj.ID
		}
	}
	if primary == "" && len(rank) > 0 {
		primary = rank[0]
	}
	if primary == "" {
		primary = caseID
	}
	out := contract.AttentionFocus{PrimaryObjectID: primary, SalienceRank: rank}
	for _, obj := range objects {
		if obj.ID != primary {
			continue
		}
		if len(obj.Actions) > 0 {
			n := obj.Name + " is actionable: " + strings.ToLower(obj.Status) + "."
			out.Narration = &n
		} else if obj.Explanation != nil {
			n := obj.Name + ": " + *obj.Explanation
			out.Narration = &n
		}
		break
	}
	return out
}

func humanList(items []string) string {
	switch len(items) {
	case 0:
		return ""
	case 1:
		return items[0]
	case 2:
		return items[0] + " and " + items[1]
	default:
		return strings.Join(items[:len(items)-1], ", ") + ", and " + items[len(items)-1]
	}
}

// titleize turns a stage id into a display name ("prep-stage" -> "Prep-stage").
func titleize(id string) string {
	if id == "" {
		return id
	}
	return strings.ToUpper(id[:1]) + id[1:]
}
