# Implementation Plan — FACE Workbench Project Foundation

**Created:** 2026-07-09
**Source of truth:** `code-assets/face/SPEC.md` (v0.1) — quote it, don't paraphrase. Design doctrine: `code-assets/face/DESIGN.md`.
**Originating context:** Greenfield bootstrap of the FACE workbench repo, planned from SPEC.md on `main` of `godspeed-growth`. The `face/` tree in SPEC Appendix B does not exist yet.
**Status of the work today:** Nothing built. Only `DESIGN.md`, `dump.md`, `design.sample.md`, `SPEC.md`, and this plan exist in `code-assets/face/`.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies: 1→2→3→4 strictly sequential (each layer guards the next); Tasks 5 and 6 are independent of each other (both need 1–4); Task 7 needs 4; Task 8 needs everything; Task 9 needs 8.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **CORE PRINCIPLE (from SPEC §10.1–10.2): every dependency lives in exactly one pin file, every external concern lives behind exactly one port, and every failure ends with a `next move:` command. If you are about to hardcode a path, import an adapter from a service, or print a raw stack trace as the primary error — stop, that's the wrong shape.**
- Match style: Python follows ruff defaults + `pyproject.toml` config you create in Task 4; TS follows `deno fmt` defaults. No other style authorities.
- Source-of-truth rule: `packages/tokens/godspeed-ui.tokens.toml` is the spec for all token outputs — generated files in `packages/tokens/dist/` change only via `just tokens`, in the same commit as the TOML edit. The color/type values come verbatim from `DESIGN.md` §§2–4.
- Work in a new repo root `code-assets/face/` subtree exactly as SPEC Appendix B lays out (it can be extracted to a standalone repo later; use relative paths everywhere so extraction is a `git mv`).

### Global verification gates (must stay green after EVERY task)

```bash
just doctor        # layers built so far all report ok (doctor grows per task)
just check         # lint + typecheck + format + boundary + hardcode scan (grows per task)
just test          # all tests, both languages (grows per task)
```

Rebuild step if you touched `packages/tokens/godspeed-ui.tokens.toml`:

