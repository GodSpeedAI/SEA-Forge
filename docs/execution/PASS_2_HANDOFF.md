# Pass 2 Handoff

Date: 2026-07-27  
Baseline: branch `frontend`, commit `8361f25ff0bbfc13c56c96ccb82161f317b4d0e6`

## Architectural Truth

SEA Forge is a local governed capability-execution product with three supported
operator interfaces: the one-shot CLI, a Unix-socket application server, and a
Tauri/React Workbench. The Workbench is a typed client; authority, identity
binding, case reduction, execution, evidence, settlement, and governance truth
remain in the Rust server/kernel. DomainForge owns `.sea` semantics through the
exact-pinned `domainforge-core` library; SEA Forge owns final authority and all
side effects.

The minimum CLI lifecycle is real and source-usable. The integrated desktop
product is not complete. The Tauri host does not install or start the server,
desktop/server socket defaults disagree, invalid server config falls back to
defaults, and packaged E2E does not exist. Current Playwright tests drive Vite
with mocked Tauri IPC.

The most serious defect is not missing frontend breadth. Server case sandbox
episodes create workspace directories before authority, store runs where
`run.get` does not look, and derive settlement from allow + exit zero rather
than the full criteria/evidence pipeline. This violates the central product
invariants and can make a case-linked run uninspectable.

The smallest completion architecture preserves current technology and process
boundaries:

1. Obtain the owner's service-lifecycle choice: Tauri sidecar or separately
   installed local server. Distribute matching Workbench/server/CLI artifacts
   with one absolute cell root and socket contract.
2. Make startup/config/reload fail closed and enforce bounded requests.
3. Route every case episode through the existing full governed lifecycle.
4. Establish one run locator with compatibility for flat minimum history.
5. Resolve actor/policy/cell context at the host/server boundary.
6. Make protected request IDs idempotent and recoverable across restart.
7. Complete the representative journey as source-backed vertical slices.
8. Prove the packaged Linux stack through real Tauri/SFWP/server/records, then
   macOS before advertising it verified.

The 123-method catalog is explicitly target-unmapped. It is a mapping backlog,
not a completion checklist. Containers, remote HTTP, Windows/mobile, hosted
services, MicroVM, NATS, and marketplaces are not required by the current local
product boundary.

## Discrepancies With Pass 1

- Its artifacts are untracked, Copilot instructions exist, and `working/face/`
  is tracked low-authority history.
- DomainForge binds real `domainforge-core 0.15.0`; route guards instead are
  overstated because production context is fabricated and only G1 normally runs.
- “0 P0,” “working desktop,” and “operator paths end-to-end” are unsupported by
  independent component builds and mocked browser IPC.
- Several gates were inferred rather than run.
- Pass 1 missed the pre-authority workspace writes, incomplete case episode
  settlement/records, and case-run versus run-view path mismatch.

## DAG And Chokepoints

- Sole graph root: `N00` truth baseline. First package layer: `N01`
  owner-selected distribution/cell contract, `N03` canonical episode pipeline,
  `N06` idempotency, and `N07` generated gates. After N01, start `N02`
  service lifecycle and `N05` identity in parallel with N04 run compatibility.

Then execute vertical slices `N08` through `N11`, packaged proof `N12`, and
release gate `N13`.

- `crates/sea-forge-server/src/case_dispatch.rs`: full episode lifecycle and
  pre-authority side effects. One package owns this file at a time.
- Run storage: full spec case-owned layout versus minimum flat compatibility and
  current SFWP run views.
- SFWP request enum/schema registry and Tauri bridge: every cross-layer slice
  touches these shared contracts.
- Server lifecycle: root/socket resolution, config preflight/reload, the
  owner-selected lifecycle, and timeout behavior must agree before UI expansion.
- Actor context: shell, host, SFWP, approvals, authority records, and SoD must use
  one resolved identity.
- Packaged E2E: the first proof that all independently tested components form a
  usable product.

## Packaging Constraints

- Authority precedes every side effect; settlement, not exit, determines success.
- One authority fabric and exact grants; no renderer/provider/sandbox bypass.
- Append-only records are truth; views and indexes are rebuildable.
- DomainForge owns semantics but only SEA Forge issues final authority.
- Renderer uses closed typed Tauri commands and never reads the store/socket.
- Synchronous kernel and standalone Tauri workspace remain isolated.
- Rust SFWP types generate committed JSON Schema and TS/AJV.
- Linux primary/macOS secondary; Bun development-only; SFWP remains local Unix
  socket v1.
- Existing minimum IDs, records, exit behavior, flat history, and P1-P4b proofs
  remain compatible.
- Unmapped catalog breadth may be deferred; preview paths cannot support a
  completion claim.

## Unknowns And Readiness

- Exact Linux package format and public signing channel. This does not block a
  local package proof.
- Whether the owner wants public crates.io distribution of all internal Rust
  libraries. Binary CLI distribution is sufficient meanwhile.
- Real external ACP/SWE_SEED/witness/IFL evidence for releases that advertise
  those optional integrations.
- macOS packaged/Seatbelt result; it must remain “not run” until executed.
- Whether tracked `working/face/` should later be archived or deleted.

**Conditionally ready for work packaging, not ready for product release.**
Backend integrity packages can be defined now. Distribution/service-lifecycle
packaging is blocked on U-06. Persisted-layout, public SFWP, dependency, CI, and
deployment changes retain the repository's ask-first gate. Remote/multi-user
scope, public library publication, release credentials, license composition, or
destructive migration also require owner input.

Authoritative package definitions: `EXECUTION_DAG.md`. Completion gate:
`PRODUCT_COMPLETION_DEFINITION.md`. Detailed reconciliation:
`CONTRADICTIONS_AND_DECISIONS.md`.
