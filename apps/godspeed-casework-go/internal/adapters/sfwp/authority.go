// Authority adapts the live SFWP client to ports.CaseAuthorityPort. Every wire shape stops here:
// what crosses out is the application's own vocabulary (REQ-ARCH-002). It adds no authority of its
// own - every method asks the governed kernel and reports what it decided.
package sfwp

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// Authority is the live case-authority adapter over one kernel socket.
type Authority struct {
	client *Client
	// clientName labels this gateway in system.hello.
	clientName string
}

// NewAuthority builds the adapter over a configured client.
func NewAuthority(c *Client) *Authority {
	return &Authority{client: c, clientName: "godspeed-casework-go"}
}

// Client exposes the underlying client (subscription management, request status recovery).
func (a *Authority) Client() *Client { return a.client }

// Health probes the authority the way preflight needs: negotiate the protocol, read the
// authority's own readiness projection, and confirm the connection's identity resolves. Every
// part must answer or the probe fails honestly.
func (a *Authority) Health(ctx context.Context) error {
	hello, err := a.client.Hello(ctx, a.clientName)
	if err != nil {
		return err
	}
	if hello.ServerProtocolVersion != ProtocolVersion {
		return apperr.New(apperr.KindUnavailable, "", "health",
			"the governed authority speaks protocol major "+hello.ServerProtocolVersion+
				", this gateway requires "+ProtocolVersion)
	}
	if _, err := a.Readiness(ctx); err != nil {
		return err
	}
	if _, err := a.ResolveIdentity(ctx, ports.Governance{}); err != nil {
		return err
	}
	return nil
}

func governance(g ports.Governance) Governance {
	out := Governance{Actor: Actor{ActorID: g.Actor.ActorID, Role: g.Actor.Role}}
	if g.OnBehalfOf != nil {
		out.OnBehalfOf = &Actor{ActorID: g.OnBehalfOf.ActorID, Role: g.OnBehalfOf.Role}
	}
	return out
}

func optsRequestID(opts ports.GovernedOptions, c *Client, verb string) string {
	if opts.RequestID != "" {
		return opts.RequestID
	}
	return c.NewRequestID(verb)
}

// ListCases implements ports.CaseAuthorityPort.
func (a *Authority) ListCases(ctx context.Context) ([]ports.CaseRecord, error) {
	resp, err := a.client.Do(ctx, NewCaseList())
	if err != nil {
		return nil, err
	}
	var view CaseListView
	if err := resp.Into(&view); err != nil {
		return nil, err
	}
	out := make([]ports.CaseRecord, 0, len(view.Cases))
	for _, row := range view.Cases {
		rec, err := caseRecordOf(row)
		if err != nil {
			return nil, err
		}
		out = append(out, rec)
	}
	return out, nil
}

func caseRecordOf(row CaseSummaryView) (ports.CaseRecord, error) {
	created, err := parseTime(row.CreatedAt, "case created_at")
	if err != nil {
		return ports.CaseRecord{}, err
	}
	rec := ports.CaseRecord{
		Ref:       ports.CaseRef(row.CaseID),
		State:     row.CaseState,
		Summary:   row.Summary,
		CreatedAt: created,
		RunCount:  row.RunCount,
	}
	if row.ClosedAt != nil && *row.ClosedAt != "" {
		closed, err := parseTime(*row.ClosedAt, "case closed_at")
		if err != nil {
			return ports.CaseRecord{}, err
		}
		rec.ClosedAt = closed
	}
	if row.CloseReason != nil {
		rec.CloseReason = *row.CloseReason
	}
	return rec, nil
}

func parseTime(value, what string) (time.Time, error) {
	if value == "" {
		return time.Time{}, nil
	}
	t, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return time.Time{}, apperr.Wrap(apperr.KindInternal, "", "decode",
			"the authority reported a non-RFC3339 "+what, err)
	}
	return t, nil
}

