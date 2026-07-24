# Implementation Audit: Four `spec-*.md` Specifications

**Date:** 2026-07-22
**Verdict:** **Not all four specifications are implemented coherently.** The workspace has substantial, tested coverage, but each specification has at least one normative requirement contradicted by executable code or left unproved. The prerequisite chain therefore cannot be green.

## Method

The specifications supplied requirements only. Evidence below is limited to Rust source, executable tests, and command output. Status files, plans, test names, and prose claims were not accepted as evidence.

## Summary

| Specification | Assessment | Decisive evidence |
|---|---|---|
| `spec-minimum.md` | **Not coherent with its declared minimum command contracts** | The workspace is now an extended system, and `validate` and `recall` do not retain their specified minimum contracts. |
| `spec-full.md` | **Partial; M0–M8 claims are not all met** | Multi-source DomainForge validation, default network denial, SQLite FTS memory, stage-prerequisite validation, and safe cell import have material gaps. |
| `spec-adlc-thoth-minimum.md` | **Partial; E11–E13 core outcome is not met** | Thoth is neither authority/ledger mediated nor joined to real capabilities, cells, or ledger evidence. ODI template provenance remains a placeholder. |
| `spec-agent-orchestration.md` | **Partial; M12–M16 fixture coverage does not establish the required outcome** | Built-in/manager endpoint identifiers cannot be configured; retention, turn-cap, transcript redaction, declaration reconciliation, and grant-bound iteration diverge from the spec. |

## Verified implementation coverage

These are examples of executable coverage, not evidence that the full specifications are complete.

- **Minimum governed lifecycle:** `crates/sea-forge-cli/src/pipeline.rs:242-858` wires planning, authority, execution, trace, evidence, settlement, and envelope creation. `cargo test -p sea-forge-cli --test lifecycle` passed **11 tests**, including deny-without-command-start, escalation, false-success, timeout, and stable artifact identity paths.
- **Authority and sandbox basics:** `crates/sea-forge-authority/src/lib.rs:1922-2149,2418-2690` implements fail-closed authority and identity behavior; `cargo test -p sea-forge-authority --test conformance_m0_authority` passed **9 tests**. `cargo test -p sea-forge-runtime` passed **5 tests**, including minimal child environment and timeout reaping.
- **Full-system implemented portions:** focused conformance suites passed for ledger (**20**), case engine (**11**), capability promotion (**13**), environment evaluation (**10**), and artifact IP (**44**). The implementations are respectively in `sea-forge-ledger/src/types.rs:488-1714`, `sea-forge-planner/src/case_engine.rs:101-543`, `sea-forge-capability/src/promotion.rs:110-464`, `sea-forge-sandbox/src/environment.rs:23-199`, and `sea-forge-artifact-ip/src/lib.rs:1160-1847`.
- **Orchestration fixture coverage:** focused suites passed for provider contract (**3**), M12 (**11**), M13 server (**18**) and planner (**3**), M14 (**9**), M15 (**5**), and M16 portable ACP (**12 passed, 2 ignored**). `just no-async-kernel` passed for **19** kernel crates.

## Findings

### 1. `spec-minimum.md`

1. **Compatibility concern — declared slice scope no longer describes the workspace (§2.2, §6.1, §6.3, §11.4).** The workspace contains 22 crates, including `sea-forge-server` and `sea-forge-agent` (`Cargo.toml:3-26`), while `sea-forge-agent/src/provider.rs:5,120` and `acp.rs:22-25` use HTTP/async dependencies. This is expected for the extension specifications, but it means the minimum document cannot describe the whole workspace as a two-crate, synchronous, no-network system. The command-contract findings below independently establish the minimum-spec divergence.
2. **Block — standalone validation contract changed (§11.1).** `crates/sea-forge-cli/src/commands/validate.rs:3-10` requires a policy and uses mediated authorization. The specification requires deterministic standalone validation without configuration. The lifecycle test passes policy/root arguments, so it proves the changed contract, not the required one.
3. **Block — recall is not the specified read-only scan (§10.5).** `crates/sea-forge-cli/src/commands/recall.rs:37-48` authorizes recall through mediation; `commands/mediated.rs:130-186` persists authority/evidence/decision records. That conflicts with the required no-run/no-trace/no-envelope read-only operation. Existing lifecycle coverage checks capability and run files, not the mediated ledger writes.
4. **Block — demo child argv changed (§11.2).** `crates/sea-forge-planner/src/lib.rs:95-108` injects `--root` and `--policy`; the required command is `[current_exe, "validate", "model.sea"]`.

