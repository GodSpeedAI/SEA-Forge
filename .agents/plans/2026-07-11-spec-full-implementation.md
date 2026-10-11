# Implementation Plan — SEA Forge Full System (spec-full.md, M0–M8)

**Created:** 2026-07-11
**Source of truth:** `.agents/specs/spec-full.md` (Draft v0.2) — quote it, don't paraphrase from memory. Where it is silent, `.agents/specs/spec-minimum.md` governs (spec-full §0 line 13).
**Originating context:** The v0.1 minimum vertical slice is merged, green on `main` (35 tests, P1–P4b pass — `.agents/CURRENT_STATUS.md`). This plan executes the draft v0.2 full spec: ten extension capabilities E1–E10 across milestones M0–M8.
**Status of the work today:** Tasks 1–9 are implemented on `full-spec`; M0, M1, the M2 case engine, and M2 templates are conformance-green. Task 9.5 is the next mandatory gate and closes the criteria-provenance contract before M3. `CURRENT_STATUS.md` carries the current proof results and blockers.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies: Tasks 1→9 remain
  milestone-ordered. After Task 9 is green, execute Task 9.5 before beginning
  Task 10. Tasks 10→17 then continue in order. Within M0, Tasks 3 and 4 remain
  independent of Task 2, Task 5 consumes Tasks 2 and 3, and Task 6 consumes
  Task 2. When in doubt, follow the dependency gates rather than task-number
  appearance.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **Core principle (spec-full §0.3, §17): the minimum kernel is the invariant substrate. P1–P4b must pass unchanged after every task. "A milestone that cannot pass the minimum spec's proofs P1–P4b unchanged has broken the kernel and MUST be rejected."** Never patch the proofs to make a milestone fit — fix the milestone.
- **Fail closed, always** (spec-full §0.6): a missing/unavailable/undecidable evaluator, ledger, jail, identity, or settlement authority denies, escalates, or halts — it never allows, never silently downgrades, never falls back to a weaker class.
- **Verify the substrate before designing against it.** At the beginning of
  every task, inspect the current code, tests, persisted record shapes, crate
  boundaries, and completed milestone behavior. Do not implement from the plan
  as though the repository were empty. Reuse an existing type, ledger record,
  hash path, store, validator, or authority surface when it already satisfies
  the required contract.
- **Do not add structure for naming symmetry.** A spec term does not
  automatically require a new crate, database, JSONL file, trait, or wrapper.
  Add a new structure only after the current substrate has been inspected and a
  concrete missing invariant has been identified. Prefer additive fields,
  existing ledger record kinds, existing writers, and existing validation
  paths over parallel implementations.
- For each task, the agent's completion report MUST include a short
  `Substrate reconciliation` section stating:
  1. what existing implementation was inspected;
  2. what was reused unchanged;
  3. what was extended;
  4. what genuinely had to be added;
  5. what tempting duplicate structure was deliberately not added.
- Match surrounding code style; the idioms to mirror live in `crates/sea-forge-core/src/` (typed errors in `errors.rs`, deterministic hashing in `authority.rs`, append-only JSONL discipline in `capability.rs`/`trace.rs`, lifecycle ordering in `pipeline.rs`).
- Source-of-truth rule: spec-full §7 record shapes and §12/§17.1 proof text are the spec — conformance tests assert those shapes and behaviors verbatim. Change a fixture only together with the spec change that justifies it, in the same commit.
- When the repository already implements an equivalent contract under a
  different internal name, preserve the working substrate and map the spec term
  onto it. Do not rename or rewrite working code merely to make terminology
  visually match the document. Add an alias, adapter, additive field, or
  documented mapping when that is sufficient.
- New crates go under `crates/`, are added to `Cargo.toml` workspace `members` (currently lines 3–6), and inherit `workspace.package` / `workspace.lints` (`unsafe_code = "deny"`).
- Milestone gates become tests named `tests/conformance_m<N>*.rs` in the owning crate (or `sea-forge-cli` for end-to-end flows), so `cargo test --workspace` runs every gate forever after.

### Global verification gates (must stay green after EVERY task)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
just proof                      # minimum-spec P1–P4b, unchanged
devbox run -- just check        # deny/licenses/gitleaks/context-check bundle
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
|---|---|
| Full spec: entities §7, config §8, flow §9, behavior §10, contracts §11, proofs §12, variation §13, failures §14, gates §17.1, DoD §18, milestone order Appendix A | `.agents/specs/spec-full.md` |
| Minimum spec (governs wherever full spec is silent; §16.1 path safety, §16.2 settlement basis vocabulary live there, NOT in spec-full — its §16 gap is intentional) | `.agents/specs/spec-minimum.md` |
| Crate graduation map: slice module → target crate → milestone | `.agents/specs/spec-full.md` §6.2 table |
| All 14 slice modules to graduate | `crates/sea-forge-core/src/{ids,types,errors,domain,authority,planner,sandbox,runtime,trace,evidence,settlement,capability,pipeline}.rs` |
| Workspace members list to extend | `Cargo.toml:3-6` |
| Proof/test/check recipes | `justfile:43` (check), `justfile:62` (context-check), `justfile:67` (test), `justfile:72` (proof) |
| DomainForge library, pinned version, feature rules | spec-full §6.3: `domainforge-core` `0.13.0`, default features off, `signing` only when a configured authority path needs it; sibling repo confirmed canonical (`.agents/CURRENT_STATUS.md`) |
| Semantic boundary decision (DomainForge owns `.sea` semantics; SEA Forge owns authority/side effects) | `docs/decisions/ADR-001-domainforge-semantic-boundary.md` |
| Ledger contract: jcs-nfc-v1, domain-separated SHA-256, CSPRNG monotonic ULIDs, append ordinals authoritative, chain+MMR, signed checkpoints, witness receipts | spec-full §7.0c |
| Exit codes incl. new `5 = awaiting_approval/parked` | spec-full §3.3, §10.9 |
| Typed deterministic resolution: deny blocks; unresolved escalation blocks while retaining boundaries; boundaries intersect; permitted degraded controls accumulate; allow is identity | spec-full §7.0 GovernanceVerdict |
| The one graduation security rule: untrusted argv0 never allow-listed on `local` class (schema_error) | spec-full §8.2, §15 |
| Confidence arithmetic is fixed, fixed-point 6-decimal, no binary floats in rebuild | spec-full §7.3 |
| Kernel stays synchronous; Tokio only in `sea-forge-server` (spawn_blocking seam) | spec-full §6.1 |
| Criteria provenance gap | Before §7.1a was added, spec-full §7.2.1 consumed `criteria_ref`, `criteria_sha256`, and `criteria_declared_at` without defining their canonical record; Task 9.5 implements the new contract before M3 |
| Substrate-first correction | Task 9.5 MUST inspect the implemented Intent, CasePlan, PlanItem, SettlementCriteria, template, ledger, migration, and settlement code before adding records; reuse existing contracts and add no new crate |
| Dedicated JobContract rule | A JobContract is conditional, not automatic: reuse a ledgered Intent, template, spec stage, policy requirement, issue, or external requirement when it already expresses the job with sufficient provenance |

## External ecosystem map (workspace siblings — reference only, none are build dependencies)

These sibling repos live under `~/projects/` and define the contracts SEA Forge's seams must stay compatible with. They inform shapes; they are NOT dependencies of any task, and NATS is never added to this workspace (spec-full §2.4, E6: bus is a later adapter behind `EventSink`).

