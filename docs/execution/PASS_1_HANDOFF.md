# Pass 1 Handoff — SEA Forge Census

**Completed**: 2026-07-27, branch `frontend`, commit `8361f25ff0bbfc13c56c96ccb82161f317b4d0e6`. Working tree clean except a Jolli debug log. All six required artifacts written under `docs/execution/`.

## What was inspected

- Full directory tree of root, `crates/`, `workbench/`, `.agents/specs/`, `.agents/plans/`, `.agents/reports/`, `docs/decisions/`, `.github/workflows/`, `scripts/`, `secrets/`, `spec/`, `models/`, `working/`.
- All key manifests: `Cargo.toml` (root), `rust-toolchain.toml`, `devbox.json`, `mise.toml`, `justfile`, `workbench/package.json`, `workbench/apps/desktop/package.json`, `workbench/apps/desktop/src-tauri/Cargo.toml`, `.gitignore`, `.envrc`, `.sops.yaml`.
- Canonical specs (read in part): `Shell-SPEC.md`, `spec-minimum.md`, `spec-full.md`, `spec-agent-orchestration.md`. Skimmed: `spec-adlc-thoth-minimum.md`.
- Architecture docs: `AGENTS.md` (root), `workbench/AGENTS.md`, `ARCHITECTURE.md`, `README.md`, all five ADRs (titles + purpose only for some).
- Source: `crates/sea-forge-cli/src/main.rs` (full), `crates/sea-forge-server/src/lib.rs` (120 lines), `crates/sea-forge-server/src/sfwp/mod.rs` (250 lines), `crates/sea-forge-server/src/config.rs` (80 lines), `crates/sea-forge-server/src/main.rs` (full, 39 lines), `crates/sea-forge-agent/src/delegation.rs` (around 4 `unimplemented!()` sites, confirmed all in test stubs).
- Workbench: `workbench/apps/desktop/src/router.tsx` (full), `workbench/apps/desktop/src-tauri/src/bridge.rs` (100 lines), `workbench/apps/desktop/src-tauri/src/` directory listing, `workbench/packages/contracts/` (generation verified), `workbench/apps/desktop/src/pages/` and `src/shell/` directory listings.
- Memory artifacts: `.agents/CURRENT_STATUS.md` (lines 1-1026; the file is ~2402 lines, the rest is historical M13 detail), `.agents/OBSERVED_DEBT.md` (lines 1-120 of ~833).
- Git: status, last 30 commits, branch list, unpushed-commit list.

## What could NOT be inspected

- **`crates/sea-forge-domainforge` actual semantic engine**: ADR-001 says it should bind to a first-party `domainforge-core` library; the adapter exists but whether the library is vendored was not verified. Treat any DomainForge-dependent claim with medium confidence.
- **Full text of `.agents/CURRENT_STATUS.md` (lines 1027-2402)**: read in part only; contains historical M13 implementation chronology. Skim before relying on specific M13 details.
- **Full text of `.agents/OBSERVED_DEBT.md` (lines 121-833)**: read in part only; additional entries likely exist beyond what was summarized.
- **`CONTRIBUTING.md`, `docs/ci-cd-architecture.md`, `docs/skills/release-management.md`**: referenced but not opened.
- **The `models/*.sea` files**: present, not parsed.
- **Jolli Memory and `.ua/` knowledge graph**: not consulted beyond confirming they exist (treated as retrieval aids, not truth, per AGENTS.md).
- **`.codex/`, `.claude/`, `.entire/`, `.serena/`, `.gemini/`, `.omc/`, `.opencode/`, `.superpowers/`**: present but gitignored / auxiliary agent tooling; not inspected.
- **Playwright e2e actual run**: not re-executed this pass (CURRENT_STATUS 2026-07-25 reports green).

## Highest-confidence findings

1. **The product builds and its tests pass**: 776 Rust + 133 TS = **909 tests, 0 failures**. `just proof` (P1-P4b) green. Live server smoke confirmed end-to-end SFWP round-trip.
2. **The implementation is far ahead of the README/ARCHITECTURE narrative**. Both docs describe a "minimum kernel (two-crate)" but reality is 22 crates covering M0-M16 of the additive specs, plus a working Tauri/React/Bun desktop frontend.
3. **The kernel async-isolation invariant holds**: only `sea-forge-server` and `sea-forge-agent` carry `tokio`/`reqwest`; `just no-async-kernel` enumerates 19 kernel crates that must remain pure.
4. **SFWP method coverage is 17% (21/123)** of the catalog target. The missing methods are planned (workbench plan Tasks 12-14), not abandoned; the gap is the largest single source of "specification preview · not live" frontend surfaces.
5. **Secrets hygiene is intact**: only `secrets/dev.enc.env` tracked; plaintext `dev.env` correctly gitignored; age key lives outside the repo.
6. **The `.sea-forge/` runtime store is not committed** and is properly gitignored.
7. **Zero `todo!`/`unimplemented!`/`FIXME`/`panic!` in production Rust paths**. All four `unimplemented!()` matches are inside test stub providers (intentional, only the `complete()` branch exercised).

