# Independent Validation: Four Specification Implementation Audit

**Date:** 2026-07-22

**Source report:** `.agents/reports/2026-07-22-spec-implementation-audit.md`

**Outcome:** The source report is substantially accurate, but two minimum-spec
findings need qualification. All other original findings are confirmed.

## Overall Verdict

| Specification | Verdict |
|---|---|
| `spec-minimum.md` | **Not conformant.** Standalone validation, exact demo argv, plan write ordering, and capability-recall output contradict executable contracts. The workspace-size claim is not itself a defect, and read-only recall is intentionally upgraded by the full spec. |
| `spec-full.md` | **Not conformant.** All five findings are confirmed, including critical cell-import traversal. Additional gaps exist in memory authorization, stale-index detection, DomainForge guards, and production M5 wiring. |
| `spec-adlc-thoth-minimum.md` | **Not conformant.** All five findings are confirmed. M9/M10/M11 tests largely exercise isolated components or injected fixtures rather than the required governed production paths. |
| `spec-agent-orchestration.md` | **Not conformant.** All six findings are confirmed. Transcript redaction is a critical defect affecting ACP and, more narrowly, HTTP delegation. |

## Original Findings

### `spec-minimum.md`

| # | Report claim | Verdict | Executable and test evidence | Recommendation |
|---|---|---|---|---|
| 1 | Workspace no longer matches two-crate synchronous slice | **Partially supported, not a behavioral defect** | Workspace has 22 crates at `Cargo.toml:3-26`; HTTP/async is isolated in `sea-forge-agent`. The minimum spec itself reserves these as full-spec additions at `.agents/specs/spec-minimum.md:68-83,175-179`. No evidence connects minimum execution to async/network code. | Treat minimum conformance as a compatibility profile and enforce dependency boundaries. Do not shrink the additive full workspace. |
| 2 | Standalone validation contract changed | **Confirmed: implementation absence and test gap** | `validate` requires policy and performs ledgered authorization at `crates/sea-forge-cli/src/commands/validate.rs:3-16` and `crates/sea-forge-cli/src/commands/mediated.rs:130-186`, contrary to `.agents/specs/spec-minimum.md:799-803`. The test supplies policy/root at `crates/sea-forge-cli/tests/lifecycle.rs:843-881`. | Restore config-free validation. Keep governed model loading as a separate command/path if required by the full system. Add a no-policy, no-state-mutation test. |
| 3 | Recall is no longer the minimum read-only scan | **Partially supported** | Capability recall mutates authority state at `crates/sea-forge-cli/src/commands/recall.rs:37-48` and `crates/sea-forge-cli/src/commands/mediated.rs:139-186`. However, `.agents/specs/spec-minimum.md:779` expressly says full-spec recall becomes authority-checked and evidenced. Current tests inspect only capabilities and run count at `crates/sea-forge-cli/tests/lifecycle.rs:780-838`, not complete mutations. | Preserve an explicit read-only compatibility scan, or formally supersede it. Implement governed memory recall separately with scope enforcement and evidence naming returned memory IDs. |
| 4 | Demo child argv changed | **Confirmed: implementation and test absence** | Planner emits extra `--root` and `--policy` arguments at `crates/sea-forge-planner/src/lib.rs:95-108`, contradicting `.agents/specs/spec-minimum.md:804-813`. Lifecycle tests do not assert persisted argv. | Restore the exact three-token argv and standalone validator, then add a persisted-plan argv assertion. |

### `spec-full.md`

