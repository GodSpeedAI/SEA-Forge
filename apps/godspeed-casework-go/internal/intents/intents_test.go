// The intent translator's unit tests: every mapped kind gets an accepted path, at least one
// refusal class, and idempotent replay; the two permanent plan teeth (operator APPROVE_HUMAN_TASK
// -> UNAUTHORIZED_ROLE with no kernel call; an older cursor -> STALE_PROJECTION with no kernel
// write) are asserted here against a recording fake and again against the real kernel in the
// live tests.
package intents

import (
	"context"
	"fmt"
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// fakeClassRefusal carries a kernel error class through the ports.ClassRefusal seam (in
// production the sfwp.Refusal at the bottom of the adapter's error chain implements it).
type fakeClassRefusal struct{ class string }

func (f *fakeClassRefusal) Error() string        { return f.class }
func (f *fakeClassRefusal) RefusalClass() string { return f.class }

func kernelDenied(class, message string) error {
	return apperr.New(apperr.KindAuthorityDenied, "", "verb", message).With(&fakeClassRefusal{class: class})
}

func kernelInvalid(class, message string) error {
	return apperr.New(apperr.KindInvalid, "", "verb", message).With(&fakeClassRefusal{class: class})
}

func kernelUnavailable(class, message string) error {
	return apperr.New(apperr.KindUnavailable, "", "verb", message).With(&fakeClassRefusal{class: class})
}

// kernelInternal mirrors the real wire shape of a class the adapter's kind() does not know
// (e.g. plan_schema_error arrives with the default internal kind).
func kernelInternal(class, message string) error {
	return apperr.New(apperr.KindInternal, "", "verb", message).With(&fakeClassRefusal{class: class})
}

// fakeAuth records every governed call (method, options) and can script an execute failure.
type fakeAuth struct {
	ports.CaseAuthorityPort // nil-embedded: any unexpected call panics loudly

	calls     []string
	mutations []string // only the kernel-writing calls, in order
	lastOpts  ports.GovernedOptions
	// lastReopenReason is the reason the last ReopenCase call carried to the authority.
	lastReopenReason string

	listCases []ports.CaseRecord
	approvals []ports.ApprovalRecord
	commits   int
	decided   []string
	commitID  string
	lastItem  ports.CaseItemProposal

	executeErr error
}

func (f *fakeAuth) ListCases(ctx context.Context) ([]ports.CaseRecord, error) {
	f.calls = append(f.calls, "list_cases")
	return f.listCases, nil
}

func (f *fakeAuth) PendingApprovals(ctx context.Context, ref ports.CaseRef) ([]ports.ApprovalRecord, error) {
	f.calls = append(f.calls, "approval_list")
	return f.approvals, nil
}

func (f *fakeAuth) CommitCase(ctx context.Context, draft ports.CaseDraft, pin ports.PreconditionDigest, opts ports.GovernedOptions) (ports.CommitReceipt, error) {
	f.calls = append(f.calls, "case_commit")
	f.mutations = append(f.mutations, "case_commit")
	f.lastOpts = opts
	f.commits++
	f.commitID = opts.RequestID
	return ports.CommitReceipt{CaseID: "case_new", State: "active"}, nil
}

func (f *fakeAuth) DecideApproval(ctx context.Context, approvalID string, approve bool, note string, opts ports.GovernedOptions) error {
	f.calls = append(f.calls, "approval_decide")
	f.mutations = append(f.mutations, "approval_decide")
	f.lastOpts = opts
	f.decided = append(f.decided, approvalID)
	return nil
}

func (f *fakeAuth) AddCaseItem(ctx context.Context, ref ports.CaseRef, item ports.CaseItemProposal, opts ports.GovernedOptions) (string, error) {
	f.calls = append(f.calls, "case_add_item")
	f.mutations = append(f.mutations, "case_add_item")
	f.lastOpts = opts
	f.lastItem = item
	return "item-disc-1", nil
}

func (f *fakeAuth) ReopenCase(ctx context.Context, ref ports.CaseRef, reason string, opts ports.GovernedOptions) error {
	f.lastReopenReason = reason
	f.calls = append(f.calls, "case_reopen")
	f.mutations = append(f.mutations, "case_reopen")
	f.lastOpts = opts
	return nil
}

func (f *fakeAuth) TerminateCase(ctx context.Context, ref ports.CaseRef, reason string, opts ports.GovernedOptions) error {
	f.calls = append(f.calls, "case_terminate")
	f.mutations = append(f.mutations, "case_terminate")
	f.lastOpts = opts
	return nil
}

func (f *fakeAuth) ExecuteItem(ctx context.Context, ref ports.CaseRef, itemID string, opts ports.GovernedOptions) (ports.AdvanceReport, error) {
	f.calls = append(f.calls, "item_execute")
	f.lastOpts = opts
	if f.executeErr != nil {
		return ports.AdvanceReport{}, f.executeErr
	}
	f.mutations = append(f.mutations, "item_execute")
	return ports.AdvanceReport{CaseID: string(ref), State: "active"}, nil
}

func (f *fakeAuth) CompleteHumanTask(ctx context.Context, ref ports.CaseRef, itemID string, justification string, opts ports.GovernedOptions) error {
	f.calls = append(f.calls, "human_task_complete")
	f.mutations = append(f.mutations, "human_task_complete")
	f.lastOpts = opts
	return nil
}

func (f *fakeAuth) GetArtifact(ctx context.Context, digest string) (ports.ArtifactContent, error) {
	f.calls = append(f.calls, "artifact_get")
	return ports.ArtifactContent{Digest: digest}, nil
}

// fakeCursors is the relay side of the staleness guard.
type fakeCursors struct {
	cursors map[string]string
}

func (f *fakeCursors) CursorForCase(caseID string) (string, bool) {
	c, ok := f.cursors[caseID]
	return c, ok
}

func (f *fakeCursors) WaitForCaseAdvance(ctx context.Context, caseID, before string) string {
	if c, ok := f.cursors[caseID]; ok {
		return c
	}
	return "01NEW"
}

func newHandler(auth *fakeAuth, cursors map[string]string) *Handler {
	return NewHandler(auth, &fakeCursors{cursors: cursors}, auth, Options{
		Gateway:   ports.ActorClaim{ActorID: "gateway", Role: "service"},
		PolicyRef: "authority/active-policy.json",
	})
}

func operatorIntent(id, action string, params map[string]any) contract.InteractionIntent {
	return contract.InteractionIntent{
		IntentID:       id,
		Kind:           "CONSEQUENTIAL_CASE",
		ActionName:     action,
		TargetObjectID: "task_prepare",
		CaseID:         "case_1",
		ClientCursor:   "01CURSOR",
		Actor:          contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters:     params,
	}
}

func rsoIntent(id, action string, params map[string]any) contract.InteractionIntent {
	in := operatorIntent(id, action, params)
	in.Actor = contract.IntentActor{ActorID: "rso_local", Role: "R-SO"}
	return in
}

func refusalKind(t *testing.T, resp contract.IntentResponse) string {
	t.Helper()
	if resp.Success || resp.Refusal == nil {
		t.Fatalf("expected a refusal, got %+v", resp)
	}
	return resp.Refusal.RefusalKind
}

func TestAcceptedPathsUseGovernanceAndIntentIDAsRequestID(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	in := operatorIntent("i-gov", "EXECUTE_ITEM", map[string]any{"item_id": "task_prepare"})
	resp := h.Handle(context.Background(), in)
	if !resp.Success {
		t.Fatalf("accepted intent refused: %+v", resp.Refusal)
	}
	if len(auth.mutations) != 1 || auth.mutations[0] != "item_execute" {
		t.Fatalf("mutations = %v", auth.mutations)
	}
	// The T02 delegation wiring: the gateway principal is the actor, the end user rides beside it.
	if auth.lastOpts.Governance.Actor.ActorID != "gateway" || auth.lastOpts.Governance.Actor.Role != "service" {
		t.Fatalf("actor block must be the gateway principal: %+v", auth.lastOpts.Governance)
	}
	if auth.lastOpts.Governance.OnBehalfOf == nil ||
		auth.lastOpts.Governance.OnBehalfOf.ActorID != "operator_local" ||
		auth.lastOpts.Governance.OnBehalfOf.Role != "operator" {
		t.Fatalf("on_behalf_of must carry the acting end user: %+v", auth.lastOpts.Governance)
	}
	if auth.lastOpts.RequestID != "i-gov" {
		t.Fatalf("the kernel request_id must be the intent id, got %q", auth.lastOpts.RequestID)
	}
	if auth.lastOpts.Policy != "authority/active-policy.json" {
		t.Fatalf("the governed verbs authorize against the configured policy, got %q", auth.lastOpts.Policy)
	}
}

func TestEveryIntentKindAcceptedPath(t *testing.T) {
	cases := []struct {
		name     string
		in       func() contract.InteractionIntent
		wantCall string
	}{
		{"propose", func() contract.InteractionIntent {
			in := operatorIntent("i-propose", "PROPOSE_CASE", map[string]any{
				"template_ref": "e2e-sentry-chain@0.1.0", "preflight_digest": "sha256:385e",
				"params": map[string]any{"dataset_name": "orders"}})
			in.ClientCursor = "" // no target case yet: staleness does not apply to a commit
			return in
		}, "case_commit"},
		{"add-disc", func() contract.InteractionIntent {
			return operatorIntent("i-disc", "ADD_DISCRETIONARY_WORK", map[string]any{
				"case_id": "case_1", "stage_id": "stage_a", "kind": "work_item",
				"title": "Rollback rehearsal", "justification": "needed"})
		}, "case_add_item"},
		{"execute", func() contract.InteractionIntent {
			return operatorIntent("i-exec", "EXECUTE_ITEM", map[string]any{"item_id": "task_prepare"})
		}, "item_execute"},
		{"complete", func() contract.InteractionIntent {
			return operatorIntent("i-comp", "COMPLETE_HUMAN_TASK", map[string]any{
				"item_id": "signoff", "result": map[string]any{"ok": true}, "justification": "done"})
		}, "human_task_complete"},
		{"approve", func() contract.InteractionIntent {
			in := rsoIntent("i-appr", "APPROVE_HUMAN_TASK", nil)
			just := "signed off"
			in.Justification = &just
			return in
		}, "approval_decide"},
		{"reject", func() contract.InteractionIntent {
			in := rsoIntent("i-rej", "REJECT_HUMAN_TASK", nil)
			just := "not good"
			in.Justification = &just
			return in
		}, "approval_decide"},
		{"reopen", func() contract.InteractionIntent {
			return operatorIntent("i-reopen", "REOPEN_CASE", map[string]any{"case_id": "case_1", "reason": "regression"})
		}, "case_reopen"},
		{"terminate", func() contract.InteractionIntent {
			return operatorIntent("i-term", "TERMINATE_CASE", map[string]any{"case_id": "case_1", "reason": "duplicate"})
		}, "case_terminate"},
		{"open", func() contract.InteractionIntent {
			in := operatorIntent("i-open", "OPEN_ARTIFACT", map[string]any{"digest": "sha256:abc"})
			in.ClientCursor = "" // a read: no staleness, no new cursor
			return in
		}, "artifact_get"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}},
				approvals: []ports.ApprovalRecord{{ApprovalID: "apr_1", CaseID: "case_1", ItemID: "task_prepare"}}}
			h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
			resp := h.Handle(context.Background(), tc.in())
			if !resp.Success {
				t.Fatalf("refused: %+v", resp.Refusal)
			}
			found := false
			for _, c := range auth.calls {
				if c == tc.wantCall {
					found = true
				}
			}
			if !found {
				t.Fatalf("calls %v must contain %s", auth.calls, tc.wantCall)
			}
		})
	}
}

