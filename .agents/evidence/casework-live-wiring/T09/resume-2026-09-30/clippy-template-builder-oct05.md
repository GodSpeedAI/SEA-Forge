# Clippy template test repair — 2026-10-05

## Trigger

The ordinary pre-push gate recorded in `/tmp/sea-hooks-oct05-push-retry2.raw`
ran `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
and failed at `crates/sea-forge-server/tests/case_templates_live.rs:270-273`
with `clippy::unnecessary_map_or`. Clippy suggested replacing the `map_or`
with `is_none_or`.

## Change

Changed only the preflight `errors` assertion:

```diff
-            .map_or(true, |e| e.is_empty()),
+            .is_none_or(|e| e.is_empty()),
```

The target file SHA-256 changed from
`79c3640bb17448ada1298799850a37e86acd026fadfa7c20be3d6452c5bb1f72` to
`5806fc58a25b7eda586019ec9abf18d664f01a996816217c1f6830995b085a1c`.

## Semantics

The expression before and after the edit has the same behavior: if `errors`
is absent or is not an array, `and_then` yields `None` and the assertion
condition is true; an empty array yields true; a nonempty array yields false.
No assertion, test scope, or error meaning changed.

## Verification and limits

`rustfmt --edition 2021 --check crates/sea-forge-server/tests/case_templates_live.rs`
passed. No compiler, Clippy, or test command was run for this repair; the
independent critic owns those checks. This note does not claim the pre-push
gate is green.
