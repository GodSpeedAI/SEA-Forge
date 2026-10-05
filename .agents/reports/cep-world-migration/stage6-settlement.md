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

## Debt closed after settlement (2026-10-05)
- World registry cache: keyed by the worlds configuration plus every file's bytes, so an edited file can never reuse a stale registry (tested: the old world_ref is refused after an edit, the new one allowed). Bounded to 8 entries.
- Per-end-user policy: optional `subjects` on `cognate_action` / `cognate_capability` rules, matched against the request's `subject_actor_ref`. Rejected on any other kind or when empty. Omitted from the serialized rule when absent, so existing bundle hashes are unchanged (tested).
- Production wiring and enforcement on the Cognate side: `timeout_secs` enforcer, `governanceFromEnv` (explicit opt-in), `just sea-forge-live` that fails rather than skips; CG-PRF-002 flake fixed at its root cause (React scheduler outliving happy-dom). See cognate `.agents/evidence/2026-10-05-stage6-debt-closure.md`, commit a3ee8fd.
- Proven live through the real server: a `subjects` rule gave one user a `boundary` with `timeout_secs: 1`; a slow provider was aborted with `deadline_exceeded` while another user's capability calls stayed gated.
- sea-rs `just test`: 1,141 passed, 0 failed (+5). Adding `subjects` to the public `PolicyRule` struct broke one struct literal in `sea-forge-sandbox` tests; fixed. Any downstream code building `PolicyRule` literally must add `subjects: None`.

## Still open
- Only `timeout_secs` has an enforcer; the other boundary dimensions (workspace, artifacts_root, env_keys, sandbox_class, max_manager_iterations) are refused by Cognate until enforcers exist.
- One unexplained single failure of Cognate gate AK-006 in one serial run (0 of 23 afterwards); cause unknown.
- The full Cognate suite has environment-dependent failures (68-78 across runs) in AG-UI interop, E2E, J-* journeys and archived validation copies; none in code changed by this migration, but not investigated.
- Live loop tests still skip without SEA_FORGE_SERVER_BIN unless run through `just sea-forge-live`.
- Escalation is still refused (Stage 7).
