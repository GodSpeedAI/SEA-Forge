# How-To: Regenerate Contracts & Schemas

> **A guide for full-stack developers updating SFWP wire contracts across Rust, JSON Schema, and TypeScript/AJV.**

---

## Goal

Update or add an SFWP protocol type in `sea-forge-server`, regenerate its JSON Schema and TypeScript definitions, and prove that all committed projections match canonical Rust truth with zero diff.

---

## Prerequisites

* Rust 1.92.0+ and Cargo installed.
* Bun 1.4.0+ installed.
* Understanding of **ADR-005** (SFWP schema-generation pipeline).

---

## 1. The Schema Generation Pipeline

To ensure the React Workbench and SFWP server never disagree on wire shapes, SEA Forge uses a single-source-of-truth pipeline:

```text
Rust SFWP Types (crates/sea-forge-server/src/sfwp/)
  │  derive(JsonSchema)
  ▼
cargo run -p sea-forge-server --bin gen_sfwp_schema
  │
  ▼
JSON Schemas (workbench/packages/contracts/schema/*.schema.json)
  │
  ▼
cd workbench && bun run generate:contracts
  │
  ├── TypeScript Interfaces (workbench/packages/contracts/src/generated/types.ts)
  └── AJV Standalone Validators (workbench/packages/contracts/src/generated/*.validator.ts)
```

**Invariant GEN-01:** Never edit files inside `schema/` or `src/generated/` by hand. Always modify the canonical Rust source and run the generator.

---

## 2. Step-by-Step Procedure

### Step 1: Define or Modify the Rust Type
Open the appropriate module under `crates/sea-forge-server/src/sfwp/` (e.g. `case.rs`, `readiness.rs`).

Ensure your struct derives `JsonSchema`:
```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CustomStatusResult {
    pub cell_id: String,
    pub is_ready: bool,
    pub active_runs: u32,
}
```

### Step 2: Register in `SCHEMA_TYPES`
Open `crates/sea-forge-server/src/sfwp/mod.rs`. Add your new type to the `SCHEMA_TYPES` registration array:
```rust
pub const SCHEMA_TYPES: &[&str] = &[
    "EventFrame",
    "Precondition",
    "RecordDigest",
    // ...
    "CustomStatusResult",
];
```

Also update `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs` to include the `schemars::schema_for!(CustomStatusResult)` invocation.

### Step 3: Emit Intermediate JSON Schemas
From the repository root, run the Rust generator:
```sh
cargo run -p sea-forge-server --bin gen_sfwp_schema
```
This writes `<TypeName>.schema.json` files to `workbench/packages/contracts/schema/`.

### Step 4: Emit TypeScript Interfaces & AJV Validators
Run the Bun generation script:
```sh
cd workbench
bun run generate:contracts
```
This reads every JSON schema file, compiles TypeScript types via `json-schema-to-typescript`, and emits standalone AJV validator functions.

---

## 3. Verify Zero Diff

Prove that the committed contract files match the canonical Rust source:

```sh
just workbench-contracts-gate
```

If the gate outputs:
```text
[contracts-gate] schema diff clean
[contracts-gate] typescript contracts clean
[contracts-gate] standalone tauri workspace preserved
```
Your contract update is complete and ready for commit.

If git reports untracked or modified files in `workbench/packages/contracts/`, stage them alongside your Rust changes:
```sh
git add crates/sea-forge-server/ workbench/packages/contracts/
```

---

## 4. Source Trail

* [`crates/sea-forge-server/src/bin/gen_sfwp_schema.rs`](file:///c:/Users/sprim/projects/sea-rs/crates/sea-forge-server/src/bin/gen_sfwp_schema.rs) — Rust JSON Schema emitter.
* [`workbench/packages/contracts/scripts/generate.ts`](file:///c:/Users/sprim/projects/sea-rs/workbench/packages/contracts/scripts/generate.ts) — Bun script generating TS types and AJV validators.
* [`docs/decisions/ADR-005-sfwp-schema-generation.md`](file:///c:/Users/sprim/projects/sea-rs/docs/decisions/ADR-005-sfwp-schema-generation.md) — Governing architectural decision record.
