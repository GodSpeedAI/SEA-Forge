# Bootstrap / Configuration Audit — SEA-rs

Date: 2026-08-31 · Branch: `ultracode/sea-forge-completion` · Host: WSL2 x86_64-unknown-linux-gnu

Every claim below is tagged **[verified via …]** (a command I ran, or a file I read in full) or **[inferred]**.

---

## 1. Executive summary

**Yes — the app is wired correctly enough to build and run end-to-end right now, with one functional caveat.**

Both Cargo workspaces compile clean (`cargo check --workspace --all-targets` and the standalone `src-tauri` crate, exit 0). The TypeScript frontend typechecks and lints clean (`bun run check`, exit 0). The generated-contract drift gate passes byte-identical (`just workbench-contracts-gate`, exit 0). Tauri versions are internally consistent across Rust and JS, the WebKitGTK/libsoup system deps for a Tauri 2 desktop window are present, and `just workbench-tauri-dev` restages the sidecar as a recipe dependency, so the stale committed sidecar binary self-heals.

The caveat is *runtime completeness, not wiring*: there is no `.sea-forge/server.yaml`, so the server boots on `ServerConfig::default()` with `agent.endpoints = []` — the cell will start, serve SFWP, and drive the UI, but no agent delegation can execute until an endpoint is configured. Separately, the SOPS-encrypted dev secrets profile cannot be decrypted with the age key on this machine.

**Scope correction, stated up front:** the audit brief names *sqlite, postgres, duckdb, and RuVector*. Only SQLite is present. Postgres, DuckDB, RuVector/pgvector, sqlx, diesel, and sea-orm appear **nowhere** in project source, manifests, docs, spec, or scripts — the only matches in the repository are inside `.tmp/ref/` (vendored third-party reference checkouts: goose, semantic-kernel, t3code) that are not part of this build. There is no multi-engine data fabric to audit; there is one embedded SQLite FTS5 index over an append-only JSONL truth store. Section 2 documents what actually exists.

---

## 2. Data layer map

**Architectural shape** [verified via `docs/execution/REPOSITORY_TRUTH.md:35,64`, `docs/execution/ULTRACODE_MISSION.md:70`, `crates/sea-forge-capability/src/memory.rs:196-297`]: append-only JSONL is the authoritative record; SQLite, JSON views, TS contracts and UI caches are declared *rebuildable projections*. This is an event-sourcing/CQRS shape, not a polyglot-persistence fabric.

| Store | Engine / pin | Role | What talks to it | Status |
|---|---|---|---|---|
| Capability memory index | `rusqlite 0.32` (`bundled`) → SQLite FTS5, file `memory/index.sqlite` under the cell root | Rebuildable full-text projection of `items.jsonl`; enables `sea-forge recall` | `sea-forge-capability` (writer, `memory.rs`), `sea-forge-cli` (`commands/recall.rs:169`, `commands/memory.rs:8`, read-only via `OpenFlags::SQLITE_OPEN_READ_ONLY`) | **Working** — compiles, conformance-tested (`crates/sea-forge-cli/tests/conformance_m4b.rs:287` asserts index and linear-scan fallback return identical results). No `.sqlite` file exists on disk yet because no memory has been ingested in this cell. |
| Record / ledger truth | Plain JSONL files on disk under `SEA_FORGE_ROOT` (default `.sea-forge/`) | Authoritative append-only records; ULID + hash chain + MMR in `sea-forge-ledger` | 20 crates write JSONL (`sea-forge-core`, `-ledger`, `-capability`, `-case-runner`, `-agent`, `-cell`, `-artifact-ip`, `sea-forge-cli/*`) | **Working but empty** — `.sea-forge/` contains `ledgers/`, `ledgers/events/`, `ledgers/quarantine/`, `requests/`, and zero record files. Fresh/unseeded cell. |
| Frontend draft store | JSON files via Tauri app-data dir (explicitly *not* SQLite) | Per-window case-authoring drafts | `src-tauri/src/drafts.rs`, IPC commands `draft_save/load/list/delete` | **Working, unused from UI** — the 4 `draft_*` commands are registered in `generate_handler!` but no `.ts`/`.tsx` file under `apps/desktop/src` invokes them. `drafts.rs:5-11` documents the deliberate choice over SQLite/the store plugin, with a note to revisit if concurrent multi-window editing arrives. |
| PostgreSQL | — | — | — | **Not present.** Zero references in `crates/`, `workbench/apps`, `workbench/packages`, `docs/`, `spec/`, `scripts/`, `justfile`, `Cargo.toml`. |
| DuckDB | — | — | — | **Not present.** Same search, zero hits. |
| RuVector / pgvector / any vector store | — | — | — | **Not present.** Same search, zero hits. |

