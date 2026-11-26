# ADR-016: Bind a Cognate capability allow to one execution profile

Status: accepted, 2026-11-26 (CognitiveOS convergence, ticket #116).

## Context

A governed spawn is decided by a CEP `authority_request` that carries no argv, executable digest, or limits.
An allow therefore said nothing about which process would run. The grant path binds an executable path but not
its content, and the CEP decision envelope exports boundary constraints as `{constraint_type, description}`.

## Decision

Extend the existing contract; add no field, schema or verb.

* A new boundary dimension `execution_profile` on `cognate_capability` rules lists up to 8 permitted launch-plan
  digests (`sha256:<64 lowercase hex>`). Rules carrying it must be allow rules with boundary disposition.
  `cognate_capability` is chosen because Cognate sends capability invocations with `claims`; the `action`
  kind carries none.
* The request claims `launch_plan_digest`. A profile rule matches only an equal digest. If a profile rule
  governs the request, only a profile rule may allow it; a miss is denied with `execution_profile_mismatch`.
* The decision narrows the constraint to the matched digest; the envelope exports it as constraint type
  `execution_profile`. The claim is already recorded in the committed decision.

## Consequences

Policy bundles that do not use the dimension keep their bytes and hashes. SEA-Forge binds the digest the host
claims; it cannot verify it. The host must recompute and compare before spawn. A decision read-back verb
remains future work.