## Highest-risk unknowns

1. **Whether `frontend` should merge to `main`** before any release. Local `main` exists but is stale relative to the 5 unpushed commits on `frontend`. CI runs against `main` per `.github/workflows/ci.yml`.
2. **Real ACP/SWE_SEED/Seatbelt release behavior**. Portable tests pass; real-host behavior is `#[ignore]`-gated and unverified.
3. **Whether `working/face/` is intentional** or leftover. It describes a different project ("face") and should not influence sea-forge decisions.
4. **Whether the DomainForge first-party library binding is real or stubbed**. ADR-001 commits to it; the adapter crate exists; the underlying library binding was not verified this pass.
5. **Whether `just check`'s composite gates (`cargo deny`, `gitleaks`) are currently green**. They were not run as a composite this pass; both were last reported green in CURRENT_STATUS 2026-07-23.
6. **Whether the workbench frontend is gated in CI**. It is not (`.github/workflows/ci.yml` has no `workbench-check` job); the frontend relies on local discipline only.

## Commands the next agent should run first

```sh
# 1. Re-confirm the baseline (<3 minutes):
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
devbox run -- just proof
cd workbench && bun run check && bun run test

# 2. Then the composite gates not run this pass:
devbox run -- just check               # adds cargo deny + gitleaks + clippy + context
cargo test --workspace --all-features --locked --no-fail-fast

# 3. To exercise the live server yourself:
mkdir -p /tmp/probe/state
SEA_FORGE_ROOT=/tmp/probe/state ./target/debug/sea-forge-server &
# then connect to .sea-forge/server.sock (NOT /tmp/probe/state/server.sock — see P2-SOCK)
python3 -c "import socket,json; s=socket.socket(socket.AF_UNIX); s.connect('.sea-forge/server.sock'); s.sendall((json.dumps({'verb':'system_hello','protocol_version':'1'})+'\n').encode()); print(s.recv(8192).decode())"

# 4. To inspect the Workbench dev experience:
cd workbench && bun run dev    # http://localhost:1420
```

## Documents the next agent should treat cautiously

- **`README.md` and `ARCHITECTURE.md`**: stale; describe a "minimum kernel" that no longer reflects reality. Trust the specs and the source code, not these summaries. (See INCOMPLETENESS_INVENTORY P1-DOCS.)
- **`.agents/CURRENT_STATUS.md`**: accurate but enormous (~2402 lines). The first 200 lines summarize the most recent work (Task 7 + Task 6 + Task 5); later sections are historical chronology that may have been superseded by later "COMPLETE" annotations.
- **`.agents/OBSERVED_DEBT.md`**: entries are dated; some may have been resolved by later work without being removed. Re-verify before treating any entry as currently open.
- **`.agents/reports/2026-07-22-spec-implementation-audit.md`**: contains at least one stale finding (Thoth `ask` is now governed). See P3-STALE-AUDIT.
- **`.ua/knowledge-graph.json`**: rebuildable projection; treat as navigation aid, not authoritative proof (per AGENTS.md "Codebase Knowledge Graph").
- **`.agents/specs/api-routes.md` and `.agents/specs/datamodel.md`**: empty placeholder files. The live API/datamodel truth is elsewhere. (See P2-SPEC-EMPTY.)
- **`working/face/*`**: unrelated project, do not use as sea-forge evidence.

## Unresolved document-precedence questions

1. **Should `frontend` be the new base, or merge to `main`?** The CI workflow and the contributor happy path assume `main`. (User decision.)
2. **Should the empty `api-routes.md`/`datamodel.md` spec placeholders be deleted, filled, or left alone?** (User decision; no behavior impact today.)
3. **Should `working/face/` be removed, moved, or gitignored?** (User decision; cosmetic.)
4. **Should `just check`/CI gate the workbench frontend?** (User/team decision per ADR-004's "separate Bun workspace" framing; either choice is defensible.)
5. **Is the DomainForge first-party library binding intended to be vendored in this repo, or is the adapter crate a stub pending an external library?** (Verify against ADR-001 before any `.sea` semantic claim is load-bearing.)

---

## Summary

SEA Forge is a **substantially implemented** governed capability-execution kernel with a working desktop frontend — not the "minimum kernel" its top-level docs describe. The implementation is clean (909 tests green, no production `panic!`/`unimplemented!`, kernel async-isolation invariant preserved), the operator paths work end-to-end (CLI + Unix-socket server + Tauri host), and the architecture matches the spec's additive-milestone design. The largest gap is SFWP method coverage (17%), which is planned rather than abandoned. No correctness or basic-usability blocker (P0) was found. The next pass should focus on (a) resolving the `frontend`→`main` integration question, (b) continuing the workbench plan's vertical slices for the missing SFWP methods, and (c) closing the real-server e2e harness gap.