*Interoperation / sync / handoff:* there is none to audit, because there is one engine. The only "handoff" is the documented rebuild direction JSONL → SQLite FTS5, one-way, idempotent (`memory.rs:212-297` writes to `index.sqlite.tmp` then renames) [verified via file read].

**Judgment [inferred]:** for a governed, auditable, single-user desktop cell, one bundled SQLite index over append-only JSONL is the right call — no daemon to install, no connection string to misconfigure, no version skew, and the index is disposable. Adding Postgres or DuckDB here would be a liability, not a feature. If the brief's four-engine model is a *target* architecture rather than a description of the code, none of it has been started.

---

## 3. Server and API layer

**There is no HTTP/REST API and no network port.** [verified via `crates/sea-forge-server/Cargo.toml` — no axum/actix/hyper/tonic; `crates/sea-forge-server/src/main.rs:19-53`]

| Boundary | Mechanism | Detail | Status |
|---|---|---|---|
| Kernel ↔ desktop host | **Unix domain socket, NDJSON frames** at `<root>/server.sock` | `SOCKET_FILE_NAME = "server.sock"`; a relative `socket_path` composes under `root` so records and socket never split across cells (`config.rs`, `resolved_socket_path`) | Working — socket file exists at `.sea-forge/server.sock` |
| Desktop host ↔ React | **Tauri 2 IPC commands** (10 registered) | `sfwp_query`, `sfwp_command`, `sfwp_identity`, `sfwp_cell`, `sfwp_initialize_cell`, `sfwp_request_status`, `draft_save`, `draft_load`, `draft_list`, `draft_delete` (`src-tauri/src/lib.rs:110-121`) | Working |
| Frontend call sites | `invoke()` from `@tauri-apps/api/core` (10 import sites) | `sfwp_query` ×7, `sfwp_command` ×3, `sfwp_identity` ×1, `sfwp_initialize_cell` ×1, `sfwp_request_status` ×1 | **Matched** — every invoked name exists in `generate_handler!`; `sfwp_cell` and the 4 `draft_*` are registered but never invoked |
| Event channel | Single Tauri event `sfwp://event` | Emitted by `src-tauri/src/events.rs` (`emit("sfwp://event", frame)`) with a persisted cursor; consumed by `useOperationsStream.ts:168`, `useGovernedEventInvalidation.ts:53` | **Matched** — one emitter name, one listener name, no drift |
| Serialization boundary | Rust `schemars` → JSON Schema → `json-schema-to-typescript` + AJV validators | 57 canonical types; Rust is authoritative, TS is a committed projection; every query hook validates the response against its generated AJV validator | **Working, gate-enforced** (see §4) |
| Sidecar lifecycle | `bundle.externalBin: ["binaries/sea-forge-server"]`, supervised by `src-tauri/src/supervisor.rs` (38 KB) | Host spawns/adopts the kernel and shuts down only a child it spawned (`lib.rs` `RunEvent::Exit`) | Working |

**Verification commands and results:**

