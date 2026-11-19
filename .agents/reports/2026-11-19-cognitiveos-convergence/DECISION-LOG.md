# Decision log

Format per decision: **Alternatives → Evidence → Trade-off → Chosen → Falsifier** (what
observation would overturn it). "Falsifier" is the honest exit condition, not a formality.

## 1. Where the RLM contract lives

* **Alternatives**: (a) types in `@godspeedai/cognate-runtime-api` + implementation in
  `runtime-bun`; (b) a new `@godspeedai/cognate-rlm` package; (c) inside Workbench.
* **Evidence**: no RLM contract exists anywhere; `ProcessPort` is buffered
  (`execution/src/types.ts:64-66`); `subagent.start` has no status/result/cancel contract
  (`src/capabilities/subagentStart.ts:43-57`); Workbench already owns too much (A1).
* **Trade-off**: (a) keeps the package count flat and puts the contract where every runtime
  consumer already looks, at the cost of growing a transport-neutral package; (b) is cleaner by
  name but adds a package and a release unit for ~200 lines of types.
* **Chosen**: (a). Public vocabulary is capability ids (`rlm.*`), not service methods, so the
  contract can evolve behind the kernel.
* **Falsifier**: if the contract needs provider-specific fields to be useful, they belong in the
  adapter, not `runtime-api`; if it grows past ~300 lines, split it out. The vocabulary
  (`isolation`, `resultDelivery`, status names) is adopted from Gauntlet's `AgentRunner` so the two
  ports cannot drift; a test enforces parity.

## 2. Which existing ports it reuses

* **Alternatives**: (a) reuse `ProcessPort`; (b) reuse `startRun` + continuations + `subagent`
  correlation; (c) a new execution port.
* **Evidence**: `ProcessPort` cannot stream or accept stdin (`execution/src/types.ts:64-66`);
  `startRun` already gives durable runs, idempotency, cancellation and events
  (`runtime-api/src/types.ts:264-279`); continuations model durable waits
  (`continuations/src/types.ts:4-62`).
* **Trade-off**: reusing runs means the child appears in run listings (desirable) and that
  provider invocation identity is encoded in correlation/idempotency strings (a little string
  discipline).
* **Chosen**: (b) for lifecycle; (a) *only* for one-shot probes (`--version`). A streaming spawn
  is still needed; the design **declares** `StdioSpawnPort` and lets the host application implement
  it, because the existing wrapper depends on six app-owned stores (review M7).
* **Falsifier**: if a provider needs mid-stream interaction Cognate's run events cannot carry,
  the run model is insufficient and the contract must expose a stream handle.

## 3. How the Prime integration is shared

* **Alternatives**: (a) new Rust crate shared by Gauntlet and a Node binding; (b) bridge the Rust
  adapter behind a sidecar; (c) extract Gauntlet's TypeScript host into an optional Cognate
  package; (d) write a new TypeScript adapter.
* **Evidence**: Gauntlet has both a Rust adapter (`runner.rs:639`) and a TS host
  (`real-prime-host.ts:1-40`); no FFI or WASM surface exists ‡; `fabric serve` is bounded and does
  not freeze/verify/settle ‡.
* **Trade-off**: (a) needs a Node↔Rust binding that does not exist; (b) adds a process and a
  protocol for the same result; (c) creates a cross-repo TS dependency and release ordering;
  (d) is a second protocol.
* **Chosen**: (c), scoped to `prime-host` + `real-prime-host` + the `terminal-session` validation
  functions; Gauntlet's policy, topology and envelope modules stay put. (d) is refused explicitly:
  it is the duplication the assignment forbids.
* **Falsifier**: if Prime's TS host depends on OpenTUI or Gauntlet state that cannot be separated
  cleanly, fall back to (b) — a sidecar speaking the same NDJSON the host already speaks.

## 4. Provider discovery and optional installation

* **Alternatives**: (a) reuse `agent-discovery` probes; (b) an operator-configured provider list
  validated at boot; (c) auto-detect and auto-install.
* **Evidence**: `agent-discovery` exports `findExecutables`, `discoverAgents`, `probeVersion` ‡;
  no provider registry exists; the owner forbids silent installation.
* **Trade-off**: (a) gives detection for free but no capability contract; (b) is explicit and
  auditable but needs operator action; (c) violates the constraint.
* **Chosen**: (b) for authority, (a) for detecting the binary path and version. Installation is a
  printed instruction, never an action.
* **Falsifier**: if operators cannot discover what to install from the UI, add a read-only
  "available but not enabled" list sourced from (a).

## 5. Experts/Teams use RLM without duplicate models

* **Alternatives**: (a) a runner field on the Assignment execution snapshot (`none` or a provider
  id); (b) a new "RLM Expert" type; (c) Orchestrator-only RLM.
