# Stage 5 settlement — Cognate GovernedOperation choke point

Date: 2026-10-04. Repo: cognate, branch `cognate/harness`, commit dbc6b27 (pushed). Evidence: `cognate/.agents/evidence/2026-10-05-governed-operation-stage5.md`.

## Delivered
- `packages/runtime-bun/src/governed-operation.ts`: `Governor`, `GovernedOperation` (action | capability), CEP dispositions (7, never boolean), `GovernanceAuthority` port, modes `sea-forge` | `off`, `worldRef` grammar check.
- All 14 `ActionPolicy` sites and the kernel `Policy` path (Invoker) route through `Governor`. A test fails if any other source file calls local policy.
- sea-forge mode: authority decides against the pinned world_ref AND local policy must allow (authority != capability). Only `allow` proceeds. Authority error/garbage -> `unknown` -> refuse. Misconfiguration fails at `createRuntime`.

## Verification
- runtime-bun 70/0 (10 new); tsc clean; `just verify` 78/78.
- Full suite 948 pass / 77 fail vs baseline 938 / 77: same pre-existing failures.

## Debt / limits
- Default mode remains `off` until Stage 6 supplies the real authority.
- constrained_allow / boundary / degraded / escalate are refused, not honored (Stages 6-7).
- 77 inherited failing tests (AG-UI interop, E2E, J-*) not investigated.
- Own mistake fixed: Cognate's revision-31 status snapshot lacked `---`, which had broken `just verify`.