// The T06 fix F2 contract: the gateway constructs a minimal VALID plan item from the intent
// payload - a governed write_file operation (the kernel refuses an operation-less sandboxed task
// with plan_schema_error), settlement criteria naming the written path, and depends_on for the
// stage anchor. proposed_by is never sent: the kernel overwrites it with the verified actor.
func TestAddDiscretionaryConstructsAValidPlanItem(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	summary := "Rehearse the rollback path before the quarter closes."
	resp := h.Handle(context.Background(), operatorIntent("i-obj", "ADD_DISCRETIONARY_WORK", map[string]any{
		"case_id": "case_1", "stage_id": "task_prepare", "kind": "work_item",
		"title": "Rollback Rehearsal (Q3)", "summary": summary, "justification": "needed"}))
	if !resp.Success {
		t.Fatalf("refused: %+v", resp.Refusal)
	}
	item := auth.lastItem
	if item.ItemID == "" || len(item.ItemID) > 128 {
		t.Fatalf("the proposal needs a well-formed item id, got %q", item.ItemID)
	}
	if item.Name != "Rollback Rehearsal (Q3)" {
		t.Fatalf("the item name comes from the payload title, got %q", item.Name)
	}
	if item.Kind != "sandboxed_task" || item.SandboxClass != "local" {
		t.Fatalf("kind/sandbox class: %+v", item)
	}
	if len(item.Operations) != 1 {
		t.Fatalf("a sandboxed task must carry exactly one operation, got %v", item.Operations)
	}
	op := item.Operations[0]
	if op.Kind != "write_file" {
		t.Fatalf("the operation must be a governed write_file, got %q", op.Kind)
	}
	if op.Path != "discretionary/rollback-rehearsal-q3.md" {
		t.Fatalf("the write path is the kebab-case slug of the title under discretionary/, got %q", op.Path)
	}
	if op.ContentHint != summary {
		t.Fatalf("content_hint is the payload summary, got %q", op.ContentHint)
	}
	if len(item.RequiredArtifacts) != 1 || item.RequiredArtifacts[0] != op.Path {
		t.Fatalf("settlement criteria must require the written path as an artifact, got %v", item.RequiredArtifacts)
	}
	if len(item.DependsOn) != 1 || item.DependsOn[0] != "task_prepare" {
		t.Fatalf("the stage anchor must become depends_on, got %v", item.DependsOn)
	}
	if item.ParentStage != "" {
		t.Fatalf("parent_stage must stay unset (the anchor may not be a stage item), got %q", item.ParentStage)
	}
	if resp.ResultingObject == nil || resp.ResultingObject.ID != "item-disc-1" ||
		resp.ResultingObject.Status != "WAITING" || resp.ResultingObject.Badge != "Proposed" {
		t.Fatalf("resulting object: %+v", resp.ResultingObject)
	}
	if resp.ResultingObject.ParentID != nil {
		t.Fatalf("the anchor is a dependency, not a parent; parent_id must be unset: %+v", resp.ResultingObject.ParentID)
	}
}

