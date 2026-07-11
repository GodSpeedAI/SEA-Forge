# Genesis Settlement-Integrity Alignment

Date: 2026-07-11
Status: Approved design; pending normative spec edits

## Context

`.tmp/genesis.md` defines settlement through four joint conditions: durable
evidence, a proof obligation declared before action, reliability weighting of
the feedback channel, and a declaration by a component with standing that is
independent of the acting agent. It defines capability as repeated settlement
under relevant variation, including recovery and reduced orchestration burden.

The implemented minimum kernel records governed attempts and evaluates
predeclared criteria, but its prose can be read as promoting one accepted run
to a capability. The full spec aggregates accepted outcomes and currently lets
one acceptance satisfy `require_proven`; it does not model feedback reliability
or an independent settlement declaration.

## Decision

Preserve the implemented v0.1 kernel and its persisted records. Clarify that its
settlement is kernel-local and that `capabilities.jsonl` is a history of
capability attempts, not proof of durable capability. Do not rename persisted
files or fields.

Extend the full v0.2 contract additively:

1. SEA Forge constructs the claim and preserves authority, execution, and
   evidence records.
2. A `SettlementAuthority` verifies the predeclared proof obligation, scores
   feedback reliability, checks declarer standing and independence, and emits a
   `SettlementDeclaration`.
3. SEA Forge persists the declaration and permits capability promotion only
   from qualifying declarations.
4. `CapabilityRecord` distinguishes attempted, demonstrated, proven, and
   metabolized states. Promotion considers repeated accepted declarations,
   declared variation coverage, disruption recovery, reliability weight, and
   orchestration burden. Raw outcome counts remain observations.
5. `require_proven` tests the configured promotion threshold; it never means
   merely `accepted >= 1`.

SWE_SEED is the first named external settlement-authority adapter. A local
adapter may support development, but its declarations are not independent and
cannot satisfy policies that require strong settlement.

## Alternatives Considered

### Redefine the minimum records now

Add settlement authority and reliability fields directly to v0.1 records.
Rejected because the minimum is implemented and green, the full spec promises
additive evolution, and changing the persisted contract would re-key a proven
baseline.

### Treat SEA Forge's evaluator as sufficient settlement authority

Keep the full spec unchanged and describe the kernel evaluator as independent
from the child process. Rejected because it leaves no explicit standing,
independence, or feedback-reliability evidence and cannot distinguish local
verification from the stronger Genesis settlement claim.

### Recommended: additive declaration layer

Keep v0.1 compatible and add a v0.2 declaration protocol. This preserves the
kernel while making strong settlement and capability promotion explicit,
auditable, and policy-selectable.

## Boundaries

SEA Forge records costs when evidence provides them, but CognitiveOS owns
payment estimation and path valuation. GodSpeed-Agent owns horizon selection
and developmental routing. This change does not add general affordance ranking,
economic optimization, or agent learning to SEA Forge.

## Failure and Security Rules

- Criteria created after execution cannot yield a qualifying declaration.
- The acting agent cannot issue a qualifying declaration for its own work.
- Missing standing, independence, reliability assessment, or evidence fails
  closed for strong settlement and capability promotion.
- A weak/local settlement remains inspectable but contributes zero qualifying
  weight under a strong-settlement policy.
- Repeated identical runs do not establish variation coverage.
- Capability status can contract after regression, revoked authority, or loss
  of qualifying evidence; projections remain rebuildable from source records.

## Validation

The full-spec conformance matrix will cover post-hoc criteria, self-declaration,
gameable or low-confidence feedback, identical-run repetition, declared
variation, disruption recovery, promotion thresholds, and contraction. The
minimum P1-P4b proofs remain unchanged.

## Implementation Impact Audit

The intended minimum changes are documentation-only. After editing both specs,
compare every normative v0.1 requirement with the current Rust types and tests.
Change minimum code only if the clarified text exposes an actual mismatch; use
a focused failing test before any behavior change.