| # | Report claim | Verdict | Executable and test evidence | Recommendation |
|---|---|---|---|---|
| 1 | Multi-file DomainForge source sets are not validated | **Confirmed: implementation and test absence** | Only the entry hash/content is validated and parsed at `crates/sea-forge-domainforge/src/lib.rs:100-136`; other files become unchecked refs at `:138-163`. Tests use one source at `crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs:21-69`, contrary to `.agents/specs/spec-full.md:376-399,1654-1658`. | Validate every URI/hash, resolve imports from the complete source set, enforce limits, and add multi-file/unresolved-import/hash-drift tests. |
| 2 | Jail does not deny network by default | **Confirmed: implementation and test absence** | Jail installs only `AccessFs` under Landlock ABI V1 at `crates/sea-forge-sandbox/src/jail.rs:260-295`. M1 tests exercise filesystem escape only at `crates/sea-forge-sandbox/tests/conformance_m1.rs:38-84`. The requirement is `.agents/specs/spec-full.md:1432-1436`. | Use Landlock network controls where supported or another network-isolating backend. Fail closed when default network denial cannot be enforced. Add outbound TCP/UDP tests. |
| 3 | Memory uses JSON instead of SQLite FTS | **Confirmed: incompatible implementation** | JSON projection is explicit at `crates/sea-forge-capability/src/memory.rs:196-242`; CLI uses `memory/index.json` at `crates/sea-forge-cli/src/commands/recall.rs:109-116`. M4b tests affirm JSON at `crates/sea-forge-capability/tests/conformance_m4b.rs:168-238`. The spec requires `index.sqlite` at `.agents/specs/spec-full.md:143,1525-1529`. | Implement the required SQLite FTS projection and retain linear fallback for missing/stale indexes. |
| 4 | Spec pipeline accepts invalid predecessor state | **Confirmed: implementation and test absence** | Validation checks only non-descending enum order at `crates/sea-forge-spec-pipeline/src/lib.rs:73-95`; processing at `:268-280` does not verify predecessor presence/status/hashes. Tests cover reversed order only at `crates/sea-forge-spec-pipeline/tests/conformance_m5.rs:302-318`. | Require every omitted predecessor's validated persisted output and verify status, hashes, schema, DomainModelRef, and DomainForge version before advancing. |
| 5 | Cell import permits path escape | **Confirmed, critical** | Manifest-controlled `bundle_id`, file paths, and `cell_id` reach `join`, `remove_dir_all`, writes, and rename at `crates/sea-forge-cell/src/bundle.rs:219-248`. Tests cover hash/manifest tampering but not traversal at `crates/sea-forge-cell/tests/conformance_m6.rs:149-186,239-337`. | Validate identifiers and safe-join every archive destination before any filesystem mutation. Reject absolute paths, `..`, prefixes, and symlink-parent escapes. |

### `spec-adlc-thoth-minimum.md`

| # | Report claim | Verdict | Executable and test evidence | Recommendation |
|---|---|---|---|---|
| 1 | Thoth cannot expose installed/demonstrated/available state | **Confirmed: production integration absent** | CLI snapshot adapter always returns no capability, declarations, or environment status at `crates/sea-forge-cli/src/commands/ask.rs:48-75`. T11 unit tests inject prejoined `CapabilityInfo` through mocks rather than exercising production. | Build `SnapshotView` from ledger-verified release/cell realization, probes, projections, settlements, and capability records. |
| 2 | `ask` bypasses governance and ledger chain | **Confirmed: implementation and test absence** | CLI reads policy directly and calls the pure engine at `crates/sea-forge-cli/src/commands/ask.rs:106-148`. It commits no question, authority decision, disclosure/query plans, evidence, or answer. Server has no ask operation. CLI tests cover errors/default denial only at `crates/sea-forge-cli/tests/ask_cli.rs:21-80`. | Introduce one mediated ask service shared by CLI/server that commits the complete required chain. |
| 3 | Disclosure can emit prohibited claim classes | **Confirmed: concrete bug** | Any one capability class permits retrieval at `crates/sea-forge-thoth/src/engine.rs:174-182`, while emitted class is independently elevated from status at `:261-289`. No partial-grant test exists. | Cap status and emitted claim class to the exact permitted class set; add declared-only and installed-only grant tests. |
| 4 | ODI provenance is placeholder-only | **Confirmed and understated** | Built-in template contains `sha256:placeholder` at `crates/sea-forge-planner/src/templates.rs:910-929`. Resolver-aware verification exists at `crates/sea-forge-planner/src/criteria.rs:274-292`, but production uses the no-model wrapper. M10 tests call isolated resolver fixtures. | Parameterize the desired-outcome ref, bind it to the validated seed DomainModelRef, derive criteria from the instantiated template, and invoke resolver-aware verification before authority. |
| 5 | Self-model is not a settled ledger projection | **Confirmed: implementation and test absence** | Snapshots/projections are direct filesystem writes at `crates/sea-forge-self-model/src/store.rs:99-115,162-196,227-253`. Projection records have empty authority/evidence/settlement refs at `crates/sea-forge-self-model/src/projections.rs:71-91`. M9 tests assert deterministic hashes, not governance linkage. | Ledger snapshot/projection creation and populate authority, validation evidence, settlement, and envelope references before materializing views. |