func TestAddDiscretionaryDefaultsAndUnanchoredItems(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	// No kind: defaults to sandboxed_task. No stage_id: no entry sentries. No summary:
	// content_hint falls back to the mandatory justification.
	resp := h.Handle(context.Background(), operatorIntent("i-def", "ADD_DISCRETIONARY_WORK", map[string]any{
		"case_id": "case_1", "title": "  --  ", "justification": "because the case needs it"}))
	if !resp.Success {
		t.Fatalf("kind and stage_id are optional now, refused: %+v", resp.Refusal)
	}
	item := auth.lastItem
	if item.Kind != "sandboxed_task" {
		t.Fatalf("an absent kind must default to sandboxed_task, got %q", item.Kind)
	}
	if item.DependsOn != nil {
		t.Fatalf("an unanchored proposal carries no dependency, got %v", item.DependsOn)
	}
	if item.Operations[0].ContentHint != "because the case needs it" {
		t.Fatalf("content_hint must fall back to the justification, got %q", item.Operations[0].ContentHint)
	}
	if item.Operations[0].Path != "discretionary/item.md" {
		t.Fatalf("a slugless title falls back to a valid path, got %q", item.Operations[0].Path)
	}
}

// Stage and milestone proposals stay operation-less: the kernel refuses operations on
// non-sandboxed items ("non-sandboxed item cannot have operations").
func TestAddDiscretionaryStageAndMilestoneCarryNoOperations(t *testing.T) {
	for _, kind := range []string{"stage", "milestone"} {
		auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}}
		h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
		resp := h.Handle(context.Background(), operatorIntent("i-"+kind, "ADD_DISCRETIONARY_WORK", map[string]any{
			"case_id": "case_1", "kind": kind, "title": "Gate review", "justification": "needed"}))
		if !resp.Success {
			t.Fatalf("%s refused: %+v", kind, resp.Refusal)
		}
		if len(auth.lastItem.Operations) != 0 || len(auth.lastItem.RequiredArtifacts) != 0 {
			t.Fatalf("a %s item must not carry operations or artifact requirements: %+v", kind, auth.lastItem)
		}
		if kind == "milestone" && resp.ResultingObject.Kind != "milestone" {
			t.Fatalf("a milestone proposal renders as a milestone object, got %+v", resp.ResultingObject)
		}
	}
}