```
$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized] target(s) in 37.12s          # exit 0, warnings only

$ cd workbench/apps/desktop/src-tauri && cargo check --all-targets
    Checking sea-forge-workbench v0.1.0 (…/src-tauri)
    Finished `dev` profile [unoptimized] target(s) in 15.87s          # exit 0

$ just workbench-contracts-gate
    generated 57 type(s): ApprovalGovernanceContext, … ValueSource
    sea-forge.tokens.css matches spec source.
    ok: generated contracts, UI tokens, and the Tauri workspace boundary are current   # exit 0

$ cd workbench && bun run check         # tsc -b --noEmit && oxlint
    2 warnings (react fast-refresh, exhaustive-deps), 0 errors        # exit 0

$ cd workbench && bun install --frozen-lockfile --dry-run
    [12.00ms] done                                                    # exit 0
```

---

## 4. Configuration audit

| Config item | Expected | Actual | Status |
|---|---|---|---|
| `SEA_FORGE_ROOT` | unset → `.sea-forge` under CWD, absolutized (`config.rs::resolve_cell_root`) | unset; `.sea-forge/` exists at repo root | OK [verified via `config.rs` + `ls .sea-forge`] |
| `SEA_FORGE_SOCKET` | optional override, outranks config + root default | unset → `<root>/server.sock` | OK [verified via `config.rs`, `main.rs:41-42`] |
| `<root>/server.yaml` | optional; absent = first run → defaults. Present-but-invalid = fail closed (`server_config_error`) | **absent** — `.sea-forge/` holds only `server.sock` and `server.sock.lock` | **Functionally incomplete** — see B-1 |
| `agent.endpoints` | at least one endpoint for delegation to run | `Vec::new()` (`sea-forge-agent/src/config.rs:118`), because no `server.yaml` | **Blocker for agent work** — see B-1 |
| `max_concurrent_runs` / `approval_ttl_hours` | 4 / 24 | defaults in effect | OK [verified via `config.rs`] |
| `SEA_ENV` | `dev` \| `test` \| `ci`, default `dev` | `.envrc` exports `${SEA_ENV:-dev}` | OK [verified via `.envrc`] |
| `RUST_LOG` | `sea_forge=info` default | `.envrc` exports `${RUST_LOG:-sea_forge=info}` | OK [verified via `.envrc`] |
| `SOPS_AGE_KEY_FILE` | auto-set to `~/.config/sops/key.txt` or `~/.config/sops/age/keys.txt` | `~/.config/sops/age/keys.txt` present (189 bytes); `key.txt` absent → `.envrc` resolves to `age/keys.txt` | Path OK, **key wrong** — see B-2 |
| `secrets/dev.enc.env` | decryptable by local age identity | **decryption fails**: `no identity matched any of the recipients` (recipient `age1mq5sj8g…nj93a5`) | **Broken (soft-fail)** — see B-2 |
| `secrets/dev.env` | plaintext placeholder, gitignored | present, `0600`, contains only `SEA_EXAMPLE_API_KEY` placeholder | OK [verified via read with values redacted] |
| direnv | `.envrc` is the activation layer; devbox pins `direnv 2.37.0` | `direnv` **not on PATH** outside the devbox shell | Expected-inside-devbox [verified via `command -v direnv`] |
| Vite dev server | port 1420, `strictPort: true` | `vite.config.ts` sets exactly that | OK — matches `tauri.conf.json` `devUrl: http://localhost:1420` [verified via both files] |
| `tauri.conf.json` `frontendDist` | `../dist` | `apps/desktop/dist/` exists with `assets/` | OK [verified via `ls`] |
| `tauri.conf.json` `$schema` | `../node_modules/@tauri-apps/cli/config.schema.json` | `apps/desktop/node_modules/@tauri-apps/cli` installed | OK [verified via `ls`] |
| CSP | `connect-src 'self' ipc: http://ipc.localhost` — IPC only, no remote origins | as expected | OK, and appropriately tight [verified via `tauri.conf.json`] |
| Tauri capabilities | at least one capability for the `main` window | `capabilities/default.json` → `["core:default"]` on window `main` | OK — the 10 custom commands need no permission entry [verified via file] |
| `bundle.externalBin` | `binaries/sea-forge-server` + host triple suffix | `binaries/sea-forge-server-x86_64-unknown-linux-gnu` present | OK but **stale** — see B-3 |
| `src-tauri` workspace isolation | own `[workspace]` root, not a member of the kernel workspace (ADR-004) | enforced and asserted by the contracts gate via `cargo metadata` | OK, gate green [verified via gate run] |
| `.cargo/config.toml` | — | `jobs = 1`, `incremental = false`, `[profile.dev] codegen-units = 1` | Works, **slow by design** — see N-1 |
| `workbench/bunfig.toml` | — | `exact = true`, `frozenLockfile = true` | OK [verified via file] |
| WSL2 GUI prereqs for Tauri | webkit2gtk-4.1, javascriptcoregtk-4.1, libsoup-3.0, a display | 2.52.3 / 2.52.3 / 3.6.6; `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0` | OK [verified via `pkg-config --modversion`] |

