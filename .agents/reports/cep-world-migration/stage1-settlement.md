# Stage 1 — DomainForge 0.18.2 convergence: settlement record (2026-10-04)

Target verified: crates.io `domainforge-core` 0.18.2; DomainForge repo commit 5003b0d; npm `@godspeedai/domainforge` 0.18.2. Installed CLI upgraded 0.18.1 -> 0.18.2 (`cargo install domainforge-core --version 0.18.2 --features cli --locked --force`).

| Consumer | Mechanism | Result | Commit | Push |
|---|---|---|---|---|
| sea-rs (`sea-forge-domainforge`) | crate, exact pin 0.16.0 -> 0.18.2; `EXPECTED_DOMAINFORGE_VERSION`; devbox pin | build ok; dependent crate tests ok; `just test` 0 failures; clippy clean on affected crates | dd2c287 (branch migration/cep-world-ref) | pushed |
| domainforge-lsp | path dep on removed `sea-core` -> published `domainforge-core =0.18.2` (alias) | did not build before; builds; 57 tests pass | a351be5 (semantic-adapter) | pushed |
| sxr (RealityTrace) | CLI binary; `with_required_version` fail-closed guard, pinned constant | workspace tests 0 failures; clippy clean; 3 real-binary tests pass on 0.18.2 | 7e7b1ab (main) | pushed |
| gauntlet | captured envelopes + sxr CLI | 0.16.0 capture marked historical; new real 0.18.2 capture; 1026 tests pass; tsc clean | 78ddd2a (migration/domainforge-0.18.2) | pushed |
| cognate | npm, already 0.18.2; bun.lock pins 0.18.2 | 29 DomainForge-facing tests pass; no change | none needed | n/a |
| domainforge-vsc-extension | grammar only | no DomainForge version dependency | none needed | n/a |
| SEA (legacy) | npm `domainforge-cli@1.0.9` | see finding 1 | f0af88c (migration/domainforge-0.18.2) | NOT pushed |
| SWE_SEED, godspeed_agent, Context_Kernel | type-gated identity / none | no DomainForge dependency to converge | none | n/a |

## Findings
1. Legacy SEA depended on npm `domainforge-cli` 1.0.9, an unrelated third-party package (not GodSpeed DomainForge). Nothing in SEA called it. Replaced by `@godspeedai/domainforge` 0.18.2 (library, no bin) and the CI pin guard retargeted. Branch is committed locally but the pre-push hook fails on generation drift that exists on `dev` independent of this change (`manifest-regen-check`, determinism check on `docs/generated/readme`). Pushing requires either fixing that drift or an operator-approved `SKIP_PREPUSH=1`. Left unpushed.
2. Semantic identity is stable across the upgrade: for the same source (`valid-basic.sea`) the 0.16.0 and 0.18.2 envelopes have identical `semantic_closure_hash` and `source_set_hash`; only `producer.version` and `self_hash` (covers the producer block) differ.
3. Stage 2 risk: if `DomainModelIdentity::canonical_digest()` covers the compiler version, `world_ref` would change on every DomainForge upgrade. Stage 2 must check this before fixing the `world_ref` rule.
4. sea-rs `just lint` is red from inherited `sfwp_supervisor` test warnings (OBSERVED_DEBT). `just manifest-regen-check` fails at baseline in SEA.
5. Gauntlet `bun run lint` fails at baseline on untouched files.
6. `domainforge` CLI version is guarded only in RealityTrace; Gauntlet gets envelopes through it.

## Gate
Met for every active consumer except legacy SEA, whose dependency is replaced on an unpushed branch. Historical 0.15.0/0.16.0 references remain only in evidence bundles, plan/spec observed-fact notes, and a deny.toml comment.