* **Evidence**: `ExpertExecution` is frozen per Assignment (`src/experts/types.ts:58-68`);
  authority intersection and depth exist (`service.ts:195-199`, `:190`); subagent is removed from
  Experts (`agentCoding.ts:374-377`).
* **Trade-off**: (a) keeps one identity model and one lineage but adds a nullable field to a
  frozen snapshot; (b) creates a second Expert kind and review path; (c) denies Specialists the
  capability they most need.
* **Chosen**: (a).
* **Falsifier**: if provider selection must change mid-assignment, the frozen snapshot is the
  wrong place and selection belongs in Team config with a recorded override event.


## 6. Standalone Expert with no provider

* **Alternatives**: (a) the existing in-process Expert loop (`runner:none`), advertised with
  `contextDecomposition:false` when a provider is compared;
  (b) refuse Expert work without a provider; (c) silently use Prime if installed.
* **Evidence**: the Expert loop and its tools already work (`agentCoding.ts:781-838`); Prime is
  optional by owner constraint.
* **Trade-off**: (a) is truthful but weaker than the marketing term "RLM"; (b) breaks the
  standalone product; (c) violates "no silent dependency".
* **Chosen**: (a), and the word RLM is not used in user-facing copy for it. There is no `native`
  provider: `runner:none` means the ordinary loop, and `rlm.providers` returns an empty list.
* **Falsifier**: if the ordinary loop cannot satisfy a required RLM behaviour, that is a stated
  limitation, not a reason to rename the capability.

## 7. Protecting the default Experts and Team

* **Alternatives**: (a) operator-only `experts.restore`; (b) immutability; (c) reuse safe mode;
  (d) a separate baseline store.
* **Evidence**: `seedExperts` short-circuits on `seed:v1` (`state.ts:57-59`); safe mode is
  unmerged and extension-scoped (`safeMode.ts:1-7`); the versioned mutation path serializes
  writes.
* **Trade-off**: (a) leaves deletion possible but recoverable; (b) blocks legitimate
  customization; (c) solves a different problem; (d) is a second source of truth.
* **Chosen**: (a), plus per-id idempotent boot seeding. `includeArchived:true` is the only path
  that reverses an operator's archive, and it is never automatic. Safe mode stays for extensions
  only.
* **Falsifier**: if operators need point-in-time snapshots of their custom teams, that is
  versioning, not baseline restore, and is a separate decision.

## 8. Where the Workbench→Gauntlet boundary belongs

* **Alternatives**: (a) one `case.escalate` capability that spawns `gauntlet run`; (b) an RPC
  client to a Gauntlet daemon; (c) `fabric serve`; (d) route all Workbench work through Gauntlet.
* **Evidence**: `gauntlet run` is the controller (`run_cmd.rs:1-11`); `fabric serve` is bounded
  and does not freeze/verify/settle ‡; the spawned-process pattern already exists in T3/T4a.
* **Trade-off**: (a) ties Workbench to Gauntlet's CLI and reference format; (b) needs a daemon
  and a protocol; (c) cannot satisfy frozen verification; (d) violates "optional".
* **Chosen**: (a). The gate is an explicit criteria list, recorded with the values that fired.
* **Falsifier**: if the reference format changes frequently, the projector's pin becomes a
  maintenance tax and a versioned small RPC would be cheaper.

## 9. How Gauntlet receives a frozen Reference

* **Alternatives**: (a) project Workbench's frozen criterion into `.wayfinder/reference.yaml` +
  `answer-key.md` and let Gauntlet freeze its own lock; (b) send a CEP envelope and have Gauntlet
  parse it; (c) pass criteria on the command line.
* **Evidence**: Gauntlet locks the reference itself and re-verifies it before settlement
  (`docs/source-map.md:13`, `ReferenceFreezer` ‡); builders cannot write the rubric; CEP has no
  Gauntlet consumer.
* **Trade-off**: (a) is a file-format projection owned by another repo but preserves Gauntlet's
  frozen-ruler invariant; (b) needs a new Gauntlet CEP contract; (c) is unauditable.
* **Chosen**: (a).
* **Falsifier**: if Gauntlet later consumes CEP natively, replace the projector and delete this
  decision.

## 10. How Horizon invokes RLM and delegates

* **Alternatives**: (a) keep `expert.converse`, add `caseContext`/`assertedBy`; (b) a new
  `case.converse` capability; (c) Horizon embeds an RLM client.
* **Evidence**: Horizon already calls `expert.converse` (`workshop.ts:174`); WP-23 already binds
  runs to case assignments ‡; Horizon owns no execution (`src/runtime/create.ts` ‡).