---

## 5. Dependency audit

### Rust — kernel workspace (`/Cargo.toml`, 22 member crates)

| Crate | Required | Installed / locked | Status |
|---|---|---|---|
| rustc / cargo | `1.92.0` (`rust-toolchain.toml`) | `rustc 1.92.0 (ded5c06cf)`, `cargo 1.92.0` | Match [verified via `--version`] |
| edition / rust-version | 2021 / 1.92.0 | as declared | Match |
| rusqlite | `0.32`, features `["bundled"]` | resolved in lock; ADR-002 documents the deliberate pin *down* from 0.40 because `libsqlite3-sys 0.38.1` uses unstable `cfg_select!` | Match, and intentional [verified via `Cargo.toml` + `docs/decisions/ADR-002…md:50-75`] |
| tokio | `1`, multi-thread+net+io-util+process+sync+time+fs+macros — **only** in `sea-forge-server` | as declared; `just no-async-kernel` gate asserts 19 kernel crates stay async-free | Match |
| reqwest | `0.12`, `default-features = false`, rustls-tls | `0.12.28` locked | Match |
| schemars / serde / serde_json / serde_yaml | `1` / `1` / `1` / `0.9` | as declared | Match |
| ed25519-dalek, chacha20poly1305, sha2, zeroize, getrandom | `2`, `0.11`, `0.10`, `1.8`, `0.3` | as declared | Match |
| Whole workspace | builds | `cargo check --workspace --all-targets` **exit 0** | **OK** |

### Rust — desktop host (`workbench/apps/desktop/src-tauri`, separate workspace + own `Cargo.lock`)

| Crate | Required (`Cargo.toml`) | Locked (`Cargo.lock`) | Status |
|---|---|---|---|
| tauri | `2.11.3` | `2.11.5` | Compatible (caret) |
| tauri-build | `2.6.3` | `2.6.3` | Exact match |
| tauri-plugin-log | `2` | `2.9.0` | Compatible |
| tauri-utils | (transitive) | `2.9.3` | Consistent with tauri 2.11.x |
| wry | (transitive) | `0.55.1` | Correct for Tauri 2.11 |
| tokio | `1` + net/io-util/sync/time/macros/rt/signal | `1.53.1` | Compatible |
| serde / serde_json | `1.0` | `1.0.229` | Compatible |
| dev-deps (path) | `sea-forge-server`, `-agent`, `-planner` | resolve across the workspace boundary; dev-only, never bundled | OK by ADR-004 |
| Whole crate | builds | `cargo check --all-targets` **exit 0** | **OK** |

### JavaScript / TypeScript (Bun workspace, `bun.lock`)