// CaseOverview implements ports.CaseAuthorityPort.
func (a *Authority) CaseOverview(ctx context.Context, ref ports.CaseRef) (ports.CaseOverview, error) {
	resp, err := a.client.Do(ctx, NewCaseGetOverview(string(ref)))
	if err != nil {
		return ports.CaseOverview{}, err
	}
	var view CaseOverviewView
	if err := resp.Into(&view); err != nil {
		return ports.CaseOverview{}, err
	}
	created, err := parseTime(view.CreatedAt, "case created_at")
	if err != nil {
		return ports.CaseOverview{}, err
	}
	out := ports.CaseOverview{
		Ref:         ports.CaseRef(view.CaseID),
		State:       view.CaseState,
		Summary:     view.Summary,
		CreatedAt:   created,
		ItemCount:   view.ItemCount,
		Stages:      view.Stages,
		Settlements: make([]ports.SettlementNote, 0, len(view.Settlements)),
	}
	if view.TemplateRef != nil {
		out.TemplateRef = *view.TemplateRef
	}
	for _, s := range view.Settlements {
		settled, err := parseTime(s.SettledAt, "settlement settled_at")
		if err != nil {
			return ports.CaseOverview{}, err
		}
		out.Settlements = append(out.Settlements, ports.SettlementNote{
			Run:            ports.RunRef(s.RunID),
			Status:         s.Status,
			Basis:          s.Basis,
			ReviewRequired: s.ReviewRequired,
			SettledAt:      settled,
		})
	}
	for _, id := range view.RunIDs {
		out.RunIDs = append(out.RunIDs, ports.RunRef(id))
	}
	return out, nil
}

// CaseHorizon implements ports.CaseAuthorityPort.
func (a *Authority) CaseHorizon(ctx context.Context, ref ports.CaseRef) (ports.CaseHorizon, error) {
	resp, err := a.client.Do(ctx, NewCaseGetHorizon(string(ref)))
	if err != nil {
		return ports.CaseHorizon{}, err
	}
	var view CaseHorizonView
	if err := resp.Into(&view); err != nil {
		return ports.CaseHorizon{}, err
	}
	out := ports.CaseHorizon{
		Ref:          ports.CaseRef(view.CaseID),
		State:        view.CaseState,
		EventsFolded: view.EventsFolded,
		Items:        make([]ports.HorizonItem, 0, len(view.Items)),
	}
	if view.LastEventID != nil {
		out.LastEventID = *view.LastEventID
	}
	for _, it := range view.Items {
		item := ports.HorizonItem{
			ItemID:     it.PlanItemID,
			Name:       it.Name,
			Kind:       it.ItemKind,
			Execution:  it.Execution,
			Settlement: it.Settlement,
			DependsOn:  it.DependsOn,
		}
		if it.ParentStage != nil {
			item.ParentStage = *it.ParentStage
		}
		for _, id := range it.RunIDs {
			item.RunIDs = append(item.RunIDs, ports.RunRef(id))
		}
		out.Items = append(out.Items, item)
	}
	return out, nil
}

// RunsList implements ports.CaseAuthorityPort.
func (a *Authority) RunsList(ctx context.Context) ([]ports.RunSummary, error) {
	resp, err := a.client.Do(ctx, NewRunList())
	if err != nil {
		return nil, err
	}
	var view RunListView
	if err := resp.Into(&view); err != nil {
		return nil, err
	}
	out := make([]ports.RunSummary, 0, len(view.Runs))
	for _, row := range view.Runs {
		run := ports.RunSummary{
			RunID:         row.RunID,
			Execution:     row.Execution,
			Settlement:    row.Settlement,
			EvidenceCount: row.EvidenceCount,
		}
		if row.CaseID != nil {
			run.CaseID = *row.CaseID
		}
		if row.PlanItemID != nil {
			run.PlanItemID = *row.PlanItemID
		}
		if row.StartedAt != nil && *row.StartedAt != "" {
			started, err := parseTime(*row.StartedAt, "run started_at")
			if err != nil {
				return nil, err
			}
			run.StartedAt, run.HasStarted = started, true
		}
		if row.FinishedAt != nil && *row.FinishedAt != "" {
			finished, err := parseTime(*row.FinishedAt, "run finished_at")
			if err != nil {
				return nil, err
			}
			run.FinishedAt, run.HasFinished = finished, true
		}
		out = append(out, run)
	}
	return out, nil
}

