# Reference: Configuration and Cell Layout

> **How the server resolves its cell, what `server.yaml` actually accepts, and where a cell keeps its records.**  
> Normative cell/socket contract: [`docs/CELL_CONTRACT.md`](../CELL_CONTRACT.md). Configuration source: [`crates/sea-forge-server/src/config.rs`](../../crates/sea-forge-server/src/config.rs).

---

## 1. Environment and Resolution Order

The server accepts **no command-line arguments**. Configuration comes from the environment and `<root>/server.yaml`.

| Rank | Input | Effect |
|---|---|---|
| 1 | `SEA_FORGE_SOCKET` | Absolute socket path. Outranks the configured `socket_path` and the root-composed default. |
| 2 | `SEA_FORGE_ROOT` | Cell root. Records live under it; a relative `socket_path` composes under it. |
| 3 | Neither set | `.sea-forge` under the current working directory. |

A relative path is made absolute against the current directory before use. At startup the resolved root overwrites any `root:` key in `server.yaml`, because the file lives inside the cell it configures and cannot relocate that cell. An absolute `socket_path` is honored verbatim; a relative one composes under the fixed root.

Logging follows Rust's `RUST_LOG` filter (default `info`). The server emits JSON tracing lines to stderr.

There are no `SEA_FORGE_CONFIG`, `SEA_FORGE_LOG`, or `SEA_FORGE_TIMEOUT` variables. Do not script against them.

### Windows CLI: `SEA_FORGE_SERVER_SID`

The Windows CLI connects to the cell's named pipe and refuses any server whose process runs under a different SID than the expected server principal. The expected principal is:

| Setting | Expected server SID |
|---|---|
| `SEA_FORGE_SERVER_SID` unset or empty (default) | The calling user. The server must run as the same user. |
| `SEA_FORGE_SERVER_SID=S-1-5-21-...` | That SID. Use it when the server runs under a service account that this user is admitted to through `identity.bindings`. |

A value that does not start with `S-1-` is refused. The CLI opens the pipe with identification-only security QoS, so a server cannot impersonate the caller, and with explicit pipe rights that exclude `FILE_CREATE_PIPE_INSTANCE`. This variable only changes which server the CLI accepts. It grants no authority to the server.

---

## 2. Startup and Reload Semantics

* **No `server.yaml`:** startup proceeds with `ServerConfig::default()`. A first run is expected to have no file.
* **Present but invalid:** startup fails before the socket is bound or cell records are created. Unknown fields are rejected (`deny_unknown_fields`); this is not a permissive schema.
* **Reload trigger:** the running server re-reads `server.yaml` on case submission (`submit` / case commit), not on every request.
* **Invalid reload:** the previous validated snapshot remains live, and the server publishes an `invalid_reload_error` operator event. It does not half-apply a configuration.
* **Identity reload on Windows:** the server pipe's security descriptor admits the SIDs bound in `identity` (and the gateway SID), and it is built once at startup. On Windows, a reload that adds or removes any such SID is refused with error class `identity_reload_requires_restart`, and the previous snapshot stays live. The refusal is published as an `invalid_reload_error` event whose `error_class` is that value. Restart the server to apply a SID change. Role changes and other non-SID settings still reload. On Unix, identity is matched by uid on the socket, so identity reloads apply live.
* **Deleted file:** a reload with no file is a no-op. It does not reset the process to defaults.
* **Fixed root and socket:** a reload cannot move the cell root or socket. Those are process-lifetime properties.
* **Immutable supervisor:** supervisor settings are read when the task is spawned at startup. Changing them requires a restart.

---

## 3. `server.yaml` Top-Level Schema

Every field is optional. The sample below shows the actual accepted keys and representative sections; comments state defaults and bounds.

