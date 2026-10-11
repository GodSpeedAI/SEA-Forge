# Original bounded task

Repair T08 artifact provenance in the live Go gateway.

- Resolve real `case_id` and `plan_item_id` from the artifact's owning run with existing SFWP `run.get` (`run_get` wire enum, `run.get` catalog method).
- Verify the returned run matches `artifact.get` and the evidence/source event is associated with that run and item when the records expose that relationship.
- Refuse unavailable ownership with a typed refusal. Do not read private ledgers directly.
- Preserve bytes, digest, size, UTF-8 refusal, and verified session perspective before artifact lookup.
- Never stamp requested metadata or invent IDs. Leave `invocation_id` empty because the artifact/run views do not expose a standalone invocation ID; do not alias `run_id` or mint one.
- Keep changes within `authority.go`'s `GetArtifact`, new provenance helper/tests, the narrow `run.get` request/result types, `ArtifactContent` fields if needed, and artifact handler/tests. Do not touch T07's projection/live/store/relay/`server.go` work or the other T08 builder's obsolete-query test.
- No dependencies, schema/ID grammar/policy/security/public TypeScript contract changes. Source-only; no compile/test/build token is assigned while Go source is active.