* **Trade-off**: (a) keeps one seam but overloads a capability name; (b) adds a second entry
  point with the same semantics; (c) duplicates execution in the wrong product.
* **Chosen**: (a). `assertedBy` is recorded as a claim and is never a refusal, because Horizon
  always presents the operator's own session (review M4); attribution comes from the session actor.
* **Falsifier**: if non-Expert case work needs the same seam, a `case.converse` that delegates to
  the same handler is justified; renaming alone is not.


## 11. Standalone acceptance vs governed settlement

* **Alternatives**: (a) keep both machines, add `settlementScope`/`receiptRef`; (b) one machine
  with a mode flag; (c) require SEA-Forge always.
* **Evidence**: two event families exist with different producers (`cognitive.ts:267`;
  `orchestrator.ts:160-164`); both feed the SXR observation stream (`observations.ts:28-37`);
  SEA-Forge owns `status`/`final` (`orchestrator.ts:9-10`).
* **Trade-off**: (a) is a small payload change; (b) collapses two authorities into one mechanism
  and invites a local claim of governed settlement; (c) breaks standalone use.
* **Chosen**: (a). Absence of the field reads as `local`.
* **Falsifier**: if a third settlement authority appears, the field becomes a three-value enum and
  each value must name its receipt issuer.

## 12. Containing and authorizing recursive tool execution

* **Alternatives**: (a) process containment + bridge-only tools + authority intersection, with
  refusal where unenforceable; (b) prompt-level restrictions; (c) no containment, audit only.
* **Evidence**: bubblewrap is the only in-tree enforcement (`executionPosture.ts:1-35`);
  `run_code` is not a sandbox (`tools.ts:208-227`); T3/T4a already express a per-kind matrix
  (`execSpawn.ts:50-70`); the owner forbids prompt-only enforcement.
* **Trade-off**: (a) refuses real use cases on macOS/Windows; (b) is dishonest; (c) is unsafe.
* **Chosen**: (a). Under SEA-Forge governance the answer is therefore "refused", not "admitted with
  an attestation", because governance mandates the posture that forbids host spawns.
* **Falsifier**: if a platform gains a real sandbox (seatbelt, job-object limits), extend the
  matrix; do not weaken it.

## 13. Which CEP profiles and versions cross each interface

* **Alternatives**: (a) reuse the seven existing `godspeed.*` 1.0.0 profiles and add none;
  (b) define an eighth "execution" profile; (c) carry no profile and rely on `world_ref` strings.
* **Evidence**: seven profiles at 1.0.0 exist (plus one `world-transition` record schema); every
  profile except the invalid-world snapshot
  branch carries `scope.world_ref`; SEA-Forge validates `evidence_packet` with 7 corroborated
  fields ‡; no consumer exists for a new profile.
* **Trade-off**: (a) constrains the handoff to fields the profiles already define; (b) is a
  second semantic source of truth; (c) loses validation.
* **Chosen**: (a). Mapping: `semantic_snapshot` for representation,
  `authority_request/decision` for authorization, `execution_trace`/`evidence_packet` for
  observed evidence, `settlement_packet` for settlement, `context_bundle` for the Horizon
  handoff.
* **Falsifier**: if a required field has no home in any profile, document the gap and raise a CEP
  change proposal — do not smuggle it into a free-form extension.

## 14. Surviving restart, cancellation and ambiguity

* **Alternatives**: (a) record-before-invoke + `unknown` status + the existing
  `run.invocation.reconcile` continuation;
  (b) retry on restart; (c) treat a lost transport as failure.
* **Evidence**: Gauntlet prescribes re-probing result contracts, never assuming
  (`agent_runner.rs:46-48`); Workbench has `reconciliation-required` (`types.ts:11`) and durable
  billing reservations (`billing.ts:1-9`).
* **Trade-off**: (a) requires operator action in the indeterminate case; (b) risks a double
  effect; (c) understates the truth.
* **Chosen**: (a).
* **Falsifier**: if every provider can offer a trustworthy result contract, automatic
  reconciliation becomes possible for those providers — still never a blind retry.

## 15. Consolidation and deletion

* **Alternatives**: (a) delete/consolidate as listed in `B4`; (b) keep everything and add
  adapters; (c) delete only what breaks.
* **Evidence**: `pluginRegistry` is self-declared legacy ‡; two Prime hosts exist; two subagent
  starters exist; `seed:v1` is a permanent lock; installs are stale.
* **Trade-off**: (a) risks removing something a hidden consumer needs; (b) guarantees permanent
  duplication; (c) leaves the debt in place.
* **Chosen**: (a), gated on an import inventory for each deletion.
* **Falsifier**: if the inventory finds a live consumer, the item moves from DELETE to EXTEND and
  the consolidation is re-scoped — not abandoned.