func TestToothOperatorApproveRefusedWithNoKernelCall(t *testing.T) {
	auth := &fakeAuth{}
	h := newHandler(auth, nil)
	resp := h.Handle(context.Background(), operatorIntent("i-tooth-1", "APPROVE_HUMAN_TASK", nil))
	if kind := refusalKind(t, resp); kind != "UNAUTHORIZED_ROLE" {
		t.Fatalf("operator approve must refuse UNAUTHORIZED_ROLE, got %s", kind)
	}
	if len(auth.calls) != 0 {
		t.Fatalf("the kernel must receive no call, got %v", auth.calls)
	}
	if resp.ErrorCode == nil || *resp.ErrorCode != "UNAUTHORIZED_ROLE" {
		t.Fatal("the legacy error_code field must mirror the refusal kind (additive contract)")
	}
}

func TestToothStaleCursorRefusedWithNoKernelWrite(t *testing.T) {
	auth := &fakeAuth{}
	h := newHandler(auth, map[string]string{"case_1": "01NEWER"})
	resp := h.Handle(context.Background(), operatorIntent("i-tooth-2", "EXECUTE_ITEM", map[string]any{"item_id": "task_prepare"}))
	if kind := refusalKind(t, resp); kind != "STALE_PROJECTION" {
		t.Fatalf("stale cursor must refuse STALE_PROJECTION, got %s", kind)
	}
	if resp.Refusal.CurrentCursor == nil || *resp.Refusal.CurrentCursor != "01NEWER" {
		t.Fatal("a stale refusal must carry the current cursor for the refetch")
	}
	if len(auth.mutations) != 0 {
		t.Fatalf("a stale intent must produce no kernel write, got %v", auth.mutations)
	}
	if len(auth.calls) != 0 {
		t.Fatalf("the staleness guard reads only gateway state: %v", auth.calls)
	}
}

