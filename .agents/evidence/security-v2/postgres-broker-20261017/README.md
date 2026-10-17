# Workload-bound PostgreSQL sessions — 2026-10-17

Server: PostgreSQL 18.1 from the local pg0 v0.15.2 installation
(`~/.pg0/installation/18.1.0`; provenance in `../pg0-20261016/`). Each test
initdb's its own disposable cluster: `local all sea_admin peer map=launcher`
(launcher OS user only), `local all all scram-sha-256`, no TCP listener.

Seam: `postgres::PostgresBroker` (control plane, psql over the Unix socket
with peer auth — no admin secret exists for a workload to find). Authority
action `database_session` (`db_resource`/`db_scope` boundaries required)
decides every issuance. Issued role: `sfw_<workload uid>_<random>`, LOGIN,
no admin attributes, `CONNECTION LIMIT 4`, `VALID UNTIL` TTL, member of the
scope role `WITH INHERIT TRUE, SET FALSE, ADMIN FALSE`. Production path:
`ServerConfig.postgres[].offer` scopes are requested for each `isolated`
ACP agent; issued conninfo arrives as `SEA_FORGE_PG_<RESOURCE>_<SCOPE>`;
sessions are revoked at episode end. Ledger stream `dbsessions-<run>` holds
`authority_decision`, `database_session_{issued,refused,revoked}` without
password material.

Observed (captured regression: 793 passed, 0 failed, 9 ignored across
authority/server/sandbox/agent, PostgreSQL and isolation required):

- `postgres_broker`: from an isolated workload, `SELECT` succeeds; DELETE,
  DROP, TRUNCATE, ALTER ROLE SUPERUSER, SET ROLE to the scope role, self
  GRANT, CREATE ROLE all fail with 42501; admin login (peer), another
  workload's role, and reading the data directory are refused. The
  unauthorized `audit-write` scope creates no role. A live `pg_sleep`
  backend is attributed to the workload role in `pg_stat_activity`;
  revocation terminates it ("terminating connection due to administrator
  command") and reconnection fails. Table digest unchanged.
- Expired credential: new logins after `VALID UNTIL` are refused.
- `conformance_m16::t16_4d`: the production ACP path issues only
  `audit-read`, PostgreSQL refuses DELETE (42501), and no `sfw_` role
  survives the episode.

Mutation controls (manual, restored): `SET TRUE` on the scope grant fails the
SET ROLE check; skipping `pg_terminate_backend` fails revocation. The second
exposed a real defect — post-DROP verification cannot see surviving
backends — fixed by verifying before dropping (the role stays NOLOGIN and is
kept if anything survives).

Limits: `VALID UNTIL` does not end established sessions; only `revoke`
does (episode end or explicit). No background reaper for crashed servers
yet. Row-level observation is the launcher digest; RLS policies are not
used. Only Unix-socket access is supported.
