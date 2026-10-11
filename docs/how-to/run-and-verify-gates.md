# How-To: Run and Verify Project Gates

> **The standard verification progression for developers: from fast inner-loop checks to normative proofs and CI gates.**

---

## Goal

Run the appropriate `just` verification recipes during development, ensuring your changes preserve Rust formatting, Clippy lints, the synchronous kernel boundary, and normative conformance proofs without paying unnecessary full-workspace compile times.

---

## Prerequisites

* `just` command runner installed.
* Repository dependencies installed (`devbox install` or `cargo fetch --locked`).

---

## 1. Fast Inner Loop (Development Iteration)

When editing a localized Rust file, do **not** run full CI. Follow this progression to minimize iteration time:

### Step 1: Type-Check the Affected Crate
```sh
just crate-check sea-forge-core
```
Runs `cargo check -p sea-forge-core --locked` in under 2 seconds.

### Step 2: Run the Targeted Test
```sh
just crate-test sea-forge-core ids_follow_the_persisted_grammar
```
Runs only the test matching the name within that specific crate.

### Step 3: Run Fast Pre-Commit Checks
```sh
just check-fast
```
Runs `scripts/check-agent-context.sh`, checks formatting (`cargo fmt -- --check`), and runs workspace type-checking in under 10 seconds.

---

## 2. Verify Key Architectural Boundaries

Before opening a pull request or submitting code for review, verify the architectural invariants:

### Check the Synchronous Kernel Boundary (BUILD-01)
```sh
just no-async-kernel
```
Inspects dependency trees across all 19 kernel crates to prove that zero async runtimes (`tokio`) or HTTP clients (`reqwest`) were accidentally introduced.

### Check Formatting & Lint Cleanliness
```sh
just fmt-check
just lint
```
Runs `cargo fmt` check and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

---

## 3. Run Normative Conformance Proofs (P1–P4b)

To prove that your changes have not broken the core governance lifecycle:

```sh
just proof
```
Executes the four normative proofs defined in `.agents/specs/spec-minimum.md`:
* **P1:** Complete lifecycle records presence, artifact hashes, and non-empty settlement basis.
* **P2:** Deterministic plan and authority hashes across identical runs.
* **P3:** Fail-closed authority denial and empty workspace.
* **P4 / P4b:** Generated zone write denial and false success rejection.

---

## 4. Run Workbench Contract Gates

If you modified types in `sea-forge-server/src/sfwp` or frontend contracts under `workbench/`:

```sh
just workbench-contracts-gate
```
Asserts three critical properties:
1. **Schema Zero-Diff:** JSON schemas in `workbench/packages/contracts/schema/` match Rust types.
2. **TypeScript AJV Zero-Diff:** TypeScript validators match generated JSON schemas.
3. **Standalone Workspace:** Confirms `src-tauri` preserves its standalone `[workspace]` boundary.

To run the full Workbench test suite:
```sh
just workbench-check
```

---

## 5. Full Platform CI Union

Before tagging a release or completing major milestone work:

```sh
just ci
```
Runs the complete platform CI suite: agent context check, formatting check, Clippy lints, workspace typecheck, supply-chain scan (`cargo-deny`), security secret scan (`gitleaks`), synchronous kernel boundary check, and full workspace unit and integration tests.

---

## 6. Summary of Recipes

| Scope | Command | When to Run |
|---|---|---|
| Inner loop check | `just check-fast` | After every small code edit |
| Crate typecheck | `just crate-check <crate>` | While editing a single crate |
| Crate test | `just crate-test <crate> <test>` | While testing a single fix |
| Async boundary | `just no-async-kernel` | After adding or upgrading Cargo dependencies |
| Normative proofs | `just proof` | Before claiming any lifecycle or kernel task complete |
| Workbench contracts | `just workbench-contracts-gate` | After editing SFWP wire contracts or UI tokens |
| Full platform CI | `just ci` | Prior to merge, PR creation, or final milestone handoff |