func TestMissingCursorOnMutationIsInvalid(t *testing.T) {
	auth := &fakeAuth{}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	in := operatorIntent("i-nocur", "EXECUTE_ITEM", map[string]any{"item_id": "task_prepare"})
	in.ClientCursor = ""
	if kind := refusalKind(t, h.Handle(context.Background(), in)); kind != "INVALID" {
		t.Fatalf("missing client_cursor must be INVALID, got %s", kind)
	}
	if len(auth.calls) != 0 {
		t.Fatalf("no kernel call may follow, got %v", auth.calls)
	}
}

func TestJustificationAndReasonRefusals(t *testing.T) {
	cases := []struct {
		name string
		in   func() contract.InteractionIntent
		want string
	}{
		{"add-disc", func() contract.InteractionIntent {
			return operatorIntent("i-j1", "ADD_DISCRETIONARY_WORK", map[string]any{
				"case_id": "case_1", "stage_id": "s", "kind": "work_item", "title": "t"})
		}, "JUSTIFICATION_REQUIRED"},
		{"complete", func() contract.InteractionIntent {
			return operatorIntent("i-j2", "COMPLETE_HUMAN_TASK", map[string]any{"item_id": "x", "result": map[string]any{}})
		}, "JUSTIFICATION_REQUIRED"},
		{"approve-envelope", func() contract.InteractionIntent {
			return rsoIntent("i-j3", "APPROVE_HUMAN_TASK", nil)
		}, "JUSTIFICATION_REQUIRED"},
		{"terminate-blank-reason", func() contract.InteractionIntent {
			return operatorIntent("i-j4", "TERMINATE_CASE", map[string]any{"case_id": "case_1", "reason": ""})
		}, "INVALID"}, // pinned by the T01 golden
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			auth := &fakeAuth{}
			h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
			if kind := refusalKind(t, h.Handle(context.Background(), tc.in())); kind != tc.want {
				t.Fatalf("want %s, got %s", tc.want, kind)
			}
			if len(auth.calls) != 0 {
				t.Fatalf("refusals must precede any kernel call, got %v", auth.calls)
			}
		})
	}
}

