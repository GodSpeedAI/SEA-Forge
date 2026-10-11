// LiveSource: the network side of the projection. It fetches a case's bounded views through
// ports.CaseAuthorityPort (case.list, case.get_overview, case.get_horizon, approval.list,
// run.list - the T05 client over the governed kernel) and hands them to the pure builder.
//
// Nothing here decides: every fact is what the authority reported, and a fetch failure surfaces
// as a typed error rather than a degraded guess.
package projection

import (
	"context"
	"fmt"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// LiveSource builds snapshots from live authority reads.
type LiveSource struct {
	auth ports.CaseAuthorityPort
	// gateway is the gateway principal's own claim: the actor block the kernel requires on
	// every governed call, perspective verification included. The kernel resolves a delegated
	// read (resolve_delegated) only when the request's actor block names the configured gateway
	// principal - the same rule intents.execute satisfies, so VerifyPerspective sends it too
	// (T06 fix F1: an empty actor block is refused with identity_required).
	gateway ports.ActorClaim
	// now stamps rendered snapshots; nil means time.Now.
	now func() time.Time

	// artifacts lists the artifacts a run captured when the authority can (optional capability).
	artifacts ports.RunArtifactLister
	// artifactCache remembers a run's artifact list per evidence count: a run's evidence journal
	// only grows, so the same count is the same list and costs no further authority read.
	artifactMu    sync.Mutex
	artifactCache map[string]cachedRunArtifacts
}

type cachedRunArtifacts struct {
	evidence int
	refs     []ports.RunArtifactRef
}

// NewLiveSource builds a source over the given authority port. gateway is the gateway
// principal's claim (config serve.gateway_actor_id/gateway_role in production); it must match
// the kernel's configured gateway principal or every delegated call is refused.
func NewLiveSource(auth ports.CaseAuthorityPort, gateway ports.ActorClaim) *LiveSource {
	src := &LiveSource{auth: auth, gateway: gateway, artifactCache: map[string]cachedRunArtifacts{}}
	if lister, ok := auth.(ports.RunArtifactLister); ok {
		src.artifacts = lister
	}
	return src
}

// runArtifacts returns the artifacts each evidence-bearing run captured. A run whose list cannot
// be read contributes nothing (its artifacts are simply not bound); the world never fails over an
// artifact it could not list, and nothing is guessed in its place.
func (s *LiveSource) runArtifacts(ctx context.Context, runs []ports.RunSummary) map[string][]ports.RunArtifactRef {
	if s.artifacts == nil {
		return nil
	}
	var out map[string][]ports.RunArtifactRef
	for _, run := range runs {
		if run.EvidenceCount <= 0 {
			continue
		}
		s.artifactMu.Lock()
		cached, hit := s.artifactCache[run.RunID]
		s.artifactMu.Unlock()
		refs := cached.refs
		if !hit || cached.evidence != run.EvidenceCount {
			listed, err := s.artifacts.RunArtifacts(ctx, run.RunID)
			if err != nil {
				continue
			}
			refs = listed
			s.artifactMu.Lock()
			s.artifactCache[run.RunID] = cachedRunArtifacts{evidence: run.EvidenceCount, refs: refs}
			s.artifactMu.Unlock()
		}
		if len(refs) == 0 {
			continue
		}
		if out == nil {
			out = map[string][]ports.RunArtifactRef{}
		}
		out[run.RunID] = refs
	}
	return out
}

// Facts fetches one case's views. The case record comes from case.list (the list is the only
// surface carrying the operator-facing summary); its views are addressed by case id.
func (s *LiveSource) Facts(ctx context.Context, caseID string, actor ports.ActorClaim, cursor string) (CaseFacts, error) {
	if strings.TrimSpace(caseID) == "" {
		return CaseFacts{}, apperr.New(apperr.KindInvalid, "", "world", "case id must not be blank")
	}
	cases, err := s.auth.ListCases(ctx)
	if err != nil {
		return CaseFacts{}, err
	}
	var record *ports.CaseRecord
	for i := range cases {
		if string(cases[i].Ref) == caseID {
			if record != nil {
				return CaseFacts{}, unavailableCapturedRuns("the authority repeated the requested case record")
			}
			record = &cases[i]
		}
	}
	if record == nil {
		return CaseFacts{}, apperr.New(apperr.KindInvalid, "", "world",
			"no case "+caseID+" exists on the authority")
	}
	overview, err := s.auth.CaseOverview(ctx, ports.CaseRef(caseID))
	if err != nil {
		return CaseFacts{}, err
	}
	horizon, err := s.auth.CaseHorizon(ctx, ports.CaseRef(caseID))
	if err != nil {
		return CaseFacts{}, err
	}
	approvals, err := s.auth.PendingApprovals(ctx, ports.CaseRef(caseID))
	if err != nil {
		return CaseFacts{}, err
	}
	runList, err := s.auth.RunsListForCase(ctx, ports.CaseRef(caseID))
	if err != nil {
		return CaseFacts{}, err
	}
	if string(record.Ref) != caseID || string(overview.Ref) != caseID || string(horizon.Ref) != caseID {
		return CaseFacts{}, unavailableCapturedRuns("the authority returned a case view for a different case")
	}
	horizonItems := make(map[string]struct{}, len(horizon.Items))
	for _, item := range horizon.Items {
		if strings.TrimSpace(item.ItemID) == "" {
			return CaseFacts{}, unavailableCapturedRuns("the authority returned a blank horizon item id")
		}
		if _, exists := horizonItems[item.ItemID]; exists {
			return CaseFacts{}, unavailableCapturedRuns("the authority repeated a horizon item id")
		}
		horizonItems[item.ItemID] = struct{}{}
	}
	seenRuns := make(map[string]struct{}, len(runList.Runs)+len(runList.UnreadableIDs))
	for _, run := range runList.Runs {
		if strings.TrimSpace(run.RunID) == "" {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority returned a blank run id")
		}
		if _, exists := seenRuns[run.RunID]; exists {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority repeated a run id")
		}
		seenRuns[run.RunID] = struct{}{}
		if run.CaseID != caseID {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority returned a run owned by another case")
		}
		if strings.TrimSpace(run.PlanItemID) == "" {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority returned a run without its plan item id")
		}
		if _, exists := horizonItems[run.PlanItemID]; !exists {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority returned a run without an actual horizon parent")
		}
		if !validCapturedExecution(run.Execution) || !validCapturedSettlement(run.Settlement) || run.EvidenceCount < 0 {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority returned a malformed run summary")
		}
	}
	for _, id := range runList.UnreadableIDs {
		if strings.TrimSpace(id) == "" {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority returned a blank unreadable run id")
		}
		if _, exists := seenRuns[id]; exists {
			return CaseFacts{}, unavailableCapturedRuns("the scoped authority repeated or overlapped a run id")
		}
		seenRuns[id] = struct{}{}
	}
	facts := CaseFacts{
		Record:           *record,
		Overview:         overview,
		Horizon:          horizon,
		Approvals:        approvals,
		Runs:             runList.Runs,
		RunArtifacts:     s.runArtifacts(ctx, runList.Runs),
		UnreadableRunIDs: runList.UnreadableIDs,
		Actor:            actor,
		Cursor:           cursor,
	}
	if s.now != nil {
		facts.Now = s.now()
	} else {
		facts.Now = time.Now()
	}
	return facts, nil
}

func unavailableCapturedRuns(message string) error {
	return apperr.New(apperr.KindUnavailable, "", "world", message)
}

func validCapturedExecution(value string) bool {
	switch value {
	case "pending", "enabled", "active", "completed", "failed", "terminated":
		return true
	default:
		return false
	}
}

func validCapturedSettlement(value string) bool {
	switch value {
	case "unsettled", "accepted", "rejected", "escalated":
		return true
	default:
		return false
	}
}

// Snapshot fetches and builds one case's snapshot for the given perspective.
func (s *LiveSource) Snapshot(ctx context.Context, caseID string, actor ports.ActorClaim, cursor string) (contract.CognitiveWorldSnapshot, error) {
	facts, err := s.Facts(ctx, caseID, actor, cursor)
	if err != nil {
		return contract.CognitiveWorldSnapshot{}, err
	}
	return Build(facts), nil
}

// NewestCaseID returns the newest case on the authority (case.list is newest-first), preferring
// active cases so a busy cell's default world is the live one. Empty when the cell has no cases.
func (s *LiveSource) NewestCaseID(ctx context.Context) (string, error) {
	cases, err := s.auth.ListCases(ctx)
	if err != nil {
		return "", err
	}
	for _, c := range cases {
		if c.State == "active" || c.State == "awaiting_approval" {
			return string(c.Ref), nil
		}
	}
	if len(cases) > 0 {
		return string(cases[0].Ref), nil
	}
	return "", nil
}

// EmptyWorld is the honest snapshot for a cell with no cases: an empty world, provenance-honest
// cursor included (the caller supplies whatever kernel cursor the gateway can vouch for).
func EmptyWorld(actor ports.ActorClaim, cursor string, now time.Time) contract.CognitiveWorldSnapshot {
	return contract.CognitiveWorldSnapshot{
		WorldID:          "world-empty",
		CaseID:           "",
		Cursor:           cursor,
		Timestamp:        now.UTC().Format(time.RFC3339),
		Perspective:      contract.ActorPerspective{ActorID: actor.ActorID, Role: actor.Role},
		Summary:          contract.WorldSummary{Headline: "No cases yet", Phase: "idle", StatusPhrase: "Commit a case from a template to begin."},
		VisibleObjects:   []contract.CognitiveObject{},
		AvailableActions: []contract.ActionDescriptor{},
		AttentionFocus:   contract.AttentionFocus{PrimaryObjectID: ""},
	}
}

// Templates maps case.entry_options into the contract's template DTOs. Parameter type spellings
// are translated kernel -> contract (int/path/bool fold into the contract's four-type union);
// anything untranslatable is an error rather than a guess.
func Templates(opts []ports.TemplateOption) ([]contract.TemplateEntryOption, error) {
	out := make([]contract.TemplateEntryOption, 0, len(opts))
	for _, o := range opts {
		entry := contract.TemplateEntryOption{
			TemplateRef: o.TemplateRef,
			Title:       templateTitle(o.TemplateRef),
			Description: nilIfEmpty(o.Description),
			Parameters:  make([]contract.TemplateParameter, 0, len(o.Parameters)),
		}
		for _, p := range o.Parameters {
			typ, err := paramType(p.Type)
			if err != nil {
				return nil, err
			}
			cp := contract.TemplateParameter{
				Name:     p.Name,
				Type:     typ,
				Required: &p.Required,
			}
			if p.HasDefault {
				cp.DefaultValue = p.Default
			}
			entry.Parameters = append(entry.Parameters, cp)
		}
		sort.SliceStable(entry.Parameters, func(i, j int) bool { return entry.Parameters[i].Name < entry.Parameters[j].Name })
		out = append(out, entry)
	}
	sort.SliceStable(out, func(i, j int) bool { return out[i].TemplateRef < out[j].TemplateRef })
	return out, nil
}

// paramType translates the kernel's declared parameter types onto the contract's union
// (string|number|boolean|enum). The E2E templates use string, int and path; int is numeric and
// path is a string on the wire.
func paramType(kernelType string) (string, error) {
	switch kernelType {
	case "string", "path":
		return "string", nil
	case "int", "number", "float":
		return "number", nil
	case "bool", "boolean":
		return "boolean", nil
	case "enum":
		return "enum", nil
	default:
		return "", apperr.New(apperr.KindInternal, "", "templates",
			fmt.Sprintf("the authority declared template parameter type %q, which this gateway cannot represent", kernelType))
	}
}

func templateTitle(ref string) string {
	// "e2e-sentry-chain@0.1.0" -> "e2e-sentry-chain"; the kernel exposes no separate title field.
	if idx := strings.IndexByte(ref, '@'); idx > 0 {
		return ref[:idx]
	}
	return ref
}

func nilIfEmpty(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

// PreflightResult maps the authority's preflight report onto the contract DTO: passed exactly
// when the authority said ok, reasons empty exactly when passed, and the commit digest present
// exactly when passed (PROPOSE_CASE must echo it - the kernel verifies the echo at commit).
func PreflightResult(report ports.PreflightReport, params map[string]any) contract.TemplatePreflightResult {
	out := contract.TemplatePreflightResult{
		TemplateRef: report.TemplateRef,
		Params:      params,
		Passed:      report.OK,
		Reasons:     append([]string(nil), report.Errors...),
	}
	if report.OK && report.Precondition.Present {
		digest := report.Precondition.ExpectedDigest
		out.Digest = &digest
	}
	if out.Reasons == nil {
		out.Reasons = []string{}
	}
	return out
}

// EntryOptions fetches and maps the kernel's template catalog (GET /api/templates).
func (s *LiveSource) EntryOptions(ctx context.Context) ([]contract.TemplateEntryOption, error) {
	opts, err := s.auth.EntryOptions(ctx)
	if err != nil {
		return nil, err
	}
	return Templates(opts)
}

// Preflight runs the authority's audit-only dry run and maps it (POST /api/templates/preflight).
func (s *LiveSource) Preflight(ctx context.Context, templateRef string, params map[string]any) (contract.TemplatePreflightResult, error) {
	scalar, err := scalarParams(params)
	if err != nil {
		return contract.TemplatePreflightResult{}, err
	}
	report, err := s.auth.PreflightCase(ctx, ports.CaseDraft{TemplateRef: templateRef, Params: scalar})
	if err != nil {
		return contract.TemplatePreflightResult{}, err
	}
	return PreflightResult(report, params), nil
}

// VerifyPerspective checks with the kernel that this connection may act as the requested
// (actor, role) - the same delegation rules intents are verified against, applied to reads. The
// gateway principal is the request's actor and the requested user rides as on_behalf_of, exactly
// as intents.execute sends it (T02/D-2): identity.get with on_behalf_of resolves through the
// kernel's resolve_delegated, so a refusal here means the kernel would refuse this connection's
// write as that actor too.
func (s *LiveSource) VerifyPerspective(ctx context.Context, actor ports.ActorClaim) error {
	report, err := s.auth.ResolveIdentity(ctx, ports.Governance{
		Actor:      s.gateway,
		OnBehalfOf: &ports.ActorClaim{ActorID: actor.ActorID, Role: actor.Role},
	})
	if err != nil {
		return err
	}
	if report.Refusal != nil {
		return apperr.New(apperr.KindAuthorityDenied, "", "identity",
			report.Refusal.Message)
	}
	if report.EffectiveActor == nil || report.EffectiveActor.ActorID != actor.ActorID {
		return apperr.New(apperr.KindAuthorityDenied, "", "identity",
			"the kernel did not resolve "+actor.ActorID+" as an effective actor for this connection")
	}
	return nil
}

// scalarParams narrows the contract's free-form params object onto the kernel's string-valued
// params (numbers/booleans render their wire spelling; anything structured is refused rather
// than stringified into ambiguity).
func scalarParams(params map[string]any) (map[string]string, error) {
	out := map[string]string{}
	for k, v := range params {
		switch t := v.(type) {
		case string:
			out[k] = t
		case float64:
			out[k] = fmt.Sprintf("%v", t)
		case bool:
			out[k] = fmt.Sprintf("%v", t)
		case nil:
			out[k] = ""
		default:
			return nil, apperr.New(apperr.KindInvalid, "", "preflight",
				fmt.Sprintf("template parameter %q must be a scalar value; structured parameters cannot cross the kernel wire", k))
		}
	}
	return out, nil
}