### `spec-agent-orchestration.md`

| # | Report claim | Verdict | Executable and test evidence | Recommendation |
|---|---|---|---|---|
| 1 | Generated topology/manager endpoint IDs are invalid | **Confirmed: implementation and end-to-end test absence** | Config IDs reject colons at `crates/sea-forge-agent/src/config.rs:124-138`; templates emit `agent:builtin` at `crates/sea-forge-planner/src/templates.rs:938-947`; manager emits `agent:default` at `crates/sea-forge-cli/src/commands/manager.rs:258-269`. M14 only inspects plans at `crates/sea-forge-planner/tests/conformance_m14.rs:247-344`; M15 stops after insertion at `crates/sea-forge-cli/tests/conformance_m15.rs:175-218`. | Require a valid endpoint parameter or resolve a configured default endpoint before plan acceptance. Dispatch generated plans in an integration test. |
| 2 | Summarized retention and precedence are absent | **Confirmed: implementation and test absence** | Config has no retention fields at `crates/sea-forge-agent/src/config.rs:22-63`; dispatch drops the plan field at `crates/sea-forge-server/src/case_dispatch.rs:437-443`; storage forces full plaintext retention at `crates/sea-forge-server/src/delegation.rs:488-520`. T13.6 proves full mode only at `crates/sea-forge-server/tests/conformance_m13.rs:379-429`. | Implement plan-item to endpoint to global to summarized-default precedence and the selected sealed summarized mode. Test both modes against the same redacted canonical transcript. |
| 3 | Turn-cap handling always rejects | **Confirmed: implementation contradiction** | Completed outcomes evaluate criteria, but `TurnCapExceeded` unconditionally rejects at `crates/sea-forge-server/src/delegation.rs:595-615`. Existing T13 test asserts rejection at `crates/sea-forge-server/tests/conformance_m13.rs:288-328`, contrary to `.agents/specs/spec-agent-orchestration.md:273-280,422`. | Evaluate available output/artifacts after cap termination; preserve `turn_cap_exceeded` in basis while allowing acceptance when criteria pass. |
| 4 | ACP transcript redaction is absent | **Confirmed, critical** | ACP copies raw content then hashes it at `crates/sea-forge-server/src/delegation.rs:1133-1182`; artifact is written at `:488-513` before ledger commit. M16 uses benign text and does not test sentinels at `crates/sea-forge-server/tests/conformance_m16.rs:539-560`. | Apply one shared sentinel-aware redactor before summary, hash, storage, logging, or error creation for both ACP and HTTP paths. |
| 5 | Late SWE_SEED declaration reconciliation is absent | **Confirmed: implementation and test absence** | Declarations are scanned once during settlement at `crates/sea-forge-server/src/delegation.rs:569-592,666-676`. Portable T16.6 checks artifact correlation but not declaration IDs at `crates/sea-forge-server/tests/conformance_m16.rs:624-645`. | Add an idempotent reconciler triggered by declaration append/replay that updates a rebuildable correlation projection without rewriting original records. |
| 6 | Manager caps are not authority grant-bounded | **Confirmed: implementation and test absence** | Authority action has empty parameters at `crates/sea-forge-cli/src/commands/manager.rs:47-56`; only caller-supplied cap is enforced at `:62-78`. T15 policy has no maximum and the test passes local `1` at `crates/sea-forge-cli/tests/conformance_m15.rs:170-173,255-294`. | Bind requested and granted `max_manager_iterations` into the authority action and enforce the minimum of CLI, config, and grant limits. |

