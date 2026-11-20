# H — Transformation brief

For the implementation workflow (the owner intends Matt Pocock's unmodified skills: clarify →
spec → tickets → TDD → review → retro). This brief does not replace that workflow's own
verification; it tells it what transformation is justified and what must not be reopened.

## Problem to be solved

Cognate has no recursive-execution capability, and the ecosystem has no way for a Workbench Expert
or Team to use one without either duplicating a second agent hierarchy or making a vendor binary
mandatory. Separately, Workbench drops case and world identity at every external boundary, and a
governed settlement never reaches the evidence stream, so "finished", "locally accepted" and
"externally settled" cannot all be read from the data.

Solve exactly that. Do not build a new agent framework, scheduler, case engine, envelope, or
authority.

## Current verified behaviour (baseline)

Baselines are pinned in [`README.md`](README.md). The short version, with citations in
[`A`](A-actual-state.md):

* Workbench Experts/Teams are real: one Orchestrator enforced in the versioned mutation path
  (`src/experts/state.ts:39-47`), delegation narrows authority by intersection
  (`src/experts/service.ts:195-199`), children are durable assignments with continuations and boot
  reconciliation (`src/agents/agentCoding.ts:806-816`, `src/experts/bridge.ts:93-103`), billing
  reserves before dispatch (`src/experts/billing.ts:1-9`), effects fail into
  `reconciliation-required` (`src/capabilities/expertEffect.ts:1-45`).
* There is **no RLM contract anywhere** (searches recorded in `recon/`), Cognate's `ProcessPort` is
  buffered (`cognate/packages/execution/src/types.ts:64-66`), and the only long-lived stdio wrapper
  is Workbench-local on `feat/capability-runtime-t4a`.
* `prime-agent` 0.10.2 is installed and is the **Rust** build with **no tool allowlist** (no
  `--tools`/`--no-builtin-tools`/`--exclude-tools`, no `defaultTools` key, zero binary hits).
* Workbench emits no CEP profile and no `world_ref`; its `worldHash` is a local manifest hash
  (`src/capabilities/cognitive.ts:161-164`).
* `world.settled` is observed by RealityTrace; `workbench.settlement.recorded` is **not**
  (`src/observations.ts:28-35`).
* Both apps run 0.8.0 of `@godspeedai/cognate-*` while their manifests pin 0.9.2 and 0.10.0.

## Desired observable behaviour

1. With no optional product installed, Workbench behaves as it does today (journey J1).
2. An Expert or Team can be configured with a provider; its Assignment then runs a child
   `agent.rlm` run through `rlm.invoke`, and the Assignment's own run id does not change (J4).
3. With no provider, `rlm.providers` is empty and nothing claims RLM (J4, J11). The adapter speaks
   ACP, so `session/update` tool calls are recorded as evidence claims and a policy violation can
   cancel the session — never presented as pre-approval.
4. A Specialist cannot exceed its grant or depth; an unauthorised child effect is refused (J3, J5).
5. A killed external invocation becomes `unknown`, opens an `ambiguous-invocation` continuation, and
   blocks retry until the operator resolves it through the existing
   `run.invocation.reconcile` (J11).
6. The built-in Experts and Team can be restored by an operator without touching customised copies
   (J12).
7. A long-horizon task can be escalated to Gauntlet with a frozen reference, and the escalation
   never marks a case settled (J7).
8. A Horizon handoff reaches a Workbench run carrying world and case identity, with an explicit
   verified/asserted flag (J8).
9. Local acceptance and governed settlement are distinguishable in the payload and both reach the
   observation stream (J9).
