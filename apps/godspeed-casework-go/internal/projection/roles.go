// Role filtering and the action-descriptor vocabulary for the live projection.
//
// The gateway is NOT the authority over who may do what - the kernel's identity bindings and
// authority policy decide that, and every kernel refusal passes through honestly. What this file
// owns is the PROJECTION vocabulary: which action descriptors a snapshot offers to which kernel
// role, on which standing. The intent translator (internal/intents) consults the SAME predicates
// as its projection-level guard, so an intent the gateway refuses as UNAUTHORIZED_ROLE is exactly
// one whose descriptor the role's action list would never have carried (plan T06 tooth).
//
// Roles are the kernel's own wire spellings (sea-forge-core ActorRole serde names: "operator",
// "R-SO", ...). The spec-04 TypeScript union spells some of these differently ("security_officer");
// the session-to-actor mapping is T07's work and the kernel verifies every claim regardless.
package projection

import "github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"

// Kernel roles this projection treats as separations of duty, deliberately coarse:
//
//   - executor roles drive work (execute items, complete human tasks) but never decide approvals;
//   - approver roles decide approvals but never drive the work they gate (the kernel's own SoD
//     compares actors record-by-record; this filter only keeps the OFFER honest);
//   - proposer roles commit cases and propose discretionary work;
//   - lifecycle roles reopen and terminate cases;
//   - pure machine roles (service, system) are offered nothing consequential.
//
// The plan's journeys ground this split: the operator executes (L4) and is denied the approval
// (L5), which a second authenticated user holding R-SO performs.
var (
	approverRoles  = map[string]bool{"R-SO": true, "R-RM": true, "R-LC": true, "R-AG": true, "R-DS": true}
	executorRoles  = map[string]bool{"operator": true, "R-DEV": true, "agent": true, "R-AA": true}
	proposerRoles  = map[string]bool{"operator": true, "R-DEV": true, "R-AG": true}
	lifecycleRoles = map[string]bool{"operator": true, "R-LC": true, "R-AG": true}
	machineRoles   = map[string]bool{"service": true, "system": true}
)

// MayDecideApproval reports whether the role is offered approval decisions.
func MayDecideApproval(role string) bool { return approverRoles[role] }

// MayExecute reports whether the role is offered item execution and human-task completion.
func MayExecute(role string) bool { return executorRoles[role] }

// MayPropose reports whether the role is offered case commit and discretionary work.
func MayPropose(role string) bool { return proposerRoles[role] }

// MayDriveLifecycle reports whether the role is offered reopen/terminate.
func MayDriveLifecycle(role string) bool { return lifecycleRoles[role] }

// RoleOffersConsequentialWork reports whether the role is offered consequential descriptors at
// all. Pure machine roles (the gateway principal itself, system) get a quiet projection: they are
// infrastructure, not actors the action list should speak for.
func RoleOffersConsequentialWork(role string) bool { return !machineRoles[role] }

// actionFor builds one descriptor. ids are deterministic ("act-<intent>-<object>") so a
// re-rendered snapshot keeps stable action identities the UI can key on.
func actionFor(objectID, intent, label, variant string, consequential bool, requiresJustification bool) contract.ActionDescriptor {
	id := "act-" + lower(intent)
	if objectID != "" {
		id += "-" + objectID
	}
	out := contract.ActionDescriptor{
		ID:            id,
		Label:         label,
		Intent:        intent,
		Consequential: consequential,
	}
	if variant != "" {
		v := variant
		out.Variant = &v
	}
	if requiresJustification {
		b := true
		out.RequiresJustification = &b
	}
	return out
}

func lower(s string) string {
	out := make([]byte, len(s))
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c >= 'A' && c <= 'Z' {
			c += 'a' - 'A'
		}
		out[i] = c
	}
	return string(out)
}
