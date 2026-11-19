# E — End-to-end acceptance journeys

Each journey states an observable pass/fail and at least one negative test. "Pass" means the
named assertion can be executed by a separate agent without reading this document.

## J1. Workbench alone (no optional GodSpeed services)

**Given** a clean install of Cognate Workbench with no Horizon, Gauntlet, SEA-Forge,
DomainForge, RealityTrace or Prime binary on `PATH`,
**When** the operator opens a coding session, reads and edits a file, and runs a test,
**Then** the session completes with a receipt and no error mentions any optional service.

* **Pass**: the run's event stream contains no `unavailable` provider entry that the operator
  did not ask for; `rlm.providers` returns an empty array; no UI element prompts for a provider
  choice.
* **Negative**: with every optional binary removed, `rlm.providers` must still answer; a run
  must not fail because a provider is missing. Assert the refusal text is absent.
* **Trace**: `src/server.ts` boot, `src/experts/bridge.ts:13-104`.

## J2. Default five-Expert Team with exactly one Orchestrator

**Given** a fresh state, **When** the operator opens `Cognate Team`, **Then** five Experts exist
(`product-manager`, `builder`, `critic`, `verifier`, `reviewer`) and exactly one is an
Orchestrator.

* **Pass**: `assertTeamInvariant` passes; removing the only Orchestrator is refused; activating
  a Team with zero or two is refused.
* **Negative**: a direct `updateSharedState` attempt to write two Orchestrators is refused by
  the command path (not merely by the UI).
* **Trace**: `src/experts/state.ts:39-47,49-76`.

## J3. Orchestrator delegation preserves identity, authority, budget, lineage

**Given** an active Team, **When** the Product Manager delegates "add input validation" to the
Builder, **Then** a child Assignment exists with `expertId:"builder"`, `parentId` = the PM
assignment, `depth:1`, an authority set that is a subset of the PM's, and a budget reservation.

* **Pass**: the child's `authorityTools` ∩ PM's `authorityTools` equals the child's set; the
  child cannot call `team_delegate`; the parent suspends on `expert-assignment:<id>` and resumes
  once; the handoff appears in the PM's conversation with the child's evidence.
* **Negative**: a Specialist calling `team.delegate` is refused; a child requesting `bash` when
  the PM lacks it is refused (not silently narrowed).
* **Trace**: `src/experts/service.ts:181-216,424-430`; `src/agents/agentCoding.ts:806-816`.

## J4. RLM invocation with and without the optional provider

**Given** a Team with `runner:none`, **When** the Builder receives an assignment, **Then** the
ordinary Expert loop runs, `rlm.providers` returns an empty array, and no `rlm.*` invocation
exists. **Given** the same Team with `runner:{providerId}`, **When** the assignment runs, **Then**
exactly one child run of `agent.rlm` exists for that invocation, correlated
`rlm:<parentRun>:<invocationId>`, and the parent Assignment's `runId` is unchanged.

* **Pass**: both runs produce a handoff with evidence; only the provider run has an
  `invocationId`; the provider's capability facts (including `isolation`) are reported, not
  inferred; the parent run id remains the Assignment's run id.
* **Negative**: an assignment whose provider is unavailable fails with
  `rlm-provider-unavailable` and creates no child run; a top-level `rlm.invoke` is refused with
  `rlm-top-level-invocation`; under SEA-Forge governance the same call is refused with
  `rlm-posture-refused` **before** any process starts.
* **Trace**: `D1`, `D4.1`, `D4.9`, `src/experts/types.ts:69-83`,
  `src/experts/service.ts:119-122`, `agent_runner.rs:162-229` for the adopted vocabulary.

## J5. Unauthorized child execution is refused

**Given** a read-only Team assignment, **When** the provider-internal child attempts `edit`,
**Then** the effect is refused and recorded; the assignment ends `failed` or
`reconciliation-required`, never `completed` with a changed file.

* **Pass**: the workspace file is unchanged; the refusal appears in evidence with the attempted
  tool name; the provider is either `bridge-only` or refused earlier (Prime 0.10.2 is
  `unattested`, so it is refused for any narrowed-authority run).
* **Negative**: an adapter declaring `toolPolicy:"unattested"` is refused at `rlm.invoke` with
  `rlm-containment-unavailable` before any process starts.
