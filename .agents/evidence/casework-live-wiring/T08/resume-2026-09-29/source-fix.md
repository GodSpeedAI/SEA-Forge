# T08 source selector fix (2026-09-29)

## Scope

Changed `apps/godspeed-cognitive-ui/src/main.tsx` only. Production source is
now fixed to `live` by Vite's compile-time `import.meta.env.DEV` condition,
regardless of `VITE_CASEWORK_SOURCE`. Development keeps the existing behavior:
local by default, explicit `live` selects HTTP, and other values select local.
The local adapter remains behind the existing dynamic import in the local
branch.

This addresses the T08 guardrail in
`.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml`: production
must be HTTP-only, while local remains available for dev and tests.

## Baseline evidence

Before the edit, the parent reported the independent critic's alternate
production-build reproduction: both `VITE_CASEWORK_SOURCE=local` and a typo
value exited 0 and emitted `localAdapter-Cy-YJ1r8.js`. This is the baseline
reported by the parent; this builder did not run those builds.

## Verification

No compile, test, or build was run by the builder because the parent reserved
that token for the independent critic. The production output check across
default, `local`, and invalid `VITE_CASEWORK_SOURCE` values remains pending.
