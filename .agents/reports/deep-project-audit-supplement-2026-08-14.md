# SEA-Forge Rust Audit Supplement — Semantic Layer, Rust Surfaces, Round-Trips

Supplement to `deep-project-audit-2026-08-14.md` (base audit). This run covers **only what the base audit did not**: the SEA/DomainForge semantic layer, `.sea` round-trip and canonicalization invariants, systematic Rust-surface sweeps (integers, indexing, recursion, allocation), and the extension/self-model crates. Findings here cross-reference base findings (F-01…F-25) instead of re-reporting them.

| Item | Value |
| --- | --- |
| Repository / revision | `/home/sprime01/projects/sea-rs` @ `df80d73` (same working tree as base audit) |
| Date | 2026-08-14 |
| Harness | `.agents/reports/audit-harness-2026-08-14/audit2/` (executes against the live crates; run with `CARGO_TARGET_DIR=<outside-repo> cargo run`) |
| Semantics contract sources read | `.agents/specs/*`, `.sea/interaction/interaction-model.sea` (canonical interaction model), `models/*.sea`, domainforge-core 0.16.0 grammar (`grammar/sea.pest`) and authority subsystem, extension/self-model/spec-pipeline/domainforge crate sources + conformance tests |
| Not covered by either audit | Landlock runtime enforcement on exotic kernels; workbench e2e-real; Windows path |

Per the supplement's taxonomy, each finding states whether the root problem is: **Rust implementation**, **parser/serializer**, **semantic transformation**, **invariant enforcement**, **architecture/integration boundary**, **specification ambiguity**, or **build/dependency/runtime** — and which of the five defect categories it falls into (defect in SEA-Forge / defect implementing an existing semantic contract / spec inconsistency / intentional limitation / model unable to represent semantics).

---

## Findings

### SUP-01 — `.sea` stack exhaustion: ~7 KB of nested unary expressions aborts the process through `load_validate`

