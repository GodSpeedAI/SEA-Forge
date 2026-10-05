# Stage 6 settlement — Cognate <-> SEA-Forge CEP authority loop

Date: 2026-10-05. sea-rs branch `migration/cep-world-ref`; cognate `cognate/harness` 920b990 (pushed). Spec/plan: `specs/cep-world-ref-migration.spec.md` (Stage 6), `plans/2026-10-05-stage6-cep-authority-loop.md`.

## Delivered
- SEA-Forge: `sea-forge-authority::cep` (parse -> world check via Stage 4 registry -> real `PolicyAuthorityEngine` as reserved `cognate_action`/`cognate_capability` -> append-only commit idempotent by operation_id -> CEP `authority_decision`). `authority_request` verb on the existing socket (protected, identity-gated, correlated by request_id, replay returns the recorded decision). Config `cep_authority` is off by default.
- Dispositions: allow, deny, escalate, boundary, degraded all emitted; constrained ones keep constraints, policy basis, compensating controls; a constrained disposition lacking them degrades to `unknown`, never allow.
- Cognate: `SeaForgeAuthority` + enforcer-gated constraint handling in the Governor (see cognate evidence 2026-10-05-cep-authority-loop-stage6.md).

## Verification
- sea-rs `just test`: 1,136 passed, 0 failed (+15). fmt, clippy, typecheck clean.
- CEP's own validator (CEP_REPO) accepts the request and all five emitted decision shapes (Rust gated test) and Cognate-built action/capability requests (Bun gated test).
- Live: real sea-forge-server + DomainForge CLI + Cognate runtime, 5/5; fail-closed on unregistered world and unreachable server.

## Findings
- World identity includes logical URIs; DomainForge's CLI derives them relative to the entry's directory. SEA-Forge's first world config used root-relative paths and minted a different world_ref. Config now models `base` + entry-relative files.
- Policy loader and engine each keep a closed vocabulary of operation kinds; added `cognate_action`/`cognate_capability` to both (additive, unlisted kinds still deny).

## Debt / limits
- Production wiring is not enabled: harness profile still runs governance `off`; no enforcers registered, so boundary/degraded are refused in practice (first candidate: `timeout_secs`).
- Evaluated actor is the verified service identity; per-end-user policy cannot be expressed by current rules.
- World registry is rebuilt per request (no cache); acceptable now, measure before production.
- Cognate gate CG-PRF-002 is flaky under `just verify` (~1 in 4, reproduced on the pre-change commit dbc6b27); not investigated.
- Live loop tests are skipped without SEA_FORGE_SERVER_BIN; a skip is not a pass.
