# Hostile thrown-value repair result receipt

Date: 2026-10-07. Source and focused test are frozen for independent review. No test, Bun, compiler, typecheck, formatter, Git, or gate command was run.

## Resulting source identities

- `localAdapter.ts`: SHA-256 `5a876ec276ff79941e17cf8e3ca472ca8108273ae0a5bb0f3f10aff6d1a0b958`.
- New `localAdapter.hostileThrownValues.test.ts`: SHA-256 `f29740cc973974bbffb16652f8ed75b4789e2d05828d16a35ed3d833d2645da4`.
- Frozen settlement supplement: `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819`.
- Frozen cursor-bounds test: `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a`.
- Frozen cursor-order test: `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`.
- Frozen conformance fixture: `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.

## Narrow change summary

`asError` now contains both `instanceof Error` and `String(value)` inside one `try/catch`; if either throws, it returns an `Error` with a fixed fallback message and performs no further conversion. Ordinary Error instances are still returned unchanged, and ordinary non-Error coercions retain their string message.

The new controlled-timer test covers a null-prototype object, throwing `Symbol.toPrimitive`, throwing `toString`, and a Proxy whose `getPrototypeOf` trap throws during `instanceof`. For each value, it asserts the drain callback does not throw, the offending subscriber receives one fallback Error and is disposed, a healthy subscriber receives that event, and only the healthy subscriber receives a later event. Timer globals are restored from `finally`. The test was added before the source fix; it was not executed and is not described as baseline-RED evidence.

No production algorithm beyond error-value normalization, public contract, dependency, schema, configuration, frozen fixture, conformance file, or settlement atomicity source/test was changed. Existing shared mutable event-object delivery remains the previously documented scope/debt note from the independent review; this repair does not authorize or include a cloning algorithm.

## Preservation and evidence boundary

Root archived exact UTF-8 JSON-content source preimages as `local-adapter-828069-preimage-utf8-json-oct07.json` and `local-settlement-35f2a8-preimage-utf8-json-oct07.json`. Root reports decoded byte-for-byte equality with the original files and expected hashes `828069b6c09d07c728778356c8921bfa553201f7882ffbbdfe83c86d96c21074` (38,106 bytes) and `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819` (7,631 bytes). These JSON content containers, not my `.raw` attempts, are the exact preservation evidence.

The package-local `.raw` attempts were created before edits with an add-file patch that appended an extra LF. They are deliberately left untouched, are **not byte-exact**, and have hashes `023809bf83d82e06027c2dc0c425a6f2af228d2134c1cf23680c595ce902b2b5` and `ba01bcade7c4ca788cc35a6355a67e23f8a919ba5ebd56afe342c63ad07fa6d1`. An earlier note incorrectly inferred that the original files lacked a final LF; this receipt corrects that inference based on root's decoded-byte comparison. No `cp` or shell file-copy operation was used.

The original full assignment and reviewer finding remain preserved in the immutable pre-edit record `local-ordinary-cursor-hostile-throws-repair-record-oct07.md`. Runtime GREEN, typecheck, full UI gates, and production/integration claims remain pending a different independent critic and root authorization.