// RunsListForCase reads only the run records owned by ref. The adapter validates the full
// authority frame before returning any rows so malformed ownership cannot cross the port.
func (a *Authority) RunsListForCase(ctx context.Context, ref ports.CaseRef) (ports.RunListResult, error) {
	req, err := NewRunListForCase(string(ref))
	if err != nil {
		return ports.RunListResult{}, err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		var refusal *Refusal
		if errors.As(err, &refusal) {
			return ports.RunListResult{}, normalizeScopedRunListRefusal(err)
		}
		return ports.RunListResult{}, err
	}
	var view RunListView
	if err := resp.Into(&view); err != nil {
		if resp.Err != nil {
			return ports.RunListResult{}, normalizeScopedRunListRefusal(resp.Err.appErr("run_list"))
		}
		return ports.RunListResult{}, apperr.Wrap(apperr.KindUnavailable, "", "run_list", "the authority returned a malformed run list", err)
	}
	if view.Runs == nil || view.Unreadable == nil {
		return ports.RunListResult{}, unavailableRunList("the authority omitted a required run list array")
	}

	result := ports.RunListResult{
		Runs:          make([]ports.RunSummary, 0, len(view.Runs)),
		UnreadableIDs: make([]string, 0, len(view.Unreadable)),
	}
	seen := make(map[string]struct{}, len(view.Runs)+len(view.Unreadable))
	for _, row := range view.Runs {
		run, err := scopedRunSummary(row, string(ref))
		if err != nil {
			return ports.RunListResult{}, err
		}
		if _, exists := seen[run.RunID]; exists {
			return ports.RunListResult{}, unavailableRunList("the authority repeated a run id")
		}
		seen[run.RunID] = struct{}{}
		result.Runs = append(result.Runs, run)
	}
	for _, id := range view.Unreadable {
		if strings.TrimSpace(id) == "" {
			return ports.RunListResult{}, unavailableRunList("the authority reported a blank unreadable run id")
		}
		if _, exists := seen[id]; exists {
			return ports.RunListResult{}, unavailableRunList("the authority repeated or overlapped a run id")
		}
		seen[id] = struct{}{}
		result.UnreadableIDs = append(result.UnreadableIDs, id)
	}
	return result, nil
}

func normalizeScopedRunListRefusal(err error) error {
	var refusal *Refusal
	if errors.As(err, &refusal) && refusal.RefusalClass() == "unavailable" {
		return apperr.Wrap(apperr.KindUnavailable, "", "run_list", "the authority refused the scoped run list", err)
	}
	return err
}

func scopedRunSummary(row RunSummaryView, caseID string) (ports.RunSummary, error) {
	if strings.TrimSpace(row.RunID) == "" {
		return ports.RunSummary{}, unavailableRunList("the authority reported a blank run id")
	}
	if row.CaseID == nil || strings.TrimSpace(*row.CaseID) == "" || *row.CaseID != caseID {
		return ports.RunSummary{}, unavailableRunList("the authority reported a missing or foreign run case id")
	}
	if row.PlanItemID == nil || strings.TrimSpace(*row.PlanItemID) == "" {
		return ports.RunSummary{}, unavailableRunList("the authority reported a missing run plan item id")
	}
	if !validRunExecution(row.Execution) || !validRunSettlement(row.Settlement) {
		return ports.RunSummary{}, unavailableRunList("the authority reported an unknown run standing")
	}
	if row.EvidenceCount < 0 {
		return ports.RunSummary{}, unavailableRunList("the authority reported a negative run evidence count")
	}
	run := ports.RunSummary{
		RunID:         row.RunID,
		CaseID:        *row.CaseID,
		PlanItemID:    *row.PlanItemID,
		Execution:     row.Execution,
		Settlement:    row.Settlement,
		EvidenceCount: row.EvidenceCount,
	}
	if row.StartedAt != nil {
		started, err := parseScopedRunTime(*row.StartedAt, "started_at")
		if err != nil {
			return ports.RunSummary{}, err
		}
		run.StartedAt, run.HasStarted = started, true
	}
	if row.FinishedAt != nil {
		finished, err := parseScopedRunTime(*row.FinishedAt, "finished_at")
		if err != nil {
			return ports.RunSummary{}, err
		}
		run.FinishedAt, run.HasFinished = finished, true
	}
	return run, nil
}

