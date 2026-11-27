# Adversarial review (EGM)

Applied to **revision 2**. Each attack states the strongest form, what the design actually does,
the evidence, and a verdict. "Open" means the design does not close it and does not pretend to.

## 1. Duplication attack — does this recreate what already exists?

**Attack.** `rlm.invoke` becomes a fourth way to start child work beside `subagent.start`,
`team.delegate` and Gauntlet's invocation graph.

**Response.** Revision 1 did not close this: `rlm.invoke` looked like a general delegation API.
Revision 2 requires `parent.runId === ctx.runId`, so a top-level invocation is impossible
(`D4.1`, `rlm-top-level-invocation`), and it does not create an Assignment — it runs one.
`team.delegate` still creates Assignments; `subagent.start` still starts unassigned coding
children; Gauntlet still owns its own graph for its own runs.

**Residual.** Three entry points still exist. They differ in owner and semantics, and a test
asserts `rlm.invoke` cannot create an Assignment. A future `rlm.delegate` would reopen the attack;
the guard is a test, not a type.

**Verdict: partly closed, residual documented.**

## 2. Abstraction attack — could a package, interface or controller be removed?

**Attack.** Delete `cognate-rlm-prime`; delete the `rlm.*` capabilities; let `agent.expert` call
the provider directly.

**Response.** Deleting the adapter costs J4-with-provider, J5, J11 and J13-provider. Deleting
`rlm.*` moves discovery, status, cancellation, unknown-outcome and recovery into Workbench, where
no other consumer can reuse them and a protocol sits in the product layer. `agent.rlm` survives
removal only if some other agent can drive a provider process; none can.

**Verdict: survives.** Each `NEW` item is load-bearing for a named journey.

## 3. Standalone attack — does a missing optional product break unrelated behaviour?

**Attack.** Boot, provider probing or escalation couples Workbench to an external binary.

**Response.** `rlm.providers` returns an empty array when nothing is installed; probing is lazy and
bounded, never at boot; `case.escalate` is an explicit operator action; the adapter is an optional
dependency.

**Residual.** I0 bumps Cognate packages, and a bad bump breaks everything. That is why it is a
separate reversible increment.

**Verdict: closed for feature absence; the version bump is isolated.**

## 4. Authority attack — can a child exceed its grant?

**Attack.** A provider process inherits host powers; its internal subagents inherit them too; a
caller can declare a generous `parent.authorityTools`.

**Response.** Three claims, three honest answers.
* Provider-internal children: **not enforceable by Cognate**. Bounded by the process tool policy
  and containment; `toolPolicy:"unattested"` is refused (`D5`). Enforcement is the adapter's.
* Caller-declared grant: the seam checks only that the request is a subset of what the caller
  declared; the real grant is Workbench's `authorizeCapabilityOp`. A caller that bypasses Workbench
  is **out of scope** and now named in the NFRs.
* Governance: external providers are refused outright.

**Residual.** The `uncontained-local` self-approval hazard predates this design and is owner
decision O2.

**Verdict: partly closed; gaps named, not hidden.**

## 5. Identity attack — wrong actor, case or revision?

**Attack.** Provider children have no Workbench identity; `assertedBy` is a free string;
`worldRef` is caller-asserted; three records are called "Assignment".

**Response.** Provider-internal children are opaque by decision, attributed to the Specialist's
Assignment and labelled `provider-internal` (`C3`). `assertedBy` is recorded, never trusted.
`worldCorroboration` separates a registry check from a claim. Workbench `Assignment` versus Cognate
`CaseAssignment` is consolidated by naming with a type-level test (`G5`).

**Residual.** A bearer session key remains a bearer key: attribution improves, authentication does
not.

**Verdict: closed for the named paths; one documented limitation.**

## 6. Settlement attack — can success be mistaken for settlement?

**Attack.** A completed run, a green test, or Gauntlet's `Settled` terminal state is presented as
operational settlement.

**Response.** `completed` is not acceptance: the Assignment still ends in
`reconciliation-required` (`src/experts/bridge.ts:86`). `case.escalate` returns
`gauntletTerminal` and never writes `world.settled` (`D3`). Local acceptance is
`settlementScope:"local"` with `receiptRef:null`.

**Residual found during review:** Gauntlet's own terminal state is literally named `Settled`. The
UI must say "Gauntlet settled its own run", never "the case is settled". A schema cannot enforce
copy.

**Verdict: closed at contract level; one UI copy rule required.**


## 7. Envelope attack — can a handoff drop authority, provenance or uncertainty?

**Attack.** Horizon sends a name and a task; Workbench accepts anything.