| Package | Declared | Installed | Status |
|---|---|---|---|
| `@tauri-apps/api` | `2` | `2.11.1` | Match — same 2.11.x line as the Rust `tauri` crate |
| `@tauri-apps/cli` | `2.11.4` | `2.11.4` | Exact match |
| react / react-dom | `^19.2.7` | `19.2.8` | Match |
| vite | `^8.1.1` | `8.1.5` | Match |
| typescript | `~6.0.2` | `6.0.3` | Match |
| vitest | `4.1.10` | `4.1.10` | Exact match |
| `@tanstack/react-query` / `react-router` | `5.101.4` / `1.170.18` | pinned exact via `bunfig.toml exact = true` | Match |
| xstate / `@xstate/react` | `5.32.5` / `6.1.0` | pinned exact | Match |
| `@astryxdesign/core`, `theme-neutral` | `0.1.8` | pinned exact | Match |
| ajv | `8.20.0` (app + contracts) | pinned exact | Match |
| oxlint / playwright / axe-core | `^1.71.0` / `1.62.0` / `4.12.1` | installed | Match |
| Bun runtime | `packageManager: "bun@1.4.0"` | **`1.3.14`** | **Mismatch** — see S-1 |
| Lockfile integrity | frozen | `bun install --frozen-lockfile --dry-run` **exit 0** | OK |
| Install completeness | all workspaces | `workbench/node_modules` holds only `.bun`; deps live in `apps/desktop/node_modules` (22) and `packages/*/node_modules` — Bun's per-workspace layout | OK — not a missing install [verified via `ls`] |

### Toolchain binaries on PATH vs `devbox.json` pins

| Tool | Pinned | On PATH | Status |
|---|---|---|---|
| gitleaks | `8.30.1` | `8.30.1` | Match |
| just | `1.55.1` | `1.58.0` | Drift (harmless) — see N-2 |
| sops | `3.10.2` | `3.9.3` | Drift (older) — see N-2 |
| age | `1.2.1` | `1.2.0` | Drift (older) — see N-2 |
| cargo-deny | `0.19.9` | `0.20.2` | Drift (newer) — see N-2 |
| direnv | `2.37.0` | absent | Only inside `devbox shell` |
| sqlite3 CLI | not required (rusqlite is `bundled`) | `3.46.1` | N/A — not used by the build |
| psql / duckdb CLI | not required | absent | N/A — no such dependency exists |

---

## 6. Broken / misconfigured items

### B-1 — No `server.yaml`; agent endpoints empty. **Severity: blocker (for agent-executing journeys only)**

**What:** `.sea-forge/` has no `server.yaml`, so `ServerConfig::load` returns `Self::default()` and `AgentConfig::endpoints` is `Vec::new()` [verified via `crates/sea-forge-server/src/main.rs:26-38`, `crates/sea-forge-agent/src/config.rs:99-125`, `find .sea-forge -type f`].

**Why it matters:** the cell starts and the UI works, but any journey that delegates to an agent has no endpoint to reach. This is not a wiring defect — `ServerConfig` deliberately treats an absent file as a valid first run and fails closed only on an *invalid* one.

**Fix** — create `.sea-forge/server.yaml`. `ServerConfig::load` validates on start, and its own tests show the two rules you must satisfy: an `open_ai_compatible` endpoint must be HTTPS unless it is loopback, and `status: probed` is rejected because that field is evidence-derived, not operator-declared [verified via `config.rs` tests].

```yaml
# .sea-forge/server.yaml
max_concurrent_runs: 4
approval_ttl_hours: 24
agent:
  timeout_secs: 60
  endpoints:
    - id: local
      kind: open_ai_compatible
      base_url: http://127.0.0.1:11434/v1   # loopback HTTP is allowed; remote must be HTTPS
```

Verify: `cargo run -p sea-forge-server --bin sea-forge-server` and confirm the startup log line names your root, socket and `max_concurrent`.

### B-2 — SOPS dev secrets cannot be decrypted on this machine. **Severity: should-fix**

**What:** `sops -d secrets/dev.enc.env` fails with `age: no identity matched any of the recipients` for recipient `age1mq5sj8gj4k5vqtgefkuvs05nghanzhgmcqkxxspk0vffq9hxm5ssnj93a5`. The local identity at `~/.config/sops/age/keys.txt` (189 B, present) is a different key [verified via the `sops -d` run above].

