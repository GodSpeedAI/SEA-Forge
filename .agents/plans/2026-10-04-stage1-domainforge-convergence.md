# Stage 1 plan — DomainForge 0.18.2 convergence (non-normative)

Governing spec: `.agents/specs/cep-world-ref-migration.spec.md`.

Targets (decided 2026-10-04): sea-rs `sea-forge-domainforge` (0.16.0 -> =0.18.2); domainforge-lsp (repair stale path dep, converge); legacy SEA (upgrade `domainforge-cli` 1.0.9 line); sxr + gauntlet (explicit `domainforge` CLI version check; installed CLI 0.18.1 -> 0.18.2); cognate already 0.18.2 (verify lockfile and tests); vsc-extension (grammar only, verify).

Per consumer: upgrade, adapt API, build, run existing tests, update lockfile intentionally, verify semantic behavior unchanged, commit. Dirty repos (gauntlet, SEA) use a separate branch or worktree.
Order: sea-rs first (highest API risk), then LSP, CLI consumers, SEA, verification sweep.
