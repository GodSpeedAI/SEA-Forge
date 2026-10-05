# Debt ledger — CEP / world_ref migration (Stages 0–11)

Recorded 2026-10-05 (updated after Stage 8) from the migration work on branch `migration/cep-world-ref` (worktree
`~/projects/sea-rs-cep-migration`, sea-rs HEAD e0f54de; cognate `cognate/harness` a3ee8fd; DomainForge
0.19.0; cep `main` + open PR #2). Each entry says what is true, how I know, and what closing it takes.

Labels: **observed** = I saw it happen in this work; **inferred** = follows from code I read but I did not
reproduce it; **unverified** = I noticed it and did not check. Resolved items are kept so the history is
honest. IDs are `M-nn` (migration) to avoid colliding with the repository's own `DEBT-nnn` series.

## Open

### M-01 Branches and PRs not landed
- **Status:** closed 2026-10-05, except legacy SEA. **Observed.**
- Merged: cep #2, Context-Kernel #1, GodSpeed-Agent #1, swe_seed #2, Gauntlet #1, Cognate #1, SEA-Forge #7
  (squashed; the repo forbids merge commits, so Stage 4-9 history lives on the remote branch
  `migration/cep-world-ref`). Legacy SEA `migration/domainforge-0.18.2` (8d91dff7b) still has no PR.
- Stage 10 work is on `migration/cep-world-transition` (sea-rs) and `cognate/world-transition` (Cognate).

### M-02 Legacy SEA: Dependabot backlog
- **Status:** open. **Observed** (GitHub reported 826 vulnerabilities on the default branch during a push).
- Not addressed; out of the migration's scope. `domainforge-cli` on npm is an unrelated third-party package
  that the legacy repo previously depended on; it was replaced by `@godspeedai/domainforge`.
- **Close:** triage separately from this migration.

### M-03 Cross-binding goldens embed the release version
- **Status:** open. **Observed** (release PR #134 failed 8 jobs).
- Every DomainForge release PR needs the Rust contract/envelope, Python, TypeScript and WASM golden
  constants regenerated plus `Cargo.lock`, or CI fails.
- **Close:** make the goldens version-independent, or generate them in the release workflow.

### M-04 world_ref changes with every release and every source edit
- **Status:** open by operator decision ("digest as-is"). **Observed.**
- `DomainModelIdentity::canonical_digest()` binds the exact source text and the producer version, so a
  comment-only edit or a DomainForge upgrade mints a new `world_ref`. `semantic_closure_hash` is stable
  across releases for the same source, which is what proves the meaning is unchanged.
- **Impact:** every consumer must re-register worlds on upgrade; stored `world_ref` values do not survive one.
- **Close:** revisit only if a release-stable world identity is wanted (would need an explicit decision).

### M-05 World identity depends on logical paths
- **Status:** open (documented gotcha). **Observed** (first live loop failed with "unknown world").
- DomainForge's CLI derives logical URIs relative to the entry file's directory. A world loaded under a
  different root-relative path has a different identity. SEA-Forge's world config now takes `base` plus
  entry-relative files to match.
- **Close:** document in operator docs; consider a tool that prints the `world_ref` a cell will compute.

### M-06 SEA-Forge world registry is in-memory only
- **Status:** open. **Observed.**
- `WorldRegistry` (Stage 4) is a derived cache; registered worlds are rebuilt from server config per cell
  (cached by config + file bytes). No durable world history exists.
- pg0 migration is not started and out of scope; `world_ref` values are plain strings in append-only
  records so a later move should not redo them.
- **Close:** decide persistence with the pg0 work.

### M-07 SEA-Forge does not require a bound world outside the CEP authority path
- **Status:** open. **Inferred.**
- `DomainModelRef.world_ref` is optional. Only the `authority_request` verb requires a registered world.
  Other governed paths (case runner, delegation, settlement) do not yet carry or require one.
- `WorldRegistry::verify_snapshot` checks DomainForge identity integrity, not full CEP profile
  conformance (cep's validators own that).
- **Close:** Stages 8–10 (execution/evidence/settlement and world transitions).

### M-08 Policy engine keeps closed vocabularies
- **Status:** open. **Observed.**
- Operation kinds are enumerated in three places in `sea-forge-authority` (policy loader, reserved-type
  list in `malformed_action`, rule matching). Adding `cognate_action` / `cognate_capability` required
  touching two. A new surface will silently fail closed until all are updated.
- **Close:** one registry for operation kinds.

### M-09 Public struct widened
- **Status:** open (low). **Observed.**
- `PolicyRule` gained `subjects: Option<Vec<String>>` (omitted from serialization when absent, so bundle
  hashes are unchanged). Any downstream code building `PolicyRule` literally must add `subjects: None`; one
  such literal in `sea-forge-sandbox` tests was fixed.

### M-10 Escalations: durable wait landed (Stage 7); residue below
- **Status:** mostly closed. **Observed.**
- Closed: an escalated capability call inside a Cognate run now waits on a durable continuation, approval is
  bound to one operation, single-use, expiring, resolved by someone other than the requester, and revalidated
  against current policy. See M-21 to M-24 for what remains.
- Still true: the engine's in-memory "opaque constraint" (an escalation parks a constraint on its resource
  within one engine instance) does not persist, because the server rebuilds the engine per request. That is
  why escalations are tracked in the ledger instead; the constraint itself is unused by this path.

### M-11 Evaluated actor is the verified service identity
- **Status:** partly closed. **Observed.**
- Rules key on the verified caller's role and operation kind. Per-end-user policy now works through
  `subjects` rules on the Cognate surfaces. The envelope subject is still recorded, never trusted for role.
- **Remaining:** no per-subject policy on other surfaces; identity of end users is asserted by Cognate and
  only the service identity is verified by the socket peer.

### M-12 Only `timeout_secs` can be enforced by Cognate
- **Status:** open. **Observed.**
- SEA-Forge boundary dimensions are `workspace`, `artifacts_root`, `timeout_secs`, `env_keys`,
  `sandbox_class`, `max_manager_iterations`. Cognate has an enforcer for `timeout_secs` only (capability
  calls). A boundary or degraded decision with any other dimension, or a timeout on a plain action, is
  refused. Compensating controls (`audit`, `short_timeout`, `minimal_environment`) have no enforcers.
- **Close:** add enforcers as consumers need them.

### M-13 Governance defaults to off in the harness
- **Status:** open by design until deployed. **Observed.**
- `governanceFromEnv` makes `sea-forge` mode an explicit opt-in; unset means `off` (local policy only).
  Nothing in a default install is governed by SEA-Forge.
- **Close:** deployment decision; Stage 11/12 hardening should state the production setting.

### M-14 Live loop tests skip without their prerequisites
- **Status:** open. **Observed.**
- `sea-forge-live.test.ts` is skipped unless `SEA_FORGE_SERVER_BIN` is set; the CEP conformance tests need
  `CEP_REPO`. A skip is not a pass. `just sea-forge-live` fails (never skips) when prerequisites are
  missing, but it is not part of `just verify` or any CI.
- Same for sea-rs `cep_validators_accept_every_emitted_decision_and_the_request` (needs `CEP_REPO`).
- **Close:** run both in CI with the binaries built.

### M-15 Cognate: flaky and failing tests not investigated
- **Status:** open. **Observed.**
- Gate `AK-006` (RealityTrace characterization, drives a real `sxr` binary against temp homes) failed once
  in one serial `just verify` run and did not reproduce in 23 further serial runs. Cause unknown.
  Isolated parallel runs fail about 1 in 3 because they share temp state, so that rate is not evidence.
- The full `bun test` suite has environment-dependent failures: 77, 78, then 68 (stable across two runs)
  in AG-UI interop, E2E, J-* journeys and archived `validation/reconstructions/**` copies. None are in
  code the migration changed; none were investigated.
- Running the suite rewrites a tracked file, `validation/reconstructions/copilot/tests/architecture/
  reference-removed.log`, which blocks `git stash pop` until reverted.
- **Close:** isolate shared temp/port state per test; stop tests writing tracked files.

### M-16 Cognate: duplicate version pins
- **Status:** open. **Observed.**
- `@godspeedai/domainforge` is pinned in three package manifests/lockfile and again as the `BINDING`
  constant in `packages/domainforge/src/types.ts`; the constant was missed by a manifest bump and caught only
  by `just verify`. Same pattern in sea-forge-domainforge (`EXPECTED_DOMAINFORGE_VERSION` constant beside the
  `=` crate pin; the version check is intentionally fail-closed).
- **Close:** derive the constant from the manifest, or test equality in one place.

### M-17 sea-forge-domainforge adapter descriptor hash may be stale
- **Status:** open. **Unverified.**
- `ADAPTER_DESCRIPTOR_SHA256` is documented as computed over adapter name, version and `domainforge_version`,
  but it did not change across the 0.16.0 -> 0.18.2 -> 0.19.0 upgrades. Either the comment is wrong or the
  hash is stale. I did not check.
- **Close:** recompute and compare; fix the comment or the constant.

### M-18 Repository git hooks are not running
- **Status:** open, deliberately (operator: enable only if the hooks need no fixing; they did). **Observed.**
- Every commit and push prints "hook was ignored because it's not set as executable". The real cause was worse than the
  mode bit: all nine `.githooks/*` were committed with CRLF line endings, so once executable they fail to start
  (`env: 'bash\r': No such file or directory`). Fixed on 2026-10-05: LF endings, pinned by `.gitattributes`
  (`.githooks/* text eol=lf`). The executable bit is NOT set.
- With LF they work: `pre-commit` (`just check-fast`) took 34 s and passed; `pre-push` (`just ci`) took 10.5 min and passed.
- **Close:** `chmod +x .githooks/*` and `git update-index --chmod=+x .githooks/*`, then commit. Costs to accept first:
  every push takes ~10 min; `core.hooksPath` is local git config (`just hooks-install` sets it), not tracked; `--no-verify` bypasses both; the `.pre-entire` files are chained wrappers.

### M-19 cep: lint/type debt in test and tool code
- **Status:** open (recorded in cep as DEBT-0010). **Observed.**
- Strict pyright over `tests/` reports 140 errors; `tools/validate_okf.py` has 53 ruff findings. The
  exporter and psycopg declaration were fixed; these were left.

### M-20 Release hygiene
- **Status:** open (low). **Observed.**
- Many crates declare both `license` and `license-file`; cargo warns on every invocation (noise that hides
  real warnings).
- Gauntlet's 0.16.0 capture test is marked historical rather than removed.
- `domainforge-lsp` uses a `sea-core = { package = "domainforge-core", version = "=0.19.0" }` alias after its
  path dependency was found broken; confirm that is the intended long-term shape.

### M-21 Escalated actions do not wait
- **Status:** open. **Observed** (design limit).
- Only capability invocations inside a run wait on an approval. Service-level actions (`run.start`,
  `continuation.resume`, state updates, remote offers) that escalate are refused with the approval id and have
  no resume path. A policy that escalates `run.start` makes that action unusable.
- **Close:** decide whether actions need an approve-then-retry flow (the approval machinery supports it; the
  client side does not).

### M-22 Crash window after an approval is consumed
- **Status:** open. **Inferred** from the code and the single-use design; not reproduced by a crash test.
- Consumption is the ledger commit of the allow decision. If the process dies after it and before the
  invocation is recorded, the replay is refused (the approval is spent) while the provider may already have run
  once. This is at-most-once *authority*, not exactly-once *effect*.
- **Close:** record the intent before the call, or make the provider idempotent for governed calls.

### M-23 Approval polling is manual by default
- **Status:** open. **Observed.**
- `Runtime.pollApprovals()` does the work; `governance.approvalPollMs` runs it on a timer, but nothing sets it
  by default and `governanceFromEnv` does not expose it. Without a poll or a manual resume an approved run
  keeps waiting until its approval window closes, then fails.
- **Close:** expose the interval in the environment configuration and set a default for sea-forge mode.

### M-24 Cognate approvals are invisible to the workbench
- **Status:** open. **Observed.**
- Approvals live in the `cognate-authority` ledger stream, derived by key. SEA-Forge's own approval inbox
  reads the case approvals journal (`approvals.jsonl`), which is keyed per case and assumes a case ledger. A
  human can list and resolve Cognate approvals only through the `authority_approvals` / `authority_approval`
  verbs, not the workbench.
- **Close:** surface them in the inbox, with their governance context, or add a workbench view.

### M-25 Escalation approval depends on a cell policy rule
- **Status:** open. **Observed.**
- Resolving an approval needs a policy rule for `approval_resolution` for the approver's role. A cell that
  escalates but has no such rule can never approve anything; the escalation then waits until it expires.
- **Close:** operator documentation, and a policy lint that flags `requires_approval` without an approver rule.

### M-26 Process: Stage 7 spec and plan written after the code
- **Status:** recorded. **Observed.**
- SEA-Forge's AGENTS.md asks for a governing spec and plan before substantial work. For Stage 7 both were
  written after the implementation and say so. The requirements were not changed to fit results, but the order
  was wrong.

### M-27 RealityTrace does not speak the GodSpeed profiles; Cognate packages for it
- **Status:** open. **Observed** (released RealityTrace 0.3.0).
- `sxr` writes its own closed envelopes: `scope` is limited to repo/checkout/run keys (no `world_ref`),
  lineage is empty, kinds are fixed by record (`evidence.created` is an `evidence_packet`, `settlement.registered`
  a `settlement_packet`). Cognate's adapter builds the GodSpeed `execution_trace` (delivered to RealityTrace as
  hash-addressed content) and the GodSpeed `evidence_packet` (from RealityTrace's ledger rows). The producer of
  the evidence packet is therefore the adapter, not RealityTrace.
- RealityTrace's own `settlement_packet` records are a *different fact* (its threshold judgment about its
  question). They are never read as SEA-Forge's settlement; the two are deliberately not conflated.
- **Close:** a RealityTrace release that emits GodSpeed profile envelopes natively (needs a crates.io release).

### M-28 Evidence is tied to an operation by the adapter's choice, not by proof
- **Status:** open. **Observed.**
- RealityTrace evidence is per question and target, not per Cognate operation. The adapter selects which
  evidence rows to attach to an operation; SEA-Forge cannot tell whether the selection is right. A mis-selected
  supporting row (for example another operation's) would settle the wrong operation. SEA-Forge does verify the
  chain it can: a committed allow decision, same world, lineage, well-formed items.
- **Close:** evidence that carries the operation id (per-operation question or plan in RealityTrace).

### M-29 One shared question and target for all operations
- **Status:** open. **Observed.**
- Cognate's RealityTrace workflow declares a single question and the target `concept:runtime_execution` for
  every governed operation. Settlement is per question, so operations share a question id.
- **Close:** declare a question per operation class or per operation.

### M-30 Settlement criteria are cell-wide and config-declared
- **Status:** open. **Observed.**
- `min_supporting` and `min_reliability` apply to every operation in the cell. They are bound into each allow
  decision by hash at decision time (so they cannot be moved afterwards; a settle under changed criteria is
  refused), but they are declared in cell configuration, not as a signed record, and cannot differ by resource,
  surface or world.
- **Close:** criteria declared per surface/resource in policy.

### M-31 Evidence reliability and source are taken as reported
- **Status:** open. **Observed.**
- SEA-Forge counts an item if its direction is `supports` and its reliability label meets the minimum. The label
  is RealityTrace's (`medium` by default for a verifier run); the `source` string is not checked against an
  identity. Any identity that policy lets use `evidence_mutation`, and that presents a valid chain, can submit
  items; the submitter is not bound to the requester or to RealityTrace.
- **Close:** bind evidence to a verified producer identity; treat reliability as policy input, not a label.

### M-32 Final settlements cannot be revised, and late evidence is silently unconsidered
- **Status:** open. **Observed.**
- `settled` and `rejected` are final per operation. Evidence submitted afterwards is recorded but a later
  settle returns the recorded final settlement (marked replayed). There is no appeal or re-evaluation path, and
  the response does not tell the caller that newer evidence was ignored.
- **Close:** decide whether finality needs a governed re-opening, and surface ignored evidence.

### M-33 Settlement goes nowhere yet
- **Status:** open — Stage 9. The `settlement_packet` is committed in the `cognate-authority` ledger and returned
  to the caller. It is not sent to SWE_SEED or any other consumer.

### M-34 Retries and ledger payload determinism
- **Status:** open (low). **Observed.**
- The ledger's idempotency key requires a byte-identical payload. A direct (non-socket) retry of
  `authority_request` for the same operation id re-evaluates, produces a different decision payload, and
  conflicts; the socket path is protected by request-id replay. Policy-gate decisions (evidence, settlement,
  approval attempts) are committed once per attempt, so retries create additional governance records. That is
  deliberate (each attempt is an event) but unbounded.
- **Close:** a cap or compaction policy, or deterministic decision payloads.

### M-35 Request ids can be burned by another caller
- **Status:** open (low). **Observed** (a tampered packet under the genuine envelope id made the genuine
  request unusable until the id included a payload hash).
- SEA-Forge binds a `request_id` to its first payload. The Cognate client now derives evidence request ids from
  the payload hash; any other client must do the same, and a verified caller can still deliberately consume
  another operation's ids.

### M-36 The live Stage 8 gate depends on a real verifier and a crates.io binary
- **Status:** open. **Observed.**
- The end-to-end test needs the released RealityTrace binary (`just setup-realitytrace`), cargo and the
  DomainForge CLI, runs a real `cargo test` (cold compile makes it slow), and is not part of `just verify` or any
  CI. The verifier is a toy project's unit test; real use needs a real verifier (Gauntlet was not made one in Stage 9; see M-41).

### M-37 The legacy E1–E7 chain checks `world_ref` syntax and equality, never the digest
- **Status:** closed 2026-10-05 (Stage 11; merged in SEA-Forge #9). **Observed.**
- Decision: the legacy E4–E6 chain is a contract library, not a server door (a second entry would give one
  decision two paths). `accept_verified_governed_work_request` recomputes the world against a `WorldRegistry`;
  the syntax-only `accept_governed_work_request` is `#[deprecated]` and kept for the synthetic-world fixture suites.
- Residual: the later edges (E5A/E5B/E6 emit) take the world from the intent and do not re-verify it; they are only
  reachable through an intent, and the deprecated entry point can still produce one. Close by making
  `GovernedWorkIntent` unconstructible without verification if the chain ever gets a transport.

### M-38 RealityTrace's legacy E7/E8 wire carries no `world_ref`
- **Status:** closed 2026-10-05 (Stage 11; merged in RealityTrace #5 and GodSpeed-Agent #2). **Observed.**
- RealityTrace ingestion requires a pinned `world_ref` equal to the deployment's (syntax and equality) and carries it
  into `ComparisonInputs`; emission stamps it and refuses when the pinned and compared worlds differ. A pinned
  GodSpeed-Agent deployment refuses E8 evidence that names no world (`world_ref_missing`); an unpinned one still
  records it `unbound`. Proven end to end by `scripts/e2e-world-loop.sh`.

### M-39 Producers and mirrors not yet pinned to a world
- **Status:** open. **Observed.**
- SWE_SEED's standalone `emit_*` helpers (`federation/emit.rs`, flag-gated) do not pin a world. Context
  Kernel's `integration/agentic_capability_loop/adapters.py` is an older copy of the Python adapter and was not
  touched. GodSpeed-Agent's developmental events and memory (E10) are not world-pinned, and its adapter still
  falls back to the namespace pseudo-hash when the legacy SEA manifest is absent.

### M-41 Stage 9 did not make Gauntlet a verifier
- **Status:** open. **Observed.**
- Boundary 10 was "MAY / confirm". Gauntlet consumes no `world_ref`; the confirmation is four tests that a
  world_ref cannot be minted as a semantic fingerprint or file identity. M-36 still stands: the Stage 8 live gate
  verifies with a toy project's unit test, not Gauntlet.

### M-42 Pre-existing lint and format failures in the Stage 9 repos
- **Status:** open (low). **Observed.** Context Kernel's fmt drift and doc drift are fixed (7ef81cc).
- Context Kernel: no `justfile` or `just context-check` although AGENTS.md requires it.
- SWE_SEED: `cargo fmt --check` fails across ~30 files; 45 clippy warnings (unchanged by Stage 9). A repo-wide
  reformat would bury the real diff, so it needs its own commit.
- GodSpeed-Agent: 44 ruff findings (unchanged). Gauntlet: 89 biome warnings (unchanged, 0 errors).

### M-43 New git worktrees under `mise` need `mise trust`
- **Status:** open (low). **Observed.**
- In a fresh worktree of SWE_SEED the `python3` shim fails ("config files are not trusted"), so scripted edits
  silently did nothing until I called `/usr/bin/python3`. A first build "succeeded" against unedited code. I did
  not run `mise trust`.

### M-44 Stage 9 branches are pushed but unreviewed
- **Status:** closed (merged 2026-10-05; sea-rs by squash). **Observed.**
- Context Kernel `migration/cep-world-ref` (new, from `harden`), GodSpeed-Agent `migration/cep-world-ref` (new,
  from `audit-corrections/neatcode-2026-07-29`), SWE_SEED `migration/cep-world-ref` (new worktree
  `~/projects/SWE_SEED-cep`, from `deploy-prep`), Gauntlet `migration/domainforge-0.18.2`. No PRs. The three new
  feature branches depend on each other (CK packets carry what SWE_SEED now requires), so they should land together.

### M-45 Transition compatibility is a claim unless the closures are equal
- **Status:** open. **Observed.**
- DomainForge has no world-level semantic diff, so SEA-Forge cannot tell `backward_compatible` from `breaking`.
  It records the sender's claim (or `unknown`) and forces a human approval for any unequal closure. File names are
  part of a world's identity, so a pure rename also needs approval.
- **Close:** a DomainForge world diff, then compute compatibility and let policy distinguish.

### M-46 A transition moves nothing
- **Status:** resolved 2026-10-05 (operator accepted the recommendation). **Observed.**
- Rule, enforced in SEA-Forge (`check_lineage_worlds`): a request whose `lineage_refs` cite an envelope this
  authority decided in another world must cite, as `extensions["godspeed.authority_request"].transition_ref`, the
  ledger entry of an allowed transition from that world to the request's world; otherwise `cep_transition_refused`.
  Work already decided stays pinned to its world and is never re-pinned. Cognate: `CarriedWork`,
  `RecordedTransition.ledgerEntry`. Proven against the real server (live suite 35/0).
- Limits: only ids this ledger decided are judged (an unknown id is not); the legacy E1-E8 chain still refuses any
  second world outright, which is stricter. A transition does not itself re-run or migrate anything.

### M-54 `decision_id` is not a unique reference (resolved with M-46)
- **Status:** resolved. **Observed** (every decision in one ledger read `auth_01`).
- `decision_id` is unique only within one operation. The world-transition record's `authority_decision_ref` used it,
  so it did not identify one decision; it now carries the ledger entry (a unique ULID). Anything else that treats
  `decision_id` as a global key has the same flaw; none found in this repo.

### M-47 Transition listing scans the whole ledger
- **Status:** open (low). **Observed.**
- `transitions()` reads every ledger entry on each call. Fine at this size; unbounded growth is not addressed.

## Resolved during the migration (kept for the record)

- **R-01** sea-rs `just check` was red on inherited debt: clippy findings in `sea-forge-server` tests, 14
  gitleaks false positives (inspected structurally without printing values; fingerprints added to
  `.gitleaksignore`). Fixed.
- **R-02** Gauntlet biome errors in four files. Fixed.
- **R-03** cep: psycopg undeclared, exporter untyped, OKF validator scanned `node_modules`, doc link to a
  gitignored file broke CI. Fixed (see M-19 for the remainder).
- **R-04** `OQ-0006` collision in cep (my entry reused an existing number and a script truncated the
  following entries). Restored and renumbered `OQ-0010` (PR #2 pending, M-01).
- **R-05** Cognate status history: my revision-31 snapshot lacked its `---` separator and broke
  `just verify`. Fixed in revision 32.
- **R-06** CG-PRF-002 flake: React's scheduler outlived the unregistered DOM (`window is not defined`).
  Reproduced 5 of 18 under load, fixed to 0 of 30; pre-existing (reproduced on the pre-change commit).
- **R-07** Registry rebuilt per request: now cached by config + file bytes (tested against stale reuse).
- **R-08** Cross-implementation world identity mismatch (M-05): config model corrected.
- **R-09** Cognate status revision 36 and the Stage 8 evidence file were missing. Added (cognate 6118ac2).
- **R-10** Fixture writers regenerated sibling checkouts by default (was M-40). They now skip unless `SWE_SEED_ROOT`, `SEA_RS_ROOT` or `SXR_ROOT` is set (sea-rs, SWE_SEED). `~/projects/SWE_SEED`'s `t06_operational_settlement.json` is still modified from earlier runs; I left that working tree alone.

## Stage 11 (production hardening) — new

### M-48 The end-to-end loop simulates execution and needs sibling checkouts
- **Status:** open. **Observed.**
- `scripts/e2e-world-loop.sh` takes real surfaces in five repos but E5B (the execution report) is a stand-in; real
  execution under a decision is Cognate's `just sea-forge-live`. The loop is run locally, not in CI, because it needs
  five checkouts on the right branches plus the DomainForge CLI.
- **Close:** a CI job that checks out the five repos at pinned SHAs (needs a cross-repo token), or a release-time
  gate. Not worth it before the repos settle on one branch convention (GSA and SWE_SEED stack on non-main bases).

### M-49 Only Context Kernel's Rust side projects to `context_bundle`
- **Status:** open. **Observed.**
- `ck-mcp::cep_bundle` maps `ContextPacketCreated` to the profile and is validated against the cep reference
  validator when `CEP_REPO` is set (in the e2e loop too). The older Python copy of the adapter in Context Kernel
  (`integration/agentic_capability_loop/adapters.py`, see M-39) and SWE_SEED's consumer do not speak the profile; the
  legacy payload remains the wire. The Rust validator in cep is not wired into Context Kernel's CI.

### M-50 CI covers the migration repos only as far as their own environments allow
- **Status:** open. **Observed.**
- Added: Cognate (`verify.yml`: just verify with the DomainForge CLI, RealityTrace, Playwright), GodSpeed-Agent
  (`tests.yml`: pytest with the ml extra and RuVector), SEA-Forge CI on PRs into any base plus `bun` for the
  workbench jobs. Not added: SWE_SEED on its `deploy-prep` base shows no checks; Gauntlet's CI was not exercised.
- `.githooks/*` were committed without the executable bit, so none ran. They work as written (`pre-commit` =
  `just check-fast`, 34 s; `pre-push` = full `just ci`, 10.5 min locally, passes), so the bit is not set (see M-18, which also records the CRLF defect).

### M-51 SWE_SEED gateway audit appends could fuse records (resolved in Stage 11)
- **Status:** resolved and merged (swe_seed #3). **Observed** (CI failure, then reproduced 3/3 with a regression test).
- `AuditWriter::write` used `writeln!` on an unbuffered file: two writes, so concurrent requests could fuse two audit
  records into one line and lose one. Fixed with a single buffered write under a lock. Worth asking whether any
  deployed gateway already has fused lines in `audit.jsonl` (a fused line is valid JSON up to its first record and
  has trailing characters after).

### M-52 macOS checkouts of SEA-Forge (resolved in Stage 11)
- **Status:** resolved and merged (SEA-Forge #9). **Observed.**
- `.agents/plans/TODO.md` and `todo.md` differed only by case; renamed the lowercase one. Any other case-colliding
  paths would show the same symptom: `git ls-files | sort -f | uniq -di`.

### M-53 Two SEA-Forge CI jobs fail on the `casework/live-wiring` base for reasons outside this migration
- **Status:** open. **Observed** (SEA-Forge #9, run 37330114391; lint, test and package pass).
- Enabling CI for PRs into any base was what exposed them; the base had never been tested by CI.
- `verify (macos)`: `sea-forge-case-runner` `conformance_m5` (`accepted_stage_settles_and_completes_case`,
  `downstream_sentry_blocked_by_rejected_predecessor`) assume the Linux Landlock jail and get "jail backend is
  unavailable on this platform". Needs the tests gated to Linux or a macOS (Seatbelt) backend.
- `workbench (ubuntu)`: `packaged_stack::records_survive_the_kernel_being_stopped_and_started_again`. After adding the
  `request_id` the server now demands (`durable_locator_required`), the submit is accepted but `run_list` is empty.
  It is T09 casework territory, so I stopped.
- Already fixed along the way: stale workbench `Cargo.lock` (domainforge-core 0.16.0), needless borrows in
  `bridge.rs`, an off-Linux unused import, `JailSandbox: Debug`, `bun` and Tauri libraries missing from the jobs, gitleaks
  fingerprints that cannot match on a shallow PR checkout, and a case-colliding pair of files.
- **Close:** fix or gate those tests, then make the checks required. Until then #9 is mergeable on the three green
  checks only if you accept two red jobs that are not caused by it.
