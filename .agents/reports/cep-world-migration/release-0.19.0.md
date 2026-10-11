# DomainForge 0.19.0 release and consumer re-convergence (2026-10-04)

## Release
- DomainForge PRs merged by the operator: #132 (world_ref, squash 86aaf6f), #133 (world-bound snapshot, 497a199), #134 (release 0.19.0). Tag `v0.19.0`; GitHub release published.
- Published: crates.io `domainforge-core` 0.19.0; PyPI `domainforge` 0.19.0 (sdist + 15 wheels, CPython 3.11-3.13, macOS x86/arm, Linux x86/arm, Windows); npm `@godspeedai/domainforge` and `@godspeedai/domainforge-wasm` 0.19.0. The Deploy run completed with every job successful.
- cep PR #1 merged (58f3bd6): canonical full schema plus the GodSpeed profiles are on `main`. Follow-up cep PR #2 (renumbers the stacked-branch question to OQ-0010; the original OQ-0006 collided with an existing entry) is open.

## What the release PR needed (and what to repeat next time)
The release-please PR failed 8 CI jobs because the cross-binding golden hashes embed the release version. The project rule (in `application_cross_binding_golden_tests.rs`) is to regenerate them with every version bump, and the 0.18.2 release commit changed the same files. Commit `2e8f4eb` on the release branch regenerated the Rust (contract + envelope), Python, TypeScript and WASM constants plus `Cargo.lock`, after verifying each binding against a local 0.19.0 build. Then all 31 checks passed. Expect to do this on every release until the goldens stop embedding the version.

## Consumers on 0.19.0 (all verified, committed, pushed)
| Consumer | Change | Verification | Commit |
|---|---|---|---|
| sea-rs `sea-forge-domainforge` | `=0.19.0`, `EXPECTED_DOMAINFORGE_VERSION`, devbox, lockfile | no API change; `just test` 1110/0; `just check` green | d679fa0 (migration/cep-world-ref) |
| sxr (RealityTrace) | `PINNED_DOMAINFORGE_VERSION`, env doc; CLI installed 0.19.0 | workspace 0 failures; 3 real-binary tests pass | ee48738 (main) |
| cognate | 2 manifests, bun.lock, `BINDING` constant, README, projection golden | adapter tests 29/0, full suite 678/0, `just verify` 78/78; golden differs only in the binding string | d49fdcf (cognate/harness) |
| domainforge-lsp | `=0.19.0` | 57/0 | 1e35f0c (semantic-adapter) |
| gauntlet | new real 0.19.0 capture; 0.18.2 capture marked historical | 1028 tests, lint, tsc; fingerprint unchanged | 0780eaf (migration/domainforge-0.18.2) |
| SEA (legacy) | `@godspeedai/domainforge` 0.19.0 | frozen install, manifest check, determinism; pushed through the hook | 8d91dff7b (migration/domainforge-0.18.2) |

Cognate carried a second pin the manifests did not show (`BINDING` in `packages/domainforge/src/types.ts`), caught by its own `just verify` skew check.

## Findings
- Semantic identity is stable across 0.16.0, 0.18.2 and 0.19.0: the same source keeps `semantic_closure_hash` 1a1b7053...; only `producer.version` and the document `self_hash` change. world_ref changes per release by design (decision: digest as-is).
- No stale 0.18.x pin remains in any active manifest, constant or lockfile; installed CLI is 0.19.0.
- Not done and not needed: domainforge-vsc-extension (grammar only).