func parseScopedRunTime(value, field string) (time.Time, error) {
	parsed, err := time.Parse(time.RFC3339, value)
	if err != nil {
		return time.Time{}, apperr.Wrap(apperr.KindUnavailable, "", "run_list", "the authority reported a malformed run "+field, err)
	}
	return parsed, nil
}

func unavailableRunList(message string) error {
	return apperr.New(apperr.KindUnavailable, "", "run_list", message)
}

func validRunExecution(value string) bool {
	switch value {
	case "pending", "enabled", "active", "completed", "failed", "terminated":
		return true
	default:
		return false
	}
}

func validRunSettlement(value string) bool {
	switch value {
	case "unsettled", "accepted", "rejected", "escalated":
		return true
	default:
		return false
	}
}

// PendingApprovals implements ports.CaseAuthorityPort. An empty ref lists every case's approvals.
func (a *Authority) PendingApprovals(ctx context.Context, ref ports.CaseRef) ([]ports.ApprovalRecord, error) {
	resp, err := a.client.Do(ctx, NewApprovalList(string(ref)))
	if err != nil {
		return nil, err
	}
	var view ApprovalListView
	if err := resp.Into(&view); err != nil {
		return nil, err
	}
	out := make([]ports.ApprovalRecord, 0, len(view.Approvals))
	for _, row := range view.Approvals {
		requested, err := parseTime(row.RequestedAt, "approval requested_at")
		if err != nil {
			return nil, err
		}
		expires, err := parseTime(row.ExpiresAt, "approval expires_at")
		if err != nil {
			return nil, err
		}
		rec := ports.ApprovalRecord{
			ApprovalID:  row.ApprovalID,
			CaseID:      row.CaseID,
			RunID:       row.RunID,
			ItemID:      row.PlanItemID,
			DecisionID:  row.DecisionID,
			RequestedAt: requested,
			ExpiresAt:   expires,
			Expired:     row.Expired,
		}
		if row.CriteriaRef != nil {
			rec.CriteriaRef = *row.CriteriaRef
		}
		out = append(out, rec)
	}
	return out, nil
}

// EntryOptions implements ports.CaseAuthorityPort.
func (a *Authority) EntryOptions(ctx context.Context) ([]ports.TemplateOption, error) {
	resp, err := a.client.Do(ctx, NewCaseEntryOptions())
	if err != nil {
		return nil, err
	}
	var view EntryOptionsView
	if err := resp.Into(&view); err != nil {
		return nil, err
	}
	out := make([]ports.TemplateOption, 0, len(view.Templates))
	for _, t := range view.Templates {
		opt := ports.TemplateOption{
			TemplateRef: t.TemplateRef,
			Description: t.Description,
			Parameters:  make([]ports.TemplateParameter, 0, len(t.Parameters)),
		}
		for name, p := range t.Parameters {
			param := ports.TemplateParameter{Name: name, Type: p.ParamType, Required: p.Required}
			if p.Default != nil {
				param.Default = *p.Default
				param.HasDefault = true
			}
			opt.Parameters = append(opt.Parameters, param)
		}
		out = append(out, opt)
	}
	return out, nil
}