### 2. `spec-full.md`

1. **Block — multi-file DomainForge source sets are not validated (§7.0a/M0).** `crates/sea-forge-domainforge/src/lib.rs:101-136,138-196` passes only entry content to DomainForge and does not validate hashes/import resolution for the remaining `SeaSourceSet.files`. `conformance_m0_domainforge.rs:21-106` uses one source only.
2. **Block — jail does not deny network by default (§10.1/M1).** `crates/sea-forge-sandbox/src/jail.rs:275-295` creates Landlock filesystem rules only. `conformance_m1.rs:38-84` exercises filesystem escape, not outbound network denial.
3. **Block — governed memory is JSON, not required SQLite FTS (§3.1, §10.5/M4b).** `crates/sea-forge-capability/src/memory.rs:197-242` and CLI commands use `memory/index.json` (`commands/recall.rs:109-116`, `commands/memory.rs:6-12`), rather than the required `memory/index.sqlite` projection.
4. **Block — spec-pipeline accepts invalid predecessor state (§10.7/M5).** `crates/sea-forge-spec-pipeline/src/lib.rs:73-95,268-280` rejects only descending stage order, not absent/rejected/unhashed prerequisites. `conformance_m5.rs:302-318` covers reversed order only.
5. **Critical — cell import permits path escape (§M6 untrusted-import boundary).** `crates/sea-forge-cell/src/bundle.rs:227-237` writes manifest-controlled paths with `staging.join(arc)` and creates the final directory with `join(&manifest.cell_id)` without safe-path validation. A internally consistent malicious manifest can write outside both roots; tamper coverage (`conformance_m6.rs:149-186`) does not test traversal.

### 3. `spec-adlc-thoth-minimum.md`

1. **Block — Thoth cannot report installed/demonstrated/available capability state (§10.3, §15, T11.1–T11.2).** `crates/sea-forge-cli/src/commands/ask.rs:48-75` supplies no capabilities, declarations, or environment statuses, so permitted capability queries return `unsupported`.
2. **Block — Thoth `ask` bypasses governance and leaves no required chain (§9.1, §10.3, §12.1, T11.3/T11.10).** `ask.rs:116-148` calls `SelfDisclosureSurface::permits` directly; it does not construct an authority request or write decision/query/answer/evidence ledger records. No server ask operation exists.
3. **Block — disclosure can leak prohibited claim classes (§10.3, §15).** `crates/sea-forge-thoth/src/engine.rs:175-182,261-289` derives a full capability then can emit demonstrated evidence when only `declared_capability` was permitted.
4. **Block — ODI provenance is placeholder-only (§7.5, §10.4, T10.3).** The built-in template contains `sha256:placeholder` (`crates/sea-forge-planner/src/templates.rs:910-929`). `verify_plan_criteria_with_resolver` exists in `criteria.rs:278`, but has no production caller.
5. **Block — self-model state is not a settled, evidence-linked ledger projection (§3.1, §7.1, §10.1).** `crates/sea-forge-self-model/src/store.rs:227-253` writes direct JSON; `projections.rs:71-91` creates empty authority/evidence/settlement references.

### 4. `spec-agent-orchestration.md`

