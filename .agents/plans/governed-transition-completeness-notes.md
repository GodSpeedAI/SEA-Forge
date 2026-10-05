Feed it back as a **new verification obligation plus a permanent regression fixture**, not as a note or lesson learned.

The incident exposed a specific missing property:

> SEA-Forge proved that forbidden transitions were unreachable, but did not prove that permitted transitions were reachable.

I would encode that as a first-class invariant called something like **Governed Transition Completeness**.

For every governed transition (T), SEA-Forge should require four proofs:

```text
1. SAFETY
   Invalid predecessor states cannot reach T.

2. REACHABILITY
   Every valid predecessor state has at least one executable path to T.

3. GATE PRESERVATION
   A valid path reaches T without bypassing any required gate.

4. PAYMENT ORDERING
   No irreversible/payment-bearing action occurs before all required gates settle.
```

Your RealityTrace failure becomes the canonical counterexample:

```text
state:
    PILOT_AUTHORIZED
    release token valid
    marker valid
    fingerprints valid
    matrix valid

expected affordance:
    construct configured ModelClient
    enter authorized pilot

actual:
    ExecutionGate requires client
    client construction occurs only after ExecutionGate
    therefore valid transition is unreachable

settlement:
    SAFE but NOT TRANSITION-COMPLETE
```

That distinction is valuable because a system can be perfectly fail-closed and still be unusable.

## How I would feed it into SEA-Forge

First, preserve the exact RealityTrace incident as a regression fixture. Do not abstract it away immediately. Something like:

```text
sea-forge/
  tests/
    fixtures/
      verification/
        governed-transition-completeness/
          sxr-pilot-client-gate/
```

The fixture should contain a minimal state machine equivalent to:

```text
FROZEN_NOT_STARTED
        ↓ owner authorization
PILOT_AUTHORIZED
        ↓ static gate
CLIENT_CONSTRUCTION
        ↓ client gate
PILOT_EXECUTION
```

with the defective implementation encoded as:

```text
PILOT_AUTHORIZED
        ↓
client-required gate(client=None)
        ✕
CLIENT_CONSTRUCTION
```

Then add the concept to the semantic model. Conceptually, your `.sea` should be able to express something like:

```text
Invariant GovernedTransitionCompleteness {
    applies_to: GovernedTransition

    requires:
        forbidden_predecessors_unreachable
        valid_predecessors_reachable
        required_gates_preserved
        payment_after_gate_settlement
}
```

I would **not** make "reachability" a generic boolean attached to transitions. SEA-Forge should derive proof obligations from the state graph.

For each transition:

```text
T : S_valid → S_target
```

SEA-Forge generates:

```text
NEGATIVE OBLIGATION
∀ s ∉ AllowedPredecessors(T):
    reachable(s, T) = false

POSITIVE OBLIGATION
∀ s ∈ AllowedPredecessors(T):
    ∃ path p:
        p starts at s
        p reaches T
        all required gates settle
        no forbidden side effect occurs before settlement
```

That positive obligation is what your current verification missed.

## Then make SEA-Forge generate two kinds of tests

Today, your verification apparently emphasized tests equivalent to:

```text
unauthorized → transport impossible
```

It should automatically demand the dual:

```text
authorized + all invariants satisfied
    → transport becomes reachable
```

For this particular fixture the generated tests would look conceptually like:

```text
test_unauthorized_pilot_cannot_construct_transport()
test_authorized_pilot_can_reach_client_construction()
test_static_gate_failure_prevents_client_construction()
test_valid_static_gate_reaches_client_gate()
test_valid_client_gate_reaches_unit_loop()
test_no_provider_call_occurs_before_final_gate()
test_full_remains_unreachable_from_pilot_authorization()
```

The second test is the one that would have caught your defect before GLM ever tried to execute.

## I would also add a verification classification

Right now "PASS" is probably too coarse for this class of thing.

For a governed transition, SEA-Forge should distinguish:

```text
SAFE
REACHABLE
GATE_PRESERVING
PAYMENT_ORDERED
```

Only:

```text
SAFE
+
REACHABLE
+
GATE_PRESERVING
+
PAYMENT_ORDERED
```

earns:

```text
TRANSITION_COMPLETE
```

RealityTrace before the live attempt would therefore have settled as:

```text
SAFE: true
REACHABLE: false
TRANSITION_COMPLETE: false
```

rather than "351 tests pass, ready."

That's the representational upgrade I'd want most from this incident.

## Give Sol this prompt in SEA-Forge