* **Trace**: `G6`, `G7`, `src/capabilities/expertEffect.ts:1-45`.

## J6. Small task completes without Gauntlet

**Given** an ordinary one-file fix, **When** the operator asks Workbench to do it, **Then** no
escalation record exists and the task completes locally.

* **Pass**: no `case.escalate` event; no Gauntlet process was spawned (assert via the spawn

## J8. Horizon case handoff preserving CEP context

**Given** a Horizon case at a pinned `world_ref` with a plan item and a frozen criterion,
**When** Horizon hands the item to the case keeper and calls `expert.converse` with
`caseContext`,
**Then** the resulting Workbench run carries `caseId`, `planItemId`, `worldRef`,
`semanticRef`, `episodeRef` (when evaluated), `worldCorroboration`, and the recorded `assertedBy`.

* **Pass**: a Workbench projection joins the run to the case and plan item; the evidence
  contains both the Workbench Assignment id and the Cognate `caseAssignmentId`; the case keeper
  shows the evidence refs; `worldCorroboration` is `verified` only when a registry check ran.
* **Negative**: `caseContext.kind:"evaluated"` without `episodeRef` is refused with a typed code
  (the published `CaseContext` requires it for evaluated work, `types.ts:28-30`).
  `assertedBy` is never a refusal (it is a claim); the test asserts it is recorded verbatim and
  that attribution still comes from the session actor.
* **Trace**: `G9`, `G10`, `G11`,
  `cognate-workbench@feat/wp23-assignment-binding (668f686):src/assignments/assignmentBinding.ts:38-42`.

## J9. Local candidate acceptance versus externally governed settlement

**Given** a local case with a verified candidate, **When** the operator promotes it, **Then**
`world.settled` records `settlementScope:"local"` and `receiptRef:null`. **Given** SEA-Forge
governance configured, **When** a consequential coding run ends, **Then**
`workbench.settlement.recorded` carries SEA-Forge's status, `final`, and a non-null receipt.

* **Pass**: the two events are distinguishable by payload alone; **both** appear in the SXR
  observation stream with their scope, and the governed one carries a receipt; the UI never labels
  a local promotion as settled. (Revision 1 of this journey could not pass: the governed family was
  not observed at all, `src/observations.ts:28-35`.)
* **Negative**: `world.settle` invoked from inside a run is refused; a governed settlement with no
  receipt is impossible (assert the field); the extra SXR ingest per settled operation is counted,
  not ignored.
* **Trace**: `G12`, `src/capabilities/cognitive.ts:253-269`,
  `src/settlement/orchestrator.ts:9-10,153-164`, `src/observations.ts:28-37`.

## J10. Independent verification without builder-private context

**Given** a Builder assignment and a Reviewer assignment with `reviews:<builderAssignmentId>`,
**When** the Reviewer runs, **Then** its prompt/context contains the builder's *outputs,
artifacts and evidence* and not the builder's private reasoning or memory.

* **Pass**: the Reviewer's execution context has no Builder private-conversation messages;
  `team.review` with `verdict:"accept"` fails unless independent observable checks exist.
* **Negative**: a Reviewer attempting to accept with only model assertions is refused with
  "Acceptance review requires an independent Expert's observable checks".
* **Trace**: `src/experts/service.ts:123` (context composition), `:316-325`.

## J11. Restart, cancellation, replay and ambiguous effects

**Given** an external provider invocation in flight, **When** the host is killed, **Then** on
boot the assignment is `reconciliation-required`, `rlm.inspect` says `unknown`, and the invocation
appears as an `ambiguous-invocation` continuation (the existing mechanism, not a new one); a retry
is refused until `run.invocation.reconcile` records an outcome.

* **Pass**: no second process is spawned before reconciliation; after a `mark-completed`
  reconciliation the assignment resumes with the original evidence; after a `reexecute` decision the
  retry uses a new `attempt` and therefore a new idempotency key.
* **Negative**: `rlm.cancel` on a terminal invocation returns `already-terminal` and performs no
  effect; replaying a completed `invocationId` returns the same run id.
* **Trace**: `G18`, `agent_runner.rs:46-48`, `src/experts/bridge.ts:93-103`,
  `src/experts/billing.ts:1-9`.

## J12. Installation, suspension, revocation, safe-mode recovery, missing provider

**Given** a provider installed and authorized, **When** the operator revokes it, **Then**
`rlm.providers` reports `unavailable` and in-flight work is cancelled with a recorded reason.
**Given** a corrupted extension bundle on a build that includes safe mode, **When** the app
boots twice, **Then** extension fragments do not mount and only the operator can clear it.

* **Pass**: revocation is durable across restart; safe mode never re-seeds Experts; the default
  Team is restorable with `experts.restore` while a renamed custom Expert is untouched.
* **Negative**: a non-operator calling `experts.restore` is refused; safe mode's clear from a
  non-operator is recorded as `unauthorized-clear`.
* **Trace**: `G13`, `cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/workbench/safeMode.ts:1-7,126-135`.

## J13. Port-governed provider, and refusal where the port cannot enforce

**Given** a provider declared with `portGovernance` and the contained streaming world registered,
**When** a read-only assignment invokes it, **Then** the child runs with a read-only workspace bind,
no network except the declared egress socket, a process group, and a recorded fs diff.
**Given** no contained streaming world, **When** the same assignment invokes it under SEA-Forge
governance, **Then** the call is refused with `rlm-port-unavailable` before any process starts.

* **Pass**: a write attempt inside the read-only port fails; a direct network connect inside fails;
  the bound egress socket answers; killing the parent removes the group; the fs diff is attached as
  evidence; the recorded `reach` matches the declaration; ACP `tool_call` events are recorded as
  claims and a disallowed `kind` triggers `session/cancel`.
* **Negative**: an ACP `tool_call` that contradicts the fs diff records `evidence-discrepancy` and
  blocks acceptance; a provider declaring `network:"proxied"` with no proxy is refused
  `rlm-egress-uncontrolled`; a provider declaring `network:"open"` is refused under governance; a
  provider whose declared reach is wider than the parent's grant is refused
  `rlm-containment-unavailable`; `unattested` + no `portGovernance` is refused
  `rlm-posture-refused`.
* **Trace**: `D8`, `F` I3b,
  `cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/capabilities/execSpawn.ts:50-70,195-197`,
  `src/executionPosture.ts:1-35`, `src/server.ts:510-511`.

## J14. Real producer/consumer CEP compatibility

**Given** DomainForge emits `domainforge envelope --emit cep --world-name <n>` for a world,
**When** the envelope reaches SEA-Forge's evidence acceptance and RealityTrace corroboration,
**Then** acceptance succeeds with zero translation and the recorded `world_ref` matches.

* **Pass**: `scope.world_ref` is byte-identical at producer, authority request, evidence packet
  and settlement; the seven `CORROBORATED_FIELDS` match RealityTrace's record.
* **Negative**: an envelope with an unknown profile id, a mismatched `world_ref`, or a missing
  corroborated field is refused, not downgraded. A Workbench local `worldHash` presented in a
  CEP `world_ref` position is refused.
* **Trace**: `A4`, `sea-rs` `cep.rs:1356,1582-1688` ‡, `recon/cep-and-domainforge-contracts.md`.

  ledger being empty); the completion path is J1's path.
* **Negative**: the escalation gate must not fire on duration alone — a 40-minute read-only
  investigation produces no escalation.
* **Trace**: `G8`, `C4` gate list.

## J7. Long-horizon work delegated through Gauntlet

**Given** a frozen criterion, an explicit gate reason, an operator-named `runners.default`, and a
posture that admits a host spawn, **When** the operator escalates, **Then** a Gauntlet project
exists, `gauntlet run --config <path>` executes under a recorded spawn intent, and its exit code
plus `terminal_state` are recorded against the escalation.

* **Pass**: the reference digest is recorded and matches; the escalation record contains `version`,
  `argvDigest`, `referenceDigest`, `exitCode`, `reportRef` and `gauntletTerminal` (one of the six
  Gauntlet states); no `world.settled` and no `receiptRef` were produced by the escalation.
* **Negative**: editing the reference after freeze makes Gauntlet refuse; Workbench surfaces that
  refusal and does not re-freeze. An unset `runners.default` is refused by Workbench and never
  filled in with `prime-agent` on the operator's behalf. Under governance the escalation is
  refused before spawn.
* **Trace**: `G8`, `gauntlet` `run_cmd.rs:1-11`, `docs/source-map.md:13`.
