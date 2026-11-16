# sea-forge-core — src/yaml

## Overview
Strict YAML boundary for the workspace. [strict.rs] is a private serde adapter
(`mod strict` in [../yaml.rs]) that enforces YAML typing rules at the parsing
boundary; the parent module [../yaml.rs] is the only module allowed to name
`serde_saphyr`. The adapter decides what each Rust target type may accept, so
no call site can be parsed with permissive coercion rules.

## Interface
- `strict::Strict<D>` — wrapping deserializer applying the rules at every depth (nested values, sequence elements, map keys/values, enum payloads) [strict.rs:Strict].
- `strict::Mode::{Shape, Spelling}` — the boundary parses each document twice: `Shape` checks types with `deserialize_any`; `Spelling` reads integer targets through `deserialize_str` and requires canonical decimal text `-?(0|[1-9][0-9]*)` with `-0` excluded [strict.rs:Mode].
- `StrictVisitor`, `BoolSpelling`, `Spelling`, `StrictSeed` — visitor/seed wrappers gating each visit by expected shape [strict.rs].
- The module is `pub(super)`: not part of the crate's public surface; it is reached only through `yaml::from_str`/`yaml::from_slice` [../yaml.rs:mod strict].

## Invariants
- bool: only a real boolean whose source text is exactly `true` or `false` [strict.rs:module docs].
- Integers: plain canonical decimal only; `1_000` and other spellings are rejected [strict.rs:module docs].
- Floats must be finite; non-finite values are rejected by the parser option `reject_non_finite_typeless_float` [strict.rs:module docs].
- Null for a collection or required string is an error; only `visit_seq`/`visit_map`/`visit_enum` are accepted for those shapes [strict.rs:module docs].
- Known hole (documented): values serde buffers into its private `Content` type (`#[serde(flatten)]`, internally tagged enums) are read with `deserialize_any`, so a `1_000` integer spelling is accepted there; quoted strings, null collections and quoted booleans are still rejected [strict.rs:module docs].

## Side Effects
- ["None"] — pure parsing adapter; no disk, network, or process effects.

## Verification
- Run: `cargo test -p sea-forge-core` (repo-wide: `just test`)