```yaml
max_concurrent_runs: 4          # 1..=64; default 4
socket_path: server.sock        # relative to root; default "server.sock"
notify_command: null            # argv array; argv0 must exist and be executable
approval_ttl_hours: 24          # 1..=8760; default 24
root: .sea-forge                # overwritten by resolved root at startup
agent: {}                       # see §4
supervisor: {}                  # see §5
gateway: null                   # absent means no delegation
identity: {}                    # see §7
cep_authority: {}               # see §8
mcp_servers: []                 # see §9
postgres: []                    # see §10
workload_read_paths: []         # absolute canonical read roots for isolated workloads
```

Unknown top-level keys are errors. The documentation previously described `version`, `server`, `policy`, and `memory` sections; those keys do not exist in `ServerConfig` and will fail startup.

### `notify_command`

`notify_command` is an argv vector, never a shell string. At startup the server resolves `argv[0]`: a path containing a separator must be an existing file; a bare name is searched on `PATH`. A missing command blocks startup with a message naming the setting. At runtime, a failed notification is logged and ignored.

### `workload_read_paths`

Each entry must be absolute and canonical, must not contain the control-plane root, and must not be a whole home directory. These paths extend the system read set for `isolated` workloads; they do not widen write access.

---

## 4. `agent`

The agent section configures outbound model endpoints. It is validated at startup.

```yaml
agent:
  max_request_bytes: 1048576
  max_response_bytes: 4194304
  timeout_secs: 60
  transcript_retention: summarized   # summarized | full
  endpoints:
    - id: prod
      kind: open_ai_compatible       # open_ai_compatible | anthropic | acp
      base_url: https://api.example.com
      credential_ref: OPENAI_API_KEY
      default_model: example-model
      allow_loopback_test: false
      max_request_bytes: 1048576
      max_response_bytes: 4194304
      timeout_secs: 60
      transcript_retention: summarized
```

ACP endpoints additionally take tokenized `argv: []` and an explicit `env: ["KEY=VALUE"]`; the parent environment is never inherited. HTTP kinds require `base_url` and use HTTPS (a loopback test endpoint may opt in with `allow_loopback_test: true`).

An endpoint `status` field is **evidence-derived**, not configuration. Supplying it is rejected.

Limits must be non-zero; endpoint ids are bounded and unique. Retention precedence is per-plan item, then endpoint, then global `agent.transcript_retention`, then `summarized`.

---

## 5. `supervisor`

The supervisor is an opt-in background task that advances eligible cases.

```yaml
supervisor:
  enabled: false                  # fail-closed default
  poll_interval_secs: 5           # 1..=3600
  max_concurrent_cases: 2         # 1..=8
  actor: supervisor               # bounded identifier
```

Even when disabled, values in a written section are validated. The supervisor actor is recorded honestly on every record a pass produces and is never an end-user identity.

---

## 6. `gateway`

The gateway section binds one OS uid that may present `on_behalf_of` claims for an allowlist of end-user actors.

```yaml
gateway:
  uid: 1000
  actor: gateway
  delegable_actors: [alice, bob]
```

Absent means delegation is unconfigured and every `on_behalf_of` block is refused. An empty allowlist delegates nothing. Duplicate actors, the gateway actor itself, or the supervisor actor in the allowlist are errors. Both the gateway and every delegable actor must resolve through `identity.bindings`; otherwise startup fails rather than promising a delegation the cell cannot enforce.

---

## 7. `identity`

`identity.bindings` maps kernel-reported uids to actors and the roles each may claim.

```yaml
identity:
  bindings:
    - uid: 1000
      actor_id: alice
      roles: [operator, R-SO]
```

The server reads the peer uid from the Unix socket (`SO_PEERCRED`); a client cannot assert it. An empty or unmatched binding authorizes nothing. A binding with no roles identifies the actor but grants no role.

---

## 8. `cep_authority`

The CEP authority loop is fail-closed off.