## Newly Discovered Material Findings

| Specification | Finding | Evidence and classification | Recommendation |
|---|---|---|---|
| Minimum | `plan.json` is written twice and before initial trace events | **Implementation defect; test absent.** Writes occur at `crates/sea-forge-cli/src/pipeline.rs:300-301` and `:350-359`, contrary to write-once/order requirements. | Remove the first write; persist once after `run_started` and immediately before `plan_created`. |
| Minimum | Recall alters matching envelopes | **Implementation defect; test absent.** `assurance` is injected at `crates/sea-forge-cli/src/commands/recall.rs:72-84`, although minimum output is the stored envelope JSONL. | Emit the unchanged envelope on compatibility recall; use a separate full-spec result wrapper for assurance. |
| Full | Memory scope is not enforced in production | **Implementation/security absence.** `scope_allows` at `crates/sea-forge-capability/src/memory.rs:179-194` is called only by tests; generic reserved-rule matching ignores `memory_scope` at `crates/sea-forge-authority/src/lib.rs:2480-2505`. | Bind requested entity/process/kinds into authority and enforce the granted scope over every result. |
| Full | Stale indexes silently omit new items | **Implementation and test absence.** `query_index` trusts stored IDs without a source digest at `crates/sea-forge-capability/src/memory.rs:244-277`; tests only delete the index. | Store the items source digest/ordinal and fall back whenever it differs. |
| Full | M5 pipeline has no production caller | **Production wiring absence.** `process_pipeline` is referenced only in its defining crate and tests. | Wire stages through ordinary governed case plans and persist authority, evidence, quarantine, and settlement records. |
| Full | DomainForge lacks URI/version/resource-limit guards | **Implementation and test absence.** `SeaSourceSet` and `load_validate` at `crates/sea-forge-domainforge/src/lib.rs:19-32,100-196` enforce none of the required count/bytes/depth/node/work or URI checks. | Add adapter limits and validate the entire source-set contract before parsing. |
| ADLC/Thoth | No-self-certification SoD is unwired | **Implementation and boundary-test absence.** `check_sod` at `crates/sea-forge-thoth/src/engine.rs:4-21` is called only by unit tests; generated claims use `authored_by: None` at `:185-198,277-289`. | Persist immutable authorship and enforce it at settlement declaration and capability promotion. |
| ADLC/Thoth | Several typed questions produce empty successful answers; `ask_why_denied` does not read a decision | **Implementation absence.** Unsupported handlers fall through at `crates/sea-forge-thoth/src/engine.rs:254-258`; denial explanation is manufactured at `:110-125`. | Implement bounded handlers for every enum kind and resolve `ask_why_denied` solely from a verified decision reference. |
| ADLC/Thoth | Required stale-snapshot refusal is absent | **Implementation/test absence.** Engine returns stale even when freshness is required at `crates/sea-forge-thoth/src/engine.rs:151-160`; CLI policy adapter does not propagate that requirement at `crates/sea-forge-cli/src/commands/ask.rs:77-89`. | Carry the matched grant's freshness boundary and refuse before query execution when required. |
| ADLC/Thoth | Self-model rebuild uses fabricated installation inputs | **Implementation absence.** CLI supplies empty registry/environment/probe data, hard-coded local sandbox, and placeholder capability digest. | Derive these values from verified registry, probe, sandbox, environment, and capability records. |
| Orchestration | HTTP redaction is also incomplete | **Security implementation/test absence.** HTTP redaction handles only exact credentials and a literal Bearer prefix at `crates/sea-forge-agent/src/delegation.rs:89-100`. | Reuse the shared sentinel-aware redactor, including streaming-boundary handling. |
| Orchestration | `response_schema` is discarded | **Implementation and test absence.** Dispatch drops the field at `crates/sea-forge-server/src/case_dispatch.rs:437-443`; delegation reconstructs it as `None` at `crates/sea-forge-server/src/delegation.rs:256-264`. | Propagate and validate the schema; return typed `schema_invalid` evidence on failure. |
| Orchestration | Transcript bytes are not specified JCS canonical JSONL | **Implementation and interoperability-test absence.** Ordinary serde serialization is used for digest/storage. | Canonicalize once with the project canonical-JSON implementation and hash/store those exact bytes. |
| Orchestration | Case dispatch overwrites detailed settlement basis | **Implementation and test absence.** Detailed basis is created at `crates/sea-forge-server/src/delegation.rs:595-640`, then replaced with `["delegation_completed"]` at `crates/sea-forge-server/src/case_dispatch.rs:468-477`. | Return and reuse the original settlement event unchanged. |

