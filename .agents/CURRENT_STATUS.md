# Current status

**Status revision:** 196

**Stage:** Security: m8 CLI tests finish the external settlement approval step with a distinct approver

**Summary:** Both conformance_m8 CLI tests pass with their original intent. An external settlement now needs approval: the m8 fixture adds an external settlement rule that requires approval, and an approval_resolution rule for an operator identity other than the requester (settlement_approver). The requester's own rules are unchanged, and the requester is refused as approver of its own settlement (asserted). The tests resolve the settlement approval through the real CEP resolution path. Before that approval, resume exits 5 with a settlement pending record and no declaration. The parks test keeps its outage check, which parks with exit 5 and declares nothing, then completes once the outage is removed.

**Verified:**
- cargo test -p sea-forge-cli --test conformance_m8_artifact -- conformance_m8_cli_capitalize_commits_one_pending_and_approval_then_exits_5 conformance_m8_cli_parks_external_gate_profiles_without_fabricating_records: both pass
- cargo test -p sea-forge-cli --no-fail-fast: 89 pass, 0 fail (exit 0)
- cargo fmt --all -- --check: clean

**Limits:**
- The CLI approve command cannot resolve a settlement approval; the tests use the CEP resolution path directly, as the server does. An owner runbook for a live round-trip is still needed
- race test not yet red-then-green (checkpoint e)
- adversarial review not done

**Next:** Checkpoint e: make the racing identical-request test red, then fix the pending-record race read.