```yaml
cep_authority:
  enabled: false
  policy: sea-forge-policy.yaml
  approval_ttl_hours: 24          # 1..=720
  settlement:
    min_supporting: 1
    min_reliability: medium       # low | medium | high
  worlds:
    - name: example
      base: worlds/example
      entry: model.sea
      files: [model.sea]
```

World `name`, `base`, `entry`, and `files` are required when a world is declared. `world_ref` is derived from DomainForge identity and is not configured.

---

## 9. `mcp_servers`

Each entry describes an upstream stdio MCP server offered only to isolated ACP agents through the per-workload broker. Upstream argv and environment stay with the trusted launcher; the workload reaches only the broker socket.

```yaml
mcp_servers:
  - name: example
    argv: ["/usr/local/bin/example-mcp"]
    env:
      EXAMPLE_TOKEN: value
    classify:
      read_secret: secret
```

`classify` maps a tool name to the exposure label recorded when a result is forwarded.

---

## 10. `postgres`

Each entry declares a workload-bound PostgreSQL resource. TCP is not used; the control plane administers over a Unix socket with peer authentication.

```yaml
postgres:
  - name: app
    psql_bindir: /usr/lib/postgresql/16/bin
    lib_dir: null
    socket_dir: /run/postgresql
    port: 5432
    database: app
    admin_role: app_admin
    session_ttl_secs: 900
    scopes:
      read: app_read
    offer: [read]
    exposure:
      read: internal
    permitted_exposure:
      read: [internal]
```

Identifiers are bounded lowercase SQL identifiers. `session_ttl_secs` defaults to `900`. Each authorized issuance creates a short-lived native login role scoped to one workload; passwords are held only in memory and the workload environment, never in ledgers.

---

## 11. Cell Layout

A cell root owns records and one socket. The common layout is:

```text
<root>/
├── server.yaml                 # optional; unknown keys are rejected
├── server.sock                 # SFWP socket, mode 0600
├── server.sock.lock            # exclusive owner lock
├── approvals.jsonl             # human approval journal
├── capabilities.jsonl          # settled capability envelopes
├── cases/
│   ├── <case_id>.json           # intent-run case record
│   └── <case_id>/
│       ├── case.json            # planned-case record
│       ├── case-events.jsonl    # persisted dispatch/settlement order
│       └── runs/
│           └── <run_id>/        # case-owned episodes
├── runs/
│   └── <run_id>/                # flat runs created by `sea-forge run`
├── ledgers/
│   ├── global-checkpoints.jsonl
│   └── <stream_id>/
├── memory/
│   ├── items.jsonl              # authoritative memory journal
│   └── index.sqlite             # derived FTS index, rebuildable
├── environments/                # pinned EnvironmentSpec YAML
├── authority/
│   ├── active-policy.json
│   ├── decisions.jsonl
│   └── audit.jsonl
├── artifacts/                  # artifact-IP registrations and gate profiles
├── extensions/                 # extension registry
├── drafts/
├── quarantine/
└── self-model/                 # snapshots and projections
```

A run episode normally contains `plan.json`, `authority.json`, `trace.jsonl`, `evidence.jsonl`, `settlement.json`, and `semantic-envelope.json`, plus `workspace/` and `artifacts/` for the workload. Settlement ids are random (`set_<...>`); they are not a fixed `set_01`.

Directories are created lazily by the command that owns them. A CLI `run` creates its run, `workspace/`, and `artifacts/` as setup; authority gates execution and workload writes, not the existence of those control-plane directories.

---

## 12. Operational Checks

* Keep `server.yaml` inside the cell it configures.
* Prefer `SEA_FORGE_ROOT` for scripts; the CLI still takes `--root` explicitly.
* Keep the socket path within the platform `sun_path` budget (104 bytes on the smaller supported platform, less the lock suffix).
* Treat `memory/index.sqlite` as derived state. Rebuild it with `sea-forge memory rebuild`.
* Never treat a migrated root as fully assured: it reports `legacy_digest_only`.
