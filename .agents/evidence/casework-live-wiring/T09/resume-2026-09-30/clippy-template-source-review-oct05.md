# Clippy template source review — 2026-10-05

## Scope and inputs

Independent source-only review of the repair requested by the ordinary pre-push
Clippy failure. The trigger is recorded in `/tmp/sea-hooks-oct05-push-retry2.raw`;
the builder note is `clippy-template-builder-oct05.md`. The reviewed target's
SHA-256 is
`5806fc58a25b7eda586019ec9abf18d664f01a996816217c1f6830995b085a1c`; the
unmodified `HEAD` version hashes to
`79c3640bb17448ada1298799850a37e86acd026fadfa7c20be3d6452c5bb1f72`.

## Source review

The complete diff in `crates/sea-forge-server/tests/case_templates_live.rs`
contains only the requested change at line 273:

```diff
-            .map_or(true, |e| e.is_empty()),
+            .is_none_or(|e| e.is_empty()),
```

No assertions, test scope, lint settings, dependencies, or other source lines
changed. The preceding `get("errors").and_then(|e| e.as_array())` produces an
`Option<&Vec<Value>>`. For `None` (missing field, or a present non-array field),
both forms return `true`; for `Some(empty)`, both return `true`; and for
`Some(nonempty)`, both apply `is_empty()` and return `false`. Thus the success
and failure conditions are preserved.

The pinned toolchain is Rust 1.92.0. Its installed official Rust core source at
`~/.rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/option.rs`
documents this method and marks it stable since Rust 1.82.0. The installed
rustdoc HTML copy was unavailable. This establishes API compatibility from the
local 1.92 source without invoking the compiler.

## Deviations and verification boundary

The task handoff notes an initial rustfmt check using its default Rust 2015
edition, followed by a successful retry with `--edition 2021`; the builder note
records only the latter command. I found no captured output for the initial
retry in the failure log or the builder note, so that first-attempt detail is
handoff-reported rather than independently evidenced here. Edition 2021 is the
repository's configured Rust edition. `git diff --check` passed for the target.

I ran no Cargo, rustc, Clippy, Go, Bun, or test command. This source review finds
the repair correct and scope-compliant, but does not approve or claim the
pending targeted Clippy or existing test gate; those remain with the designated
compiler owner.