**Category of defect:** defect in SEA-Forge (missing depth budget) interacting with external-dependency behavior
**Root cause:** parser boundary (external `domainforge-core` pest parser is recursive-descent with no depth cap) + Rust implementation (SEA Forge's own `count_expression_nodes` recursion runs *before* the `MAX_AST_NODES` budget can reject)
**Severity:** Medium-High (uncatchable process abort — `SIGABRT` — on any thread that validates a crafted model)
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-domainforge/src/lib.rs:122-199` (`load_validate`: the only pre-parse bound is `MAX_AGGREGATE_BYTES = 1 MiB`; the AST budget check at :195 runs after `parse_source` and after the recursive count at :189-193)
- `crates/sea-forge-domainforge/src/lib.rs:390-460` (`count_ast_node`/`count_expression_nodes` — recursion depth proportional to input nesting)
- External: `domainforge-core-0.16.0` `grammar/sea.pest` — `unary_expr = { ("-" ~ unary_expr) | cast_expr }` and `primary_expr = { "(" ~ expression ~ ")" | ... }` (one byte per recursion level); pest has no built-in recursion limit.

**Evidence (harness `audit2`, child processes)**

```
depth=   1000  exit_code=Some(0)  → Err("semantic validation failed: p: Policy 'p' evaluation is UNKNOWN (NULL)")  ← parses, recursion survives
depth=  10000  exit_code=None signal=Some(6)  ← SIGABRT (stack overflow) — file is ~7 KB, far under the 1 MiB cap
depth=  50000 / 100000 / 300000  signal=Some(6)
```

Input: `@namespace "t" … policy p as: ----…-1` (N dashes).

**Execution path / reachability**
`load_validate` is reached from: authority policy ingest (`sea-forge-authority/src/lib.rs:857-885` — reads the `.sea` named by the bundle's `domain_model` source at policy-load time; the base audit's F-16 showed the server honors client-named policy paths), self-model load (`sea-forge-self-model/src/lib.rs:152`; assets compile-time-pinned today), and CLI `project` (`commands/project.rs:220`, workspace `.sea` files). A stack overflow aborts the **whole process** (Rust turns it into `abort`), so on the server this escapes the normal handler-panic containment (base audit verified panics are contained via `spawn_blocking` + join-error handling — an abort is not). Any trust path that delivers foreign `.sea` content (federation bundle import hash-verifies against the bundle's own manifest — the manifest travels with the content) reaches this.

**Violated invariant**
Finite-limits contract (spec-full §7.0a): the limits exist precisely to bound "pathological CPU or memory consumption" — but nesting depth is enforced only *after* the recursion that consumes it.

**Counter-evidence considered**
`MAX_SOURCE_COUNT=64`, byte cap, and `MAX_IMPORT_DEPTH=16` (memoized, cycle-detected — sound) all fire before/around this; import-graph recursion is bounded by module count. Only *expression nesting* is unbounded pre-crash. A depth-1,000 model parses and is rejected semantically — the failure is purely depth-proportional.

**Recommended remediation**
Enforce a maximum expression/parse nesting depth *before* or *during* parsing: either a depth-limited walk rejecting early in `count_expression_nodes` (depth counter, bail at e.g. 256), or a pre-parse lexical nesting check (paren depth, unary-chain length) in `load_validate`. Upstream (domainforge-core) should carry the limit; SEA Forge needs its own guard regardless.

**Regression test**
The harness `nest` subcommand verbatim: depth ≤ 1,000 must return a typed error; depth 10,000+ must not abort the process (typed `domain_model_error`, not `SIGABRT`).

---

### SUP-02 — Equivalent entry spellings produce different semantic-model identities

**Category:** defect implementing an existing semantic contract (DomainForge normalizes entry spellings; SEA Forge hashes the raw string)
**Root cause:** semantic transformation / canonicalization (+ specification ambiguity: no rule pins identity spelling)
**Severity:** Medium
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-domainforge/src/lib.rs:253-264` — `parse_options_sha256` hashes `entry_path: source_set.entry_uri` **verbatim**; `:266-273` — `semantic_model_sha256` includes `parse_options_sha256`.
- The same comment block at `:316-320` documents that callers may supply alternate spellings (`./entry.sea`) which DomainForge normalizes.

**Evidence (harness S3)**
```
entry "model.sea"   → semantic_model_sha256 = faafcdde…
entry "./model.sea" → semantic_model_sha256 = 033e4bc8…   (same file content, same resolved graph, both accepted)
```

**Impact**
`DomainModelRef` equality is what downstream machinery compares: spec-pipeline prerequisite validation (`validate_stage_prerequisites`, `output.domain_model_ref != input.domain_model_ref` → quarantine), rebuild hashes (`compute_rebuild_hash`), authority determinism context (`domain_model_ref` in the canonical action). The *same model* referenced with two spellings pins under two identities — a stage chain mixing spellings quarantines despite byte-identical inputs, and provenance/proven records under one spelling don't match the other.

**Counter-evidence considered**
Whether identity should be spelling-sensitive is genuinely ambiguous — the spec does not define it. DomainForge's resolved import graph (used for `max_import_depth`) treats both spellings as the same module; the graph and projections (harness S7) are identical. That internal inconsistency (resolution says same; identity says different) is the defect-shaped part; the rest is a spec question to settle deliberately.

**Recommended remediation**
Hash the *normalized* entry (the closure root's logical id from the resolved envelope, already available) instead of the raw caller string; or reject non-canonical entry spellings outright. Either is defensible; document the choice.

**Regression test**
`load_validate` twice with `x.sea` vs `./x.sea` must either fail the second or produce equal `semantic_model_sha256`.

---

### SUP-03 — Generated-zone guard is substring-based: `src//gen//`, `src/./gen/`, root `fixtures/semantic/`, and bare `src/gen` escape it (compounds base F-01 into a full two-layer bypass)

**Category:** defect in SEA-Forge (implementation)
**Root cause:** parser/invariant enforcement (same non-canonical-path class as base F-01, different enforcement point)
**Severity:** High *as a chain with F-01* (both layers miss the same spellings; individually Medium)
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-spec-pipeline/src/lib.rs:252-258` — `is_generated_zone` = `path.contains("src/gen/") || ends_with(.ast.json/.ir.json/.manifest.json) || path.contains("/fixtures/semantic/")`
- Consumers: `crates/sea-forge-case-runner/src/lib.rs:199-227` (the pre-authority gate that quarantines non-generator stages declaring generated-zone outputs).

**Evidence (harness S5)**
```
src/gen/model.rs                          generated=true  edit_denied=true   (control)
src//gen//model.rs                        generated=false edit_denied=false  ← BYPASS
src/./gen/model.rs                        generated=false edit_denied=false  ← BYPASS
src/gen                                   generated=false edit_denied=false  ← BYPASS (bare)
fixtures/semantic/m.semantic.fixture.yaml generated=false edit_denied=false ← BYPASS (root-level: no leading "/")
mysrc/gen/x.rs                            generated=true  edit_denied=true   ← over-block (conservative)
```

**Chain:** a plan item whose stage declares output `src//gen//model.rs` (or root `fixtures/semantic/…`) passes the case-runner gate (`is_generated_zone` false) **and** — per base audit F-01, verified there — the authority `**/src/gen/**` / `docs/specs/**/fixtures/semantic/*.semantic.fixture.yaml` deny globs. Both the "denied before authority is ever consulted" layer and the authority layer miss the same non-canonical spellings; the write then materializes at the denied location via `safe_join` (F-01 evidence). Two supposedly independent defenses share one blind spot.

**Recommended remediation**
Normalize the path lexically (reject `//`, `.` segments — the `safe_lexical_join` rules again) before zone matching, and make the zone check segment-aware (`src` `/` `gen` as segments, any depth; `fixtures/semantic` as segments). Land together with F-01's fix so both layers share one canonicalization helper.

**Regression test**
The four bypass strings (plus root-level `fixtures/semantic/…`) must be denied by `check_generated_zone_edit`; `mysrc/gen` must be allowed.

---

### SUP-04 — `build_projection_record` stamps `Accepted` / `projection_validated` without performing any validation; `verify_projection` never checks output bytes

**Category:** likely intentional milestone simplification that has become an evidence-integrity defect (the record asserts validation that never runs)
**Root cause:** invariant enforcement
**Severity:** Medium
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-spec-pipeline/src/lib.rs:344-366` — `ProjectionValidation { status: Accepted, validator_ref: <sentinel>, basis: ["projection_validated"] }` unconditionally; `validator_ref` = `ADAPTER_DESCRIPTOR_SHA256` = `sha256:0000…0001` (domainforge lib.rs:84-85).
- `crates/sea-forge-self-model/src/projections.rs:184-201` — `verify_projection` recomputes `rebuild_hash` from the record's **own declared** `output_refs`; the recorded hashes are inputs, never compared against the materialized files (the doc comment promising the check is not implemented; conformance t9.3 tampers `rebuild_hash` itself, not outputs — untested gap).

**Evidence (harness S6)**
```
outputs = {"model.ttl": "garbage not validated at all"}
→ status=Accepted basis=["projection_validated"] validator_ref=sha256:0000…0001
```

**Impact**
Projection records are durable evidence ("views rebuildable, truth verifiable"); a tampered or garbage projection validates clean. `store::validate` iterates `projection-record.json` only, so replaced output files pass self-model validation.

**Recommended remediation**
Either run a real validator (re-parse the projection against the model, or at minimum hash the materialized files and compare to `output_refs`) before stamping `Accepted`, or record `basis: ["projection_unvalidated"]`/status `Declared` until one exists — the interaction model's own vocabulary distinguishes declared/probed/demonstrated; use it. Fix `verify_projection` to compare file bytes.

**Regression test**
Harness S6 verbatim (arbitrary output must not yield `projection_validated`), plus a tampered-output-file case for `store::validate`.

---

### SUP-05 — `parse_fixed`: UTF-8 boundary slice panic and silent i64 wrap in capability/settlement math

**Category:** defect in SEA-Forge (implementation)
**Root cause:** Rust implementation (Unicode + integer overflow)
**Severity:** Medium
**Confidence:** High (verified by execution)

**Affected components**
- `crates/sea-forge-capability/src/promotion.rs:15-32` and the duplicated `crates/sea-forge-settlement/src/declaration.rs:587-604`.

**Evidence (harness S1/S2, debug build)**
```
parse_fixed("1.ab∀∀")       → PANIC "byte index 6 is not a char boundary" (promotion.rs:29)
parse_fixed("9300000000000")→ PANIC "attempt to multiply with overflow"   (promotion.rs:31)
```

In **release**, the multiply wraps: a positive declared weight ≥ 9,223,372,036,854 becomes *negative* in `qualifying_decls` weight sums and settlement confidence math (silent sign flip), and the slice panic remains a panic.

**Reachability**
Capability side: `d.reliability.weight` parsed from persisted `declarations.jsonl` / policy config (`promotion.rs:138-139, 231-237`). Settlement side: `attribution_confidence`/`gaming_exposure`/`hidden_debt_blindness` parsed from the **external SWE-seed authority's JSON response** (`declaration.rs:579-581`) — a data-facing boundary, not just operator config. (The base audit noted this fn's overflow only as an Info item; the multibyte panic and the external-response reach elevate it.)

**Recommended remediation**
Char-boundary-safe truncation (`char_indices().take_while(...)`), `checked_mul` with a range-clamped or erroring policy, and range validation of parsed values before use. Deduplicate the two copies.

**Regression test**
Both harness inputs must return a typed error or clamped value, never panic/wrap.

---

### SUP-06 — Agent-endpoint descriptor: hardcoded version + config-sensitive hash → any endpoint edit permanently bricks future probes; failure leaves an allowed-but-unsettled run

**Category:** defect in SEA-Forge (integration)
**Root cause:** architecture/integration boundary (server violates the registry's immutability contract) + invariant enforcement (unsettled allowed run)
**Severity:** Medium
**Confidence:** High (statically verified; mechanism certain)

**Affected components**
- `crates/sea-forge-server/src/agent_probe.rs:468` (`version: "0.1.0".into()` hardcoded), `:477` (`input_contract.sha256 = endpoint.descriptor_config_sha256` — proven config-sensitive by `sea-forge-agent/src/config.rs:318-323` tests), `:479-482` (`output_contract.sha256 = format!("sha256:{}", "b".repeat(64))` — a fabricated digest binding nothing)
- `crates/sea-forge-extension/src/lib.rs:152-176` (`register_immutable_runtime_adapter`: same `(id, version)` with a different descriptor hash is a hard error)
- `crates/sea-forge-server/src/lib.rs:1615-1617` (the error returns as `{"error":…}` with **no settlement** — the failure fires after the authority decision and grant were committed).

**Mechanism**
First successful probe registers `agent_endpoint_X@0.1.0`. Any later edit to that endpoint's config (model, argv, timeout, credential ref…) changes the descriptor hash while the version stays `0.1.0` → every subsequent probe fails at registration. Recovery requires a source change (bump the hardcoded version) or hand-editing `registry.json`. Secondary: the failed probe is an allowed, granted run with committed intent/plan/authority views and **no settlement record** — the same "complete outcomes for allow" invariant violated by base F-02, on a different path.

**Recommended remediation**
Derive the descriptor version from the config hash (or a config-supplied version); settle the probe (rejected settlement with typed basis) instead of returning a bare error after the grant.

**Regression test**
Probe endpoint → edit endpoint config → probe again: must succeed with a new version (or a typed, settled outcome), not a permanent registration error.

---

### SUP-07 — Self-model root convention: double-nested reads fabricate a coherent "zero extensions / no capabilities" cell state

**Category:** defect in SEA-Forge (integration) — the self-model instance of base F-12, with an aggravating coherence twist
**Root cause:** architecture/integration boundary
**Severity:** Medium (same as F-12; recorded here because the self-model/thoth/readiness consumers are *mutually consistent* with the wrong path)
**Confidence:** High (statically verified)

**Affected components**
`crates/sea-forge-self-model/src/store.rs:43-45` (prepends `.sea-forge`), CLI `self_model.rs:31-35, 74` (reads `.sea-forge/.sea-forge/…`; **absent registry = "zero extensions", not an error**), server writes at `.sea-forge/extensions/registry.json` and `.sea-forge/capabilities.jsonl` (`agent_probe.rs:486`, `cli/pipeline.rs:858`), thoth `service.rs:74, 112` and readiness `readiness.rs:207, 356-363` read the double-nested paths.

**Impact vs F-12**
With default roots, `self-model rebuild` snapshots a cell with zero extensions and no demonstrated capabilities even when endpoints are registered and probed — and because thoth/readiness read the same wrong path, the fabricated state is served *coherently*: nothing errors, exit 0. The system's answer to "what is this cell?" is silently wrong rather than failing closed. (F-12 documented the convention split and silent empty exports; this adds the fail-open absent-registry semantics and the coherent-wrong-readers aspect.)

**Recommended remediation**
As F-12: one root convention; additionally, an absent registry/capabilities file in a cell that has ledger-committed `extension_registry` records should surface as stale/degraded, not "zero extensions".

---

### SUP-08 — Extension registry: raw-disk trust on load; `register_immutable_runtime_adapter` can replace quarantined entries with FirstParty+Active unauthenticated

**Category:** defect in SEA-Forge (invariant enforcement)
**Root cause:** invariant enforcement
**Severity:** Medium (M1) / Low-Medium latent (M2)
**Confidence:** High (statically verified)

**Affected components**
- `crates/sea-forge-extension/src/lib.rs:69-81` — `load()` = read+parse; no cross-check against the `extension_registry` ledger record that `save()` (:84-104) commits. A rewritten/rolled-back `registry.json` (`quarantined`→`active`, trust level stripped) is silently honored; `is_active` (:271-277) checks status only, ignoring `trust_level`; no duplicate `(id, version)` validation on load.
- `:152-176` — the immutable-replace path matches on `extension_id` alone and installs `FirstParty`+`Active` with `authority_ref: None`, discarding the prior entry's trust level/status. `adopt` (:237-268, authority-checked, `disabled→active` only) and `import_authorized` (:210-233, grant-required) enforce what this path skips. Today's only production caller passes fixed `agent_endpoint_*` ids; `import`/`adopt` have no production callers — latent public-API breach.
- Aggravator: `server/sfwp/assets.rs:253-256` assumes registration implies a prior authority grant, but runtime-adapter entries persist `authority_ref: None` (:173, :184) — the linkage lives only in the ledger, which `load()` doesn't read.

**Recommended remediation**
Verify `registry.json` against its ledger commit on load (or make the ledger view the read path); guard the replace path on prior trust level and require an authority ref; validate ids/versions/duplicates on load.

---

### SUP-09 — Remaining sweep items (verified statically unless noted)

| ID | Finding | Root cause | Severity |
| --- | --- | --- | --- |
| SUP-09a | `SubmitPayload.timeout` chrono panic (base F-19) — new detail: `timeout as i64` at values ≥ 2^63 flips **negative** → approval created already-expired (silent logic inversion, not just panic/permit-pin) | Rust implementation (conversion) | Low addendum to F-19 |
| SUP-09b | CLI `migrate` `enumerate_files_recursive`/`remove_empty_dirs_recursive` (`migrate.rs:191-204, 234-250`): `Path::is_dir()` follows symlinks, no cycle guard/depth cap → symlink cycle = stack overflow + unbounded Vec | Rust implementation | Medium-Low |
| SUP-09c | Unbounded whole-file reads with no `fs::metadata` cap: server `case_views.rs:260,273`, `run_views.rs:407,417` (runtime files a child/crashed run can inflate); `trace/lib.rs:22-38` (full-file read+reparse *only to count lines*, on the internal-error path — worst moment to OOM); `jail.rs:335` (reads child-written stderr unbounded for the permission-denied heuristic; cf. settlement's capped `take(65_537)`); `cli/main.rs:501-509` (`InternalTestSweSeed` unbounded stdin read in the shipped binary) | allocation amplification | Low-Medium |
| SUP-09d | Canonicalization family: three agreeing NFC implementations (self-model `lib.rs:59-77`, ledger `types.rs:31-49`, evidence `lib.rs:38-50` — evidence's relies *implicitly* on BTreeMap ordering, the only copy without an explicit sort); keys are **not** NFC'd (asymmetric but conservative — no collision possible); the label `jcs-nfc-v1` is **not** RFC 8785 JCS (no UTF-16 key ordering, no number canonicalization) so external JCS verifiers compute different hashes; `canonical_sha256` doc says "domain-separated" but no domain tag exists (contrast `sha256_domain`) | spec ambiguity / documentation | Low |
| SUP-09e | `ComposedModel` overlay has no cross-model conflict rule: `concepts()` sort+dedup silently collapses the **30 concept names already shared** by the two bundled models (`Case`, `PlanItem`, `AuthorityDecision`, …); consumers cannot observe divergent definitions. Compile-time-pinned today — latent until a release ships diverging definitions | spec ambiguity (overlay semantics undefined) | Low |
| SUP-09f | Self-model rebuild idempotency: `generated_at` honors `SOURCE_DATE_EPOCH` (`lib.rs:241-249`) but `commit_typed_once` keys on constant `release_id` — a rebuild in a different build environment (or any realization edit without a `RELEASE_ID` bump) hits the ledger's idempotency payload conflict → **every later rebuild fails** until ledger surgery | semantic transformation × integration boundary | Low-Medium |
| SUP-09g | `store.rs:58-60,519`: manifest `current_snapshot_id` joined unchecked — `../../x` escapes the snapshots dir (`.sea-forge`-write boundary, same class as base F-14) | Rust implementation | Low |
| SUP-09h | `evaluate_authority` (domainforge `lib.rs:499-537`) is a filename-stem heuristic (`write_file` → Allow iff `Path::file_stem(resource_id)` ASCII-case-insensitively equals an entity/resource name) while its trace asserts *"DomainForge evaluated validated model against canonical action"* and is composed into real authority evidence (`authority lib.rs:305, 2272-2311`). domainforge-core 0.16.0 ships a full authority subsystem (compiler/resolver/fact_resolver/lowered policies) that the adapter does not consult. Possibly a deliberate milestone approximation — but the evidence string overstates what ran, and case-insensitive stem matching means `docs/SAMPLE.md` is "semantically targeted" for entity `Sample` (harness S8). **Resolve deliberately: either wire the real evaluator or reword the trace/basis to describe the approximation.** | architecture/integration boundary (+ spec ambiguity) | Low-Medium (evidence integrity) |
| SUP-09i | CEP-0008: `deny_unknown_fields` + `ALLOWED_TOP_LEVEL` including `causation_id` means the struct **cannot deserialize** a schema-legal incoming envelope carrying `causation_id` (outbound-only today); `payload_hash`/`entry_ulid`/`ledger_id` get emptiness checks only, no grammar validation; `validate_descriptor` checks hash grammar but not `extension_id`/`version` grammar (empty or `../` ids accepted); `Compatibility`/`ExtensionInstallRecord` are dead spec surface (declared, never enforced — the string-version-compare bug class is absent only because enforcement is absent); `register_built_in` treats same-(id,version)-different-hash as idempotent success (silent ignore vs the runtime-adapter path's error); `import` doesn't dedupe | parser-serializer / invariant enforcement (all latent: no production callers for import/adopt) | Low |

---

## Rejected hypotheses (this run)

1. **"Projections are nondeterministic"** — disproven by execution: CALM (post timestamp-strip), RDF, and KG projections are byte-identical across repeated `project()` calls (harness S7). The `sea:timestamp` strip is effective; `to_vec_pretty` over BTreeMap is sorted.
2. **"NFC normalization causes a cross-layer hash split-brain"** — disproven: self-model, ledger, and evidence canonicalizers agree byte-for-byte today (values NFC'd, keys sorted, arrays ordered); `preserve_order` confirmed absent from the lockfile. Residual risks documented as SUP-09d (implicit sort in evidence's copy; keys-not-normalized asymmetry; misleading `jcs-nfc-v1` label).
3. **"NFC model-identity collisions"** — disproven: `semantic_model_sha256` hashes **raw** bytes (no NFC), so NFD vs NFC spellings produce *different* identities (no collision); the normalization only affects the three canonical-hash implementations, which agree.
4. **"Import-graph recursion is unbounded"** — disproven: memoized + cycle-detecting, bounded by module count (≤ 64); only *expression* nesting (SUP-01) is unbounded.
5. **"pest/AST conversions overflow at small depths"** — quantified instead: depth 1,000 survives and returns a typed semantic error; the abort threshold is between 1,000 and 10,000 unary levels (~7 KB).
6. **Horizontal sweeps clean (searches run, nothing found)**: `unsafe` blocks — zero across all 22 crates (workspace `deny` + per-crate `[lints] workspace = true` + 11 in-source `forbid!`); `build.rs` — none; `[features]` — none; proc-macros — none; unchecked `[N]` indexing — all verified guarded (enforced-by-construction sites confirmed at their enforcement points); non-test `unwrap/expect` — all 15 verified guarded; RFC 3339 parsing — all 10 sites `map_err`'d, no unwraps; child-process spawn sites beyond the base audit's coverage — `CommandSweSeedTransport` verified hardened (argv validation, env_clear, capped pipes, group kill with liveness re-check).

---

## Systemic observations (supplement-specific)

1. **The non-canonical-path class is now three layers deep**: authority globs (F-01), the generated-zone substring guard (SUP-03), and the various id joins (F-09/F-14/F-15, SUP-09g). Every layer independently invented raw-string matching where the repo already owns a canonicalization primitive (`safe_lexical_join`, `ids::valid_*`). One shared helper, applied at the enforcing function, retires the entire class.
2. **"Validated" vocabulary without validators**: `projection_validated` (SUP-04), the fabricated `sha256:b×64` output contract (SUP-06), `evaluate_authority`'s overstated reason (SUP-09h), and `verify_projection`'s unimplemented doc promise all describe checks that never run. The interaction model itself distinguishes declared/probed/demonstrated — the implementation should borrow that vocabulary instead of asserting demonstration.
3. **Identity vs resolution**: SUP-02 (entry spelling), SUP-09f (SOURCE_DATE_EPOCH vs constant release id), and F-25.e (bundle hash over `source_base`) share one shape: identity inputs that are *not* the normalized semantic content. Pin identities to resolved/normalized values, never to caller spellings or ambient environment.
4. **External-dependency boundary**: `domainforge-core` is pinned (`=0.16.0`, version-checked at load — good) but its resource behavior (pest recursion) and its unused authority subsystem show the adapter boundary was specified for *shape*, not for *resource bounds* or *capability coverage*. The finite-limits contract (§7.0a) should name nesting depth explicitly.
5. **Immutability contracts need version sources**: the extension registry enforces immutability per `(id, version)` while its only production writer hardcodes the version (SUP-06) — an invariant enforced against a constant is a time bomb.

---

## Prioritization (supplement only; merges into the base report's tiers)

1. **Immediate (chain with base Immediate):** SUP-03 + F-01 (one shared canonicalization fix, both layers); SUP-01 (depth cap before/during parse — an abort class the server cannot contain).
2. **High:** SUP-05 (external-response-reachable panic/wrap in settlement math); SUP-06 (bricked probes + unsettled-allowed-run); SUP-04 (projection evidence integrity).
3. **Important:** SUP-02 (identity spelling rule — decide and document); SUP-07/F-12 (root convention + fail-open absent-file semantics); SUP-08 (registry ledger verification + replace-path guard); SUP-09b, SUP-09f.
4. **Hardening:** SUP-09a/c/d/e/g/h/i.

**Ordering dependencies:** the shared path-canonicalization helper (SUP-03/F-01) should land before any new deny-pattern or zone work; SUP-02's decision determines the fix shape for SUP-09f's release-id pinning (same "identity from normalized content" principle); SUP-08's ledger verification should precede enabling any production caller for `import`/`adopt`.

---

*End of supplement. No production implementation code was modified during this audit. Harness: `.agents/reports/audit-harness-2026-08-14/audit2/` (`cargo run` executes SUP-01–SUP-05, S3/S5/S6/S8 verifications; the `nest <depth>` subcommand reproduces SUP-01 in a child process).*