func TestShapeRefusals(t *testing.T) {
	auth := &fakeAuth{}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})

	if kind := refusalKind(t, h.Handle(context.Background(), contract.InteractionIntent{Kind: "CONSEQUENTIAL_CASE"})); kind != "INVALID" {
		t.Fatal("missing intent id must be INVALID")
	}
	if kind := refusalKind(t, h.Handle(context.Background(), operatorIntent("i1", "TELEPORT", nil))); kind != "INVALID" {
		t.Fatal("unknown action must be INVALID")
	}
	in := operatorIntent("i2", "EXECUTE_ITEM", map[string]any{"item_id": "x"})
	in.Kind = "REACT_LOCAL"
	if kind := refusalKind(t, h.Handle(context.Background(), in)); kind != "INVALID" {
		t.Fatal("a non-consequential kind must not cross the gateway")
	}
	in = operatorIntent("i3", "EXECUTE_ITEM", map[string]any{"surprise": 1})
	if kind := refusalKind(t, h.Handle(context.Background(), in)); kind != "INVALID" {
		t.Fatal("unknown payload fields must be INVALID")
	}
	in = operatorIntent("i4", "EXECUTE_ITEM", nil)
	if kind := refusalKind(t, h.Handle(context.Background(), in)); kind != "INVALID" {
		t.Fatal("missing typed payload must be INVALID")
	}
	in = operatorIntent("i5", "PROPOSE_CASE", map[string]any{"template_ref": "t"})
	if kind := refusalKind(t, h.Handle(context.Background(), in)); kind != "INVALID" {
		t.Fatal("PROPOSE_CASE without a preflight digest must be INVALID")
	}
	if len(auth.calls) != 0 {
		t.Fatalf("shape refusals must never reach the kernel, got %v", auth.calls)
	}
}

func TestEscalateIsHonestUnavailable(t *testing.T) {
	auth := &fakeAuth{}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	in := operatorIntent("i-esc", "ESCALATE_OR_OVERRIDE", nil)
	just := "the custodian must decide"
	in.Justification = &just
	if kind := refusalKind(t, h.Handle(context.Background(), in)); kind != "UNAVAILABLE" {
		t.Fatalf("escalate must refuse UNAVAILABLE honestly, got %s", kind)
	}
	if len(auth.calls) != 0 {
		t.Fatalf("no kernel call may be fabricated, got %v", auth.calls)
	}
}

