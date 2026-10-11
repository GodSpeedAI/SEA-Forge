#!/usr/bin/env python3
"""Sequential Behavioral Proof CLI Runner.

Provides renderer-neutral terminal navigation through canonical journeys,
exercising IS1-IS8 phases without browser/GUI dependencies.
"""

import argparse
import json
import sys
from pathlib import Path

try:
    from .engine import ProofEngine
except ImportError:
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
    from src.engine import ProofEngine


def run_catalog_mode(engine: ProofEngine, args: argparse.Namespace) -> None:
    print("\n=== SEA Forge Canonical Journey Catalog (Proof Mode) ===")
    entries = engine.get_catalog_entries()
    for e in entries:
        print(f"[{e['id']}] {e['title']} ({e['operation_class']}) - {e['mapped_stories']} stories")

    jid = args.journey or "CJ01"
    print(f"\nNavigating selected journey: {jid}")
    
    actor_id = args.actor or "operator-alice"
    actor_role = args.role or ("CellAdministrator" if jid in ["CJ01", "CJ12"] else "CaseOwner")

    # Sample context for test run
    ctx = {
        "cell_root_path": "/var/sea/cells/primary",
        "actor_identity": actor_id,
        "configuration_inputs": {"profile": "standard"},
        "existing_ledger_history_when_present": True,
        "subject_of_inquiry": "capabilities",
        "disclosure_scope_declared": "standard",
        "trusted_cell_and_actor_context": True,
        "sea_source_files": ["models/core.sea"],
        "import_dependencies": [],
        "projection_target_specification": "rust",
        "actor_attribution": actor_id,
        "cell_root": "/var/sea/cells/primary",
        "work_source_descriptor": "intent_spec",
        "semantic_references": ["core_concept"],
        "environment_and_evaluator_bindings": ["jail_evaluator"],
        "committed_case_id": "CASE-2026-0901",
        "attributable_actor_context": actor_id,
        "run_records_index": "runs/index",
        "approval_request_id": "APPR-101",
        "acting_human_identity": actor_id,
        "request_context_bundle": {"scope": "deploy"},
        "downstream_effect_descriptor": "unblock item 2",
        "enabled_item_id": "ITEM-01",
        "authority_grant_token": "GRANT-XYZ",
        "sandbox_and_environment_spec": {"type": "bwrap"},
        "execution_limits": {"timeout": 300},
        "case_and_run_identities": ["CASE-2026-0901", "RUN-01"],
        "ordered_event_log_cursor": 100,
        "intervention_scope_descriptor": "scoped_run",
        "run_episode_id": "RUN-01",
        "immutable_case_plan_criteria": ["criterion_1"],
        "authority_and_trace_records": ["trace_01"],
        "disclosure_boundary": "local",
        "query_context_or_intent": "find demonstrated capabilities",
        "actor_disclosure_scope": "auditor",
        "memory_store_target": "derived_memory",
        "source_artifact_id_and_hash": "ART-01:hash",
        "target_maturity_stage": "intellectual",
        "transformation_adapter_or_pipeline_rule": "spec_pipeline_v1",
        "declared_parameters": {},
        "source_cell_identity": "cell_alpha",
        "destination_cell_identity": "cell_beta",
        "transfer_boundary_specification": "bundle_manifest_v1",
        "local_administrator_authority": True,
    }

    session, results, affordances = engine.run_full_proof(
        journey_id=jid,
        actor_identity=actor_id,
        actor_role=actor_role,
        context_inputs=ctx,
    )

    print("\n--- Sequential IS1–IS8 Execution Trace ---")
    for r in results:
        status_sym = "[OK]  " if r.status == "passed" else "[FAIL]"
        print(f"{status_sym} {r.phase_id} {r.name}: {r.status.upper()}")
        if r.error_or_reason:
            print(f"      Reason: {r.error_or_reason}")
        for ev in r.evidence:
            print(f"      Evidence: {ev.item_id} ({ev.evidence_class}) => {ev.digest_or_value}")

    print("\n--- Phase IS8 Derived Next Affordances ---")
    for aff in affordances:
        status_flag = "[AVAILABLE]" if aff.status == "available" else "[BLOCKED]  "
        reason_txt = f" ({aff.blocked_reason.value}: {aff.blocked_explanation})" if aff.blocked_reason else ""
        print(f"  {status_flag} {aff.id} -> target: {aff.target_journey or 'local'}{reason_txt}")

    if session.settlement:
        print(f"\nSettlement Sovereign Decision: {session.settlement.outcome.upper()}")
        print(f"Explanation: {session.settlement.explanation}")


def run_case_aware_mode(engine: ProofEngine, args: argparse.Namespace) -> None:
    print("\n=== SEA Forge Case-Aware Navigation (Proof Mode) ===")
    sample_active_cases = [
        {"case_id": "CASE-2026-0812-A", "current_journey": "CJ05", "status": "active", "next_action": "start_enabled_work"},
        {"case_id": "CASE-2026-0814-B", "current_journey": "CJ06", "status": "waiting_approval", "next_action": "resolve_blocker_approval"},
        {"case_id": "CASE-2026-0820-C", "current_journey": "CJ09", "status": "execution_terminated", "next_action": "evaluate_and_settle_outcome"},
    ]
    entries = engine.get_case_aware_entries(sample_active_cases)
    print("What needs your attention?")
    for idx, e in enumerate(entries, 1):
        print(f"  {idx}. Case {e['case_id']} [{e['status']}] -> Next Affordance: {e['pending_affordance']} (Journey {e['journey_id']})")


def main() -> None:
    parser = argparse.ArgumentParser(description="Sequential Behavioral Proof CLI")
    parser.add_argument("--mode", choices=["catalog", "case_aware"], default="catalog", help="Entry mode")
    parser.add_argument("--journey", type=str, default="CJ01", help="Journey ID (CJ01..CJ12)")
    parser.add_argument("--actor", type=str, default="operator-alice", help="Actor identity")
    parser.add_argument("--role", type=str, default="", help="Actor role")
    args = parser.parse_args()

    engine = ProofEngine()
    if args.mode == "case_aware":
        run_case_aware_mode(engine, args)
    else:
        run_catalog_mode(engine, args)


if __name__ == "__main__":
    main()
