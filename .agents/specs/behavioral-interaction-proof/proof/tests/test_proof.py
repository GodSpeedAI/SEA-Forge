#!/usr/bin/env python3
"""Comprehensive Deterministic Behavioral Test Suite for SEA Forge.

Verifies:
- All 12 Canonical Journeys (CJ01-CJ12)
- IS1-IS8 Phase Transitions
- Affordance Gate Evaluation (7 gates)
- Blocked Action Reasons
- Execution vs Settlement Sovereignty
- Anti-Self-Approval and Anti-Self-Certification Rules
- Affordance Dependency Degradation
- Fixture Matrix Conformance
"""

import json
import sys
import unittest
from pathlib import Path

# Setup import path
proof_root = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(proof_root))
sys.path.insert(0, str(proof_root / "src"))

from src.engine import ProofEngine
from src.models import BlockedReason


class TestBehavioralInteractionProof(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.spec_root = proof_root.parent
        cls.engine = ProofEngine(spec_root=cls.spec_root)
        with open(proof_root / "fixtures" / "cases.json", "r", encoding="utf-8") as f:
            cls.fixtures = json.load(f).get("test_cases", [])

    def test_catalog_completeness(self):
        """All 12 Canonical Journeys must be present in the catalog and loaded."""
        catalog_journeys = self.engine.get_catalog_entries()
        self.assertEqual(len(catalog_journeys), 12)
        expected_ids = [f"CJ{i:02d}" for i in range(1, 13)]
        actual_ids = [j["id"] for j in catalog_journeys]
        self.assertEqual(sorted(actual_ids), sorted(expected_ids))
        self.assertEqual(len(self.engine.journeys), 12)

    def test_journey_phase_completeness(self):
        """Every journey must contain all 8 phases IS1..IS8 without omission."""
        required_phases = [f"IS{i}" for i in range(1, 9)]
        for jid, doc in self.engine.journeys.items():
            phases = doc.get("phases", {})
            for p in required_phases:
                matching = [k for k in phases.keys() if k.startswith(p)]
                self.assertTrue(
                    len(matching) == 1,
                    f"Journey {jid} must have exactly one phase matching prefix {p}, found: {matching}"
                )
                phase_body = phases[matching[0]]
                self.assertTrue(bool(phase_body), f"Journey {jid} phase {p} must not be empty")

    def test_happy_path_all_twelve_journeys(self):
        """Each of the 12 journeys must complete IS1..IS8 cleanly under valid conditions."""
        sample_context_map = {
            "CJ01": {
                "role": "CellAdministrator",
                "ctx": {
                    "cell_root_path": "/var/sea/cells/primary",
                    "actor_identity": "admin-1",
                    "configuration_inputs": {"profile": "prod"},
                    "existing_ledger_history_when_present": True,
                },
            },
            "CJ02": {
                "role": "Operator",
                "ctx": {
                    "trusted_cell_and_actor_context": True,
                    "subject_of_inquiry": "capabilities",
                    "disclosure_scope_declared": "standard",
                },
            },
            "CJ03": {
                "role": "DomainAuthor",
                "ctx": {
                    "sea_source_files": [".sea/interaction/interaction-model.sea"],
                    "import_dependencies": [],
                    "projection_target_specification": "rust_core",
                },
            },
            "CJ04": {
                "role": "CaseOwner",
                "ctx": {
                    "actor_attribution": "case-owner-1",
                    "cell_root": "/var/sea/cells/primary",
                    "work_source_descriptor": "intent_packet",
                    "semantic_references": ["sea_forge.interaction"],
                    "environment_and_evaluator_bindings": ["jail_evaluator"],
                },
            },
            "CJ05": {
                "role": "CaseOwner",
                "ctx": {
                    "committed_case_id": "CASE-101",
                    "attributable_actor_context": "case-owner-1",
                    "run_records_index": "runs/index",
                },
            },
            "CJ06": {
                "role": "Approver",
                "ctx": {
                    "approval_request_id": "APPR-101",
                    "acting_human_identity": "approver-1",
                    "request_context_bundle": {"action": "scale_jail"},
                    "downstream_effect_descriptor": "unblock item 3",
                    "item_proposer_or_creator": "different-actor-creator",
                },
            },
            "CJ07": {
                "role": "Operator",
                "ctx": {
                    "committed_case_id": "CASE-101",
                    "enabled_item_id": "ITEM-1",
                    "authority_grant_token": "GRANT-123",
                    "sandbox_and_environment_spec": {"type": "bwrap"},
                    "execution_limits": {"timeout_seconds": 120},
                },
            },
            "CJ08": {
                "role": "Operator",
                "ctx": {
                    "case_and_run_identities": ["CASE-101", "RUN-1"],
                    "ordered_event_log_cursor": 42,
                    "intervention_scope_descriptor": "run_level",
                },
            },
            "CJ09": {
                "role": "Auditor",
                "ctx": {
                    "run_episode_id": "RUN-1",
                    "immutable_case_plan_criteria": ["criterion_A"],
                    "authority_and_trace_records": ["trace_A"],
                    "disclosure_boundary": "local",
                    "executor_identity": "worker-runner",
                },
            },
            "CJ10": {
                "role": "Operator",
                "ctx": {
                    "query_context_or_intent": "find demonstrated knowledge",
                    "actor_disclosure_scope": "standard",
                    "memory_store_target": "derived_index",
                },
            },
            "CJ11": {
                "role": "CaseOwner",
                "ctx": {
                    "source_artifact_id_and_hash": "ART-1:hash",
                    "target_maturity_stage": "intellectual",
                    "transformation_adapter_or_pipeline_rule": "spec_pipeline_v1",
                    "declared_parameters": {},
                },
            },
            "CJ12": {
                "role": "CellAdministrator",
                "ctx": {
                    "source_cell_identity": "cell-source",
                    "destination_cell_identity": "cell-dest",
                    "transfer_boundary_specification": "manifest_v1",
                    "local_administrator_authority": True,
                },
            },
        }

        for jid in [f"CJ{i:02d}" for i in range(1, 13)]:
            cfg = sample_context_map[jid]
            session, results, affordances = self.engine.run_full_proof(
                journey_id=jid,
                actor_identity="test-actor",
                actor_role=cfg["role"],
                context_inputs=cfg["ctx"],
            )
            # All 8 phases must be executed and pass
            self.assertEqual(len(results), 8, f"Journey {jid} did not complete 8 phases")
            for r in results:
                self.assertEqual(r.status, "passed", f"Journey {jid} phase {r.phase_id} failed: {r.error_or_reason}")
            self.assertIsNotNone(session.settlement)
            self.assertEqual(session.settlement.outcome, "settlement_accepted")
            self.assertTrue(len(affordances) > 0)
            available = [a for a in affordances if a.status == "available"]
            self.assertTrue(len(available) > 0, f"Journey {jid} must have spendable affordances")

    def test_missing_precondition_block_is1(self):
        """IS1 blocks immediately when required context is missing."""
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ01",
            actor_identity="admin-1",
            actor_role="CellAdministrator",
            context_inputs={"actor_identity": "admin-1"},  # missing cell_root_path, etc.
        )
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0].phase_id, "IS1")
        self.assertEqual(results[0].status, "blocked")
        self.assertIn("Missing required context elements", results[0].error_or_reason)
        self.assertEqual(len(affordances), 0)

    def test_preflight_check_failure_is2(self):
        """IS2 blocks when preflight checks fail (e.g. uncompromised keys)."""
        ctx = {
            "cell_root_path": "/var/sea/cells/primary",
            "actor_identity": "admin-1",
            "configuration_inputs": {},
            "existing_ledger_history_when_present": True,
        }
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ01",
            actor_identity="admin-1",
            actor_role="CellAdministrator",
            context_inputs=ctx,
            preflight_overrides={"identity_keys_present_and_uncompromised": False},
        )
        self.assertEqual(len(results), 2)
        self.assertEqual(results[1].phase_id, "IS2")
        self.assertEqual(results[1].status, "blocked")
        self.assertIn("identity_keys_present_and_uncompromised", results[1].details["failed_checks"])

    def test_authority_role_block_is3(self):
        """IS3 blocks if actor does not have the required role."""
        ctx = {
            "trusted_cell_and_actor_context": True,
            "subject_of_inquiry": "capabilities",
            "disclosure_scope_declared": "standard",
        }
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ02",
            actor_identity="random-user",
            actor_role="UnassignedGuest",
            context_inputs=ctx,
        )
        self.assertEqual(len(results), 3)
        self.assertEqual(results[2].phase_id, "IS3")
        self.assertEqual(results[2].status, "blocked")
        self.assertIn("not authorized for journey", results[2].error_or_reason)

    def test_anti_self_approval_block_is3(self):
        """CJ06 enforces anti-self-approval: proposer cannot approve their own work."""
        ctx = {
            "approval_request_id": "APPR-101",
            "acting_human_identity": "actor-eve",
            "request_context_bundle": {"action": "scale"},
            "downstream_effect_descriptor": "unblock item",
            "item_proposer_or_creator": "actor-eve",  # SAME ACTOR
        }
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ06",
            actor_identity="actor-eve",
            actor_role="Approver",
            context_inputs=ctx,
        )
        self.assertEqual(len(results), 3)
        self.assertEqual(results[2].phase_id, "IS3")
        self.assertEqual(results[2].status, "blocked")
        self.assertIn("Separation of duty violation", results[2].error_or_reason)

    def test_anti_self_certification_block_is3(self):
        """CJ09 enforces anti-self-certification: executor cannot settle their own work."""
        ctx = {
            "run_episode_id": "RUN-1",
            "immutable_case_plan_criteria": ["criterion_A"],
            "authority_and_trace_records": ["trace_A"],
            "disclosure_boundary": "local",
            "executor_identity": "actor-frank",  # SAME ACTOR
        }
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ09",
            actor_identity="actor-frank",
            actor_role="Auditor",
            context_inputs=ctx,
        )
        self.assertEqual(len(results), 3)
        self.assertEqual(results[2].phase_id, "IS3")
        self.assertEqual(results[2].status, "blocked")
        self.assertIn("anti-self-certification", results[2].error_or_reason)

    def test_execution_failure_with_recovery_is4(self):
        """IS4 handles sandbox/execution failure; leaves evidence and halts cleanly."""
        ctx = {
            "committed_case_id": "CASE-101",
            "enabled_item_id": "ITEM-1",
            "authority_grant_token": "GRANT-123",
            "sandbox_and_environment_spec": {"type": "bwrap"},
            "execution_limits": {"timeout_seconds": 120},
        }
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ07",
            actor_identity="operator-bob",
            actor_role="Operator",
            context_inputs=ctx,
            execution_success=False,
        )
        self.assertEqual(len(results), 4)
        self.assertEqual(results[3].phase_id, "IS4")
        self.assertEqual(results[3].status, "failed")
        self.assertIn("Execution terminated with error", results[3].error_or_reason)

    def test_settlement_rejection_despite_execution_success(self):
        """Execution exit 0 with failed criteria produces SETTLEMENT_REJECTED in IS7, and blocks downstream affordances."""
        ctx = {
            "committed_case_id": "CASE-101",
            "enabled_item_id": "ITEM-1",
            "authority_grant_token": "GRANT-123",
            "sandbox_and_environment_spec": {"type": "bwrap"},
            "execution_limits": {"timeout_seconds": 120},
        }
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ07",
            actor_identity="operator-bob",
            actor_role="Operator",
            context_inputs=ctx,
            execution_success=True,
            criteria_met=False,  # Settlement rejected
        )
        self.assertEqual(len(results), 8)
        self.assertEqual(results[6].phase_id, "IS7")
        self.assertEqual(results[6].status, "failed")
        self.assertEqual(session.settlement.outcome, "settlement_rejected")

        # In IS8, check that affordances requiring settlement are blocked
        self.assertTrue(len(affordances) > 0)
        for aff in affordances:
            self.assertEqual(aff.status, "blocked")
            self.assertEqual(aff.blocked_reason, BlockedReason.NO_SETTLEMENT_ACCESS)

    def test_affordance_dependency_degradation(self):
        """Affordance Dependency Test: Invalidate lower-level settlement and assert higher affordances disappear."""
        # 1. Under happy settlement in CJ04: start_lawful_work, resolve_likely_approvals, navigate_live_case are available
        ctx = {
            "actor_attribution": "case-owner-1",
            "cell_root": "/var/sea/cells/primary",
            "work_source_descriptor": "intent_packet",
            "semantic_references": ["sea_forge.interaction"],
            "environment_and_evaluator_bindings": ["jail_evaluator"],
        }
        session_happy, _, aff_happy = self.engine.run_full_proof(
            journey_id="CJ04",
            actor_identity="case-owner-1",
            actor_role="CaseOwner",
            context_inputs=ctx,
            criteria_met=True,
        )
        available_happy = [a.id for a in aff_happy if a.status == "available"]
        self.assertIn("start_lawful_work", available_happy)

        # 2. Invalidate settlement in CJ04 (criteria_met=False)
        session_degraded, _, aff_degraded = self.engine.run_full_proof(
            journey_id="CJ04",
            actor_identity="case-owner-1",
            actor_role="CaseOwner",
            context_inputs=ctx,
            criteria_met=False,
        )
        available_degraded = [a.id for a in aff_degraded if a.status == "available"]
        self.assertNotIn("start_lawful_work", available_degraded)
        self.assertEqual(len(available_degraded), 0)

    def test_evidence_integrity_inspection_is6(self):
        """IS6 catches corrupt evidence and halts before settlement."""
        ctx = {
            "committed_case_id": "CASE-101",
            "enabled_item_id": "ITEM-1",
            "authority_grant_token": "GRANT-123",
            "sandbox_and_environment_spec": {"type": "bwrap"},
            "execution_limits": {"timeout_seconds": 120},
        }
        session, results, affordances = self.engine.run_full_proof(
            journey_id="CJ07",
            actor_identity="operator-bob",
            actor_role="Operator",
            context_inputs=ctx,
            corrupt_evidence=True,
        )
        self.assertEqual(len(results), 6)
        self.assertEqual(results[5].phase_id, "IS6")
        self.assertEqual(results[5].status, "blocked")
        self.assertIn("Evidence inspection failed", results[5].error_or_reason)
        self.assertIsNone(session.settlement)

    def test_fixtures_matrix(self):
        """Execute all declared fixtures from cases.json and assert matching expectations."""
        self.assertTrue(len(self.fixtures) > 0)
        for tc in self.fixtures:
            tc_id = tc["id"]
            jid = tc["journey_id"]
            actor = tc["actor_identity"]
            role = tc["actor_role"]
            inputs = tc["inputs"]
            preflight_overrides = tc.get("preflight_overrides")
            exec_success = tc.get("execution_success", True)
            crit_met = tc.get("criteria_met", True)

            session, results, affordances = self.engine.run_full_proof(
                journey_id=jid,
                actor_identity=actor,
                actor_role=role,
                context_inputs=inputs,
                preflight_overrides=preflight_overrides,
                execution_success=exec_success,
                criteria_met=crit_met,
            )

            if "expected_phase_blocked" in tc:
                blocked_p = tc["expected_phase_blocked"]
                last_res = results[-1]
                self.assertEqual(
                    last_res.phase_id,
                    blocked_p,
                    f"Testcase {tc_id} expected phase {blocked_p} blocked, but {last_res.phase_id} was last"
                )
                self.assertIn(last_res.status, ["blocked", "failed"])
                if "expected_blocked_reason" in tc:
                    self.assertIn(
                        tc["expected_blocked_reason"].lower(),
                        (last_res.error_or_reason or "").lower()
                    )
            elif "expected_settlement" in tc:
                self.assertIsNotNone(session.settlement, f"Testcase {tc_id} expected settlement")
                self.assertEqual(session.settlement.outcome, tc["expected_settlement"])
                if "expected_affordances" in tc:
                    avail_ids = [a.id for a in affordances if a.status == "available"]
                    for exp_a in tc["expected_affordances"]:
                        self.assertIn(exp_a, avail_ids, f"Testcase {tc_id} missing affordance {exp_a}")


if __name__ == "__main__":
    unittest.main()