// PreflightCase implements ports.CaseAuthorityPort.
func (a *Authority) PreflightCase(ctx context.Context, draft ports.CaseDraft) (ports.PreflightReport, error) {
	resp, err := a.client.Do(ctx, NewCasePreflight(draft.TemplateRef, draft.Params))
	if err != nil {
		return ports.PreflightReport{}, err
	}
	var view PreflightView
	if err := resp.Into(&view); err != nil {
		return ports.PreflightReport{}, err
	}
	report := ports.PreflightReport{
		OK:          view.OK,
		TemplateRef: view.TemplateRef,
		Errors:      view.Errors,
		Items:       make([]ports.PlanItemSummary, 0, len(view.Items)),
	}
	if view.Precondition != nil {
		report.Precondition = ports.PreconditionDigest{
			Present:        true,
			TemplateRef:    view.Precondition.Ref,
			ExpectedDigest: view.Precondition.ExpectedDigest,
		}
	}
	for _, it := range view.Items {
		report.Items = append(report.Items, ports.PlanItemSummary{ItemID: it.PlanItemID, Name: it.Name, Kind: it.ItemKind})
	}
	return report, nil
}

// CommitCase implements ports.CaseAuthorityPort.
func (a *Authority) CommitCase(ctx context.Context, draft ports.CaseDraft, pin ports.PreconditionDigest, opts ports.GovernedOptions) (ports.CommitReceipt, error) {
	var precond *PreconditionWire
	if pin.Present {
		precond = &PreconditionWire{Records: []RecordDigestWire{{Ref: pin.TemplateRef, ExpectedDigest: pin.ExpectedDigest}}}
	}
	req, err := NewCaseCommit(draft.TemplateRef, draft.Params, opts.Policy, "godspeed-casework-go", opts.TimeoutSec,
		optsRequestID(opts, a.client, "case_commit"), precond, governance(opts.Governance))
	if err != nil {
		return ports.CommitReceipt{}, err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return ports.CommitReceipt{}, err
	}
	var view CommitView
	// The kernel verifies carried preconditions BEFORE committing: a stale preflight digest is
	// rejected with a normal (non-error) frame whose outcome is "rejected_as_stale" and whose
	// code is "precondition_failed" - no case exists, no side effect. Probed live in T06 and
	// surfaced as a typed refusal rather than a zero-value receipt.
	var stale struct {
		Outcome string          `json:"outcome"`
		Code    string          `json:"code"`
		Changed json.RawMessage `json:"changed_records"`
	}
	if err := json.Unmarshal(resp.Raw, &stale); err == nil && stale.Code == "precondition_failed" {
		return ports.CommitReceipt{}, apperr.New(apperr.KindInvalid, "", "case_commit",
			"the kernel rejected the commit as stale: the preflight this digest pins no longer matches the template").
			With(&Refusal{Class: "precondition_failed", Message: string(stale.Changed), NoSideEffect: true})
	}
	if err := resp.Into(&view); err != nil {
		return ports.CommitReceipt{}, err
	}
	return ports.CommitReceipt{CaseID: view.CaseID, State: view.State, ExitCode: view.ExitCode}, nil
}

// DecideApproval implements ports.CaseAuthorityPort. The approval ids come from PendingApprovals;
// the adapter resolves it through the same decision path the inbox verbs use, correlated.
func (a *Authority) DecideApproval(ctx context.Context, approvalID string, approve bool, note string, opts ports.GovernedOptions) error {
	// The decision verb targets (case_id, approval_id); the port carries only the approval id, so
	// resolve its case first through the governed inbox view.
	caseID, err := a.caseOfApproval(ctx, approvalID)
	if err != nil {
		return err
	}
	decision := "reject"
	if approve {
		decision = "approve"
	}
	req, err := NewApprovalDecide(caseID, approvalID, decision, note,
		optsRequestID(opts, a.client, "approval_decide"), governance(opts.Governance))
	if err != nil {
		return err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return err
	}
	// The decided view carries the outcome; validate it decoded cleanly.
	var outcome json.RawMessage
	return resp.Into(&outcome)
}

func (a *Authority) caseOfApproval(ctx context.Context, approvalID string) (string, error) {
	approvals, err := a.PendingApprovals(ctx, "")
	if err != nil {
		return "", err
	}
	for _, ap := range approvals {
		if ap.ApprovalID == approvalID {
			return ap.CaseID, nil
		}
	}
	return "", apperr.New(apperr.KindInvalid, "", "approval_decide",
		"approval "+approvalID+" is not pending on the authority")
}

