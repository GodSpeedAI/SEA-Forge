"""Data models for Behavioral Interaction Proofs.

Represents IS1-IS8 phases, affordance gates, evidence items, and execution sessions.
Renderer-neutral: contains no widget, prompt, button, modal, or display primitives.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional


class OperationClass(str, Enum):
    EPISTEMIC = "epistemic"
    CONSEQUENTIAL = "consequential"


class EvidenceClass(str, Enum):
    DECLARED = "declared"
    OBSERVED = "observed"
    INFERRED = "inferred"
    PROPOSED = "proposed"
    UNKNOWN = "unknown"


class BlockedReason(str, Enum):
    MISSING_PRECONDITION = "missing_precondition"
    UNAUTHORIZED = "unauthorized"
    UNPAYABLE = "unpayable"
    MISSING_CAPABILITY = "missing_capability"
    UNRECOVERABLE = "unrecoverable"
    NO_SETTLEMENT_ACCESS = "no_settlement_access"
    POLICY_PROHIBITED = "policy_prohibited"


@dataclass
class AffordanceGate:
    visible: bool = True
    reachable: bool = True
    authorized: bool = True
    payable: bool = True
    executable: bool = True
    recoverable: bool = True
    settleable: bool = True

    @property
    def is_available(self) -> bool:
        return (
            self.visible
            and self.reachable
            and self.authorized
            and self.payable
            and self.executable
            and self.recoverable
            and self.settleable
        )


@dataclass
class ActionCandidate:
    id: str
    label: str
    target_journey: Optional[str] = None
    action_type: str = "navigation"
    operation_class: OperationClass = OperationClass.EPISTEMIC
    gates: AffordanceGate = field(default_factory=AffordanceGate)
    blocked_reason: Optional[BlockedReason] = None
    blocked_explanation: Optional[str] = None

    @property
    def status(self) -> str:
        if self.gates.is_available:
            return "available"
        return "blocked"


@dataclass
class EvidenceItem:
    item_id: str
    description: str
    evidence_class: EvidenceClass
    digest_or_value: str
    verified: bool = False


@dataclass
class PhaseResult:
    phase_id: str  # IS1 .. IS8
    name: str
    status: str    # passed | blocked | failed | escalated
    details: Dict[str, Any] = field(default_factory=dict)
    evidence: List[EvidenceItem] = field(default_factory=list)
    error_or_reason: Optional[str] = None


@dataclass
class SettlementDecision:
    outcome: str  # settlement_accepted | settlement_rejected | settlement_escalated | settlement_unavailable
    criterion_statement: str
    machine_evaluable: bool
    evidence_matched: bool
    explanation: str


@dataclass
class ProofStepRecord:
    step_index: int
    phase_id: str
    action_invoked: str
    result: PhaseResult
    state_after: str


@dataclass
class ProofSession:
    session_id: str
    entry_mode: str  # "catalog" | "case_aware"
    journey_id: str
    current_state: str
    actor_identity: str
    actor_role: str
    case_context: Dict[str, Any] = field(default_factory=dict)
    history: List[ProofStepRecord] = field(default_factory=list)
    evidence_ledger: List[EvidenceItem] = field(default_factory=list)
    settlement: Optional[SettlementDecision] = None
    completed: bool = False
    cancelled: bool = False
