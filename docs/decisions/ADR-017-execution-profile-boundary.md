# ADR-017: Bind a Cognate capability allow to one execution profile

Status: accepted, 2026-11-26 (CognitiveOS convergence, ticket #116).

## Context

A governed spawn is decided by a CEP `authority_request` that carries no argv, executable digest, or limits.
An allow therefore said nothing about which process would run. The grant path binds an executable path but not
its content, and the CEP decision envelope exports boundary constraints as `{constraint_type, description}`.

## Decision

Extend the existing contract; add no field, schema or verb.

* A new boundary dimension `execution_profile` on `cognate_capability` rules lists up to 8 permitted execution-profile
  digests (`sha256:<64 lowercase hex>`). Rules carrying it must be allow rules with boundary disposition.
  `cognate_capability` is chosen because Cognate sends capability invocations with `claims`; the `action`
  kind carries none.
* The request claims `execution_profile_digest`. A profile rule matches only an equal digest. If a profile rule
  governs the request, only a profile rule may allow it; a miss is denied with `execution_profile_mismatch`.
* The decision narrows the constraint to the matched digest; the envelope exports it as constraint type
  `execution_profile`. The claim is already recorded in the committed decision.

* An approval of a profile rule is bound to the escalated request's digest; the approved allow is a boundary
  decision carrying it.
* Governance is per rule scope. A subject-, resource- or exposure-scoped profile rule does not govern requests
  outside its scope, and those can reach a broader allow with no `execution_profile` constraint. The host
  refuses any contained spawn whose decision lacks the exact constraint. Matching semantics are unchanged.
* The digest list order is part of the bundle hash and is not re-sorted.

## Two digests

The host produces two digests. `executionProfileDigest` is stable: launcher and executable digests, restriction
spec, flag template, mount roles and environment names. `launchPlanDigest` is per launch: it binds run ids, the
workspace inode and per-port environment HMACs, and embeds the profile digest. A per-launch value can never be a
fixed policy value, so SEA binds only the profile digest, in the claim `execution_profile_digest`. The Workbench
host recomputes the profile digest, checks that the per-launch plan embeds it, and only then spawns. The branch
is unreleased, so there is no alias for the earlier claim key.

## Consequences

Policy bundles that do not use the dimension keep their bytes and hashes. SEA-Forge binds the digest the host
claims; it cannot verify it. The host must recompute and compare before spawn. A decision read-back verb
remains future work.
