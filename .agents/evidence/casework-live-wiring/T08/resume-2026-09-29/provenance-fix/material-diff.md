# Material diff

- Added the existing `run_get` SFWP request and a narrow run/evidence/trace result view.
- `Authority.GetArtifact` now calls `artifact_get`, decodes bytes, then calls `run_get` for the exact returned run ID. It binds the returned run, case, plan item, evidence ID, URI, SHA-256, source event, and source event plan item before returning application-owned content.
- Extended `ports.ArtifactContent` with the actual case and plan item resolved by the adapter. The HTTP artifact response carries those source-backed values; incomplete ownership produces a typed unavailable response.
- Kept `invocation_id` empty because neither available kernel view records the artifact's standalone invocation ID.
- Added adapter tests for the `artifact_get` → `run_get` request sequence, real case/item/evidence association, mismatched or missing run ownership, and cross-item capture events. Updated HTTP artifact expectations and added an incomplete-ownership refusal case.
- No requested provenance values, direct private ledger reads, dependencies, schema/ID grammar, policy, or TypeScript contract changes.