**Response.** Revision 1's `contextDigest` over *ids* was weak: referenced artifacts could change
under a stable digest. Revision 2 hashes resolved content and re-checks at child start (`D4.10`).
The handoff carries `world_ref`, `semantic_refs`, `acceptance`, `uncertainties` and
`cep:{profile,version}` (`G10`); an evaluated case without `worldRef` is refused; unknown profiles
are refused rather than translated.

**Residual.** A required field may have no home in the seven profiles. That is recorded as a gap,
and a CEP change proposal is the correct response (`DECISION 13`).

**Verdict: closed for supported fields; gap named.**

## 8. Recovery attack — can a retry repeat an uncertain effect?

**Attack.** Crash after an external call, then retry → double effect.

**Response.** Intent is recorded before invoke; lost transport yields `unknown`, never `failed`
(`D4.7`); retry is refused with `rlm-reconcile-required` until `rlm.reconcile` records an outcome
with evidence; a failed reconciliation creates a new attempt with a new idempotency key. This is
Gauntlet's rule, adopted (`agent_runner.rs:46-48`).

**Residual.** A caller can dodge it by choosing a fresh `invocationId` for the same work. The guard
is that a second non-terminal invocation for the same `(assignmentId, attempt)` is refused — a
check, not a proof.

**Verdict: closed for the named path; the fresh-id bypass is an explicit test requirement.**

## 9. Verification attack — can the builder influence its own acceptance?

**Attack.** `case.escalate` projects criteria; if it reads the builder's Assignment, the builder
writes its own rubric.

**Response.** The projection reads only the `criterion.frozen` event and refuses if absent (`I6`).
Acceptance is set by the Orchestrator at delegation and frozen in the snapshot
(`src/experts/service.ts:181-216`); Reviewer acceptance requires independently observed checks
(`:316-325`); a Reviewer does not inherit the Builder's private context (`J10`).

**Residual.** A solo Expert with a generic default acceptance can propose work it then accepts,
because there is no second party. The human settlement gate is the only protection there — existing
product behaviour, not a regression.

**Verdict: closed for Team work; solo relies on the human gate, stated.**

## 10. Complexity attack — is this more complicated than adapting what exists?

**Attack.** Eight increments, a new agent, a new port, a new package, five capability ids.

**Response.** Implementing RLM in Workbench is fewer files in one repo and three duplicated
mechanisms across products. Doing nothing fails requirement A. The count that matters: four
mechanisms stop having two implementations (Prime protocol, stdio spawn, child-run lifecycle,
settlement labelling). I6 and I7 are separable and can be dropped without touching the core.

**Residual.** I0–I4 is still a multi-repo change. No smaller route satisfies both
"provider-independent" and "Workbench standalone".

**Verdict: survives, narrowly.**

## 11. Capability attack — claiming RLM after one tool call?

**Attack.** "Cognate provides a first-class RLM capability" is false until a provider actually
performs recursive decomposition.

**Response.** The design refuses that claim. There is no `native` provider, so Cognate ships **no**
RLM capability until I4 installs one; `rlm.providers` is empty, and I1's acceptance is a test
double. Capability facts report `contextDecomposition:false` unless the provider does it. Product
copy must not use the term before I4.

**Verdict: closed by honesty, at the cost of a weaker marketing claim. Correct trade.**

## 12. Evolution attack — does replacing the provider force changes in Cognate or consumers?

**Attack.** A future runner replaces Prime; provider names leak into semantics.

**Response.** Provider ids are configuration; capability facts are declarative; the `prime`-token
test forbids the name in `runtime-api` and capability ids; `resultDelivery` names the mechanism, so
consumers never branch on an id.

**Residual.** A provider that cannot deliver an artifact at all needs `resultDelivery:"return"`,
with the result in the run value rather than an artifact.

**Verdict: closed.**

## Summary

| # | Attack | Verdict |
|---|---|---|
| 1 | Duplication | partly closed (three entry points, guarded by test) |
| 2 | Abstraction | survives |
| 3 | Standalone | closed (version bump isolated) |
| 4 | Authority | partly closed (provider-internal bounds are the adapter's) |
| 5 | Identity | closed, one documented limitation |
| 6 | Settlement | closed at contract, UI copy rule required |
| 7 | Envelope | closed for supported fields, gap named |
| 8 | Recovery | closed, fresh-id bypass is a test |
| 9 | Verification | closed for Teams, human gate for solo |
| 10 | Complexity | survives narrowly |
| 11 | Capability | closed by refusing the claim until I4 |
| 12 | Evolution | closed |

**What the review changed, not merely noted:** the `native` provider was deleted; the spawn
"extraction" became a declared port; the kernel-authority claim was withdrawn; the governance
admission claim was reversed to a refusal; `requestedBy` became `assertedBy`; the
settlement-observation premise was corrected and the governed family added to the stream; the
invented Gauntlet vocabulary was replaced with the real one; `experts.restore` semantics were made
non-contradictory; branch-qualified citations replaced worktree paths.
