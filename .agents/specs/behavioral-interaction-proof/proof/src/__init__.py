"""Behavioral Interaction Proof Engine for SEA Forge.

Provides renderer-neutral Behavioral IR execution, affordance derivation,
and deterministic verification.
"""

from .engine import ProofEngine
from .models import (
    ActionCandidate,
    AffordanceGate,
    BlockedReason,
    EvidenceItem,
    PhaseResult,
    ProofSession,
    SettlementDecision,
)

__all__ = [
    "ProofEngine",
    "ActionCandidate",
    "AffordanceGate",
    "BlockedReason",
    "EvidenceItem",
    "PhaseResult",
    "ProofSession",
    "SettlementDecision",
]