| System | Role vs SEA Forge | Contract to respect | Where |
|---|---|---|---|
| **SWE_SEED** | Dev-work harness (routing, hooks, traces, proof gating); spec-full names it the **first external `SettlementAuthority` adapter** (E4/M4a) — its proof-gated completion claims are what make a declaration independent | `declare()` adapter boundary §7.2.1; SWE_SEED emits `WorkRequested`/`ContextRequired`/`RouteSelected` events | `~/projects/SWE_SEED/README.md` (Boundary section) |
| **godspeed_agent** | Settlement/navigation runtime; owns settlement *classification* (usable/partial/unusable) and the capability lifecycle ("metabolized after ≥3 usable settlements" — the source of spec-full §7.3's `metabolized` status). Appears in SEA Forge as the **GovernedSpeed candidate-verdict engine** (§7.0, §8.2 `policy_engines`), never as decision owner; spec-full §2.4 assigns it developmental routing/horizon changes as a non-goal here | `GovernanceVerdict` normalization + fail-closed unavailability (Task 5); capability-status vocabulary alignment (Task 11) | `~/projects/godspeed_agent/README.md` (Boundary section) |
| **agent-memory-ledger** (hassos add-on) | "Local-first persistence, governance, memory, and event-bridge substrate **for SEA Forge**": Postgres/TimescaleDB canonical event ledger + bidirectional **NATS JetStream bridge**. From SEA Forge's side it is a future `EventSink`/import adapter — it must never become a second truth for SEA Forge records (§2.2) | SEA event contract: envelope fields `event_id, trace_id, correlation_id, causation_id, idempotency_key, source_agent, occurred_at, schema_version`; subject taxonomy `sea.{domain}.{action}.{qualifier}`; one JetStream stream `SEA_LEDGER` over `sea.agent.event.>`, `sea.governance.request/decision.>`, `sea.memory.write/lifecycle.>`, `sea.ledger.outbox.>` | `~/projects/hassos-addon-agent-memory-ledger/docs/SEA_EVENT_CONTRACT.md`, `docs/plans/013_nats_jestream_bridge.md` |
| **Context_Kernel** | Single-binary MCP service for durable context/doc retrieval (SQLite+FTS5). External caller/context layer; E7 memory stays SEA Forge-internal — a Context Kernel adapter would be a later plugin behind the recall/authority seam | MCP tools; no contract obligation in M0–M8 | `~/projects/Context_Kernel_Service_MCP/README.md` |
| **CEP / semantic envelope** | The Canonical Evaluation Protocol (`~/projects/cep`, vendored in DomainForge) defines the normative **Semantic Envelope** (Identity/Boundary/Profile/Completeness); agent-memory-ledger ships a conformance JSON schema for it | SEA Forge's per-run `semantic-envelope.json` should converge on CEP-0008; check in Task 17 | `~/projects/domainforge/docs/specs/cep/CEP-0008-semantic-envelope.md`; schema fixture `~/projects/hassos-addon-agent-memory-ledger/tests/fixtures/cep/semantic-envelope.schema.json` |
| **DomainForge `Cell`** (ADR-012) | `.sea` declarations for an agent's hermetic *execution environment*: `Cell, SystemDependency, Runtime, Tool, DependencySet, Service, Mount, Endpoint, NetworkFlow, Credential`, with a normative SEA→Cell-IR mapping (Mount→sandbox mount policy, Endpoint/NetworkFlow→deny-by-default network allow-list, Credential→authority credential contract, never a literal) | E9 `EnvironmentSpec` (Task 15) must stay projectable FROM a validated Cell declaration — do not invent conflicting field semantics | `~/projects/domainforge/docs/specs/ADR-012-cell-environment-declarations.md`, `docs/cell-environment-projections.md` |

**Disambiguation (do not conflate):** spec-full "SeaCell"/`cell_id` (E6/M6) = a *federation node identity*. DomainForge `Cell` (ADR-012) = an *execution-environment declaration* in `.sea`. Same word, different concepts, different tasks (14 vs 15).

---

## Task 1 — M0a: Crate graduation (mechanical split)  (M0 · blocking everything)

**Goal:** The workspace builds as the §6.2 kernel crates with `sea-forge-core` reduced to ids/types/errors, and the 35 existing tests plus P1–P4b pass byte-for-byte unchanged.

**Why this shape:** Spec-full §6.2 fixes the crate map and calls the move "mechanical". Doing it first, with zero behavior change, isolates every later diff to one crate and keeps the kernel-regression gate cheap.

### Steps

1. Create crates `sea-forge-domain`, `sea-forge-authority`, `sea-forge-planner`, `sea-forge-sandbox`, `sea-forge-runtime`, `sea-forge-trace`, `sea-forge-evidence`, `sea-forge-settlement`, `sea-forge-capability`, `sea-forge-extension` under `crates/`, each receiving its module verbatim from `crates/sea-forge-core/src/` per the §6.2 table (`pipeline.rs` stays in core or moves to cli — keep wherever the diff is smallest; record the choice in the commit message).
2. `sea-forge-core` keeps `ids.rs`, `types.rs`, `errors.rs`; every new crate depends on it. Fix imports only — no logic edits.
3. Extend `Cargo.toml:3-6` members; inherit workspace package/lints in every new crate.
4. Add a CI-enforceable no-async-kernel check (spec-full §18): a `just` recipe or test asserting `cargo tree -i tokio` matches nothing in kernel crates (trivially true today; it must exist so M3 can't regress it).

### Gate

```bash
cargo test --workspace --all-features --locked && just proof
```

**Done when:** all pre-existing tests and P1–P4b pass with zero test-file edits; deliberately reverting one module move breaks the build (teeth: imports are real).

**Redesign trigger:** a module split forces a behavior change (circular dependency). Then merge the two offending crates and note the deviation from §6.2 in the plan's commit — do not change behavior to satisfy the crate map.

---

## Task 2 — M0b: `sea-forge-ledger` — integrity ledger  (M0 · the trust root)

**Goal:** Every v0.2 application source record becomes a ledger entry per §7.0c, and the M0 ledger proofs in §12 (append/tamper/fork/crash/witness/redaction/key-rotation fixtures) pass.

**Why this shape:** §5 claim table: "implement `sea-forge-ledger` before other full-spec state grows." Every later milestone writes through it; retrofitting would re-key history.

### Steps

1. New crate `crates/sea-forge-ledger`. Implement canonical encoding `jcs-nfc-v1` (RFC 8785 profile per §7.0c: NFC strings, sorted set-arrays, fixed-scale decimal strings, no NaN/inf) and the four domain-separated hashes exactly as the §7.0c `payload_hash`/`entry_hash`/`stream_root`/`global_root` definitions.
2. ULID generation: CSPRNG-backed, monotonic under clock regression; duplicate ULID or RNG failure ⇒ typed `ledger_integrity_error` (§7.0c "ULIDs and ordering"). Use a reviewed ULID crate or a small audited internal impl (§6.3).
3. `LedgerEntry` append path: single writer per stream, gap-free `append_ordinal`, previous-hash validation, write→flush→fsync file→fsync parent dir before exposing (§7.0c "Ledger streams and entries"). Streams: one per case + the global streams listed in §7.0c.
4. MMR construction with inclusion and consistency proofs; `LedgerCheckpoint` + global checkpoint + Ed25519 signing (keys outside `.sea-forge/`, referenced by opaque key IDs — §6.3, §8.1a); witness receipt verification and standing rules (§7.0c "Checkpoints, signatures, and independent witnesses"). Ship a test-only in-process witness for fixtures.
5. Verification/recovery: startup verify from last trusted global checkpoint; quarantine invalid tails; signed recovery checkpoints; never rewrite history (§7.0c "Verification, recovery, and privacy"). Redaction: reject plaintext/bare-hash secrets, accept approved ciphertext commitments; tombstones + crypto-shredding for retention.
6. CLI: `sea-forge ledger verify|prove <entry_ulid>|consistency <a> <b>` — read-only, no repair (§10.0a). Add `integrity_ledger` policy schema §8.1a with its rejection rules (production `min_witnesses: 0` invalid, witness == acting entity invalid, etc.).
7. Conformance tests `crates/sea-forge-ledger/tests/conformance_m0_ledger.rs` implementing every §12 M0 ledger bullet: 1,000 mixed records/two streams, one-byte alteration, delete/reorder/duplicate/truncate/fork, witness-detected full-fork substitution, pre-action witnessed checkpoint before `command_started`, kill-during-append recovery, clock rollback, secret-sentinel rejection, key rotation.

### Gate

```bash
cargo test -p sea-forge-ledger --locked && just proof
```

**Done when:** every §12 M0 ledger fixture passes and each tamper fixture fails verification with a *typed* reason (teeth: flip the assertion on the one-byte-alteration fixture and the test fails).

**Redesign trigger:** RFC 8785 canonicalization of an existing record type proves ambiguous (float fields). Then change the record field to a fixed-scale decimal string per §7.0c — never loosen the canonicalization profile.

---

## Task 3 — M0c: DomainForge semantic adapter  (M0 · independent of Task 2)

**Goal:** Real multi-file `.sea` parses and validates through `domainforge-core` 0.13.0 producing a stable `DomainModelRef`; every invalid input fails closed as `domain_model_error` with zero side effects (§7.0a, §12 M0 DomainForge bullets).

**Why this shape:** ADR-001: DomainForge owns `.sea` semantics, SEA Forge owns final authority. The adapter is a side-effect-free library boundary — `load_validate / evaluate / project` (§7.0a) — never the CLI, never a filesystem writer.

### Steps

1. New crate `crates/sea-forge-domainforge` depending on `domainforge-core = "=0.13.0"`, default features off (§6.3). Do NOT add this dependency to any v0.1 kernel crate.
2. Implement `SeaSourceSet` loading with the namespace-aware parser; enforce the descriptor's finite limits (source count, bytes, import depth, AST nodes) — breach = `domain_model_error`, never retry-weaker (§7.0a).
3. Build `DomainModelRef` exactly per §7.0a (`semantic_model_sha256` over the canonical 4-tuple; sorted `source_refs`/`concept_refs`; non-empty `validation_evidence_refs`).
4. Authority normalization table §7.0a (Reject/Deny→deny, Escalate→escalate, Allow→allow, NotApplicable→no candidate or deny-if-required), preserving the DomainForge trace as evidence.
5. Commit a real multi-file `.sea` fixture (authored for this repo; the v0.1 JSON `model.sea` stub MUST NOT enter this path — §7.0b).
6. Tests `tests/conformance_m0_domainforge.rs`: valid fixture → stable ref across runs; invalid syntax / unresolved import / semantic error / source-hash drift / unsupported version → `domain_model_error` before side effects; the five normalization fixtures.

### Gate

```bash
cargo test -p sea-forge-domainforge --locked && just proof
```

**Done when:** the valid fixture yields byte-identical `DomainModelRef` on repeat runs, and every invalid-input test asserts no file was written (teeth: temporarily writing a scratch file in the adapter makes the no-side-effect test fail).

**Redesign trigger:** `domainforge-core` 0.13.0's public API can't express a required call in-memory. Record it in `OPEN_QUESTIONS.md` with the exact missing API and stop — do NOT shell out to the `domainforge` CLI (§6.3 forbids it).

---

## Task 4 — M0d: Extension ABI and registry  (M0 · independent, small)

**Goal:** `ExtensionRegistry`, `ExtensionInstallRecord`, and the `ProjectionAdapter`/`ProjectionRecord` contracts of §7.0b validate, with built-in descriptors registered and imported extensions inert until adopted.

**Why this shape:** §2.2: plugin contracts "must be stable before plugins exist" or later plugins re-key records.

### Steps

1. In `crates/sea-forge-extension`: registry/descriptor/install-record types per §7.0b; persistence at `.sea-forge/extensions/registry.json` and `descriptors/<extension_id>@<version>.json`; compatibility checks (`min/max_kernel_version`, required record versions/surfaces).
2. `ProjectionAdapter` trait: `plan/project/validate/explain/rebuild`; `ProjectionRecord` with `rebuild_hash` per §7.0b. No adapter implementations yet beyond a test stub.
3. Tests: descriptor validation, empty-registry validation, authority-surface→policy-surface mapping, imported descriptor defaults to `status: disabled` until an adopt operation (§8.2 `install_extension|adopt_extension` rules land in Task 5's policy schema; stub the authority call here).

### Gate

```bash
cargo test -p sea-forge-extension --locked
```

**Done when:** §12 M0 "validate built-in ExtensionDescriptors and an empty extension registry" passes; an imported descriptor cannot reach `active` without an adopt record (teeth-check in test).

**Redesign trigger:** none plausible — pure data contracts.

---

## Task 5 — M0e: Authority fabric  (M0 · consumes Tasks 2 & 3)

**Goal:** One mediator gates every protected ingress with deterministic decisions, identity onboarding, verdict precedence, opaque constraints, fail-closed engines, and mirrored decision/audit logs — the full §12 M0 authority bullet list passes.

**Why this shape:** §2.1: "Authority is not an extension capability; it is the invariant substrate." §10.0: "Direct calls to lower layers are bugs."

### Steps

1. Extend `crates/sea-forge-authority`: `IdentityBinding` (roles incl. `R-AA`+sponsor rule), `AuthorityPolicyBundle` (all §7.0 required surfaces + `sod_rules`), `CanonicalActionRequest` (minimum `AuthorityRequest` + optional fields only), `AuthorityDecision` with `candidate_verdicts`/`winning_source`/`sandbox_class_granted`, `GovernanceVerdict`, `AuthorityAuditRecord`, `OpaqueConstraint` — shapes verbatim from §7.0.
2. Resolver: implement the complete pairwise typed matrix and deterministic
   fold: deny blocks; unresolved escalation blocks while retaining boundaries;
   boundaries intersect; explicitly permitted degraded controls accumulate; and
   allow is identity. Prove
   evaluator-order independence and the applicable commutative, associative,
   and idempotent properties. Missing required evidence ⇒ deny; all-escalate ⇒
   create/reference OpaqueConstraint and halt as escalate (§7.0).
3. Policy v0.2 schema §8.2: engines must declare `fail_mode: closed` for action gating (else `authority_engine_config_error`); the untrusted-argv0-on-`local` rule is a `schema_error`; all new `rules[].operation_kind` variants parse now (even where their executors land later) so one schema owns the file.
4. Wire the mediator as the single entry for every current ingress (CLI run/validate/recall/inspect). Define the opaque grant in `sea-forge-authority`; remove its dependency on `sea-forge-sandbox` by keeping lexical request validation in authority/core, then make runtime and sandbox depend on authority and require that grant at their public side-effect entry points. Bind the grant to the exact request, execution context, validity interval, sandbox class, approval, and compensating controls; reject mismatch, expiry, replay, or downgrade. Persist `.sea-forge/authority/{policy-bundles/<hash>.json, decisions.jsonl, audit.jsonl, opaque-constraints.json}` and the Task 4 extension registry/descriptors through the ledger before materializing compatibility views, with detectable failed/stale view state and byte-identical rebuild proof (Task 2).
5. Pre-action integrity sequencing per §10.0a: under `required_for_side_effects`, append intent/identity/policy/request/decision, force checkpoint + witness receipts BEFORE the side effect.
6. Engine adapters: `local` rules engine now; `domainforge` candidate via Task 3; `opa`/`governedspeed` as config-declared stubs whose *unavailability* paths are real (deny/escalate, never pass) — real transports are later plugins.
7. Tests `tests/conformance_m0_authority.rs` covering the §12 M0 authority bullets: onboarding fixtures (human/service/R-AA+sponsor; unresolved ⇒ escalate), same-request-twice determinism, generated-zone write / unknown host / private-network host / protected git commit / PR-merge-without-checks / dangerous shell / prompt-risk all fail closed with evidence, engine-unavailable fail-closed, complete resolver properties, opaque-constraint halt, unauthorized/forged/mismatched/expired/replayed/downgraded authority rejection, approval and degraded-control enforcement, and ledger-before-view failure/rebuild semantics for decisions/audit mirrors.

### Gate

```bash
cargo test -p sea-forge-authority --locked && cargo test --workspace --locked && just proof
```

**Done when:** the full M0 gate row of §17.1 passes (ledger+authority+DomainForge+ABI+graduation, minimum suite unchanged, "no ingress bypasses the mediator"). Teeth: adding a direct `runtime::execute` call in a test without a decision must panic/fail.

**Redesign trigger:** deterministic decision hashing conflicts with a candidate-verdict field that carries timestamps — exclude `recorded_at` from the decision hash input (mirror how the minimum spec hashes decisions), never drop determinism.

---

## Task 6 — M0f: `sea-forge migrate` — legacy import + case layout  (M0 closeout)

**Goal:** A v0.1 root migrates losslessly: `legacy_import` genesis entries commit every legacy file's bytes/path/size/hash, runs move under `.sea-forge/cases/<case_id>/runs/<run_id>/`, old IDs stay resolvable, and imported history reports `legacy_digest_only` (§7.0c Migration, §7.1, §12 M0 first bullet).

### Steps

1. `sea-forge migrate` in `crates/sea-forge-cli/src/commands/`: enumerate v0.1 files, emit genesis entries via Task 2's writer, sign initial global checkpoint, relocate run dirs per §7.1 without rewriting record bytes.
2. Assurance labeling: any inspect over imported records reports `legacy_digest_only`, never stronger (§10.0a assurance ladder).
3. Test: build a v0.1 fixture root (reuse the P1 demo run), migrate, verify: byte hashes committed, IDs resolvable, `sea-forge ledger verify` green, re-running `migrate` is a refused no-op.

### Gate

```bash
cargo test -p sea-forge-cli conformance_m0_migrate --locked && just proof
```

**Done when:** migration is lossless and idempotence-guarded; teeth: corrupting one legacy file byte after recording makes `ledger verify` flag the mismatch.

**Redesign trigger:** none plausible.

---

## Task 7 — M1: Jail sandbox backend  (E1)

**Goal:** An untrusted command under `jail` class cannot write outside its workspace (OS-enforced), jail unavailability refuses to run, and policy can never grant `local` to an untrusted argv0 (§10.1, §12 M1, §17.1 M1).

**Why this shape:** §15 graduation rule is the load-bearing invariant; §6.3: jail setup failure settles `rejected` basis `jail_unavailable` — "it MUST NOT fall back to `local` silently."

### Steps

1. In `crates/sea-forge-sandbox`: `SandboxClass` enum (`local|jail|microvm`), the `ExecutionSandbox` trait exactly as §11.2 (no checkpoint/restore — deliberately excluded), `local` backend = existing behavior.
2. `jail` backend: Linux Landlock via the `landlock` crate (ruleset from workspace root, ReadOnly/ReadWrite per grant; network default-deny, Landlock v4+ TCP where available); macOS `sandbox-exec` profile generation behind `cfg(target_os)`, with Linux CI marking Seatbelt tests *skipped, not passed* (§17.2).
3. Runtime selection in `crates/sea-forge-runtime`: exactly `sandbox_class_granted` — weaker class is debug-panic/release-refuse (§7.1 AuthorityDecision).
4. Preflight probe: policy referencing an unavailable class ⇒ `unsupported_sandbox_class_error`, nothing runs (§8.3, §13).
5. Tests `conformance_m1.rs`: escape attempt (write `../outside`) fails at the OS and settles `rejected` basis `jail_violation`; the same intent under `local` for an untrusted argv0 is unconfigurable (`schema_error` — already Task 5, re-assert here); jail-unavailable refusal.

### Gate

```bash
cargo test -p sea-forge-sandbox --locked && cargo test -p sea-forge-runtime --locked && just proof
```

**Done when:** §12 M1 proof passes on a Landlock-capable kernel; teeth: running the escape test with the jail ruleset commented out makes it fail (the write would succeed).

**Redesign trigger:** Landlock proves impractical for the workload — §0.8: the `ExecutionSandbox` trait is the isolation seam; a MicroVM backend replaces jail without kernel changes. Do not weaken `local`.

---

## Task 8 — M2a: CMMN-subset case engine  (E2 · the biggest task)

**Goal:** Cases run stages/plan-items driven by sentries over the ledger; the two §12 M2 scenario tests pass and every activation decision replays deterministically from the ledger alone.

**Why this shape:** §10.2: "Sentry evaluation MUST be a pure function of the case ledger… no hidden state." §5: if predicate needs grow, "extend the predicate vocabulary, never add hidden engine state."

### Steps

1. `crates/sea-forge-planner`: extend `PlanItem` per §7.1 (`item_kind`, `sandbox_class`, `parent_stage`, `markers`, `entry_criteria`/`exit_criteria`, `max_instances`); `Sentry` shape per §7.1; `depends_on` compiles to entry sentries at validation; static satisfiability check on the "on" graph (`plan_cycle_error`).
2. Case loop engine (in `sea-forge-planner` or a thin `sea-forge-case` module inside it — smallest diff wins): the §9.1 flow verbatim — decide-all authority up front, sentry re-evaluation after every appended event, milestone emission (`milestone_achieved` trace kind), repetition re-instantiation ≤ `max_instances` with fresh workspace per instance, required-item failure ⇒ case `terminated rejected` citing the item, auto-complete condition (§9.1), parked-active is a normal state.
3. Case states + reopen (authority-checked, evidenced), discretionary add (`sea-forge case add-task` — full validation + authority BEFORE joining; `plan_mutated` event), human-task item kind + `sea-forge task complete` (§7.1, §10.2).
4. Plan-proposal ingestion `sea-forge run --plan <file.json>` per §8.6 (schema, path/charset, satisfiability, per-operation authority; typed rejections). DomainForge binding: plans carrying `DomainModelRef`/concept refs reload the hash-pinned source set via Task 3 and resolve every concept before authority (§10.2 last bullet).
5. Exit codes per §10.9 (0/3/4/5/1/2) in `sea-forge-cli`.
6. Tests `conformance_m2.rs`: the full A/B(rep×2)/C/M scenario from §12 M2 including ledger-replay reproduction of every activation; discretionary-item allowed/denied copies; DomainModelRef binding valid/changed-source/unknown-concept; unsatisfiable sentry rejection; parked-case-is-not-failure; reopen authority.

### Gate

```bash
cargo test -p sea-forge-planner --locked && cargo test -p sea-forge-cli conformance_m2 --locked && just proof
```

**Done when:** the §12 M2 scenario passes AND replaying the ledger through the sentry evaluator reproduces the identical activation sequence (teeth: injecting one synthetic event into the replay input changes the output — proves the replay isn't a stub).

**Redesign trigger:** a needed activation condition can't be expressed as `on`+`if` over the ledger — extend the `if` predicate vocabulary in §7.1 (spec edit + fixtures in the same commit); never add engine-private state.

---

## Task 9 — M2b: Plan templates  (E8 · small, lands with M2)

**Goal:** `sea-forge run --template <name>@<version> --param k=v` instantiates deterministic CasePlans; substitution can never reach `kind`, `argv[0]`, IDs, or sentry structure (§7.6 PlanTemplate, §17.1 M2-templates row).

### Steps

1. Template load/store in `crates/sea-forge-planner`: YAML at `.sea-forge/templates/<name>@<version>.yaml`, immutable-once-referenced (hash pin; mismatch = `template_changed_error` — §10.6).
2. Typed parameter resolution; substitution-site restriction enforced at LOAD (a `${param}` in a forbidden position is `schema_error` at template load, never at run time — §12 M2-templates).
3. Instantiation output goes through the FULL §8.6 proposal path (zero privilege from templates); `template_ref` provenance into plan + envelope; ship built-in `sea_model_demo@0.1.0` from the slice demo plan.
4. Tests: same template+params twice ⇒ byte-identical CasePlans; forbidden substitution site rejected at load; missing required param = input error.

### Gate

```bash
cargo test -p sea-forge-planner template --locked && just proof
```

**Done when:** determinism and load-time restriction tests pass; teeth: moving a `${param}` into `argv[0]` in the fixture flips the test to the schema_error assertion.

**Redesign trigger:** none plausible.

---

## Task 9.5 — M2c: Settlement-criteria origin and provenance

(M2 closeout · blocks M3)

**Goal:** Close the missing derivation path between the requested job or
requirement and the SettlementCriteria used to judge work, while reusing the
implemented M0–M2 substrate and avoiding a parallel requirements or provenance
system.

**Why this shape:** The full spec already expects `criteria_ref`,
`criteria_sha256`, and `criteria_declared_at` in the M4a declaration protocol,
but no canonical criteria record currently defines those values. Task 10 would
otherwise spread that unresolved identity into ApprovalRequest and server
state. This task defines the smallest missing contract before those consumers
exist.

### Substrate inspection — mandatory first step

Before editing code, inspect and report the actual implementation of:

- `Case`, `CasePlan`, `PlanItem`, `Intent`, and `SettlementCriteria`;
- template loading, hash pinning, and deterministic instantiation from Task 9;
- ledger typed-record submission and immutable committed refs;
- current plan persistence/materialized views;
- current ID and canonical-hash helpers;
- migration behavior for v0.1 plans;
- any existing criteria IDs, origin refs, source refs, authorship fields,
  record hashes, declaration placeholders, or provenance types;
- authority validation before PlanItem activation.

Produce a `Substrate reconciliation` section in the task report.

The report MUST answer:

1. Can the existing Intent serve as the origin for the built-in demo?
2. Can the existing template record serve as an origin for template-created
   criteria?
3. Can existing ledger record refs and canonical hashing be reused directly?
4. Does the current code already have a generic source/provenance reference that
   should be extended instead of replaced?
5. Is a dedicated JobContract actually needed for any current path?
6. Can the criteria record live in existing core/settlement types and the
   existing ledger writer without a new crate or independent store?

Do not begin structural implementation until these questions are answered from
the repository.

### Steps

1. Reconcile the new spec §7.1a with the actual code:
   - reuse existing provenance/source-ref types when equivalent;
   - reuse existing canonical hashing;
   - reuse the ledger's typed-record commit path;
   - reuse existing planner and proposal validation;
   - do not add a new crate;
   - do not add a separate database;
   - do not add a second authoritative plan or criteria store.

2. Implement only the missing canonical contract:
   - a ledgered SettlementCriteriaRecord or an existing equivalent extended to
     satisfy the spec;
   - `PlanItem.settlement_criteria_ref`;
   - embedded-criteria/hash agreement;
   - non-empty attributable OriginRefs;
   - pre-execution declaration timing;
   - `criteria_provenance_error`.

3. Add a dedicated JobContract only for a path where inspection proves that no
   existing Intent, PlanTemplate, SpecPipelineStage, policy requirement, issue,
   or external requirement can satisfy the origin contract.
   - The built-in demo SHOULD normally reuse its Intent and built-in template.
   - A template-created plan SHOULD reuse the immutable template version and
     originating Intent.
   - Do not generate ceremonial JobContract records that restate existing text.

4. Integrate Task 9 templates:
   - same template version + typed params must yield identical criteria content,
     origin refs, criteria hash, and criteria-record content hash;
   - template substitution must not alter provenance structure or declared
     author identity;
   - template_ref remains the existing provenance link and is reused rather
     than copied into a competing model.

5. Integrate plan proposal and activation validation:
   - resolve criteria and origins before authority;
   - reject missing, stale, hash-mismatched, or post-execution criteria;
   - discretionary items pass the same validation;
   - no hidden planner state may substitute for persisted provenance.

6. Preserve legacy behavior:
   - read v0.1 embedded-only criteria as `legacy_unattributed_criteria`;
   - do not synthesize fake authorship, rationale, timestamps, or origin refs;
   - permit local legacy evaluation only when policy allows;
   - refuse strong settlement qualification for unattributed legacy criteria.

7. Add focused tests in the existing owning crates. Prefer extending
   `sea-forge-planner`, `sea-forge-settlement`, `sea-forge-core`, and
   `sea-forge-ledger` tests rather than creating a new crate:
   - every new settling PlanItem resolves to one committed criteria record;
   - embedded snapshot matches `criteria_sha256`;
   - every OriginRef resolves and hash-verifies;
   - same template+params produces identical criteria/origin hashes;
   - changing criteria or an origin changes the applicable hash;
   - missing origin fails before authority or side effects;
   - embedded-record mismatch fails before authority or side effects;
   - post-execution declaration fails;
   - legacy plan remains readable but cannot qualify strong;
   - a test asserts the built-in demo reused Intent/template origins and did not
     create a redundant JobContract.

8. Update the M2 conformance gate and documentation only after tests pass.
   Record the actual reused substrate in `.agents/CURRENT_STATUS.md`.
   Use `.agents/OBSERVED_DEBT.md` only for a real remaining mismatch.

### Gate

```bash
cargo test -p sea-forge-planner criteria_provenance --locked
cargo test -p sea-forge-settlement criteria_provenance --locked
cargo test --workspace --all-features --locked
just proof
devbox run -- just check
```

If the repository places the focused tests under different existing crates,
adjust the first two commands to those actual owners and record why in the task
report. Do not create a crate merely to preserve these example command names.

**Done when:** every new settling PlanItem resolves to an attributable,
pre-execution, hash-verified criteria record; the built-in and template paths
reuse existing Intent/template/ledger substrate; legacy records remain readable
without fabricated provenance; and deliberately removing an origin or
mismatching the embedded criteria causes failure before authority or side
effects.

**Teeth check:** modify one embedded criterion after the criteria record is
committed. The conformance test must fail with `criteria_provenance_error`
before any `authority_evaluated`, `command_started`, or workspace mutation
event.

**Redesign trigger:** inspection proves the existing ledgered CasePlan or another
existing canonical record already provides criteria identity, timing,
authorship, origin linkage, and immutable hash verification. In that case,
extend that record and add the missing reference/checks only. Do not introduce
SettlementCriteriaRecord or JobContract as duplicate storage merely to mirror
the specification's logical names.

---

## Task 10 — M3: Server, approvals, operator loop  (E3)

**Goal:** `sea-forge-server` runs concurrent cases, `escalate` becomes a TTL-bound ApprovalRequest resolvable via `sea-forge approve|reject`, and the §12 M3 + §13 concurrency/race/crash drills pass.

**Why this shape:** §6.1: kernel stays synchronous; Tokio only in the server, one run per `spawn_blocking`. Kernel traits never become `async fn`.

### Steps

1. First verify Task 9.5's implemented criteria contract and reuse its exact
   refs and validators. Implement `ApprovalRequest` per §7.2 in the existing
   settlement/evidence ownership boundary:
   - append-only, latest-line-wins resolution, no re-resolution;
   - bind `plan_item_id`, `criteria_ref`, `criteria_sha256`, and
     `criteria_record_hash`;
   - include nullable `job_contract_ref` only when the plan actually uses one;
   - revalidate the criteria record before approval or rejection;
   - stale or mismatched criteria refuse resolution with
     `criteria_provenance_error`;
   - add `SettlementCriteria.require_approval`;
   - emit evidence kind `approval`.
   Do not copy the criteria body into a second approval-owned source of truth.
2. New crates `crates/sea-forge-interface` (subscribe/notify/approve-reject plumbing) and `crates/sea-forge-server` (Tokio): run queue, `max_concurrent_runs`, Unix socket 0600 with the §11.1 NDJSON verbs (`submit`, `status`, `approve/reject`, `subscribe`), 10s per-request timeout, `notify_command` exec'd argv-style with JSON on stdin — failure logged and ignored (§10.3).
3. `server.yaml` config §8.1; dynamic reload between dispatches with last-known-good on invalid reload (§8.4); startup preflight §8.5 (socket, sandbox probe, notify argv0, integrity checkpoint, DomainForge version match).
4. CLI: `watch` (tails event stream), `resume <case_id>`, `runs --unsettled`, `tasks`, `approve|reject <run_id> <approval_id> [--note]` (unauthorized approver refused — approving is itself authority-checked); one-shot exit 5 for `awaiting_approval` (§9.2).
5. TTL expiry ⇒ `rejected` basis `authority_escalate_expired` (§14.2); approval/expiry race resolved by append order, loser is a visible no-op (§13); server crash leaves resumable self-describing run dirs, no auto-resume (§14.4).
6. Tests `conformance_m3.rs`: escalate→exit 5→approve→resume→accepted; TTL-expire copy; 8 concurrent submits ⇒ 8 uncorrupted run dirs + 8 valid `capabilities.jsonl` lines (§13 R-jsonl); reload valid/invalid; notify-failure-ignored; unauthorized approver; approval of the original criteria succeeds; mutating or superseding the criteria before resolution does not silently rebind the ApprovalRequest; a stale/mismatched criteria reference is refused and evidenced; the built-in approval flow does not create a redundant JobContract.

### Gate

```bash
cargo test -p sea-forge-server --locked && cargo test -p sea-forge-cli conformance_m3 --locked && just proof
```

**Done when:** §17.1 M3 row green including the concurrency test; the no-Tokio-in-kernel check from Task 1 still exits 0 (teeth: adding `tokio` to `sea-forge-planner`'s Cargo.toml makes that check fail). Every approval is bound to the exact immutable criteria record that caused the escalation.

**Redesign trigger:** the R-jsonl test fails at 8 concurrent runs — §5 already prescribes the fix: move the two shared files (`capabilities.jsonl`, `approvals.jsonl`) behind a single writer task. Do not switch to a database.

---

## Task 11 — M4a: Settlement declarations + capability promotion  (E4)

**Goal:** Independent, reliability-weighted `SettlementDeclaration`s gate capability promotion; `capability rebuild` is byte-identical-pure; `require_proven` denies with citation — the long §12 M4a fixture list passes.

**Why this shape:** §15: manufactured settlement is a primary threat; criteria hashes, immutable manifests, independent standing, and reliability weighting are *separate* controls — "passing one never substitutes for another."

### Steps

1. Inspect and reuse Task 9.5's actual criteria/origin record implementation.
   Do not define a second declaration-only criteria model.
   `SettlementDeclarationRequest` and `SettlementDeclaration` must carry and
   verify the existing `criteria_ref`, `criteria_sha256`,
   `criteria_record_hash`, declaration time, and resolved origin chain. Then
   implement the remaining §7.2.1 shapes (fixed-scale decimal strings for
   reliability fields; `declaration_hash` excludes only itself); `declare()`
   trait; `local` adapter (strength `local` always); and `swe_seed` adapter
   contract with a test-double transport (real service = §17.4 integration
   test, behind a feature/env flag). SWE_SEED's proof gating supplies independent
   verification; godspeed_agent remains a GovernedSpeed candidate-verdict engine
   and downstream capability-status consumer. Preserve §7.3's
   `attempted < demonstrated < proven < metabolized` vocabulary.
2. Qualification predicate exactly per §7.2.1: a declaration contributes
   qualifying weight only when the criteria record resolves and hash-verifies;
   its embedded PlanItem snapshot agrees; every OriginRef resolves; derivation
   authorship is attributable; criteria predate execution; and the criteria are
   not `legacy_unattributed_criteria`. Rejected-strong increments
   `regression_weight` only; escalated contributes nothing until resolved. A
   missing origin chain preserves the minimum event but contributes zero
   qualifying weight.
3. `crates/sea-forge-capability`: `CapabilityPromotionPolicy` snapshots (`policies/<sha256>.json`), `CapabilityRecord` per §7.3 with the FIXED confidence arithmetic (fixed-point, 6 decimals, clamp, zero-denominator rule); default v0.2 policy thresholds from §7.3; contraction with recorded reason; `capability list|show|rebuild` — rebuild pure over `capabilities.jsonl` + `declarations.jsonl` + policy snapshots, byte-identical modulo `rebuilt_at`.
4. Policy: `settlement_authorities[]`, `settlement.required_strength/min_reliability_weight/promotion_policy_ref`, `rules[].require_proven` (§8.2); config error classes `settlement_authority_config_error`/`settlement_integrity_error` (§8.3); strong-outage behavior per §13/§14.9 (event inspectable, zero weight, strong-required case can't complete; resume appends exactly one declaration).
5. Tests `conformance_m4a.rs`: every §12 M4a bullet — raw counts, local-declaration-zero-weight, post-hoc criteria, self-declaration, gameable/low-attribution weight, three-qualifying-declarations promotion, identical-variation-no-coverage, regression/revocation/policy-change contraction (separate copies), rebuild byte-identity, require_proven denial citing record + policy hash; strong declaration with unresolved criteria origin is non-qualifying; strong declaration with criteria-record hash mismatch is non-qualifying; strong declaration over `legacy_unattributed_criteria` is non-qualifying; valid declaration traverses criteria → origin → Intent/template/spec source; declaration implementation reuses the Task 9.5 record rather than persisting a duplicate criteria body as authority.

### Gate

```bash
cargo test -p sea-forge-settlement --locked && cargo test -p sea-forge-capability --locked && just proof
```

**Done when:** §17.1 M4a row green; teeth: perturbing one declaration's weight in the rebuild fixture changes the rebuilt bytes (proves rebuild reads sources, not caches).

**Redesign trigger:** none plausible — the arithmetic and predicates are fully specified.

---

## Task 12 — M4b: Governed semantic memory  (E7)

**Goal:** Deterministic extraction fills `memory/items.jsonl`, SQLite FTS is a pure rebuildable index with a linear-scan fallback, and recall is an authority-scoped operation leaving `recall` evidence (§7.5, §10.5, §12 M4b).

### Steps

1. In `crates/sea-forge-capability`: `MemoryItem` per §7.5 (provenance REQUIRED non-empty; `dedup_key` update semantics); fixed extraction rule set as the final pipeline step of every settled run — never fails the run (§10.5).
2. `rusqlite` FTS index at `memory/index.sqlite`, `sea-forge memory rebuild`, stale/missing ⇒ linear scan of `items.jsonl` (slower, never wrong).
3. `recall_memory` operation kind + `memory_scope` policy rules (`own`/`entity:<id>`/`any`; default deny — §8.2); in-case recall writes `recalled/<plan_item_id>.jsonl` + `recall` evidence; plan-time consultation must inject the same operation (no private read channel — §10.5); upgrade the slice `recall` CLI to FTS with `--kind`, contract preserved; recall capped at 50 items / 1000-char statements; imported memory excluded absent an explicit foreign-entity rule.
4. Tests `conformance_m4b.rs`: two-entity extraction dedup + provenance; own-scope isolation + evidence naming; cross-entity denial (run rejected, denial evidenced); index-delete ⇒ identical fallback results; extraction-error-never-fails-run.

### Gate

```bash
cargo test -p sea-forge-capability memory --locked && just proof
```

**Done when:** §17.1 M4b row green; teeth: deleting `index.sqlite` mid-test yields identical recall output via fallback.

**Redesign trigger:** deterministic extraction is too shallow to be useful — §5 prescribes it: LLM-assisted extraction as untrusted *proposals* through the §8.6 pattern, never in-kernel writes.

---

## Task 13 — M5: Spec-to-code pipeline + DomainForge projections  (E5)

**Goal:** The ADR→…→acceptance chain runs as ordinary case plans with hash-linked stages, generated zones are read-only, regeneration is deterministic, and `sea-forge project` emits validated CALM+RDF ProjectionRecords from one `DomainModelRef` — §12 M5 passes.

### Steps

1. New crate `crates/sea-forge-spec-pipeline`: `SpecPipelineRun`/`SpecPipelineStage` per §7.8; stage-order + start-later-only-if-priors-validate rule (§10.7); proof-classification ceiling (`generated-contract` until last-mile+runtime+acceptance accepted); quarantine with provenance; `nondeterministic_projection` rejection.
   - Accepted ADR/PRD/SDS/SEA or other requirement-bearing stages MAY serve
     directly as OriginRefs for SettlementCriteriaRecord derivation.
   - Reuse the existing stage record and digest. Do not create a parallel
     JobContract merely to restate an accepted specification stage.
   - When a dedicated JobContract is necessary, it references the accepted stage
     rather than copying it without provenance.
2. Generated-zone guard: direct edits to `src/gen`/AST/IR/manifest/fixtures are denied `run_spec_pipeline` operations, basis `generated_zone_direct_edit` (§10.7) — enforce via Task 5's file-surface rules.
3. `.sea` synthesis adapter (distinct from the DomainForge adapter; output MUST be re-parsed/validated by Task 3 before becoming a `DomainModelRef` — §7.0b).
4. `sea-forge project <case_id..>`: governed projection run (plan = project op, settlement = output validates); in-memory DomainForge projections, CALM + RDF required, per-target `ProjectionRecord`s sharing one `DomainModelRef`; prove no DomainForge-owned filesystem/network/CLI side effect (§10.4a); remote-delivery adapters get retry+DLQ semantics (later plugins).
5. Declarative Evaluator form only (`kind: predicate` — §7.6; command form is Task 15 per Appendix A) wired into `SettlementCriteria.evaluator`.
6. Policy `rules[].operation_kind: run_spec_pipeline|run_projection` enforcement (schema from Task 5).
7. Tests `conformance_m5.rs`: full small-context pipeline hash-chain; direct-edit denial; regeneration byte-identity; classification ceiling; two-case `project` ⇒ CALM/RDF validate + rebuild byte-identically; quarantine completeness; projection-is-a-full-case; no-side-effect proof; criteria derived from an accepted spec stage resolve directly to that stage's hash-linked record; changing the source stage invalidates or supersedes the dependent criteria; no duplicate requirement store is introduced by the spec pipeline.

### Gate

```bash
cargo test -p sea-forge-spec-pipeline --locked && cargo test -p sea-forge-domainforge projection --locked && just proof
```

**Done when:** §17.1 M5 row green; teeth: touching one input byte of an accepted stage forces re-run and changes the chain hash assertion.

**Redesign trigger:** a DomainForge projection target is nondeterministic across identical inputs — settle `rejected` basis `nondeterministic_projection` per §7.8 and pin/report the DomainForge issue; never fuzzy-match outputs.

---

## Task 14 — M6: Federation prep (SeaCell)  (E6 · cheap, additive)

**Goal:** `cell_id` stamps new records, signed-by-hash export/import bundles work, and imported evidence never leaks into local capability records (§7.4, §12 M6).

### Steps

1. `sea-forge-core`: `cell_id` field (absent = local legacy, valid); `crates/sea-forge-cell` (new): `cell.json` generation, tar bundle export with `manifest.json` per-file sha256, atomic-reject import to `.sea-forge/imported/<cell_id>/runs/` (§14.8 — no partial import).
2. `EventSink` trait in `crates/sea-forge-trace` + JSONL implementation (NATS is a later adapter behind it — §2.3 E6). Shape sink events so the agent-memory-ledger NATS bridge can consume them without re-keying: include (or make derivable) the SEA event contract fields `event_id, trace_id, correlation_id, causation_id, idempotency_key, source_agent, occurred_at, schema_version`, and pick event kinds that map onto the `sea.{domain}.{action}.{qualifier}` subject taxonomy (see ecosystem map). Do NOT add a NATS client or publish anything — JSONL only; the bridge is external.
3. Bundles MAY carry templates/environments; imported ones require explicit `template adopt` before instantiable (§10.6).
4. Tests `conformance_m6.rs`: export 2 runs → fresh-root import verifies hashes; local capability counts unchanged; tampered bundle rejected atomically; imported template not instantiable pre-adopt.

### Gate

```bash
cargo test -p sea-forge-cell --locked && just proof
```

**Done when:** §17.1 M6 row green; teeth: flipping one byte in an imported run file fails the whole import.

**Redesign trigger:** none plausible.

---

## Task 15 — M7: Environment contracts + command evaluators  (E9)

**Goal:** `EnvironmentSpec` declares sandbox content orthogonal to isolation and permission, command-form evaluators score inside the episode, and batch criteria quarantine failures — §12 M7 passes.

### Steps

1. `crates/sea-forge-sandbox`: `EnvironmentSpec` per §7.6 (YAML store, hash-pinned immutable; mismatch ⇒ `environment_unavailable`); `base` materialization at prepare; `provides.commands` intersection semantics with policy `environment:` matching (§7.6 — three orthogonal axes). Compatibility constraint (ecosystem map): DomainForge ADR-012 `Cell` declarations project execution environments from `.sea` (Mount→sandbox mount policy, Endpoint/NetworkFlow→deny-by-default network allow-list, Credential→authority contract, never a literal value). Keep `EnvironmentSpec` field semantics compatible with that Cell-IR mapping so a future `cell→EnvironmentSpec` projection adapter (a plugin behind Task 4's `ProjectionAdapter` ABI) needs no schema break. Building that projection is NOT in scope; conflicting semantics are.
2. Command-form Evaluator (`kind: command`, `score_from: exit|stdout_float`) running in the same sandbox/episode under full authority; score recorded in settlement basis as `evaluator_score:<env>.<name>=<value>` (§10.6) — verification evidence, never settlement standing.
3. Batch criteria: `records`/`per_record_evaluator`/`min_pass_ratio`; failures to `quarantine/<plan_item_id>.jsonl` with `{record, score, evidence_ref}` (§7.6).
4. `sea-forge env list|show`; ship `demo_env@0.1.0` fixture.
5. Tests `conformance_m7.rs`: the §12 M7 scenario verbatim (0.8 ratio, 10 records, 2 vs 3 planted failures; intersection grant; tampered file ⇒ nothing executes) + the three-axis independence test (§17.1 M7).

### Gate

```bash
cargo test -p sea-forge-sandbox environment --locked && just proof
```

**Done when:** §17.1 M7 row green; teeth: the 3-failure batch fixture rejects while the 2-failure one accepts.

**Redesign trigger:** declarative+command evaluators too weak — §5 already resolves it: evaluators stay sandboxed commands whose exit/stdout is the score; same governance, more power. No new mechanism.

---

## Task 16 — M8: Artifact-to-IP pipeline  (E10)

**Goal:** Artifacts carry four independent dimensions — type/profile, recognized maturity stage, lifecycle status, identity/attestation status (§7.9) — and recognized maturity moves only along `cognitive→intellectual→product→capital` via governed `synthesize`/`productize`/`capitalize` TransitionTokens in explicit `derive` (content-changing, new artifact version) or `promote` (byte-identical) mode, with versioned gate profiles, no teleportation, separation-of-duty capitalization backed by out-of-case value evidence, and fail-closed rebuildable projections — §12 M8 and §17.1 M8 pass. Source of truth for all semantics: spec-full §2.3 E10, §7.8a, §7.9, §8.2 (`transition_artifact_stage`/`attest_artifact_identity`), §10.8, §12 M8, §17.1 M8. Do NOT invent ontology; if the spec and this task disagree, stop and file `.agents/OPEN_QUESTIONS.md`.

**Why this shape:** M8 is a recognition overlay on completed M0–M7 substrate, not a new engine. Everything it needs — ledger identity/append, authority mediation, criteria, approvals + SoD, settlement strength, evaluators, DomainModelRef semantic refs, projection rebuild conventions, case/PlanItem execution — already exists and is conformance-green. The crate adds only artifact-specific validation and token/projection logic.

### Mandatory substrate inspection (report before coding)

Before writing any code, locate and record (paths + type names) in the task's substrate-reconciliation note: `ArtifactDescriptor`/`artifact_id`/`pre_mint_identity` (sea-forge-core `types.rs`, evidence crate); canonical encoding/hashing (jcs-nfc-v1/sha256-v1, ledger crate); ledger record identity + append path; authority mediator / `AuthorizedAction` and existing `transition_artifact_stage`/`attest_artifact_identity` operation kinds; `SettlementCriteriaRecord` + OriginRefs; approval records + SoD checks; `SettlementDeclaration` strength/reliability; `EnvironmentSpec`/`Evaluator` (sea-forge-sandbox); `DomainModelRef`/concept refs (domainforge crate); projection rebuild conventions (capability/domainforge crates); CasePlan/PlanItem + CLI command registration; typed error conventions. Reuse each unchanged; where an equivalent exists under another name, document the mapping — never rename or duplicate.

### Steps

1. New crate `crates/sea-forge-artifact-ip` containing ONLY missing M8 logic: additive identity types (`lineage_id`, `content_identity`, `descriptor_hash`, `identity_scheme`, `legacy_pre_mint_identity`, `declared_stage` per §7.8a); `ArtifactRegistrationRecord` and `TransitionToken` per §7.9; recognized-stage reconstruction; legal-edge + derive/promote validation; lineage DAG validation; `ArtifactGateProfile` binding (generic invariants in Rust; type-specific fitness delegated to M7 Evaluators — no second evaluator engine); artifact-state and capital projection rebuild. No new authority system, approval record, criteria store, settlement protocol, semantic identity system, ledger, mutable artifact DB, migration, or extra artifact ID.
2. Identity compatibility (§7.8a): existing `artifact_id` stays the identity of one immutable artifact version; `pre_mint_identity` is preserved verbatim as `legacy_pre_mint_identity` and never used as v0.2 content identity; `content_identity` hashes only `{artifact_type, content_sha256}`; stage/owner/license/review/lifecycle/semantic/attestation state never enter it; a byte change ⇒ new artifact version + new content identity. No historical record is re-keyed or rewritten; no migration.
3. Registration overlay (§10.8): registration only from durable work-product `ArtifactDescriptor`s in verified run evidence — never stdout/stderr, uncommitted files, narrative claims, or arbitrary paths. Historical descriptor `stage` retained as `declared_stage`; recognized stage starts `cognitive` absent a verified token chain; no fabricated historical tokens; content-dedup by `content_identity` allowed while distinct provenance instances stay distinguishable.
4. Commands `sea-forge artifact synthesize|productize|capitalize` as ordinary case operations (existing CasePlan/PlanItem + CLI registration). `synthesize` = cognitive→intellectual, `productize` = intellectual→product (each derive or promote per whether bytes change), `capitalize` = product→capital, always promote (byte-identical). There is NO `refine` verb and no alias for it. No-teleportation + all §7.9 invariant validation runs before any transform side effect, workspace mutation, result registration, or token commit. Derivation: new artifact_id/content_identity, `derived_from` + source IDs separate from supporting evidence, sources untouched. Promotion: artifact_id + content_identity preserved.
5. Policy + gates: extend `transition_artifact_stage` rules per §8.2 (transition_kind, mode restriction, gate_profile_ref, required settlement strength, qualifying value-evidence kinds; capitalization `requires_approval: true`, promote-only, ≥1 value-evidence kind) and `attest_artifact_identity` (identity status only, `degraded_mode`). Gate profiles per §10.8 reference M7 evaluators, DomainModelRef semantic anchors, rights/review state, and strong-settlement requirements through immutable refs — reuse the one criteria truth, the one mediator, the existing approval + SoD records. Reuse/value evidence is immutable source references from outside the originating case; never an authoritative manual `reuse_count` or mutable `current_stage` field (projections may display derived values).
6. IFL attestation adapter: flips identity status `pre_mint→attested` only; never satisfies a maturity gate. Required-attestation policy + outage ⇒ reject before capitalization; pre-mint-only policy ⇒ accept with explicit `degraded_controls` in the token.
7. Projections: `.sea-forge/artifacts/catalog.jsonl` (artifact state) and `.sea-forge/ip/capital/<artifact_id>.json` rebuild byte-identically from ledgered registration records + accepted tokens + referenced gate/authority/criteria/settlement/approval/rights/semantic/attestation/value-evidence sources, per existing projection rebuild conventions; fail closed on incomplete, forked, cyclic, hash-invalid, or missing-source chains; contain no independent mutable fact.

### Focused tests

`crates/sea-forge-artifact-ip/tests/conformance_m8.rs` implementing the §12 M8 scenario in full (naming per existing `conformance_m*.rs` files, reusing existing fixtures/helpers):

- Registration/compatibility: work-product descriptor registers; stdout/stderr cannot; demo `model.sea` keeps original descriptor, `declared_stage: intellectual`, and legacy pre-mint identity; recognized stage starts cognitive; P1–P4b unchanged.
- Content-preserving promotion: `model.sea` cognitive→intellectual via promote; artifact_id + content_identity unchanged; one new accepted token.
- Genuine synthesis: separate fixture composes cognitive source(s) → new intellectual artifact with new identities, correct `derived_from`/parent-token lineage, sources unchanged.
- Productization: full product-contract metadata + accepted evaluator evidence required; content-changing packaging ⇒ new derived product version; promote allowed only when bytes already carry the full contract.
- Capitalization: byte-identical; requester ≠ approver, authority-checked; semantic anchors via DomainModelRef; rights/review + strong settlement pass; value evidence from outside the originating case; quality-only rejected; approval-only rejected; capital rebuild byte-identical.
- No-teleportation/lineage: cognitive→product, intellectual→capital, stale `from_stage` all reject **before** side effects (assert no transform execution, no workspace mutation, no result registration, no token commit); removed middle token, tampered parent token, missing parent, cycle, conflicting fork ⇒ typed failures; edited materialized stage/reuse-count/state/capital files do not alter rebuilt truth.
- Identity/attestation: content-changing promote and content-changing capitalize rejected; IFL outage under required policy rejects; pre-mint-only policy succeeds only with explicit compensating controls in the token; attested cognitive artifact remains cognitive.

### Gate

```bash
cargo test -p sea-forge-artifact-ip --locked && just proof
```

**Done when:** §17.1 M8 row green; P1–P4b and all M0–M7 gates unchanged and green.

**Teeth:** (a) deleting one middle TransitionToken from the fixture makes capital reconstruction fail the complete-chain invariant; (b) quality-only capitalization rejected; (c) approval-only capitalization rejected; (d) content-changing capitalization rejected; (e) hand-editing the materialized capital/state file does not survive rebuild; (f) attestation without a passed maturity gate leaves recognized stage unchanged.

**Redesign trigger:** an existing M0–M7 type cannot express a required reference (e.g., no immutable rights-profile snapshot exists) — file `.agents/OPEN_QUESTIONS.md` and extend additively; never fork a parallel record.

**Substrate reconciliation:** commit the inspection report (paths, reused types, name mappings, deliberately rejected duplicates) with the milestone; no status claim ahead of a passing test.

---

## Task 17 — Definition-of-Done sweep  (closeout)

**Goal:** Every unchecked box in spec-full §18 is verifiably true, including the cross-cutting checks no single milestone owns.

### Steps

1. Version-skew test: 0.2 records with additive fields readable by 0.1 readers (§18).
2. Confirm the no-Tokio-kernel check (Task 1) runs in `just check` or CI.
3. Execute and document the server crash-recovery drill once against a real run (§13/§18) — record it in `.agents/CURRENT_STATUS.md`.
4. Semantic-envelope conformance check: validate a produced `semantic-envelope.json` against the CEP-0008 schema fixture at `~/projects/hassos-addon-agent-memory-ledger/tests/fixtures/cep/semantic-envelope.schema.json` (copy the schema into `tests/fixtures/` here — do not path-depend on a sibling repo). If the v0.1/v0.2 envelope diverges from CEP-0008, file the divergence in `.agents/OBSERVED_DEBT.md` with the field diff — do NOT silently change the envelope shape (that is a spec change with version bump).
5. Walk §0's eight questions and §18's checklist; fix or file anything unanswerable; update `CURRENT_STATUS.md` and check off §18 items only after their tests pass (no claim ahead of proof).

### Gate

```bash
cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && just proof && devbox run -- just check
```

**Done when:** every §18 box checkable with a pointer to a passing test or documented drill.

**Redesign trigger:** none plausible.

---

## Final acceptance checklist (whole plan)

- [x] Workspace builds as graduated crates; pre-existing 35 tests + P1–P4b unchanged *(Task 1)*
- [x] All §12 M0 ledger fixtures pass; every tamper class detected with typed reason *(Task 2)*
- [x] Real `.sea` validates via `domainforge-core` 0.13.0; invalid inputs fail closed, zero side effects *(Task 3)*
- [x] Extension registry/descriptors validate; imported extensions inert until adopted *(Task 4)*
- [x] M0 authority gate green: determinism, fail-closed engines, precedence, opaque constraints, no ingress bypass *(Task 5)*
- [x] `sea-forge migrate` lossless; imports report `legacy_digest_only` *(Task 6)*
- [x] Jail escape blocked at OS; untrusted-argv0-on-local is schema_error; unavailability refuses *(Task 7)*
- [x] §12 M2 scenario passes; sentry replay deterministic from ledger alone *(Task 8)*
- [x] Template instantiation byte-deterministic; forbidden substitution rejected at load *(Task 9)*
- [x] Every new settling PlanItem resolves to an attributable, pre-execution,
  hash-verified criteria record; built-in/template paths reuse the existing
  Intent/template/ledger substrate; no redundant JobContract, criteria store,
  or crate was introduced *(Task 9.5)*
- [x] Escalate→approve→resume and TTL-expiry flows pass; 8-way concurrency uncorrupted; approvals bind to and revalidate the exact criteria record *(Task 10)*
- [x] Promotion requires qualifying independent declarations; rebuild byte-pure; require_proven cites record+policy; unresolved, mismatched, or legacy-unattributed criteria cannot qualify *(Task 11)*
- [x] Scoped recall with evidence; FTS index pure + fallback-equivalent *(Task 12)*
- [x] Pipeline hash-chain + determinism + classification ceiling; CALM/RDF projections rebuild byte-identically *(Task 13)*
- [x] Bundles export/import with atomic rejection; no capability leakage *(Task 14)*
- [x] Environment/evaluator/batch semantics per §12 M7; three-axis independence *(Task 15)*
- [x] No-teleportation + approved capitalization + capital rebuild purity *(Task 16)*
- [x] `cargo clippy … -D warnings` exits 0; `devbox run -- just check` green *(all)*
- [x] spec-full §18 checklist fully checked, each item pointing at a passing test *(Task 17)*
- [x] `.agents/CURRENT_STATUS.md` reflects reality — no status ahead of a passing test.

## Guardrails (do not violate)

- **Never modify P1–P4b or the minimum-spec conformance tests to make a milestone pass** — "a milestone that cannot pass the minimum spec's proofs P1–P4b unchanged has broken the kernel and MUST be rejected" (spec-full §0.3).
- **No fallback across trust boundaries:** jail→local, strong→local settlement, witnessed→local assurance, FTS→(anything but the specified linear scan) — each downgrade path in the spec is explicitly forbidden; unavailable means halt/deny/escalate with evidence.
- **Kernel crates never gain Tokio or `async fn` traits** (spec-full §6.1, §18); async lives only in `sea-forge-server` (and, if ever needed, an isolated MicroVM adapter in `sea-forge-runtime`).
- **No new authorization stack:** every new protected surface extends the one mediator's resource classes + policy schema + tests (§10.0). A second permission system is a rejected design.
- **Plugins/projections never own truth** (§2.2): anything they persist must be rebuildable from source records + descriptor versions or captured as evidenced artifacts.
- **Do not build the non-goals** (§2.4): no UI, no DMN/CaseTask, no vector DB, no NATS dependency, no EnvHub, no marketplace, no MicroVM (roadmap seams only).
- **No sibling-repo dependencies:** the ecosystem map is contract intelligence, not a build graph. Never add a path/git dependency on SWE_SEED, godspeed_agent, agent-memory-ledger, or Context_Kernel; interaction is via the `SettlementAuthority`, `Evaluator`/policy-engine, `EventSink`, and federation-bundle seams, with copied-in schema fixtures where a contract needs testing.
- **Substrate before structure:** inspect the current implementation before
  adding a type, crate, trait, file, store, or adapter. The plan describes
  required contracts, not permission to duplicate working substrate.
- **No ceremonial JobContracts:** a dedicated JobContract exists only when
  existing Intent/template/spec/policy/issue records cannot express the job with
  sufficient identity, attribution, immutability, and upstream linkage.
- **One criteria truth:** approval, settlement, declaration, capability, and
  memory consumers all resolve the same canonical criteria record. None may
  persist an independently mutable criteria copy.
- **No rewrite for terminology:** when existing code satisfies the contract
  under another name, document the mapping and extend minimally rather than
  renaming or replacing it.
- Scope deviations, missing-API blockers, and debt discovered mid-task go to `.agents/OPEN_QUESTIONS.md` / `.agents/OBSERVED_DEBT.md` in the discovering commit — not silently absorbed.
- Commit hygiene: one milestone-task per commit series; the mechanical Task 1 graduation stays in its own commit(s) with zero behavior change.
