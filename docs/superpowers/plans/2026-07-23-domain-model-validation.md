# Domain Model Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reject DomainForge import closures deeper than 16 and surface every adapter validation failure as the stable `domain_model_error` class.

**Architecture:** SEA Forge will call DomainForge's existing public `resolve_semantic_envelope` to obtain the canonical resolved import graph, calculate the entry-rooted maximum depth, and reject over-limit closures before graph construction or any side effect. The adapter will return `ForgeError::Plan { class: "domain_model_error" }`; post-start `ForgeError::Run` failures retain their existing `internal_error` trace class.

**Tech Stack:** Rust 2021, `domainforge-core 0.15.0`, existing SEA Forge conformance tests.

## Global Constraints

- No DomainForge repository or public API change.
- No dependency, persisted-schema, ID, or exit-code change.
- Import depth is edge count: entry depth is 0; a direct import is depth 1; depth 16 is accepted and depth 17 is rejected.
- Enforce before returning `DomainModel`; no authority or side effect is permitted on rejected input.
- Preserve existing `domain_model_error: ...` display text.

---

### Task 1: Preserve Typed Domain-Model Error Classes (Complete)

**Files:**
- Modify: `crates/sea-forge-domainforge/src/lib.rs:293-295`
- Modify: `crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs:248-262`

**Interfaces:**
- Produces: `domain_model_error(message) -> ForgeError::Plan { class: "domain_model_error", .. }`.

- [ ] **Step 1: Write focused failing assertions**

```rust
let err = load_validate(&missing_import_source_set()).unwrap_err();
assert_eq!(err.class(), "domain_model_error");
assert!(matches!(err, ForgeError::Plan { class: "domain_model_error", .. }));

```

- [ ] **Step 2: Run focused tests and confirm failure**

Run: `cargo test -p sea-forge-domainforge no_source_error_reaches_planning_or_authority`

Expected: FAIL because validation returns `internal_error`.

- [ ] **Step 3: Implement the minimal classification change**

```rust
fn domain_model_error(message: String) -> ForgeError {
    ForgeError::Plan {
        class: "domain_model_error",
        message,
    }
}
```

- [ ] **Step 4: Run the focused tests and format check**

Run: `cargo fmt --all -- --check && cargo test -p sea-forge-domainforge no_source_error_reaches_planning_or_authority`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sea-forge-core/src/errors.rs crates/sea-forge-domainforge/src/lib.rs crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs
```

### Task 2: Enforce Canonical Import-Graph Depth (Complete)

**Files:**
- Modify: `crates/sea-forge-domainforge/src/lib.rs:10-18,198-208`
- Modify: `crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs`

**Interfaces:**
- Consumes: `resolve_semantic_envelope(entry_uri, sources_json) -> CanonicalSemanticEnvelopeDocument`.
- Produces: `max_import_depth(&[CanonicalModuleRef], &[CanonicalImportEdge]) -> Result<usize, ForgeError>`.
- Produces: `domain_model_error("import depth 17 exceeds limit 16")` for an over-limit closure.

- [ ] **Step 1: Write failing depth conformance tests**

```rust
assert!(load_validate(&linear_import_chain(16)).is_ok());

let err = load_validate(&linear_import_chain(17)).unwrap_err();
assert_eq!(err.class(), "domain_model_error");
assert!(err.to_string().contains("import depth 17 exceeds limit 16"));
```

`linear_import_chain(depth)` creates `entry.sea` plus `depth` modules using relative imports; every source has its declared SHA-256.

- [ ] **Step 2: Run the focused depth test and confirm failure**

Run: `cargo test -p sea-forge-domainforge import_depth_limit`

Expected: the depth-17 case FAILS because `MAX_IMPORT_DEPTH` is only provenance metadata.

- [ ] **Step 3: Implement canonical graph-based depth calculation**

```rust
let envelope = resolve_semantic_envelope(&source_set.entry_uri, &sources_json)
    .map_err(|diags| domain_model_error(format_diagnostics(&diags)))?;
let import_depth = max_import_depth(&envelope.envelope.modules, &envelope.envelope.import_graph)?;
if import_depth > MAX_IMPORT_DEPTH {
    return Err(domain_model_error(format!(
        "import depth {import_depth} exceeds limit {MAX_IMPORT_DEPTH}"
    )));
}
let graph = resolve_application_graph(&source_set.entry_uri, &sources_json)
    .map_err(|diags| domain_model_error(format_diagnostics(&diags)))?;
```

Use the sole zero-inbound canonical module as the root, then perform a visited, depth-carrying traversal. This retains DomainForge's normalization of the caller's entry spelling; an inconsistent graph produces `domain_model_error`, never an allow.

- [ ] **Step 4: Run focused and crate tests**

Run: `cargo test -p sea-forge-domainforge import_depth_limit && cargo test -p sea-forge-domainforge`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sea-forge-domainforge/src/lib.rs crates/sea-forge-domainforge/tests/conformance_m0_domainforge.rs
```

### Task 3: Verify and Record the Completed Deferrals (Complete)

**Files:**
- Modify: `.agents/CURRENT_STATUS.md`
- Modify: `.tmp/cr.md` (gitignored local review ledger)

- [ ] **Step 1: Update status**

Record the two resolved review findings, exact test commands, no DomainForge repository change, and the decision to use `resolve_semantic_envelope`’s public canonical import graph.

- [ ] **Step 2: Run repository gates**

Run: `devbox run -- just context-check && devbox run -- just check && devbox run -- just test`

Expected: all applicable gates pass; report documented platform skips exactly as emitted.

- [ ] **Step 3: Commit**

```bash
git add .agents/CURRENT_STATUS.md
```

## Self-Review

- Spec coverage: Task 1 implements the stable pre-run validation class while preserving the existing post-start `Run` trace classification; Task 2 makes `MAX_IMPORT_DEPTH` an active pre-model bound without a DomainForge API change; Task 3 records evidence.
- No placeholders: every task includes files, typed interfaces, a failing test, expected failure, implementation, verification, and commit command.
- Type consistency: `resolve_semantic_envelope` returns a public envelope with `CanonicalImportEdge { importer, imported }`; depth calculation consumes those IDs and `MAX_IMPORT_DEPTH` remains `usize`.