// AddCaseItem implements ports.CaseAuthorityPort.
func (a *Authority) AddCaseItem(ctx context.Context, ref ports.CaseRef, item ports.CaseItemProposal, opts ports.GovernedOptions) (string, error) {
	wire, err := itemWire(item)
	if err != nil {
		return "", err
	}
	req, err := NewCaseAddItem(string(ref), wire, opts.Policy,
		optsRequestID(opts, a.client, "case_add_item"), governance(opts.Governance))
	if err != nil {
		return "", err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return "", err
	}
	var view AddItemView
	if err := resp.Into(&view); err != nil {
		return "", err
	}
	return view.PlanItemID, nil
}

// itemWire translates the application's item proposal into the kernel's plan-item shape. The
// server re-validates everything and overwrites the proposer; nothing here is trusted.
func itemWire(item ports.CaseItemProposal) (map[string]any, error) {
	if item.ItemID == "" || item.Name == "" || item.Kind == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "case_add_item",
			"a proposed item needs an id, a name, and a kind")
	}
	wire := map[string]any{
		"plan_item_id":        item.ItemID,
		"name":                item.Name,
		"item_kind":           item.Kind,
		"settlement_criteria": settlementWire(item),
		"max_instances":       max(item.MaxInstances, 1),
	}
	if item.SandboxClass != "" {
		wire["sandbox_class"] = item.SandboxClass
	}
	if item.ParentStage != "" {
		wire["parent_stage"] = item.ParentStage
	}
	if len(item.DependsOn) > 0 {
		wire["depends_on"] = item.DependsOn
	}
	if item.EntryCriteriaMode != "" {
		wire["entry_criteria_mode"] = item.EntryCriteriaMode
	}
	if len(item.EntryCriteria) > 0 {
		criteria := make([]any, 0, len(item.EntryCriteria))
		for _, c := range item.EntryCriteria {
			cond := map[string]any{"kind": c.Kind}
			if c.Status != "" {
				cond["status"] = c.Status
			}
			criteria = append(criteria, map[string]any{
				"on": map[string]any{"source": c.Source, "event": c.Event},
				"if": cond,
			})
		}
		wire["entry_criteria"] = criteria
	}
	if len(item.Operations) > 0 {
		ops := make([]any, 0, len(item.Operations))
		for _, op := range item.Operations {
			entry := map[string]any{"kind": op.Kind}
			if op.Path != "" {
				entry["path"] = op.Path
			}
			if op.ContentHint != "" {
				entry["content_hint"] = op.ContentHint
			}
			ops = append(ops, entry)
		}
		wire["operations"] = ops
	}
	return wire, nil
}

func settlementWire(item ports.CaseItemProposal) map[string]any {
	criteria := map[string]any{}
	if len(item.RequiredArtifacts) > 0 {
		criteria["required_artifacts"] = item.RequiredArtifacts
	}
	if item.RequireApproval {
		criteria["require_approval"] = true
	}
	return criteria
}

// ReopenCase implements ports.CaseAuthorityPort.
func (a *Authority) ReopenCase(ctx context.Context, ref ports.CaseRef, reason string, opts ports.GovernedOptions) error {
	req, err := NewCaseReopen(string(ref), reason, opts.Policy,
		optsRequestID(opts, a.client, "case_reopen"), governance(opts.Governance))
	if err != nil {
		return err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return err
	}
	var view ReopenView
	return resp.Into(&view)
}

// TerminateCase implements ports.CaseAuthorityPort.
func (a *Authority) TerminateCase(ctx context.Context, ref ports.CaseRef, reason string, opts ports.GovernedOptions) error {
	req, err := NewCaseTerminate(string(ref), reason, opts.Policy,
		optsRequestID(opts, a.client, "case_terminate"), governance(opts.Governance))
	if err != nil {
		return err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return err
	}
	var view TerminateView
	return resp.Into(&view)
}

