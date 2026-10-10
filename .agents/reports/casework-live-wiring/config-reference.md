# Casework stack: configuration reference and packaging (T11)

Scope: what the two processes need to run as a service. Sources of truth are the code
(`crates/sea-forge-server/src/config.rs`, `apps/godspeed-casework-go/internal/config/`); this page
describes them and does not override them. Units and examples live in `deploy/systemd/`.

## 1. Processes and ordering

| Process | Binary | Configured by |
| --- | --- | --- |
| Kernel | `sea-forge-server` | `SEA_FORGE_ROOT`, `SEA_FORGE_SOCKET`, `<root>/server.yaml`. No CLI arguments. |
| Gateway | `godspeed-casework -serve` | `-config` or `GODSPEED_CONFIG` (JSON), `GODSPEED_CELL_ROOT`, `-addr`, `-metrics-addr` / `GODSPEED_METRICS_ADDR` |

Start order: kernel first, then gateway. `godspeed-casework.service` has `Requires=` + `After=` on
the kernel unit and an `ExecStartPre` that waits until `/run/sea-forge/server.sock` exists (a started
unit is not yet a listening socket). The gateway refuses to serve when the kernel authority fails
preflight, so a gateway started too early exits non-zero and `Restart=on-failure` retries it.
Restarting or stopping the kernel *unit* restarts/stops the gateway with it; a kernel *crash* does
not, and the gateway's SFWP client reconnects by itself (ladder step L-RECOV proves this).

## 2. Install (systemd)

```
useradd --system --home /var/lib/sea-forge --shell /usr/sbin/nologin sea-forge
install -d -m 0750 -o root -g sea-forge /etc/sea-forge
install -m 0755 target/release/sea-forge-server /usr/local/bin/
install -m 0755 godspeed-casework /usr/local/bin/        # go build -o godspeed-casework ./cmd/godspeed-casework
install -m 0640 -g sea-forge deploy/systemd/server.env.example  /etc/sea-forge/server.env
install -m 0640 -g sea-forge deploy/systemd/gateway.env.example /etc/sea-forge/gateway.env
install -m 0640 -g sea-forge deploy/systemd/gateway.production.json.example /etc/sea-forge/gateway.json
install -m 0644 deploy/systemd/*.service /etc/systemd/system/
# create /var/lib/sea-forge/cell/server.yaml (section 3), then:
systemctl daemon-reload && systemctl enable --now sea-forge-server godspeed-casework
```

Validation performed in this repo: `systemd-analyze verify` on both units (with `ExecStart`
temporarily pointed at `/bin/true`, because the real binaries are not installed on the dev host) is
clean. The units were NOT started under a real systemd system instance on this host, so the
hardening sets (`ProtectSystem=strict`, `SystemCallFilter=@system-service ~@privileged @resources`,
`MemoryDenyWriteExecute`) are unproven at runtime. Check `journalctl -u ...` on first start and
`systemd-analyze security <unit>` and loosen only with a recorded reason.

### Socket permissions (0600 + group)

The kernel binds its socket on a staging path, sets mode `0600`, and renames it into place, so the
socket is owner-only from its first reachable instant (`crates/sea-forge-server/src/lib.rs`). The
group part is the *directory*: `RuntimeDirectory=sea-forge` with `RuntimeDirectoryMode=0750` makes
`/run/sea-forge` reachable only by the service group. Consequence, stated honestly: a gateway
running as a different uid than the kernel cannot connect to a 0600 socket, so both units run as
`sea-forge`, and `server.yaml` must set `gateway.uid` to that user's uid. Splitting the users would
need a kernel change to the socket mode and is deliberately not done here (identity model is
unchanged). Do not widen the socket's mode by hand.

## 3. Kernel: `server.yaml` keys the stack needs

`deny_unknown_fields` is on at every level: a typo is a startup error, not a silent default.

```yaml
identity:                 # who may connect: uid -> actor, roles
  bindings:
    - {uid: 995, actor_id: gateway,             roles: ["service"]}
    - {uid: 995, actor_id: operator_local,      roles: ["operator"]}
    - {uid: 995, actor_id: security_officer,    roles: ["R-SO"]}
    - {uid: 995, actor_id: lifecycle_custodian, roles: ["R-LC"]}
gateway:                  # absent => every on_behalf_of is refused (fail closed, no uid default)
  uid: 995                # SO_PEERCRED uid of the gateway process = the service user's uid
  actor: gateway          # default "gateway"
  delegable_actors: [operator_local, security_officer, lifecycle_custodian]   # empty => delegates nothing
# Optional, with defaults: max_concurrent_runs (4), approval_ttl_hours (24, cap 8760),
# socket_path (server.sock under root), root (.sea-forge), notify_command, agent: {...},
# supervisor: {enabled: false, poll_interval_secs: 5, max_concurrent_cases: 2, actor: supervisor}
```

