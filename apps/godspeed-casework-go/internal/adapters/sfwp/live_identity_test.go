//go:build live

// TOOTH (c), permanent (plan T05 teeth): drive on_behalf_of from a cell whose
// server.yaml has NO gateway section. Expected: a typed identity refusal
// surfaced to the caller, and no retry storm - proven durably, because the
// identity gate sits before admission, so a refused delegation leaves no
// correlation record, no delegation audit write, and no case.
//
// The seed mirrors crates/sea-forge-server/tests/sfwp_delegated_identity.rs's
// negative case: T02's default is fail-closed (no gateway section -> every
// on_behalf_of is identity_delegation_refused).
package sfwp

import (
	"context"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestLiveDelegationRefusedWithoutGatewayBinding(t *testing.T) {
	cell := newLiveCell(t) // seeds identity bindings WITHOUT a gateway section
	cl := cell.client()
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	authority := NewAuthority(cl)

	// The inspect-level view reports the same refusal the protected verb
	// raises: identity.get with a governance block is the gateway's cheap
	// preflight for "would this delegated request be attributed?"
	identity, err := authority.ResolveIdentity(ctx, govDelegated("operator_a", "operator"))
	if err != nil {
		t.Fatalf("identity.get: %v", err)
	}
	if identity.Refusal == nil || identity.Refusal.Class != "identity_delegation_refused" {
		t.Fatalf("identity.get must report the delegation refusal, got %+v", identity.Refusal)
	}

	// The protected verb is refused with the typed class, mapped to
	// authority_denied, with the authority's own no-side-effect assertion.
	requestID := cl.NewRequestID("case_commit")
	delegOpts := governedOpts(requestID)
	delegOpts.Governance = govDelegated("operator_a", "operator")
	draft := caseDraftOf(sentryChainRef, sentryChainParams())
	_, err = authority.CommitCase(ctx, draft, ports.PreconditionDigest{}, delegOpts)
	assertRefusal(t, err, "identity_delegation_refused", apperr.KindAuthorityDenied)

	// No retry storm, from durable truth: nothing was admitted (no
	// correlation record exists for the request id) and nothing was written
	// (no case anywhere in the cell). The client's failure discipline never
	// re-sends a refused mutation, so there is exactly one refusal to observe.
	if rec := cell.requestRecord(requestID); rec != nil {
		t.Fatalf("a refused delegation must leave no correlation record, got %+v", rec)
	}
	if created := cell.countCaseCreated(); created != 0 {
		t.Fatalf("a refused delegation must create no case, cell has %d CaseCreated events", created)
	}
}