func TestApproveResolvesThePendingApprovalFromTheKernelInbox(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}},
		approvals: []ports.ApprovalRecord{{ApprovalID: "apr_1", CaseID: "case_1", ItemID: "task_prepare"}}}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	just := "looks good"
	in := rsoIntent("i-apr", "APPROVE_HUMAN_TASK", nil)
	in.Justification = &just
	resp := h.Handle(context.Background(), in)
	if !resp.Success {
		t.Fatalf("approve refused: %+v", resp.Refusal)
	}
	if len(auth.decided) != 1 || auth.decided[0] != "apr_1" {
		t.Fatalf("decided approvals: %v", auth.decided)
	}

	// Nothing pending on the item: INVALID.
	auth2 := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}}
	h2 := newHandler(auth2, map[string]string{"case_1": "01CURSOR"})
	in2 := rsoIntent("i-apr2", "APPROVE_HUMAN_TASK", nil)
	just2 := "looks good"
	in2.Justification = &just2
	if kind := refusalKind(t, h2.Handle(context.Background(), in2)); kind != "INVALID" {
		t.Fatalf("approve with no pending approval must be INVALID, got %s", kind)
	}
}

func TestKernelRefusalsMapOntoTypedKinds(t *testing.T) {
	cases := []struct {
		name     string
		err      error
		wantKind string
	}{
		{"role-not-held", kernelDenied("identity_role_not_held", "role not held"), "UNAUTHORIZED_ROLE"},
		{"sod", kernelDenied("separation_of_duty", "soD"), "SOD_VIOLATION"},
		{"self-approval", kernelDenied("separation_of_duty_unverifiable", "soD2"), "SOD_VIOLATION"},
		{"delegation", kernelDenied("identity_delegation_refused", "nope"), "AUTHORITY_DENIED"},
		{"input", kernelInvalid("input_error", "bad input"), "INVALID"},
		{"plan-schema", kernelInternal("plan_schema_error", "sandboxed task item-disc-x must declare at least one operation"), "INVALID"},
		{"busy", kernelUnavailable("server_busy", "busy"), "UNAVAILABLE"},
		{"stale-preflight", kernelInvalid("precondition_failed", "stale"), "STALE_PROJECTION"},
		{"unmapped", kernelDenied("cell_dark", "mystery"), "AUTHORITY_DENIED"},
		{"plain-unavailable", apperr.New(apperr.KindUnavailable, "", "verb", "socket gone"), "UNAVAILABLE"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}, executeErr: tc.err}
			h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
			resp := h.Handle(context.Background(), operatorIntent("i-"+tc.name, "EXECUTE_ITEM", map[string]any{"item_id": "x"}))
			if kind := refusalKind(t, resp); kind != tc.wantKind {
				t.Fatalf("want %s, got %s (%+v)", tc.wantKind, kind, resp.Refusal)
			}
		})
	}
}

func TestIdempotentReplay(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	in := operatorIntent("i-replay", "EXECUTE_ITEM", map[string]any{"item_id": "task_prepare"})
	first := h.Handle(context.Background(), in)
	second := h.Handle(context.Background(), in)
	if first != second {
		t.Fatalf("replay must return the recorded outcome verbatim:\n%+v\n%+v", first, second)
	}
	if len(auth.calls) != 1 {
		t.Fatalf("the kernel must be called exactly once across the replay, got %v", auth.calls)
	}
	// Same id, different body: refused as invalid.
	other := operatorIntent("i-replay", "EXECUTE_ITEM", map[string]any{"item_id": "task_other"})
	if kind := refusalKind(t, h.Handle(context.Background(), other)); kind != "INVALID" {
		t.Fatalf("a conflicting body must be refused, got %s", kind)
	}
}