**Why it matters:** `.envrc` is written to fail soft — it logs `no age key — secrets not loaded (offline mode)` and continues, which is why every offline gate still passes. But any real credential in the dev profile is unavailable. Note the *only* variable in the plaintext placeholder is `SEA_EXAMPLE_API_KEY`, so nothing in the current build appears to depend on it [inferred — no Rust or TS code reads `SEA_*` secret variables; the env-var scan found only `SEA_FORGE_*` path/test variables].

**Fix — pick one:**
- Add your existing public key as a recipient and re-encrypt (needs someone who already holds a matching identity):
  `sops updatekeys secrets/dev.enc.env`
- Or, if the encrypted profile has no content you need, regenerate it from `secrets/dev.env` under your own key:
  `age-keygen -o ~/.config/sops/age/keys.txt` (only if you want a fresh identity), update `.sops.yaml` recipients, then `sops -e secrets/dev.env > secrets/dev.enc.env`.

Verify: `sops -d secrets/dev.enc.env | head -1` prints without error.

### B-3 — Staged Tauri sidecar is 116 source files out of date. **Severity: should-fix**

**What:** `workbench/apps/desktop/src-tauri/binaries/sea-forge-server-x86_64-unknown-linux-gnu` is dated 2026-08-03 11:45; 116 `.rs` files under `crates/` are newer [verified via `stat` + `find … -newer … | wc -l`].

**Why it matters, and why it is not a blocker:** `just workbench-sidecar` is a declared *dependency* of `workbench-tauri-dev`, `workbench-tauri-test` and `workbench-package`, so any supported path restages it before use [verified via `justfile:412-484,615`]. It only bites if someone runs `cargo build --manifest-path …/src-tauri/Cargo.toml` or `bun run tauri dev` directly, bypassing `just`.

**Fix:**
```
just workbench-sidecar debug     # or: just workbench-sidecar release
```

### B-4 — `sea-forge-server` parses no CLI arguments; unknown flags silently start a server. **Severity: should-fix**

**What:** `main()` reads no `std::env::args` and has no `clap` dependency [verified via `crates/sea-forge-server/src/main.rs` full read and its `Cargo.toml`]. I confirmed this the hard way: running `…/binaries/sea-forge-server-x86_64-unknown-linux-gnu --version` did not print a version and did not exit — it started a real server that bound `.sea-forge/server.sock` (visible as PID 306180; `.sea-forge/server.sock` mtime moved to 19:31).

**Why it matters:** a typo'd flag, a health probe, or a packaging smoke test that calls `--version` will start and hold a cell instead of failing fast. Every other binary in the repo is `clap`-based, so this one is the outlier.

**Fix:** either add a minimal `clap` front-end to `sea-forge-server`, or reject any argv beyond the program name:

```rust
// crates/sea-forge-server/src/main.rs, after the tracing init
if std::env::args_os().len() > 1 {
    eprintln!("sea-forge-server takes no arguments; configure via SEA_FORGE_ROOT, \
               SEA_FORGE_SOCKET and <root>/server.yaml");
    std::process::exit(2);
}
```

**Immediate cleanup — the process I started is still running** (I was not permitted to kill it). Run:
```
pkill -f 'binaries/sea-forge-server-x86_64-unknown-linux-gnu'
```

### S-1 — Bun runtime is behind the declared `packageManager`. **Severity: should-fix**

**What:** `workbench/package.json` declares `"packageManager": "bun@1.4.0"`; the Bun on PATH is `1.3.14` [verified via `bun --version`]. `REPOSITORY_TRUTH.md` notes 1.4.0 is the canary Zig→Rust rewrite.

**Why it matters:** every gate currently passes on 1.3.14, so this is drift, not breakage — but `bun.lock` and the frozen-lockfile gate were authored against 1.4.0 [inferred].

**Fix:** `bun upgrade` (or install 1.4.0 via your version manager), then re-run `just workbench-contracts-gate`.

### N-1 — Build parallelism disabled repo-wide. **Severity: nice-to-have**

