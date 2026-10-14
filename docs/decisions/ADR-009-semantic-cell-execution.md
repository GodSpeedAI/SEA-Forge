# ADR-009: Semantic Cell execution — DomainForge + OPA/Rego + OpenShell + SEA-Forge

- Status: Accepted. Implementation in progress. The OPA engine, the OpenShell adapter and the governed case-runner episode (`cell_episode.rs`) exist, with offline tests. The live MicroVM proof is BLOCKED on the development host (no `/dev/kvm`) and UNPROVEN (see Known limitations).
- Date: 2026-10-12
- Scope: `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-authority`, new `sea-forge-changeset`; DomainForge `projection::rego` and `projection::openshell`

## Context

Agents need an execution environment that is defined by the semantic model, authorized by SEA-Forge, isolated by something stronger than the local jail, and whose file changes reach the authoritative repository only through settlement. The pieces exist separately. This record fixes how they compose and what is deliberately not built.

## Ownership (unchanged)

| System | Owns |
|---|---|
| DomainForge | `.sea` semantics, Cell IR, policy and environment projections, model identity |
| OPA/Rego | Evaluating DomainForge-derived policy |
| SEA-Forge | Authority decisions, grants, evidence, settlement, ledger |
| OpenShell | Sandbox lifecycle, filesystem/process containment, network enforcement, credential mediation |
| Changeset layer | Private workspace, explicit changeset, compare-and-swap promotion |

## Pinned versions

| Component | Pin | Why |
|---|---|---|
| DomainForge | `domainforge-core` 0.19.0 line (`origin/main` 21924d4) + the two new projections | SEA-Forge already pins `=0.19.0` |
| OPA | 1.21.1 | Reference evaluator for the Rego projection |
| OpenShell | v0.1.3 (`e1f3c82c`), CLI + gateway + prover + `openshell-driver-vm` | Newest stable tag; contains the fix for upstream issue #2358 (merge `cb6e88ac`) |
| Contracts | `domainforge.openshell.lock/v1`, `domainforge.openshell.sandbox/v1`, `domainforge.openshell.mapping/v1`, `domainforge.rego.decision/v1`; projection `domainforge-rego/v1` | Schema identities checked at load and at decision validation |

## Decisions

1. **Rego is a DomainForge projection, verified against DomainForge's own evaluator.** `domainforge project --format rego` lowers graph policies into a narrow policy IR and renders Rego v1. Anything outside the IR is a typed `REGO001 unrepresentable` error. Per-policy True/False/Unknown/error results are compared with `Policy::evaluate_with_mode` by running real OPA on generated inputs. The allow/deny/escalate aggregate is the projection's own contract and is labelled `adapter`, not `isomorphic`.

2. **The evaluator's actual behaviour is the reference, including where it is not Kleene.** A source-level `and`/`or` over two literal-reducing operands absorbs NULL (`false and NULL` is NULL), while unreduced trees use Kleene logic, and `exists_unique` never counts a condition containing `not`. The Rego library reproduces this rather than a cleaner semantics the evaluator does not implement. It is recorded as observed debt in DomainForge, not changed here.

3. **OpenShell policy is a projection of the existing Cell IR.** `domainforge project --format openshell` reuses `CellIr::build`; there is no second parser. It emits the policy, the sandbox creation parameters OpenShell keeps outside the policy (CPU, memory, exec timeout), a boundary file, and a mapping report naming the source element of every rule. Requirements OpenShell cannot enforce fail with a typed error or are listed as residual obligations the executor must discharge; none is dropped.

4. **Generated Rego is not installed into OpenShell.** OpenShell's embedded Rego (`package openshell.sandbox`) is baked in; file overrides are a developer mode refused under gateway-managed activation. The two Rego uses stay separate: DomainForge Rego informs SEA-Forge authority; OpenShell enforces its own policy schema.