10. Under SEA-Forge governance a provider runs only in a governed port (contained streaming, a
    reach matrix no wider than the parent's grant, one declared egress channel, fs-diff evidence).
    Where the port cannot enforce that, the call is refused with a typed code — never degraded
    (J13).

## Governing constraints (do not trade away)

* Workbench standalone; Prime optional and never silently installed; Gauntlet optional.
* No second identity, scheduler, work queue, authority or settlement source.
* Recursive execution cannot enlarge its parent's authority.
* Unavailable containment or governance refuses; it never runs uncontained.
* Unknown cost is never zero; interrupted consequential work is reconciled before retry.
* No Prime identifier in `runtime-api`, capability ids, CEP profiles or DomainForge semantics.
* Do not weaken the one-Orchestrator check, authority intersection, review independence, or the

## Accepted design and why

One `rlm.*` contract in Cognate, expressed as an ordinary run: `rlm.invoke` starts exactly one
child `agent.rlm` run *for* an existing Assignment, under a principal derived from the parent, with
a caller-attested grant that is re-checked (non-subset refused) and the caller's own budget
reservation. There is **no built-in provider** — absence is reported as absence. External providers
are adapters reached through a `StdioSpawnPort` the host implements, and Prime 0.10.2 is classified
`unattested` because it cannot be restricted to bridged tools.

Why this and not the alternatives: a Workbench-owned RLM would be duplicated in three products; a
Gauntlet-owned one would make Gauntlet mandatory; a new service would duplicate runs, continuations
and billing. The nine refused alternatives and the fifteen per-decision trade-offs are in
[`concepts_brief.md`](concepts_brief.md) and [`DECISION-LOG.md`](DECISION-LOG.md).

## Repository scope

| Repo | Work | Increment |
|---|---|---|
| `cognate` | `runtime-api` contract + `StdioSpawnPort`; `runtime-bun` RLM component, registry, `agent.rlm` | I1 |
| `cognate` | release tag consumed by the two apps | I0 |
| `cognate` | optional `packages/rlm-prime` (protocol + driver only) | I4 |
| `cognate-workbench` | `runner` on the execution snapshot; `caseContext`/`assertedBy`; continuation wait; `case.escalate`; `settlementScope`; `experts.restore`; `affordance.field` + operations catalogue; spawn-port implementation | I2, I3, I5, I6 |
| `Horizon` | handoff fields + `caseContext`/`assertedBy` on the existing call | I7 |
| `gauntlet` | import the extracted package (or mark its copy deprecated); no behavioural change | I4 |
| `sea-rs`, `cep`, `domainforge`, `sxr` | **no change**. Verification only | — |

## Acceptance journeys

All fourteen are in [`E`](E-acceptance-journeys.md). The workflow must not accept a claim that a
journey passes without its negative test also passing. The four that decide whether the
transformation happened: J1 (standalone), J4 (with and without a provider), J7 (escalation cannot
settle), J11 (unknown is reconciled, never retried blind).

## Required evidence

* Focused tests per increment (admission, terminal stability, cancel idempotency, `unknown` →
  ambiguous continuation, authority/depth/budget refusals, `prime`-token absence, vocabulary parity
  with `agent_runner.rs`).
* Workbench's existing expert/team/E2E suites green and unmodified.
* The `pluginRegistry` suite green (it is not being deleted).
* A recorded refusal test for each of: governance, contained posture, non-local world, win32,
  `unattested` provider.
* One cross-repo fixture for J14 (DomainForge envelope → SEA-Forge validator).
* An explicit note of anything not run, in the same spirit as `G6`.

## Blockers and rules before that workflow starts

**No blockers.** Every decision is closed in [`I-closure-register.md`](I-closure-register.md); both former
open items are decided (package + ACP; I3b in scope). What remains is work, not uncertainty.

Three rules the workflow must keep:

1. I0 must actually install and assert the Cognate version; if 0.10.0 breaks Workbench, stop and
   report rather than work around it.
2. Do not add a `bridge-only` claim for Prime. Its tool list is unrestricted by design; the port is
   the boundary, and ACP observation is a claim, not an approval.
3. If a guard in `I` fires, apply the response written next to it rather than reopening the design.

## Design document references

[`README.md`](README.md) · [`concepts_brief.md`](concepts_brief.md) · [`design.md`](design.md) ·
[`A`](A-actual-state.md) · [`B`](B-gap-and-convergence-ledger.md) ·
[`C`](C-target-architecture.md) · [`D`](D-minimal-contracts.md) · [`E`](E-acceptance-journeys.md) ·
[`F`](F-implementation-sequence.md) · [`G`](G-residual-uncertainty.md) ·
[`DECISION-LOG.md`](DECISION-LOG.md) ·
[`egm-review/`](egm-review/) · [`recon/`](recon/)

  posture refusal to make a test pass.