1. **Block — shipped topology and manager tasks cannot reference any valid endpoint (§2 G6–G7, T14.1–T15.1).** Templates emit `agent:builtin` (`crates/sea-forge-planner/src/templates.rs:943`) and manager emits `agent:default` (`crates/sea-forge-cli/src/commands/manager.rs:263`). `AgentEndpointConfig::validate` allows only lowercase alphanumeric, `_`, and `-` IDs (`crates/sea-forge-agent/src/config.rs:124-138`); colons are invalid. The passing M14/M15 tests do not dispatch either generated item.
2. **Block — selectable summarized retention is not implemented (§7.4, §10.5, T13.6).** `crates/sea-forge-server/src/delegation.rs:488-513` explicitly implements full retention only. `case_dispatch.rs:437-443` ignores the plan-item retention field. The T13.6 test validates only the full artifact.
3. **Block — turn-cap handling contradicts settlement rule (§9.5/T13.4).** `delegation.rs:611-615` unconditionally rejects `TurnCapExceeded`; the spec requires criteria evaluation and possible acceptance. The conformance test asserts rejection (`conformance_m13.rs:316-324`).
4. **Critical — ACP transcripts are written and hashed without demonstrated redaction (§7.4, §15).** `acp_outcome_to_delegation` copies ACP text (`delegation.rs:1133-1144`), then the same code serializes it to disk (`:504-513`). No M16 test injects a credential/sentinel into ACP output.
5. **Block — SWE_SEED declaration reconciliation is absent (§2 G9/T16.6).** `delegation.rs:572-592,666-676` snapshots declarations during initial settlement only; it has no later reconciliation path. M16 tests check initial harvested references, not a declaration arriving afterward.
6. **Block — manager iteration caps are not authority grant-bounded (§2 G7/T15.3).** `commands/manager.rs:47-78` authorizes an empty-parameter `manager_iteration` action and enforces only the CLI-supplied local cap; no authority action binds that maximum.

## Verification record

Focused commands run during this audit:

```text
cargo test -p sea-forge-cli --test lifecycle                         PASS (11)
cargo test -p sea-forge-authority --test conformance_m0_authority    PASS (9)
cargo test -p sea-forge-runtime                                      PASS (5)
cargo test -p sea-forge-self-model --test conformance_m9             PASS (7)
cargo test -p sea-forge-planner --test conformance_m10               PASS (6)
cargo test -p sea-forge-cli --test ask_cli                           PASS (3)
cargo test -p sea-forge-thoth                                        PASS (20)
cargo test -p sea-forge-agent --test provider_contract               PASS (3)
cargo test -p sea-forge-server --test conformance_m12                PASS (11)
cargo test -p sea-forge-server --test conformance_m13                PASS (18)
cargo test -p sea-forge-planner --test conformance_m13               PASS (3)
cargo test -p sea-forge-planner --test conformance_m14               PASS (9)
cargo test -p sea-forge-planner --test template_conformance          PASS (8)
cargo test -p sea-forge-cli --test conformance_m15                   PASS (5)
cargo test -p sea-forge-server --test conformance_m16                PASS (12, 2 ignored)
devbox run -- just proof                                              PASS (P1–P4b)
devbox run -- just no-async-kernel                                   PASS (19 kernel crates)
devbox run -- just check                                             PASS (format, clippy, typecheck, supply-chain, secret scan)
devbox run -- cargo test -p sea-forge-cli --bin sea-forge \
  --all-features --locked -- --nocapture                             PASS (5)
```

`devbox run -- just test` failed twice: both runs ended with `SIGSEGV` before the `sea-forge-cli` `src/main.rs` unit binary reported any test output. The isolated binary command above passed, so the fault is suite-context dependent and remains unresolved. Real ACP-host and SWE_SEED-host release tests remain ignored because operator configuration is unavailable; they are not counted as passing evidence.

## Recommended order

1. Resolve the critical cell-import traversal and ACP redaction defects.
2. Decide whether the minimum specification describes a preserved compatibility slice or must be revised; its command and boundary contracts currently conflict with the executable system.
3. Add failing end-to-end tests for each block above before claiming the corresponding milestone green.