`.cargo/config.toml` sets `[build] jobs = 1`, `incremental = false`, `[profile.dev] codegen-units = 1` [verified via file read]. This is a correct choice for reproducibility and for a memory-constrained WSL2 host, and it is why `cargo check` took 37 s on an already-warm target dir. If iteration speed matters more than determinism locally, raise `jobs` and re-enable `incremental` in a **local, uncommitted** override — do not change the committed file, since the reproducibility gates depend on it [inferred].

### N-2 — Devbox tool pins vs. host PATH drift. **Severity: nice-to-have**

`just` 1.58.0 (pinned 1.55.1), `sops` 3.9.3 (pinned 3.10.2), `age` 1.2.0 (pinned 1.2.1), `cargo-deny` 0.20.2 (pinned 0.19.9) [verified via each `--version`]. The pins are only authoritative inside `devbox shell`; the host binaries are what actually ran during this audit, and every gate passed with them. Note `cargo-deny` 0.20 changed some `deny.toml` semantics, so `just deny` may behave differently outside devbox [inferred].

**Fix:** run gates inside `devbox shell` for pin-accurate results.

### N-3 — Untracked example and stale committed artifacts. **Severity: nice-to-have**

- `crates/sea-forge-server/examples/journey_gauntlet_bootstrap.rs` is untracked (created 2026-08-31 16:21) and *is* compiled by `cargo check --all-targets` — it passed, but it is not under version control [verified via `git status` + `ls`].
- Six `cc*.cdtor.o` object files sit in the repository root [verified via `ls /`], leftover build droppings unrelated to any target.
- Registered-but-unused IPC surface: `sfwp_cell` and `draft_save`/`draft_load`/`draft_list`/`draft_delete` have no frontend caller [verified via the invoke-name scan]. Either wire the drafts UI or drop the commands; a registered command with no caller is untested attack surface [inferred].

---

## 7. Recommended fix order

Minimal sequence to a working end-to-end run. Steps 1–3 are enough to launch; 4–6 are hygiene.

1. **Kill the stray server I started** (B-4 cleanup) — it holds `.sea-forge/server.sock` and will collide with the one the desktop host wants to spawn:
   `pkill -f 'binaries/sea-forge-server-x86_64-unknown-linux-gnu'`
2. **Write `.sea-forge/server.yaml` with at least one agent endpoint** (B-1) — the only change that converts "boots and renders" into "can actually execute delegated work". Use the YAML in B-1; keep remote endpoints on HTTPS and never set `status:`.
3. **Launch via `just`, not directly** — this restages the stale sidecar (B-3) and runs a frozen install as part of the recipe:
   `just workbench-tauri-dev`
   Expect: Vite on `http://localhost:1420` (strict port), then a Tauri window; the host spawns the sidecar and streams `sfwp://event`.
4. **Repair the SOPS dev profile** (B-2) — `sops updatekeys secrets/dev.enc.env`, or re-encrypt under your own age recipient. Not needed for step 3 because `.envrc` fails soft.
5. **Align Bun to 1.4.0** (S-1), then re-run `just workbench-contracts-gate` to confirm the byte-identical projection still holds on the pinned runtime.
6. **Harden and tidy** — add the argv guard to `sea-forge-server` (B-4), commit or delete `crates/sea-forge-server/examples/journey_gauntlet_bootstrap.rs`, remove the root `cc*.cdtor.o` files, and decide whether the 5 uncalled IPC commands get a caller or get deleted (N-3).

**Full verification sweep once the above is done:**
```
cargo check --workspace --all-targets                                    # expect: exit 0
cargo check --all-targets --manifest-path workbench/apps/desktop/src-tauri/Cargo.toml
just workbench-contracts-gate                                            # expect: "ok: generated contracts…"
cd workbench && bun run check && bun run test
```
All four passed at audit time except the not-yet-run `bun run test` (Vitest was not executed in this audit — the frontend `check` and the Rust `cargo check --all-targets` were).
