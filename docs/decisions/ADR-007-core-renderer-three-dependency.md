# ADR-007 — CORE renderer depends on npm `three`

Status: accepted (2026-09-22, Gargantua white-label integration).

## Context

The adopted CORE implementation is the Gargantua single-file WebGL renderer,
whose donor loads `three.min.js` r128 from cdnjs at runtime. The Workbench is
a Tauri 2 desktop app with a strict CSP and an offline-first packaging story
(`just workbench-package` ships static renderer assets + a supervised
sidecar): a runtime CDN script is not shippable.

## Decision

Depend on `three@0.180.0` (+ `@types/three@0.180.0`, dev) from the existing
Bun workspace instead of vendoring the donor's r128 UMD build or the CDN tag.

## Consequences

- One mechanical adaptation follows: donor `THREE.PlaneBufferGeometry` is
  `THREE.PlaneGeometry` (same buffer-backed fullscreen quad; documented as
  deviation D-02 in `src/core/PROVENANCE.md`). No shader, constant, ordering,
  or behavior change.
- Supply-chain surface grows by one pinned dependency + its types; both are
  exact-pinned per `bunfig.toml` (`exact = true`, frozen lockfile updated in
  the same change). No other dependency was added.
- The donor's runtime CDN fetch disappears from the product path; the golden
  reference at `public/reference/gargantua.html` keeps its original script
  tag untouched for comparison.

## Alternatives considered

- Vendoring `three.min.js` r128 locally: preserves the identifier verbatim
  but pins an unmaintained UMD build with no types and no security updates.
  Rejected.
- Rewriting passes on another engine/postprocessing stack: explicitly
  forbidden by the adoption brief (donor preservation). Rejected.