```text
FEED THE REALITYTRACE PILOT-GATE FAILURE BACK INTO SEA-FORGE AS A
GENERAL VERIFICATION CAPABILITY.

This is NOT a RealityTrace repair task.

The RealityTrace incident is a counterexample demonstrating a missing
verification property in SEA-Forge.

======================================================================
OBSERVED COUNTEREXAMPLE
======================================================================

RealityTrace reached a scientifically and governably valid state:

    status = PILOT_AUTHORIZED
    execution_authorized = true
    authorized_stage = pilot
    release token valid
    release marker valid
    fingerprints valid
    matrix valid
    evidence invariants valid
    FULL unauthorized

Offline tests had strongly established that unauthorized execution could not
reach provider transport.

However, the first real authorized invocation failed before transport:

    ExecutionGate required ModelClient != None
    _cmd_run invoked the gate with client=None
    _live_client() was positioned after that gate
    therefore client construction was unreachable

A further explicit NotImplementedError also prevented the authorized path from
reaching the unit loop.

No provider/model call occurred.

The architecture was SAFE but the authorized transition was NOT REACHABLE.

======================================================================
THE MISSING VERIFICATION PROPERTY
======================================================================

Introduce a first-class SEA-Forge verification property:

    Governed Transition Completeness

For every governed transition T, require proof of all four:

1. SAFETY

   Forbidden predecessor states cannot reach T.

2. REACHABILITY

   Every valid predecessor state has at least one executable path to T.

3. GATE PRESERVATION

   Valid paths to T do not bypass required gates.

4. PAYMENT ORDERING

   No irreversible, external, payment-bearing, or consequential action occurs
   before all required gates settle.

A transition is:

    TRANSITION_COMPLETE

iff all four properties settle true.

Do not collapse safety and reachability into one boolean.

======================================================================
SEMANTIC MODEL
======================================================================

Extend the SEA-Forge semantic model/IR using the existing architecture rather
than inventing an isolated checker.

Represent, at minimum:

    State
    Transition
    AllowedPredecessor
    ForbiddenPredecessor
    RequiredGate
    ConsequentialAction
    PaymentBearingAction
    ReachabilityObligation
    SafetyObligation
    GatePreservationObligation
    PaymentOrderingObligation
    TransitionCompletenessSettlement

Use existing SEA/.sea conventions and terminology wherever equivalents already
exist.

Do not introduce duplicate ontology unnecessarily.

======================================================================
PROOF OBLIGATIONS
======================================================================

For each governed transition:

    T : S -> S'

SEA-Forge should derive BOTH negative and positive obligations.

NEGATIVE:

    for every forbidden predecessor state:
        no executable path may reach T

POSITIVE:

    for every valid predecessor state:
        at least one executable path must reach T

and that positive path must:

    traverse all required gates
    preserve gate ordering
    avoid forbidden bypasses
    avoid irreversible/payment-bearing action before settlement

The RealityTrace defect must fail the POSITIVE obligation even though it passes
the NEGATIVE obligation.

======================================================================
CANONICAL REGRESSION FIXTURE
======================================================================

Preserve a minimalized version of the RealityTrace incident as a permanent
SEA-Forge regression fixture.

Create an appropriate fixture under the existing test/fixture conventions,
conceptually equivalent to:

    tests/fixtures/verification/governed-transition-completeness/
        sxr-pilot-client-gate/

The defective fixture should encode:

    PILOT_AUTHORIZED
        ->
    client-required gate(client=None)
        ->
    unreachable client construction

Expected verification:

    safety = PASS
    reachability = FAIL
    transition_complete = FAIL

Also create the corrected fixture:

    PILOT_AUTHORIZED
        ->
    static authorization gate
        ->
    client construction
        ->
    client-dependent gate
        ->
    execution

Expected:

    safety = PASS
    reachability = PASS
    gate_preservation = PASS
    payment_ordering = PASS
    transition_complete = PASS

======================================================================
GENERATED TEST OBLIGATIONS
======================================================================

SEA-Forge should be capable of generating or requiring tests equivalent to:

    unauthorized state cannot reach consequential action

    authorized state can reach the next permitted transition

    static gate failure prevents client construction

    static gate success permits client construction

    valid client state can reach execution

    required gates cannot be skipped

    provider/payment action cannot occur before final gate

    authorization for PILOT cannot make FULL reachable

Do not hardcode these as RealityTrace-specific behavior.

Generalize them from the transition model.

======================================================================
VERIFICATION RESULT MODEL
======================================================================

A generic PASS is insufficient.

Expose at least:

    safety
    reachability
    gate_preservation
    payment_ordering
    transition_complete

For example, the defective RealityTrace fixture must settle:

    safety = true
    reachability = false
    gate_preservation = indeterminate/not_reached as appropriate
    payment_ordering = true
    transition_complete = false

Use precise three-state/typed semantics where boolean would destroy useful
information.

======================================================================
AFFORDANCE INTERPRETATION
======================================================================

Preserve this architectural distinction:

    permitted destination != executable affordance

A governed transition is an affordance only when it is:

    visible
    reachable
    payable
    governable
    settleable

Therefore an authorized-but-unreachable transition must NOT be represented as
an available affordance.

This should integrate with existing SEA-Forge/CognitiveOS semantics rather than
become an unrelated lint rule.

======================================================================
IMPLEMENTATION STRATEGY
======================================================================

First inspect SEA-Forge and identify:

    existing transition/state representation
    existing invariant representation
    existing graph/reachability machinery
    existing verification obligations
    existing generated tests
    existing settlement/result types

Produce a short implementation map.

Then make the smallest coherent extension.

Prefer:

    semantic representation
        ->
    derived obligation
        ->
    verifier
        ->
    generated test / proof artifact
        ->
    settlement

over an ad-hoc source grep.

======================================================================
ACCEPTANCE TEST
======================================================================

The key settlement criterion is:

Had this capability existed before the RealityTrace pilot authorization,
SEA-Forge would have rejected the runner as NOT TRANSITION_COMPLETE because
the valid PILOT_AUTHORIZED state could not reach the live execution loop.

Prove that with the regression fixture.

======================================================================
DELIVERABLES
======================================================================

Return:

    semantic/IR changes
    verification-rule changes
    canonical regression fixture
    generated/required test obligations
    tests
    example verification result for defective fixture
    example verification result for corrected fixture
    commit SHA(s)

Explicitly state:

    Would the original RealityTrace runner have failed this new verification?
        YES / NO

The correct acceptance result is YES.

Do not modify RealityTrace in this task.
```

That gives you a proper **metabolization loop**:

```text
runtime failure
    ↓
counterexample
    ↓
new representation
    ↓
new verification obligation
    ↓
new generated test
    ↓
permanent regression fixture
    ↓
future projects cannot repeat the same failure silently
```

That's how I would feed failures back into SEA-Forge generally: **every escaped defect should either falsify an existing invariant or reveal a missing invariant; then the smallest reproducible counterexample becomes part of the compiler/verifier's permanent developmental memory.**
