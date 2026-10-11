# Stage 7 settlement — escalation and durable continuation

Date: 2026-10-05. sea-rs `migration/cep-world-ref`; cognate `cognate/harness` def311f (pushed). Spec: `specs/cep-world-ref-migration.spec.md` (Stage 7). Plan: `plans/2026-10-05-stage7-escalation.md`.

**Process deviation:** the spec and plan were written after the implementation, not before (SEA-Forge's AGENTS.md asks for them first). The requirements in the spec are the ones the code and tests answer; they were not reverse-fitted to results, but the order was wrong.

## Delivered
- SEA-Forge: policy-driven escalations issue an approval (`apr-<op>`, 24h default, `cep_authority.approval_ttl_hours`); standing derived only from the append-only `cognate-authority` ledger (escalation / resolution / use entries keyed by idempotency key, single-shot under the ledger lock); resolution only by an actor policy allows and never the requester (the engine's `approval_resolution` surface; attempts are committed either way); revalidation binds world, kind, name, resource, subject and requester, requires lineage, re-evaluates policy now, never overrides a deny and does not spend the approval when policy now allows outright. Verbs `authority_approval` (protected) and `authority_approvals` (read-only).
- Cognate: escalation recorded once per call, durable `governance-approval` continuation expiring with the approval window, revalidated resume, `Runtime.pollApprovals()`.

## Verification
- sea-rs `just test` 1,157 passed, 0 failed (+16); `just check` green (see commit).
- Authority-level: 13 tests plus a CEP-validator test (escalate with approval fields, approved allow, revalidation request all accepted by cep's own validator). Socket-level: 2 new.
- Cognate: 94 pass in runtime-bun (+7 scripted); `just sea-forge-live` 17/17, 0 skipped; `just verify` 78/78 x3.

## Debt and limits (also in `.agents/DEBT.md`)
- Escalated actions (startRun etc.) are refused with the approval id; only capability calls inside runs wait.
- Crash window: approval consumed then process dies before the invocation is recorded -> replay refused; provider may have run once. At-most-once authority, not exactly-once effect.
- Polling is manual unless `governance.approvalPollMs` is set.
- Cognate approvals are a separate ledger stream and do not appear in SEA-Forge's workbench approval inbox.
- Approval ids derive from the operation id; the cep request profile has no first-class `approval` field (carried in the `godspeed.authority_request` extension).