Also required in the cell: `templates/*.yaml` (case templates) and `authority/active-policy.json`
(the policy the governed verbs authorize against; its path is the gateway's `serve.policy_ref`).
`just casework-cell-init` seeds a dev cell with all of this; production cells are written by hand.
The environment: `SEA_FORGE_ROOT` (cell root), `SEA_FORGE_SOCKET` (socket override; Unix socket paths
are limited to ~100 bytes, hence `/run/sea-forge/server.sock`). The kernel takes two exclusive
flocks for its lifetime, `<root>/.server.lock` and `<socket>.lock`; a second instance fails fast.

## 4. Gateway: JSON config keys (`GODSPEED_CONFIG`)

Environment overrides: `GODSPEED_CELL_ROOT` supplies `cell_root` (deliberately absent from the
file), `GODSPEED_EVIDENCE_ROOT` supplies `evidence_root`. Secrets are never values in the file; they
are `env:NAME` or `file:/path` indirections.

| Key | Meaning / default |
| --- | --- |
| `version` | `"1"` |
| `evidence_root` | evidence directory |
| `capabilities[]` | `name`, `kind`, `required`, `adapter`, `endpoint`. Live serve requires an `authority` capability with `adapter: "sfwp"` and `endpoint: "unix:///run/sea-forge/server.sock"` (absolute: relative paths resolve against the gateway cwd). Set `required: true` in production so a dead kernel blocks startup. `adapter: "fixture"` is refused in the production build. |
| `serve.gateway_actor_id` / `gateway_role` | default `gateway` / `service`; must mirror `server.yaml` |
| `serve.policy_ref` | default `authority/active-policy.json` (relative to the cell) |
| `serve.perspective_actor_id` / `perspective_role` | whose view relay-built revisions are stored for (default: the gateway principal). Sessions re-render for the verified session actor. |
| `serve.production` | **default false.** `true` is the production posture, see section 5. |
| `serve.trusted_origins` | exact browser `Origin` allowlist for credentialed requests. Empty = loopback-only dev allowlist. In production origins must be `https://`. |
| `serve.static_root` | directory of the built UI (`apps/godspeed-cognitive-ui/dist`), served with cache headers, CSP and SPA fallback. Empty disables static serving. |
| `serve.rate_limit.intents_per_minute` / `burst` | token bucket on `POST /api/intents` per session and per IP. Defaults 60 / 20. |
| `serve.execution_timeout_sec` | bounds kernel-side execution of advance/execute AND is the lifetime of any approval those verbs open. Default 60, valid 1..604800 (kernel clamps at 7 days). A second user signing off in a browser needs minutes: the ladder uses 900. Too small and approvals expire before a human can act. |
| `auth.mode` | `local`, `oidc` or `dev`. Required; empty is a startup error. |
| `auth.users[]` | `username`, `display_name`, `password_hash` (argon2id PHC, local mode), `actor_id`, `role` (kernel standing; must be in the kernel's `delegable_actors`). |
| `auth.session_idle_minutes` / `session_absolute_hours` | 30 / 12. Sessions are in memory (see runbook). |
| `auth.insecure_cookie` | drops `Secure` from cookies. Refused in production. |
| `auth.static_token`, `auth.static_token_user` | dev bearer token. Only valid with `auth.mode: dev`. |
| `auth.oidc.*` | `issuer`, `client_id`, `client_secret` (indirection), `redirect_url`, `scopes`, `username_claim` (default `preferred_username`), `mappings[]` (`claim`, `equals`, `actor_id`, `role`; first match wins, no match refuses the login). |

Flags: `-serve`, `-addr` (default `127.0.0.1:4179`; keep loopback and terminate TLS in a trusted
reverse proxy), `-config`, `-metrics-addr`.

## 5. Auth modes and why dev auth is forbidden in production

- `local`: users from config, argon2id password verification.
- `oidc`: authorization-code flow against the issuer; claims map to kernel standing via `mappings`.
- `dev`: **passwords are not checked** (any password for a configured username logs in) and an
  optional static bearer token acts as a fixed user. It exists for tests and the local ladder only.

Production guard (`internal/config/auth.go`, `validateAuth`), enforced at config load as a fatal
document-level problem so the process exits with code 2 before listening:
`serve.production=true` with `auth.mode=dev` is REFUSED; `auth.insecure_cookie` with production is
REFUSED; `auth.static_token` outside dev mode is REFUSED; an absent or empty `auth.mode` is refused
in any serve posture; non-HTTPS or non-canonical `trusted_origins` are refused in production.
Tests: `TestValidateProductionRefusesDevAuth_T07ToothB`, `TestValidateProductionWithAbsentAuthDefaultsToDevAndRefuses`,
`TestValidateProductionRefusesInsecureCookie`, and (T11) `TestDeployProductionExampleLoadsAndDevAuthIsRefused`,
which also keeps the shipped production example loadable.

**Residual risk, by design of the existing config:** the guard only fires when `serve.production`
is `true`, and that key defaults to `false`. A production config that omits it is validated as a
dev posture. Treat `"production": true` as mandatory in the deployed file (the shipped example sets
it), and review it in change control. Making production the default would change behavior for the
local ladder and was not done in T11; it is listed as a hardening proposal.

## 6. Metrics (Prometheus text)

Enabled by `-metrics-addr` or `GODSPEED_METRICS_ADDR` (empty = off). It is a **separate listener**
that must be a loopback address (startup is refused otherwise) and serves only `GET /metrics`.
It is not on the browser-facing mux (a test pins that `/metrics` and `/api/metrics` are not served
there). The exposition has no authentication and no user-, case- or id-bearing labels: label values
come from closed sets (mux route patterns, error kinds, wire verbs). To scrape from another host,
put an authenticated, TLS-terminating proxy or an SSH tunnel in front of the loopback port.

| Metric | Type | Notes |
| --- | --- | --- |
| `casework_http_request_duration_seconds{route,status_class}` | histogram | route is the mux pattern, e.g. `POST /api/intents`; unmatched paths are `unmatched`; `GET /api/events` excluded (stream lifetime) |
| `casework_sfwp_errors_total{class,verb}` | counter | class = apperr kind (`unavailable`, `authority_denied`, `invalid`, `internal`, `config`) or `canceled`; verb = SFWP wire verb |
| `casework_sse_clients` | gauge | open event streams |
| `casework_sse_clients_opened_total` | counter | |
| `casework_sse_cursor_lag_max` | gauge | most retained kernel revisions newer than the last cursor delivered to any open client (kernel cursor vs last delivered) |
| `casework_sse_queue_depth_max` | gauge | most revisions queued but not yet written for one client |
| `casework_sse_delivery_lag_seconds` | histogram | gateway-recorded time to client write |

Not exported: kernel-side counters (the Rust server has no metrics facility; adding one is a
separate kernel change, skipped). Suggested alerts: `rate(casework_sfwp_errors_total{class="unavailable"}[5m]) > 0`
for 5m, `casework_sse_cursor_lag_max` above the budget the T11 load test records.

## 7. Log correlation

One id follows an intent through every layer. The browser sends `intent_id`; the gateway makes it
the request's `correlation_id` (logfmt line), echoes it as `X-Correlation-Id`, and sends it to the
kernel as SFWP `request_id`; the kernel stores `<cell>/requests/<request_id>.json` and appends a
`delegated_request` record (with `request_id`, verb and both principals) to the `delegation-audit`
ledger. Non-intent requests get a minted id, echoed in the same header. Trace one id:

```
grep 'correlation_id="ID"' gateway.log          # journalctl -u godspeed-casework | grep ...
cat  <cell>/requests/ID.json
grep ID <cell>/ledgers/delegation-audit/entries.jsonl
```

Proof: live test `TestLiveCorrelationIDTracesBrowserGatewayKernelRequestAndLedger`
(`go test -tags live -p 1 -run TestLiveCorrelationID ./internal/server`). Writing it found a real
defect while the metrics middleware was being added (a wrapper that hid the correlation setter would
have silently stopped the intent-id join); `TestCorrelationIDEchoedAndEqualsIntentIDInLog` pins it
at unit level.