func TestUntrackedAndUnknownCasesRefuseHonest(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}}}
	h := newHandler(auth, map[string]string{}) // the relay saw no frame for case_1 yet
	if kind := refusalKind(t, h.Handle(context.Background(), operatorIntent("i-sync", "EXECUTE_ITEM", map[string]any{"item_id": "x"}))); kind != "UNAVAILABLE" {
		t.Fatalf("a syncing projection must refuse UNAVAILABLE, got %s", kind)
	}

	auth2 := &fakeAuth{listCases: []ports.CaseRecord{}}
	h2 := newHandler(auth2, map[string]string{})
	if kind := refusalKind(t, h2.Handle(context.Background(), operatorIntent("i-unknown", "EXECUTE_ITEM", map[string]any{"item_id": "x"}))); kind != "INVALID" {
		t.Fatalf("an unknown case must be INVALID, got %s", kind)
	}
}

func TestRefusalMessageNamesTheKernelClassVerbatim(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "active"}},
		executeErr: kernelDenied("cell_dark", "mystery refusal")}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	resp := h.Handle(context.Background(), operatorIntent("i-verb", "EXECUTE_ITEM", map[string]any{"item_id": "x"}))
	if !strings.Contains(resp.Refusal.Message, "cell_dark") || !strings.Contains(resp.Refusal.Message, "mystery refusal") {
		t.Fatalf("the kernel's own class and message must ride verbatim: %q", resp.Refusal.Message)
	}
}

// The reopen reason the operator typed reaches the authority (it used to be validated and dropped,
// so the kernel's CaseReopened event could never say why).
func TestReopenCaseForwardsItsReasonToTheAuthority(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "completed"}}}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	resp := h.Handle(context.Background(), operatorIntent("i-reopen-reason", "REOPEN_CASE", map[string]any{"case_id": "case_1", "reason": "late evidence arrived"}))
	if !resp.Success {
		t.Fatalf("refused: %+v", resp.Refusal)
	}
	if auth.lastReopenReason != "late evidence arrived" {
		t.Fatalf("authority received reason %q", auth.lastReopenReason)
	}
}

// T11 review F7: the free-text reason lands in the kernel ledger and trace, and the intent id becomes
// a header, a log field and the SFWP request_id: both are bounded and control-byte free, and the
// replay cache is a bounded FIFO.
func TestT11IntentInputBounds(t *testing.T) {
	auth := &fakeAuth{listCases: []ports.CaseRecord{{Ref: "case_1", State: "completed"}}}
	h := newHandler(auth, map[string]string{"case_1": "01CURSOR"})
	for name, reason := range map[string]string{
		"too long": strings.Repeat("x", maxReasonBytes+1),
		"control":  "ok\x1b[31mred",
		"nul":      "a\x00b",
	} {
		resp := h.Handle(context.Background(), operatorIntent("i-"+strings.ReplaceAll(name, " ", "-"), "REOPEN_CASE", map[string]any{"case_id": "case_1", "reason": reason}))
		if resp.Success || resp.Refusal == nil || resp.Refusal.RefusalKind != RefInvalid {
			t.Fatalf("%s reason must be INVALID, got %+v", name, resp)
		}
	}
	if auth.lastReopenReason != "" {
		t.Fatalf("a refused reason reached the authority: %q", auth.lastReopenReason)
	}
	for _, id := range []string{strings.Repeat("a", maxIntentIDLen+1), "has space", "line\nbreak", "café"} {
		resp := h.Handle(context.Background(), operatorIntent(id, "REOPEN_CASE", map[string]any{"case_id": "case_1", "reason": "fine"}))
		if resp.Success {
			t.Fatalf("intent id %q must be refused", id)
		}
	}
	for i := 0; i < maxReplayRecords+50; i++ {
		h.record(fmt.Sprintf("r-%d", i), [32]byte{}, contract.IntentResponse{})
	}
	if len(h.outcomes) > maxReplayRecords || len(h.order) > maxReplayRecords {
		t.Fatalf("replay cache unbounded: %d", len(h.outcomes))
	}
}