5. **SEA-Forge drives OpenShell through its CLI, not the Rust SDK.** The SDK is async (tokio/tonic), git-only, and needs Rust 1.94; kernel crates are synchronous, `deny.toml` forbids git sources, and the toolchain is 1.92. The SDK also lacks policy attachment on create, file transfer, and effective-policy reads. A small synchronous adapter over `openshell` (the same surface upstream's own e2e tests use) covers all of them and adds no dependency. It is a new `SandboxClass::OpenShell` behind the existing `ExecutionSandbox` trait.

6. **MicroVMs are the primary backend; no container daemon is required.** The gateway runs as a native process with `compute_driver = "vm"` (libkrun). Linux needs KVM; the backend probes for it and fails explicitly, including on WSL2 without nested virtualization. Docker and Podman remain optional gateway drivers and are never selected implicitly.

7. **No silent downgrade.** A grant for `openshell` that cannot be honoured (no gateway, no KVM, version mismatch, prover failure, unexpected effective policy) is an error. Nothing falls back to `local` or `jail`.

8. **The effective policy is what gets verified and recorded.** After creation the adapter reads the policy the gateway composed (including `_provider_*` rules), checks it against the Cell boundary with `openshell-prover`, and records its hash, revision and the prover's result and coverage as evidence. A prover result of `unsupported` or `inconclusive` is a failure. The prover is not claimed to prove runtime security.

9. **Workspaces are transactional through Git, with no new store.** The sandbox receives an export of the base tree, never the repository. Results are staged in a separate repository, turned into an explicit changeset, validated, and promoted with `git update-ref <ref> <new> <expected-old>`. A moved ref is a conflict, not a merge. Rejected changesets are kept as evidence and never touch the authoritative repository. The compare-and-swap is performed by the trusted promotion path, never by a guest command.

10. **External effects stay outside the transaction.** A network allowance is not authority for a state-changing call: flows project to read-only HTTP unless the Cell declares otherwise, and anything that must change external state needs its own pre-execution authorization.

11. **Guest execution is a separate typed action.** A command that runs inside a sandbox is `ExecuteSandboxCommand {argv, cwd, cell_id, model_hash, policy_digest, image}` (policy kind `execute_sandbox_command`). The host `ExecuteCommand` action and its `untrusted_executable` gate are unchanged. `sandbox_class: openshell` is accepted only on `execute_sandbox_command` rules. Such a rule must pin exactly one `cell_id`, `model_hash`, `openshell_policy_digest` and `sandbox_image`, and must name an `argv0`. The host runtime `execute` refuses a sandbox operation (`sandbox_command_requires_openshell`). The local and jail backends refuse it too. `execute_openshell` refuses host operations (`sandbox_command_required`) and checks the projection identity and image against the grant before it creates anything. There is no fallback to a weaker class.

12. **Promotion is a separate typed action that is never allowed outright.** Promotion is `PromoteChangeset {repository, reference, expected_base, proposed_commit, proposed_tree, changeset_digest, paths}` (policy kind `promote_changeset`).
    - A rule without `requires_approval: true` fails bundle load. Evaluation escalates.
    - The only grant path is the existing `grant_after_approval`. The requester must differ from the resolver, the resolver must hold the SOD approver role, and the approval and criteria must be bound to committed ledger records.
    - Consumption is one-time and ledger-backed (`approval_grant_consumption`).
    - The grant is bound to the exact action hash. A changed proposal, tree, base or reference needs a new decision and a new approval.
    - `GitCommit` and `GithubPr` behaviour is unchanged.

    Three things are kept separate: permission to execute (a sandbox-command grant), acceptance of settlement evidence (a settlement event), and authority to promote (an approved `PromoteChangeset` grant). A stale base fails with `stale_base` and leaves the ref unchanged. Known behaviour: a grant attempt with a mismatched action consumes the approval. It fails closed, and a fresh approval is then required.

13. **The OPA engine is real.** `OpaCandidate::evaluate` runs the pinned `opa` 1.21.1 binary as a subprocess: `opa eval --format json --bundle <bundle> --stdin-input data.domainforge.policy.decision`, with a cleared environment and a timeout. The bundle is the DomainForge-generated Rego, declared as the policy's `rego_provenance` source surface. Every bundle file is hash-checked against `provenance.json`, and the file set must be exact. The decision is validated against contract `domainforge.rego.decision/v1`, the model hash, the outcome and the results. Any failure becomes a Deny candidate with a failure code. The candidate takes part in `resolve_candidates`, where Deny wins and then Escalate. With the engine configured and no candidate supplied, the result is Deny if required and Escalate otherwise. It is never Allow. There is no OPA server.

## Rejected alternatives

- **Exempt `ExecuteCommand` for the `openshell` class.** Rejected. It would make the host gate, including `untrusted_executable`, conditional on a sandbox declaration, which weakens it. Guest commands get their own action instead (decision 11).
- **Auto-approve `GitCommit` for promotion.** Rejected. Promotion moves an authoritative ref. It needs its own action, its own approval and a grant bound to the exact proposal (decision 12). `GitCommit` stays non-grantable (`git_commit_still_escalates_as_unsupported_and_is_not_grantable`).
- **regorus 0.12.0 as an in-process Rego evaluator.** Rejected. It loads the generated Rego but returns an evaluation error for every well-formed input: 0 of 605 cases, against OPA's 605 of 605. The comparison is recorded in the DomainForge PR. The reference binary is OPA itself (decision 13).

## Not built

A second parser, orchestrator, policy authority, ledger, content-addressed store, container engine or hypervisor; an OPA server; an in-process Rego evaluator (see Rejected alternatives); a fork of OpenShell; a dependency on BoxLite (it documents copy-on-write disks and clones that OpenShell's VM driver does not, which makes it a candidate donor for forks/snapshots later, not a present need).

## Consequences

- The live proof needs a KVM-capable host. Offline tests (projection, OPA differential, prover boundary, adapter against a stub CLI, changeset/promotion) run anywhere and are reported separately from live runs.
- Disk quota has no per-sandbox enforcement in OpenShell v0.1.3; see the mapping report and the proof matrix for how it is handled and what remains unproven.
- Real isolation (network, filesystem and process containment inside a MicroVM) is UNPROVEN. The prover checks policy containment, not runtime behaviour. Only a live run on a KVM host can show it.

## Episode flow

`run_cell_episode` in `crates/sea-forge-case-runner/src/cell_episode.rs` runs these steps:

1. Read the projection identity (`openshell.lock`) and require an image. Fail closed before any authority call.
2. Optional OPA candidate, when the policy configures `opa`. It is recorded as ledger kind `opa_evaluation` before the decision.
3. Authorize `ExecuteSandboxCommand` (`argv`, `cwd`, `cell_id`, `model_hash`, `policy_digest`, `image`). The decision is committed as `authority_decision`. The same OPA candidate is passed to the decision.
4. Only on Allow: capture the base, export it to the staging area, and execute in OpenShell. The result is recorded as `openshell_execution`, then the evidence files are captured.
5. Changeset review (`workspace_changeset`) and validation.
6. Settlement (`settlement_event`) on the execution verdict and the validation verdict.
7. If settlement accepted and validation accepted a non-empty changeset, evaluate `PromoteChangeset`. The evaluation escalates, is committed as a second `authority_decision`, and yields a `PromotionRequest`. The episode never moves the ref.

Promotion is a separate call, `promote_cell_changeset`. It checks that the request describes its own changeset and this repository, then consumes an approval through `grant_after_approval` (`approval_grant_consumption`), and moves the ref by compare-and-swap (`workspace_promotion`).

Ledger kinds for the happy path, in order: `authority_decision`, `openshell_execution`, `workspace_changeset`, `settlement_event`, `authority_decision`, `settlement_criteria`, `approval_resolution`, `approval_grant_consumption`, `workspace_promotion`. The `settlement_criteria` and `approval_resolution` records are committed by the caller, not by the episode.

## Known limitations

- Live MicroVM execution is BLOCKED and UNPROVEN on the development host: WSL2 with no `/dev/kvm`.
- `crates/sea-forge-case-runner/tests/cell_episode_live.rs` holds six live tests. They compile and have never executed: the development host has no `/dev/kvm`, so `scripts/openshell/live-proof.sh` stops at its preflight with exit 3 (BLOCKED).
- The VM driver has no per-sandbox CPU or memory limits in OpenShell 0.1.3. Only gateway-level `vcpus` and `mem_mib` apply.
- A grant attempt with a mismatched action consumes the approval. It fails closed, and a fresh approval is then required (decision 12).
- The RepairAgent model's single projected policy is constant true. Real-OPA deny and escalate outcomes are therefore exercised with a separate fixture, `fixtures/policies/gate` (added alongside this change).
- The real-OPA tests are not yet in CI. The `authority-opa` job that runs them is proposed separately on branch `ci/authority-opa`, pending review, and has never run on GitHub.
- The changeset guarantees, the authority tests and the OPA stub tests run offline. Their results are not re-run for this document.