// AdvanceCase implements ports.CaseAuthorityPort.
func (a *Authority) AdvanceCase(ctx context.Context, ref ports.CaseRef, opts ports.GovernedOptions) (ports.AdvanceReport, error) {
	req, err := NewCaseAdvance(string(ref), opts.Policy, opts.TimeoutSec,
		optsRequestID(opts, a.client, "case_advance"), governance(opts.Governance))
	if err != nil {
		return ports.AdvanceReport{}, err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return ports.AdvanceReport{}, err
	}
	return advanceOf(resp)
}

// ExecuteItem implements ports.CaseAuthorityPort.
func (a *Authority) ExecuteItem(ctx context.Context, ref ports.CaseRef, itemID string, opts ports.GovernedOptions) (ports.AdvanceReport, error) {
	req, err := NewItemExecute(string(ref), itemID, opts.Policy, opts.TimeoutSec,
		optsRequestID(opts, a.client, "item_execute"), governance(opts.Governance))
	if err != nil {
		return ports.AdvanceReport{}, err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return ports.AdvanceReport{}, err
	}
	return advanceOf(resp)
}

func advanceOf(resp *Response) (ports.AdvanceReport, error) {
	var view AdvanceView
	if err := resp.Into(&view); err != nil {
		return ports.AdvanceReport{}, err
	}
	report := ports.AdvanceReport{
		CaseID:   view.CaseID,
		State:    view.State,
		Episodes: make([]ports.EpisodeReport, 0, len(view.Episodes)),
	}
	for _, ep := range view.Episodes {
		report.Episodes = append(report.Episodes, ports.EpisodeReport{
			ItemID: ep.ItemID, RunID: ep.RunID, Settlement: ep.SettlementStatus,
		})
	}
	return report, nil
}

// CompleteHumanTask implements ports.CaseAuthorityPort.
func (a *Authority) CompleteHumanTask(ctx context.Context, ref ports.CaseRef, itemID string, justification string, opts ports.GovernedOptions) error {
	req, err := NewHumanTaskComplete(string(ref), itemID, justification, opts.Policy,
		optsRequestID(opts, a.client, "human_task_complete"), governance(opts.Governance))
	if err != nil {
		return err
	}
	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return err
	}
	var view HumanTaskCompleteView
	return resp.Into(&view)
}

// artifactReadError types the kernel's artifact_integrity_error (it re-hashes the stored bytes before
// serving them) as an integrity failure of the evidence rather than an outage of the authority.
func artifactReadError(err error) error {
	if strings.Contains(err.Error(), "artifact_integrity_error") {
		return apperr.Wrap(apperr.KindInternal, "", "artifact.get", "the stored artifact no longer matches its digest", ports.ErrArtifactIntegrity)
	}
	return err
}

// GetArtifact implements ports.CaseAuthorityPort.
func (a *Authority) GetArtifact(ctx context.Context, digest string) (ports.ArtifactContent, error) {
	resp, err := a.client.Do(ctx, NewArtifactGet(digest))
	if err != nil {
		return ports.ArtifactContent{}, artifactReadError(err)
	}
	var view ArtifactView
	if err := resp.Into(&view); err != nil {
		return ports.ArtifactContent{}, err
	}
	data, err := view.Bytes()
	if err != nil {
		return ports.ArtifactContent{}, err
	}

	runResp, err := a.client.Do(ctx, NewRunGet(view.RunID))
	if err != nil {
		return ports.ArtifactContent{}, artifactOwnershipError(err)
	}
	var run RunArtifactProvenanceView
	if err := runResp.Into(&run); err != nil {
		return ports.ArtifactContent{}, artifactOwnershipError(err)
	}
	caseID, planItemID, err := artifactProvenanceOf(view, run)
	if err != nil {
		return ports.ArtifactContent{}, err
	}
	return ports.ArtifactContent{
		Digest:     view.Digest,
		RunID:      view.RunID,
		CaseID:     caseID,
		PlanItemID: planItemID,
		EvidenceID: view.EvidenceID,
		URI:        view.URI,
		Size:       int64(view.SizeBytes),
		Data:       data,
	}, nil
}