```bash
just tokens        # regenerates dist/; a dirty git diff after this is a gate failure
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
| --- | --- |
| Complete target file tree (authoritative layout) | `SPEC.md` Appendix B |
| Full dependency list + selection rules (do not add anything else) | `SPEC.md` Appendix C |
| Error taxonomy, blast radius per error class | `SPEC.md` §8.3 |
| ErrorReport shape (`code`, `what_happened`, `next_move`, `layer`) | `SPEC.md` §7.3.4 |
| Config precedence: env (`FACE_*`) > `config/<env>.toml` > `config/default.toml` | `SPEC.md` §8.1 |
| Token source values (hex, type scale, spacing) to copy into TOML | `DESIGN.md` §§2–4 |
| Token compiler outputs: `tokens.css`, `tailwind.theme.ts`, `cli-theme.json` | `DESIGN.md` §2, `SPEC.md` Appendix B |
| Corrective-error tone rules ("name missing evidence + next affordable move") | `DESIGN.md` §8 |
| Justfile ergonomics rules: grouped menu, verb-first, ≤20 recipes, `next move:` on failure | `SPEC.md` §10.3 |
| Ports must have ≥2 adapters + shared contract tests; composition root is `wiring.py` | `SPEC.md` §10.2 |
| Test env uses `telemetry_adapter = noop` — tests never need Docker | `SPEC.md` §9.5 |
| Dev bind address `127.0.0.1:8300`; `0.0.0.0` fails the hardcode scan | `SPEC.md` §8.2, §10.1 |

---

## Task 1 — Bootstrap layer: devbox + mise + direnv + justfile skeleton  (foundation · P0)

**Goal:** A fresh shell in `face/` resolves pinned `python`, `uv`, `deno`, `just`, `sops`, `age`, `gitleaks` with zero host-global installs, and `just` prints a grouped menu.

**Why this shape:** Everything downstream guards on these tools existing; per SPEC §6.2 the bootstrap layer is component 1 and per §10.1 no README step may say "install X globally" beyond devbox/direnv/git.

### Steps
1. Create `face/devbox.json` with the Appendix C system list (git, just, sops, age, gitleaks, docker client, direnv) — file: `devbox.json`. Run `devbox install` to produce `devbox.lock`.
2. Create `.mise.toml` pinning exact versions: python 3.x latest stable, uv, deno 2.x — file: `.mise.toml`.
3. Create `.envrc`: `use devbox`-equivalent (`eval "$(devbox generate direnv --print-envrc)"` or `use flake`-free devbox integration), then mise activation, then a placeholder for the Task 2 sops line. Set `FACE_ENV=${FACE_ENV:-dev}` — file: `.envrc`.
4. Create `justfile` with `set shell := ["bash", "-euo", "pipefail", "-c"]`, groups `[setup] [dev] [quality] [secrets] [obs]`, and initial recipes: default (grouped menu via `just --list --list-heading`), `setup` (devbox/mise/uv/deno sync, idempotent), `doctor` (v1: checks each pinned tool is on PATH and matches pin; per-layer `ok/fail` rows + `next move:` on fail) — file: `justfile`.
5. Create `.gitignore` (`var/`, `.venv/`, `.direnv/`, `node_modules/`, `apps/ui/dist/`), empty `var/.gitkeep` not needed (gitignored) — file: `.gitignore`.
6. Create `README.md` stub with the three-command bootstrap (`devbox shell`, `direnv allow`, `just setup`) — file: `README.md`.

### Gate
```bash
cd code-assets/face && devbox run -- just setup && devbox run -- just doctor
```
**Done when:** gate exits 0; `just` with no args prints the grouped menu; teeth-check: temporarily remove `age` from `devbox.json` → `just doctor` exits non-zero with a `next move:` line naming the fix, then restore.

**Redesign trigger:** If devbox+direnv+mise triple-activation conflicts (PATH ordering fights), drop mise activation into devbox's `init_hook` instead of `.envrc` and document it in README.

## Task 2 — Secrets layer: sops + age + direnv wiring  (foundation · P0)

**Goal:** `secrets/dev.enc.env` and `secrets/ci.enc.env` are committed encrypted, decrypt automatically through direnv into `FACE_*` vars, and an unencrypted secret cannot be committed.

**Why this shape:** SPEC §15 mandates secrets only via sops/age; the `.sops.yaml` recipients list IS the access control.

### Steps
1. Create `.sops.yaml` with an age recipient rule scoped to `secrets/.*\.enc\.env` — file: `.sops.yaml`. Add `just secrets init` (generates age key at `~/.config/sops/age/keys.txt` if absent, prints public key) and `just secrets edit env` (sops edit wrapper) recipes — file: `justfile`.
2. Create `secrets/dev.enc.env` (contents: `FACE_SECRET_KEY=dev-placeholder`) and `secrets/ci.enc.env`, encrypted — files: `secrets/*.enc.env`.
3. Wire `.envrc`: after env activation, `sops exec-env` or `eval "$(sops -d secrets/${FACE_ENV}.enc.env | sed 's/^/export /')"` guarded so a missing key produces the §8.3 `missing_credential_error` message with `next move: just secrets init` — file: `.envrc`.
4. Extend `just doctor` with a `secrets` layer row (key present, decrypt round-trip works) and extend `just check` with `gitleaks detect` + a scan failing on unencrypted files under `secrets/` — file: `justfile`.

### Gate
```bash
cd code-assets/face && devbox run -- just doctor && devbox run -- sh -c 'sops -d secrets/dev.enc.env | grep -q FACE_SECRET_KEY'
```
**Done when:** gate exits 0; teeth-check: `echo "FACE_SECRET_KEY=oops" > secrets/plain.env && just check` fails, then delete the file.

**Redesign trigger:** none plausible.

## Task 3 — Config layer: TOML files + typed Settings + error taxonomy  (core · P0)

**Goal:** `Settings` loads with env > env-TOML > default-TOML precedence, every §8.3 error class exists as a typed exception rendering through `ErrorReport`, and tests prove precedence and each failure mode.

**Why this shape:** SPEC §8 is the contract; pydantic-settings gives schema errors nearly free (Appendix C rung 6 pick). Errors come first because every later component reports through them.

### Steps
1. Scaffold `apps/core/` with `uv init` → `pyproject.toml` (deps per Appendix C: fastapi, uvicorn, pydantic-settings, structlog, otel trio, typer; dev: pytest, pytest-cov, httpx, ruff, pyright, import-linter) — file: `apps/core/pyproject.toml`.
2. Write `errors.py`: `ErrorReport` dataclass (`code`, `what_happened`, `next_move`, `layer`, optional `detail`) + one exception per §8.3 class + a `render(report) -> str` producing the `✗ [layer/code] … next move: …` format from SPEC §11.2 — file: `apps/core/src/face_core/errors.py`.
3. Write `config.py`: `Settings` (fields + validation per SPEC §8.2/§7.3.1, `FACE_` env prefix, TOML layering via `tomllib`), raising the typed errors — file: `apps/core/src/face_core/config.py`.
4. Create `config/default.toml`, `config/dev.toml`, `config/test.toml` (`telemetry_adapter = "noop"`), `config/ci.toml` — files: `config/*.toml`.
5. Tests: precedence (env beats TOML beats default), each error class fires on its trigger, error rendering never contains a secret value — file: `apps/core/tests/unit/test_config.py`.
6. Extend `justfile`: `test` (uv run pytest), `check` (ruff + pyright + hardcode scan grepping committed sources for `/home/`, `/Users/`, `0.0.0.0`), doctor `config` layer row — file: `justfile`.

### Gate
```bash
cd code-assets/face && devbox run -- just check && devbox run -- just test
```
**Done when:** gate exits 0; teeth-check: set `persistence_adapter = "postgres"` in `config/test.toml` → the `unsupported_kind_error` test fails the suite (proving validation is live), then revert.

**Redesign trigger:** If pydantic-settings' TOML layering fights the precedence spec, load TOML manually with `tomllib` and feed merged dicts to a plain pydantic model — do not bend the precedence contract to the library.

## Task 4 — Core app: ports, adapters, wiring, walking skeleton API + CLI  (core · P0)

**Goal:** `PersistencePort`/`ClockPort`/`TelemetryPort` exist as Protocols with ≥2 adapters each, selected only in `wiring.py` from `Settings`; `GET /health` returns `{"status":"ok"}` via FastAPI; `face doctor` and `face config show` work; boundary lint forbids domain→adapter imports.

**Why this shape:** SPEC §10.2 — swapping is a config change only; the contract-test pattern (one parametrized suite over all adapters) is what makes that claim testable rather than aspirational.

### Steps
1. Write `ports/` Protocols (PascalCase, `Port` suffix per SPEC §7.5) — files: `apps/core/src/face_core/ports/*.py`.
2. Write adapters: `adapters/sqlite/` (stdlib `sqlite3`, db file under `var/`), `adapters/memory/`, `adapters/otel/`, `adapters/noop/` — files: `apps/core/src/face_core/adapters/`.
3. Write `wiring.py`: registry dict `adapter_key -> factory(Settings)`; unknown key raises `unsupported_kind_error` at startup — file: `apps/core/src/face_core/wiring.py`.
4. Write `api/` FastAPI app with `/health` (touches persistence + clock ports so the skeleton actually walks) and `cli.py` (typer: `doctor`, `config show` redacting secrets) — files: `apps/core/src/face_core/api/app.py`, `cli.py`.
5. Contract tests parametrized over every registered adapter per port — file: `apps/core/tests/contract/test_ports.py`. Integration test: in-process httpx against `/health` — file: `apps/core/tests/integration/test_health.py`.
6. Add `import-linter` contract (domain/services import ports only) to `pyproject.toml`; wire into `just check`. Add `just dev` (uvicorn --reload on `127.0.0.1:8300`) — files: `apps/core/pyproject.toml`, `justfile`.

### Gate
```bash
cd code-assets/face && devbox run -- just check && devbox run -- just test && \
  devbox run -- sh -c 'FACE_ENV=test FACE_PERSISTENCE_ADAPTER=memory uv --directory apps/core run pytest'
```
**Done when:** gate exits 0 including the memory-adapter full-suite leg (the SPEC §4 swap proof); teeth-check: add `from face_core.adapters.sqlite import ...` to a domain module → `just check` fails via import-linter, then remove.

**Redesign trigger:** If typer pulls excessive deps or fights uv scripts, fall back to stdlib argparse (Appendix C names this fallback explicitly).

## Task 5 — Tokens package + UI shell (Deno/Vite/React)  (surface · P1)

**Goal:** `godspeed-ui.tokens.toml` (values verbatim from DESIGN.md §§2–4) compiles via one Deno script into `tokens.css`, `tailwind.theme.ts`, `cli-theme.json`; the React shell renders on the workspace surface colors; `just check` fails if dist/ is stale.

**Why this shape:** DESIGN.md §2 — one token source projected to GUI/CLI/docs is the design system's core mechanism; committing dist/ keeps UI builds compiler-independent while the diff-check keeps it honest (SPEC §9.5 nuance).

### Steps
1. Author `packages/tokens/godspeed-ui.tokens.toml` — copy every hex/scale value from `DESIGN.md` §2 (surface, semantic state, text palettes), §3 (type), §4 (spacing/density) — file: `packages/tokens/godspeed-ui.tokens.toml`.
2. Write `compile.ts` (Deno stdlib TOML parse only, zero deps) emitting the three outputs to `dist/` — file: `packages/tokens/compile.ts`.
3. Scaffold `apps/ui/`: `deno.json` (react, react-dom, vite, tailwind per Appendix C; tasks: `dev`, `build`, `check` = fmt+lint+check+test), minimal shell importing `tokens.css`, `ports/api.ts` interface + `adapters/http.ts`/`adapters/mock.ts` — files: `apps/ui/*`.
4. Justfile: `tokens` (compile), extend `check` (deno task check + `just tokens && git diff --exit-code packages/tokens/dist/`), extend `dev` to also start Vite — file: `justfile`.

### Gate
```bash
cd code-assets/face && devbox run -- just tokens && git diff --exit-code packages/tokens/dist/ && \
  devbox run -- sh -c 'cd apps/ui && deno task check && deno task build'
```
**Done when:** gate exits 0; teeth-check: hand-edit one hex in `dist/tokens.css` → `just check` fails on the diff, then `just tokens` restores it.

**Redesign trigger:** If Vite-under-Deno breaks (plugin incompatibility), add pinned `node` to `.mise.toml` and run Vite via node while keeping deno for `compile.ts`, lint, and test — Appendix C explicitly reserves this path.

## Task 6 — Observability: structlog + OTel + local LGTM stack  (surface · P1)

**Goal:** Health requests emit JSON logs with `trace_id` and export spans to a local `grafana/otel-lgtm` container started by `just obs up`; when the container is down the app serves anyway with a single warning (`infra_unavailable_error` degrade path, SPEC §8.3).

**Why this shape:** SPEC §10.4 — telemetry must never block requests; the noop/otel adapter pair from Task 4 already gives the seam, this task fills the `otel` side and the infra.

### Steps
1. Write `telemetry.py`: structlog JSON config (fields per SPEC §12.3: `event`, `level`, `logger`, `trace_id`, `face_env`), OTel tracer/meter setup behind `TelemetryPort`, FastAPI auto-instrumentation, export failures → warn-and-drop — file: `apps/core/src/face_core/telemetry.py`.
2. Create `infra/compose.yaml`: single `grafana/otel-lgtm` service pinned by digest, ports for OTLP + Grafana — file: `infra/compose.yaml`.
3. Justfile: `obs up`, `obs down`; doctor `infra` layer row (docker reachable → `ok`, else `warn` not `fail` — degrade, not block) — file: `justfile`.
4. Tests: in-memory span exporter integration test asserting one span per `/health` request + log record contains `trace_id`; a test with an unreachable endpoint asserting the request still returns 200 — file: `apps/core/tests/integration/test_telemetry.py`.

### Gate
```bash
cd code-assets/face && devbox run -- just test && devbox run -- just obs up && \
  devbox run -- uv --directory apps/core run pytest tests/integration/test_telemetry.py && devbox run -- just obs down
```
**Done when:** gate exits 0 and a trace from a manual `just dev` health hit is visible in Grafana (record the trace ID in `var/evidence/`); teeth-check: stop the container mid-`just dev` → health still returns 200 with a warning log, not an error.

**Redesign trigger:** If `grafana/otel-lgtm` is too heavy for WSL2 dev machines, swap to console-exporter default with the LGTM stack opt-in via `FACE_OTEL_ENDPOINT` — the port/adapter seam makes this a config default change only.

## Task 7 — Doctor completion + justfile cognitive-ergonomics pass  (UX · P1)

**Goal:** `just doctor` covers all six layers (system, runtime, secrets, config, app, infra) emitting `DoctorResult` rows and writing `var/evidence/doctor-<timestamp>.json`; the whole `just` surface meets SPEC §10.3 (grouped menu, verb-first, ≤20 recipes, every failure ends `next move:`).

**Why this shape:** Doctor is "the only authority on which state the environment is in" (SPEC §9.4); the justfile is the product's first cognitive-ergonomic surface, per the user's explicit requirement.

### Steps
1. Move doctor logic into `face_core.cli` (`face doctor --json`), with the justfile recipe as a thin wrapper — cross-language checks (deno, docker) shell out — file: `apps/core/src/face_core/cli.py`.
2. Write evidence output to `var/evidence/doctor-<ts>.json`, keep last 10 — file: `cli.py`.
3. Audit every recipe: verb-first names, one-line doc comments (`# description` above each recipe renders in `just --list`), group headers, failure paths end with `next move:` — file: `justfile`.
4. Count recipes; if >20, merge subcommand-style (`just secrets edit`, `just obs up`) — file: `justfile`.
5. Update `README.md` with the final command surface table and key-loss recovery procedure (SPEC §14.3) — file: `README.md`.

### Gate
```bash
cd code-assets/face && devbox run -- just doctor && test -n "$(ls var/evidence/doctor-*.json)" && \
  devbox run -- sh -c "just --list | grep -c '^\s' | awk '{exit (\$1>20)}'"
```
**Done when:** gate exits 0; teeth-check: rename the age key file → doctor exits non-zero, the failing row names `secrets` layer and `next move: just secrets init`, then restore.

**Redesign trigger:** none plausible.

## Task 8 — CI/CD: GitHub Actions running identical gates  (delivery · P0)

**Goal:** Push/PR runs `devbox run -- just check && just test` plus a memory-adapter matrix leg on ubuntu, decrypting CI secrets via a repo-secret age key; red gate blocks merge.

**Why this shape:** SPEC §3.2 requires CI to run "the exact same `just` gates as local" — no parallel CI-only logic that can drift.

### Steps
1. Create `.github/workflows/ci.yml`: checkout → install devbox (official action) → cache nix/uv/deno stores → write `SOPS_AGE_KEY` repo secret to the key file → `devbox run -- just setup && just check && just test` with `FACE_ENV=ci` — file: `.github/workflows/ci.yml`.
2. Matrix leg: `FACE_PERSISTENCE_ADAPTER=memory` full suite (the SPEC §13 variation case).
3. Generate a CI age keypair, add its public key to `.sops.yaml` recipients, `sops updatekeys secrets/ci.enc.env`, store the private key as the `SOPS_AGE_KEY` repository secret (manual step — document in README) — files: `.sops.yaml`, `secrets/ci.enc.env`.
4. Add a recovery job step: `rm -rf apps/core/.venv && just setup` reconverges (SPEC §17.2 recovery case).

### Gate
```bash
cd code-assets/face && devbox run -- sh -c 'FACE_ENV=ci just check && FACE_ENV=ci just test'   # local rehearsal
# then: push branch, confirm green run:  gh run watch --exit-status
```
**Done when:** a real GitHub Actions run is green on both matrix legs; teeth-check: a branch with a deliberately failing test produces a red run that blocks merge.

**Redesign trigger:** If devbox-in-CI is too slow even cached (>10 min), install just+uv+deno+sops directly via mise in CI while keeping devbox locally — accept the documented divergence in README as a known ceiling.

## Task 9 — Fresh-clone capability proof  (acceptance · P0)

**Goal:** The SPEC §4 capability claim upgrades to Evidence-backed: a second environment reproduces green gates from clone with no undocumented steps.

**Why this shape:** SPEC §5 claim table — a single author-machine success explicitly does not count.

### Steps
1. Clone into a clean directory (or fresh WSL2 distro / container with only git+devbox+direnv), follow only README's three commands.
2. Run `just doctor && just check && just test`; save the doctor evidence JSON.
3. Update SPEC §5 claim table rows to `Evidence-backed` with pointers to the CI run URL + evidence files — file: `SPEC.md`.
4. Check off the SPEC §18 Definition of Done — file: `SPEC.md`.

### Gate
```bash
git clone <repo> /tmp/face-proof && cd /tmp/face-proof && devbox run -- just setup && \
  devbox run -- just doctor && devbox run -- just check && devbox run -- just test
```
**Done when:** gate exits 0 in the clean environment AND CI is green on the same commit (that's the two independent reproductions); teeth-check: the run used zero commands not in README.

**Redesign trigger:** Any undocumented manual step discovered here goes into `just setup` or README *before* checking the box — the fix is automation, never documentation of a workaround alone.

---

## Final acceptance checklist (whole plan)

- [ ] Pinned toolchain resolves in devbox shell; removing a pin makes doctor fail with a next move *(Task 1)*
- [ ] Encrypted secrets decrypt via direnv; an unencrypted secret cannot pass `just check` *(Task 2)*
- [ ] Settings precedence + all §8.3 error classes proven by tests; errors never leak secrets *(Task 3)*
- [ ] Full suite passes against both persistence adapters via config swap only; domain→adapter import fails lint *(Task 4)*
- [ ] Token TOML→3 outputs round-trip clean; hand-edited dist fails the gate *(Task 5)*
- [ ] Traces visible in Grafana; app serves 200 with the stack down *(Task 6)*
- [ ] Doctor covers 6 layers + evidence JSON; justfile ≤20 grouped verb-first recipes, failures end `next move:` *(Task 7)*
- [ ] GitHub Actions green on both matrix legs; failing test blocks merge *(Task 8)*
- [ ] Fresh-clone proof green in a clean environment; SPEC §5 claims upgraded to Evidence-backed *(Task 9)*
- [ ] `just check` exits 0 (includes hardcode scan, gitleaks, boundary lint, token diff).
- [ ] SPEC.md §18 checklist fully checked — no status claim ahead of a passing gate.
- [ ] All global gates green.

## Guardrails (do not violate)

- Do NOT add dependencies beyond SPEC Appendix C without following the Appendix C.3 procedure — every extra dep needs its rung-by-rung justification in the PR.
- Do NOT let any committed file contain absolute paths, usernames, `0.0.0.0`, or plaintext secrets — the scans exist to fail, keep them sharp.
- Do NOT import adapters outside `wiring.py`/`adapters/`/tests — the import-linter contract is the architecture.
- Do NOT edit `packages/tokens/dist/` by hand — only `just tokens` writes there, in the same commit as the TOML change.
- Do NOT create CI-only logic — CI calls the same `just` recipes developers run.
- Commit hygiene: one task = one commit minimum, titled `feat(face): task N — <title>`; the CI key provisioning (Task 8 step 3) is its own commit since it touches `.sops.yaml` recipients.