## Commands Run

| Command | Result |
|---|---|
| `devbox run -- just test` | **FAIL**, exit 101. The `sea-forge-cli` `src/main.rs` unit binary `target/debug/deps/sea_forge-8fa4cf68b27e02a8` exited with signal 11/SIGSEGV before reporting test output. |
| `devbox run -- cargo test -p sea-forge-cli --bin sea-forge --all-features --locked -- --nocapture` | **PASS**, 5 tests. This confirms the crash is suite-context dependent, not that the workspace gate passes. |
| `cargo test -p sea-forge-cli --test lifecycle` | **PASS**, 11 tests. |
| `cargo test -p sea-forge-domainforge --test conformance_m0_domainforge` | **PASS**, 5 tests. |
| `cargo test -p sea-forge-sandbox --test conformance_m1` | **PASS**, 5 tests. |
| `cargo test -p sea-forge-capability --test conformance_m4b` | **PASS**, 8 tests. |
| `cargo test -p sea-forge-spec-pipeline --test conformance_m5` | **PASS**, 10 tests. |
| `cargo test -p sea-forge-cell --test conformance_m6` | **PASS**, 11 tests. |
| `cargo test -p sea-forge-self-model --test conformance_m9` | **PASS**, 7 tests. |
| `cargo test -p sea-forge-planner --test conformance_m10` | **PASS**, 6 tests. |
| `cargo test -p sea-forge-planner --test criteria_provenance` | **PASS**, 18 tests. |
| `cargo test -p sea-forge-thoth` | **PASS**, 20 tests. |
| `cargo test -p sea-forge-cli --test ask_cli` | **PASS**, 3 tests. |
| `cargo test -p sea-forge-agent --test provider_contract` | **PASS**, 3 tests. |
| `cargo test -p sea-forge-server --test conformance_m13` | **PASS**, 18 tests. |
| `cargo test -p sea-forge-planner --test conformance_m14` | **PASS**, 9 tests. |
| `cargo test -p sea-forge-cli --test conformance_m15` | **PASS**, 5 tests. |
| `cargo test -p sea-forge-server --test conformance_m16` | **PASS**, 12 passed, 2 ignored. |

## Not Proven From Executable Evidence

- The root cause of the workspace SIGSEGV is not established; only the failing
  binary and suite-dependent behavior are proven.
- Real ACP-host and SWE_SEED-host behavior is unproved because two M16 tests are
  ignored pending operator configuration.
- macOS Seatbelt behavior is unproved on this Linux host.
- Actual outbound network success from the jail was not attempted; source
  inspection proves no network-denial mechanism is installed.
- No malicious traversal archive was executed. Source inspection proves
  manifest-controlled paths reach destructive/write operations without
  safe-join validation.
- Documentation claims marking gates green are not executable evidence.

No code, specifications, tests, or the source audit report were modified during
the validation. The cross-model second opinion was explicitly skipped after
fresh-context source reviews were completed.
