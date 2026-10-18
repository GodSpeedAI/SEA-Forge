# ADR-010: Security acceptance v1

**Status:** Accepted for the operator-requested test-only freeze

**Date:** 2026-10-16

**Scope:** Behavioral security contract, acceptance tests, controlled fixtures,
evidence and integrity protection. No production enforcement changes.

## Context

Policy decisions and caller-reported successful execution do not independently
prove protected resources enforced access or produced the declared outcome.
The historical local identity slice uses a UID plus an actor claim; the new
operator contract additionally requires non-impersonable workload identity even
when workloads share that UID. These are observable acceptance obligations,
not a selection of a new identity or execution architecture.

## Decisions

1. Freeze the normative SEC-01 through SEC-12 contract and affordable Gate A/B/C
   specifications under `tests/security-v1`, referenced by
   `.agents/specs/security-acceptance-v1.spec.md`.
2. Reuse actual SFWP identity/approval, authority-grant/runtime, ledger and
   operational settlement interfaces. Observe real approval persistence and
   filesystem outcomes independently of policy verdicts. Script adversarial
   attempts so model refusal cannot substitute for boundary enforcement.
3. Keep PostgreSQL assertions provider-neutral. A configurable URI or optional
   local provisioner supplies real PostgreSQL; privileged setup and generated
   metadata-only runtime credentials remain separate. Optional pg0 may provide
   a URI without becoming a runtime dependency or changing assertions.
4. Exercise MCP through its standard protocol and independently instrumented,
   unrestricted upstream. The current absence of product MCP ingress is a
   visible blocker, not a fixture-provided policy implementation.
5. Preserve SECURITY-RED, INFRASTRUCTURE-BLOCKED and NOT-YET-EXECUTABLE as
   distinct non-green outcomes. Every full-gate claim must include remaining
   integration/observer gaps, even when narrower subcases pass.
6. Hash protected files against the pre-change baseline, commit the suite and
   independently protect a remote frozen tag. Verify using the externally
   pinned commit's manifest and verifier, with local replacement refs disabled.
   Retain exact remote protection/publication evidence; never claim a local
   editable manifest or movable tag is independent integrity protection.

## Rejected alternatives

Policy-only assertions, mocked resource success, verbal model refusal, missing
symbol stubs, unconditional failed assertions and skipped-as-passed database
tests cannot establish the requested behavioral contract. Adding enforcement
inside a fixture would substitute a different system for the product.

## Consequences and limitations

Production behavior stays unchanged during this phase and some acceptance
assertions intentionally remain RED. Independent per-workload authentication,
credential issuance, active-session revocation, exposure/context propagation,
all output transports and hostile same-UID observer tampering are not fully
observable in the initial harness. The inventory states those gaps. Supplementary
versions must be separately reviewed/frozen; v1 cannot be rewritten to become
green. Protected remote refs still rely on repository administrators retaining
their protection; full commit IDs should additionally be retained externally.
