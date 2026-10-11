"""Sequential Behavioral Interaction Proof Engine.

Executes and verifies journeys against the IS1-IS8 contract.
Maintains stateful affordance derivation, evidence separation,
and explicit blocker reasoning.
"""

import hashlib
import json
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import yaml

try:
    from .models import (
        ActionCandidate,
        AffordanceGate,
        BlockedReason,
        EvidenceClass,
        EvidenceItem,
        OperationClass,
        PhaseResult,
        ProofSession,
        ProofStepRecord,
        SettlementDecision,
    )
except ImportError:
    from models import (  # type: ignore
        ActionCandidate,
        AffordanceGate,
        BlockedReason,
        EvidenceClass,
        EvidenceItem,
        OperationClass,
        PhaseResult,
        ProofSession,
        ProofStepRecord,
        SettlementDecision,
    )


class ProofEngine:
    def __init__(self, spec_root: Optional[Path] = None):
        if spec_root is None:
            spec_root = Path(__file__).resolve().parent.parent.parent
        self.spec_root = spec_root
        self.contract = self._load_yaml(spec_root / "contract.yaml")
        self.catalog = self._load_yaml(spec_root / "catalog.yaml")
        self.journeys: Dict[str, Dict[str, Any]] = {}
        for j_file in (spec_root / "journeys").glob("*.yaml"):
            doc = self._load_yaml(j_file)
            jid = doc.get("journey", {}).get("id")
            if jid:
                self.journeys[jid] = doc

    def _load_yaml(self, path: Path) -> Dict[str, Any]:
        with open(path, "r", encoding="utf-8") as f:
            return yaml.safe_load(f) or {}

    def get_catalog_entries(self) -> List[Dict[str, Any]]:
        """List all canonical journeys for the Journey Catalog entry mode."""
        return self.catalog.get("journeys", [])

    def get_case_aware_entries(self, active_cases: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """List current spendable actions for the Case-Aware entry mode."""
        entries = []
        for case in active_cases:
            entries.append({
                "case_id": case.get("case_id"),
                "journey_id": case.get("current_journey", "CJ05"),
                "status": case.get("status"),
                "pending_affordance": case.get("next_action"),
            })
        entries.append({
            "case_id": "NEW",
            "journey_id": "CJ04",
            "status": "unstarted",
            "pending_affordance": "form_and_commit_governed_case",
        })
        return entries

    def start_session(
        self,
        journey_id: str,
        actor_identity: str,
        actor_role: str,
        entry_mode: str = "catalog",
        initial_context: Optional[Dict[str, Any]] = None,
    ) -> ProofSession:
        if journey_id not in self.journeys:
            raise ValueError(f"Unknown journey {journey_id}. Valid: {list(self.journeys.keys())}")
        
        j_doc = self.journeys[journey_id]
        entry_states = j_doc.get("entry", {}).get("states", ["initial"])
        current_state = entry_states[0] if entry_states else "initial"
        
        session = ProofSession(
            session_id=f"proof-session-{journey_id}-{int(hashlib.sha256(actor_identity.encode()).hexdigest()[:8], 16)}",
            entry_mode=entry_mode,
            journey_id=journey_id,
            current_state=current_state,
            actor_identity=actor_identity,
            actor_role=actor_role,
            case_context=initial_context or {},
        )
        return session

    def execute_phase_is1_context(
        self, session: ProofSession, context_inputs: Dict[str, Any]
    ) -> PhaseResult:
        j_doc = self.journeys[session.journey_id]
        is1 = j_doc.get("phases", {}).get("IS1_context_selection", {})
        requires = is1.get("requires", [])

        missing = [r for r in requires if r not in context_inputs and r not in session.case_context]
        if missing:
            return PhaseResult(
                phase_id="IS1",
                name="Context Selection",
                status="blocked",
                details={"missing_keys": missing},
                error_or_reason=f"Missing required context elements: {missing}",
            )

        session.case_context.update(context_inputs)
        digest = hashlib.sha256(json.dumps(session.case_context, sort_keys=True).encode()).hexdigest()
        
        ev = EvidenceItem(
            item_id="is1-context-digest",
            description="Cryptographic digest of pinned context elements",
            evidence_class=EvidenceClass.OBSERVED,
            digest_or_value=digest,
            verified=True,
        )
        session.evidence_ledger.append(ev)

        res = PhaseResult(
            phase_id="IS1",
            name="Context Selection",
            status="passed",
            details={"pinned_context_digest": digest, "scope": is1.get("context_scope")},
            evidence=[ev],
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS1", "select_context", res, session.current_state))
        return res

    def execute_phase_is2_preflight(
        self, session: ProofSession, preflight_overrides: Optional[Dict[str, bool]] = None
    ) -> PhaseResult:
        j_doc = self.journeys[session.journey_id]
        is2 = j_doc.get("phases", {}).get("IS2_preflight", {})
        checks = is2.get("checks", [])
        overrides = preflight_overrides or {}

        failed_checks = []
        for check in checks:
            passed = overrides.get(check, True)
            if not passed:
                failed_checks.append(check)

        if failed_checks:
            res = PhaseResult(
                phase_id="IS2",
                name="Preflight",
                status="blocked",
                details={"failed_checks": failed_checks},
                error_or_reason=f"Preflight checks failed: {failed_checks}",
            )
            session.history.append(ProofStepRecord(len(session.history) + 1, "IS2", "run_preflight", res, session.current_state))
            return res

        ev = EvidenceItem(
            item_id="is2-readiness-receipt",
            description="Verified readiness and constraint validation receipt",
            evidence_class=EvidenceClass.OBSERVED,
            digest_or_value="readiness_ready_token",
            verified=True,
        )
        session.evidence_ledger.append(ev)

        res = PhaseResult(
            phase_id="IS2",
            name="Preflight",
            status="passed",
            details={"checks_passed": checks, "readiness": "ready"},
            evidence=[ev],
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS2", "run_preflight", res, session.current_state))
        return res

    def execute_phase_is3_authority(
        self, session: ProofSession, grant_override: Optional[bool] = None
    ) -> PhaseResult:
        j_doc = self.journeys[session.journey_id]
        is3 = j_doc.get("phases", {}).get("IS3_authority_resolution", {})
        req_role = is3.get("required_role")
        alt_role = is3.get("alternate_role")

        # Anti-self-approval / Anti-self-certification check
        if session.journey_id == "CJ06":
            creator = session.case_context.get("item_proposer_or_creator")
            if creator and creator == session.actor_identity:
                return PhaseResult(
                    phase_id="IS3",
                    name="Authority Resolution",
                    status="blocked",
                    error_or_reason="Separation of duty violation: Item creator cannot approve their own work",
                    details={"separation_of_duty_violated": True},
                )
        elif session.journey_id == "CJ09":
            executor = session.case_context.get("executor_identity")
            if executor and executor == session.actor_identity:
                return PhaseResult(
                    phase_id="IS3",
                    name="Authority Resolution",
                    status="blocked",
                    error_or_reason="Separation of duty violation: Executor cannot settle their own work (anti-self-certification)",
                    details={"anti_self_certification_violated": True},
                )

        # Role verification
        allowed_roles = [r for r in [req_role, alt_role] if r]
        if session.actor_role not in allowed_roles:
            return PhaseResult(
                phase_id="IS3",
                name="Authority Resolution",
                status="blocked",
                details={"actor_role": session.actor_role, "allowed_roles": allowed_roles},
                error_or_reason=f"Actor role '{session.actor_role}' not authorized for journey. Requires {allowed_roles}",
            )

        if grant_override is False:
            return PhaseResult(
                phase_id="IS3",
                name="Authority Resolution",
                status="blocked",
                details={"grant_decision": "denied"},
                error_or_reason="Policy denied requested operational authority grant",
            )

        grant_token = hashlib.sha256(f"grant-{session.actor_identity}-{session.journey_id}".encode()).hexdigest()
        ev = EvidenceItem(
            item_id="is3-authority-grant",
            description="Verified authority grant token and policy scope",
            evidence_class=EvidenceClass.OBSERVED,
            digest_or_value=grant_token,
            verified=True,
        )
        session.evidence_ledger.append(ev)

        res = PhaseResult(
            phase_id="IS3",
            name="Authority Resolution",
            status="passed",
            details={"grant_token": grant_token, "role": session.actor_role, "scope": is3.get("grant_scope")},
            evidence=[ev],
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS3", "resolve_authority", res, session.current_state))
        return res

    def execute_phase_is4_capability(
        self, session: ProofSession, execution_success: bool = True, sim_output: Optional[Dict[str, Any]] = None
    ) -> PhaseResult:
        j_doc = self.journeys[session.journey_id]
        is4 = j_doc.get("phases", {}).get("IS4_capability_invocation", {})
        cap = is4.get("capability", {})
        cap_id = cap.get("semantic_id", "capability.invoke")
        bounds = is4.get("bounds", {})

        if not execution_success:
            res = PhaseResult(
                phase_id="IS4",
                name="Capability Invocation",
                status="failed",
                details={"capability_id": cap_id, "bounds": bounds},
                error_or_reason="Execution terminated with error within granted sandbox bounds",
            )
            session.history.append(ProofStepRecord(len(session.history) + 1, "IS4", cap_id, res, session.current_state))
            return res

        output_payload = sim_output or {"status": "success", "result": f"Simulated output of {cap_id}"}
        output_hash = hashlib.sha256(json.dumps(output_payload, sort_keys=True).encode()).hexdigest()
        ev = EvidenceItem(
            item_id="is4-execution-output",
            description="Hash of execution output / process termination evidence",
            evidence_class=EvidenceClass.OBSERVED,
            digest_or_value=output_hash,
            verified=True,
        )
        session.evidence_ledger.append(ev)

        res = PhaseResult(
            phase_id="IS4",
            name="Capability Invocation",
            status="passed",
            details={"capability_id": cap_id, "output_hash": output_hash, "payload": output_payload},
            evidence=[ev],
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS4", cap_id, res, session.current_state))
        return res

    def execute_phase_is5_state_effect(
        self, session: ProofSession, target_state_override: Optional[str] = None
    ) -> PhaseResult:
        j_doc = self.journeys[session.journey_id]
        is5 = j_doc.get("phases", {}).get("IS5_state_and_artifact_effect", {})
        expected_state = is5.get("expected_state", {})
        new_state = target_state_override or expected_state.get("to", "completed")
        artifacts = is5.get("artifacts", [])

        from_state = session.current_state
        session.current_state = new_state

        art_hashes = [hashlib.sha256(f"{a}-{session.session_id}".encode()).hexdigest() for a in artifacts]
        ev = EvidenceItem(
            item_id="is5-artifact-state-transition",
            description="Appended state transition tuple and generated artifact digests",
            evidence_class=EvidenceClass.OBSERVED,
            digest_or_value=f"{from_state}->{new_state}:{len(artifacts)}_artifacts",
            verified=True,
        )
        session.evidence_ledger.append(ev)

        res = PhaseResult(
            phase_id="IS5",
            name="State and Artifact Effect",
            status="passed",
            details={
                "transition": {"from": from_state, "to": new_state},
                "artifacts": artifacts,
                "artifact_hashes": art_hashes,
            },
            evidence=[ev],
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS5", "apply_state_effect", res, session.current_state))
        return res

    def execute_phase_is6_evidence_inspection(
        self, session: ProofSession, corrupt_evidence: bool = False
    ) -> PhaseResult:
        j_doc = self.journeys[session.journey_id]
        is6 = j_doc.get("phases", {}).get("IS6_evidence_inspection", {})
        required = is6.get("required_evidence", [])

        if corrupt_evidence:
            return PhaseResult(
                phase_id="IS6",
                name="Evidence Inspection",
                status="blocked",
                error_or_reason="Evidence inspection failed: cryptographic digest mismatch or broken provenance chain",
                details={"integrity_check": "failed"},
            )

        ev_receipt = EvidenceItem(
            item_id="is6-inspection-receipt",
            description="All required evidence items verified against immutable ledger",
            evidence_class=EvidenceClass.OBSERVED,
            digest_or_value=f"inspected_{len(required)}_items_ok",
            verified=True,
        )
        session.evidence_ledger.append(ev_receipt)

        res = PhaseResult(
            phase_id="IS6",
            name="Evidence Inspection",
            status="passed",
            details={"inspected_items": [r.get("item") for r in required if isinstance(r, dict)]},
            evidence=[ev_receipt],
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS6", "inspect_evidence", res, session.current_state))
        return res

    def execute_phase_is7_settlement(
        self, session: ProofSession, criteria_met: bool = True
    ) -> PhaseResult:
        j_doc = self.journeys[session.journey_id]
        is7 = j_doc.get("phases", {}).get("IS7_settlement_or_decision", {})
        crit = is7.get("criterion", {})
        statement = crit.get("statement", "Execution criterion satisfied")
        machine_eval = crit.get("machine_evaluable", True)

        if not criteria_met:
            decision = SettlementDecision(
                outcome="settlement_rejected",
                criterion_statement=statement,
                machine_evaluable=machine_eval,
                evidence_matched=False,
                explanation="Observed evidence failed to satisfy the completion criterion. (Execution succeeded, but settlement rejected)",
            )
            session.settlement = decision
            res = PhaseResult(
                phase_id="IS7",
                name="Settlement or Decision",
                status="failed",
                details={"settlement": decision.__dict__},
                error_or_reason=decision.explanation,
            )
            session.history.append(ProofStepRecord(len(session.history) + 1, "IS7", "evaluate_settlement", res, session.current_state))
            return res

        decision = SettlementDecision(
            outcome="settlement_accepted",
            criterion_statement=statement,
            machine_evaluable=machine_eval,
            evidence_matched=True,
            explanation="Observed evidence completely satisfied the completion criterion.",
        )
        session.settlement = decision

        ev = EvidenceItem(
            item_id="is7-settlement-token",
            description="Independent settlement approval token",
            evidence_class=EvidenceClass.OBSERVED,
            digest_or_value=hashlib.sha256(f"settled-{session.session_id}".encode()).hexdigest(),
            verified=True,
        )
        session.evidence_ledger.append(ev)

        res = PhaseResult(
            phase_id="IS7",
            name="Settlement or Decision",
            status="passed",
            details={"settlement": decision.__dict__},
            evidence=[ev],
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS7", "evaluate_settlement", res, session.current_state))
        return res

    def derive_phase_is8_next_affordances(self, session: ProofSession) -> List[ActionCandidate]:
        """Derive spendable affordances using the 7 gates:
        visible, reachable, authorized, payable, executable, recoverable, settleable.
        """
        j_doc = self.journeys[session.journey_id]
        is8 = j_doc.get("phases", {}).get("IS8_next_affordance_selection", {})
        current = is8.get("current_affordances", [])
        blocked = is8.get("blocked_affordances", [])

        candidates: List[ActionCandidate] = []

        # If settlement was rejected, subsequent promotion/commitment actions are blocked
        settled_ok = session.settlement and session.settlement.outcome == "settlement_accepted"

        for aff in current:
            aid = aff.get("id")
            label = aff.get("action") or aff.get("id")
            target = aff.get("target_journey")

            # Check affordance dependency on settlement
            gates = AffordanceGate()
            b_reason = None
            b_expl = None

            if not settled_ok:
                gates.settleable = False
                gates.executable = False
                b_reason = BlockedReason.NO_SETTLEMENT_ACCESS
                b_expl = "Prior work has not been accepted by governed settlement"

            candidate = ActionCandidate(
                id=aid,
                label=label,
                target_journey=target,
                action_type="navigation",
                operation_class=OperationClass.CONSEQUENTIAL if target in ["CJ04", "CJ07", "CJ11", "CJ12"] else OperationClass.EPISTEMIC,
                gates=gates,
                blocked_reason=b_reason,
                blocked_explanation=b_expl,
            )
            candidates.append(candidate)

        for blk in blocked:
            bid = blk.get("id")
            target = blk.get("target_journey")
            reason_str = blk.get("reason", "policy_prohibited")
            try:
                reason_enum = BlockedReason(reason_str)
            except ValueError:
                reason_enum = BlockedReason.POLICY_PROHIBITED

            gates = AffordanceGate(authorized=False, executable=False)
            candidate = ActionCandidate(
                id=bid,
                label=bid,
                target_journey=target,
                action_type="blocked_diagnostic",
                operation_class=OperationClass.CONSEQUENTIAL,
                gates=gates,
                blocked_reason=reason_enum,
                blocked_explanation=blk.get("explanation", "Action blocked by policy or missing precondition"),
            )
            candidates.append(candidate)

        res = PhaseResult(
            phase_id="IS8",
            name="Next Affordance Selection",
            status="passed",
            details={
                "available": [c.id for c in candidates if c.status == "available"],
                "blocked": [c.id for c in candidates if c.status == "blocked"],
            },
        )
        session.history.append(ProofStepRecord(len(session.history) + 1, "IS8", "derive_affordances", res, session.current_state))
        session.completed = True
        return res, candidates

    def run_full_proof(
        self,
        journey_id: str,
        actor_identity: str,
        actor_role: str,
        context_inputs: Dict[str, Any],
        preflight_overrides: Optional[Dict[str, bool]] = None,
        grant_override: Optional[bool] = None,
        execution_success: bool = True,
        criteria_met: bool = True,
        corrupt_evidence: bool = False,
    ) -> Tuple[ProofSession, List[PhaseResult], List[ActionCandidate]]:
        """Run all 8 phases consecutively, halting if a blocker is encountered."""
        session = self.start_session(journey_id, actor_identity, actor_role, initial_context=context_inputs)
        results: List[PhaseResult] = []

        # IS1
        r1 = self.execute_phase_is1_context(session, context_inputs)
        results.append(r1)
        if r1.status != "passed":
            return session, results, []

        # IS2
        r2 = self.execute_phase_is2_preflight(session, preflight_overrides)
        results.append(r2)
        if r2.status != "passed":
            return session, results, []

        # IS3
        r3 = self.execute_phase_is3_authority(session, grant_override)
        results.append(r3)
        if r3.status != "passed":
            return session, results, []

        # IS4
        r4 = self.execute_phase_is4_capability(session, execution_success)
        results.append(r4)
        if r4.status != "passed":
            return session, results, []

        # IS5
        r5 = self.execute_phase_is5_state_effect(session)
        results.append(r5)
        if r5.status != "passed":
            return session, results, []

        # IS6
        r6 = self.execute_phase_is6_evidence_inspection(session, corrupt_evidence)
        results.append(r6)
        if r6.status != "passed":
            return session, results, []

        # IS7
        r7 = self.execute_phase_is7_settlement(session, criteria_met)
        results.append(r7)
        if r7.status != "passed":
            # Note: even if settlement is rejected, IS8 derives blocked affordances
            r8, affordances = self.derive_phase_is8_next_affordances(session)
            results.append(r8)
            return session, results, affordances

        # IS8
        r8, affordances = self.derive_phase_is8_next_affordances(session)
        results.append(r8)
        return session, results, affordances
