# Stage 8 settlement — execution, RealityTrace, evidence, settlement

Date: 2026-10-05. sea-rs `migration/cep-world-ref` (SEA-Forge half 12ccf4a plus lint fix); cognate `cognate/harness`. Spec: `specs/cep-world-ref-migration.spec.md` (Stage 8). Plan: `plans/2026-10-05-stage8-evidence-settlement.md` (written before coding).

## Delivered
- SEA-Forge: `submit_evidence` and `settle` over the socket (`authority_evidence`, `authority_settle`). Evidence is an append-only ledger fact keyed by envelope id; settlement criteria are bound into the allow decision by hash, so they cannot be moved afterwards; settlement is derived from ledger keys (`cep-settle:<op>:<hash>`, `cep-settle-final:<op>`). Only an `allow` operation can settle; repeat settle of an unsettled operation returns the recorded packet.
- Cognate: `SeaForgeAuthority.submitEvidence/settle`; RealityTrace `godspeed.ts` builds execution-trace and evidence-packet envelopes (execution trace is not evidence, evidence is not settlement); `sxr` CLI derives evidence from verifier `exit_code`; evidence request id includes the payload hash so a tampered packet under the same envelope id cannot poison the id.

## Verification
- sea-rs `just test` 1170 passed, 0 failed. cep's own validator accepts the trace, evidence and settlement packets.
- Cognate: typecheck clean; `just verify` 78/78 x3; `just sea-forge-live` 22 pass, 0 fail (includes the end-to-end governed-settlement test against real `sxr verify`).

## Debt (in `.agents/DEBT.md`, M-27..M-36)
- RealityTrace's own settlement remains a separate fact from SEA-Forge's.
- Only `sxr verify` records an exit code; other evidence sources have no derived direction.
- Unrelated: one-off AK-006 gate failure not reproduced; environment-dependent Cognate full-suite failures.