// ResolveIdentity implements ports.CaseAuthorityPort.
func (a *Authority) ResolveIdentity(ctx context.Context, g ports.Governance) (ports.IdentityReport, error) {
	wireg := governance(g)
	resp, err := a.client.Do(ctx, NewIdentityGet(&wireg))
	if err != nil {
		return ports.IdentityReport{}, err
	}
	var view IdentityView
	if err := resp.Into(&view); err != nil {
		return ports.IdentityReport{}, err
	}
	report := ports.IdentityReport{
		Configured: view.Configured,
		Available:  make([]ports.AvailableActor, 0, len(view.Available)),
	}
	if view.UID != nil {
		report.UID = int64(*view.UID)
	}
	for _, actor := range view.Available {
		report.Available = append(report.Available, ports.AvailableActor{ActorID: actor.ActorID, Roles: actor.Roles})
	}
	if view.EffectiveActor != nil {
		report.EffectiveActor = &ports.ActorClaim{ActorID: view.EffectiveActor.ActorID, Role: view.EffectiveActor.Role}
	}
	if view.Refusal != nil {
		report.Refusal = &ports.IdentityRefusal{
			Class:            view.Refusal.ErrorClass,
			Message:          view.Refusal.Message,
			NextLawfulAction: view.Refusal.NextLawfulAction,
		}
	}
	return report, nil
}

// Readiness implements ports.CaseAuthorityPort.
func (a *Authority) Readiness(ctx context.Context) (ports.ReadinessReport, error) {
	resp, err := a.client.Do(ctx, NewReadinessGet())
	if err != nil {
		return ports.ReadinessReport{}, err
	}
	var view ReadinessView
	if err := resp.Into(&view); err != nil {
		return ports.ReadinessReport{}, err
	}
	report := ports.ReadinessReport{
		Overall:      view.Overall,
		Foundations:  conditionsOf(view.Foundations),
		Capabilities: conditionsOf(view.OperationalCapabilities),
	}
	return report, nil
}

func conditionsOf(items []ReadinessItemView) []ports.ReadinessCondition {
	out := make([]ports.ReadinessCondition, 0, len(items))
	for _, it := range items {
		out = append(out, ports.ReadinessCondition{
			ID: it.ID, Name: it.Name, Category: it.Category,
			Status: it.Status, Reason: it.Reason, NextLawfulAction: it.NextLawfulAction,
		})
	}
	return out
}

// RequestStatus implements ports.CaseAuthorityPort: the explicit correlated-outcome recovery
// entry point.
func (a *Authority) RequestStatus(ctx context.Context, requestID string) (ports.RequestOutcome, error) {
	view, err := a.client.RequestStatus(ctx, requestID)
	if err != nil {
		return ports.RequestOutcome{}, err
	}
	out := ports.RequestOutcome{RequestID: view.RequestID, Status: view.Status, Method: view.Method}
	if len(view.Outcome) > 0 {
		result, err := operationResultOf(view.Outcome)
		if err != nil {
			return ports.RequestOutcome{}, err
		}
		out.Result = result
	}
	return out, nil
}

// operationResultOf normalizes a recorded terminal outcome into the application's result shape.
func operationResultOf(raw json.RawMessage) (*ports.OperationResult, error) {
	var view struct {
		CaseID     string  `json:"case_id"`
		State      string  `json:"state"`
		ExitCode   *int    `json:"exit_code"`
		Error      *string `json:"error"`
		ErrorClass string  `json:"error_class"`
	}
	if err := json.Unmarshal(raw, &view); err != nil {
		return nil, apperr.Wrap(apperr.KindInternal, "", "recover",
			"the authority recorded an outcome this gateway cannot decode", err)
	}
	result := &ports.OperationResult{
		OK:         view.Error == nil,
		CaseID:     view.CaseID,
		State:      view.State,
		ErrorClass: view.ErrorClass,
	}
	if view.ExitCode != nil {
		result.ExitCode = *view.ExitCode
	}
	if view.Error != nil {
		result.Error = *view.Error
	}
	return result, nil
}
