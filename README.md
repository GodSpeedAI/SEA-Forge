# SEA Forge

A governed capability-execution kernel. SEA Forge normalizes operator intent
into typed operations, decides authority **before** any side effect, executes
allowed work in an isolated workspace, records trace and evidence, settles the
declared outcome, and appends a semantic capability envelope.

This repository is **specification-first**. The development foundation
(`.agents/specs/Shell-SPEC.md`) and governed minimum kernel
(`.agents/specs/spec-minimum.md`) are implemented here. The full system
(`.agents/specs/spec-full.md`) remains an additive roadmap.

## Minimum kernel

Create a local authority policy from the schema in `spec-minimum.md` §8.2, then
run the governed lifecycle:

```sh
cargo run -p sea-forge-cli -- run \
  --policy sea-forge-policy.yaml \
  --intent "Generate and validate a simple DomainForge .sea model"
cargo run -p sea-forge-cli -- recall generate
```

Every invocation records its case, plan, authority decisions, trace, evidence,
settlement, and semantic envelope under `.sea-forge/`. Denied and failed work is
recorded too; only an accepted settlement exits zero.

Known limitation: `LocalWorkspaceSandbox` is a process-level sandbox — a
malicious child can write outside the workspace because there is no OS jail.
The minimum kernel therefore only allow-lists its trusted self-validator. An OS
jail is required before any untrusted command is allow-listed.

## Prerequisites

Before entering the shell you need only three host tools:

- [Git](https://git-scm.com/)
- [Devbox](https://www.jetify.com/devbox)
- [direnv](https://direnv.net/)

Everything else (Rust toolchain, `just`, `sops`, `age`, `gitleaks`,
`cargo-deny`) is pinned by Devbox and `rust-toolchain.toml`.

## Quick start

```sh
git clone <repo> sea-rs && cd sea-rs
devbox shell          # enter the pinned environment
direnv allow          # activate direnv (loads encrypted secrets if present)
just setup            # converge on the toolchain + dependencies
just doctor           # machine-readable environment check
just check            # fmt + clippy + check --locked + cargo deny + gitleaks
just test             # cargo test --workspace --all-features --locked
```

A fresh clone is foundation-ready when `just doctor`, `just check`, and
`just test` all exit zero (Shell-SPEC §3.2). Once the minimum slice is
implemented, `just proof` runs the conformance commands in
`spec-minimum.md` §12.2.

## Commands

`just` with no arguments prints the grouped recipe list. Required recipes
(Shell-SPEC §10.2):

| Recipe | Purpose |
| --- | --- |
| `just setup` | Converge on the pinned toolchain and dependencies |
| `just doctor` | Machine-readable environment check → `target/bootstrap-evidence/doctor.jsonl` |
| `just context-check` | Validate agent handoff structure and freshness |
| `just build` | `cargo build --workspace` |
| `just check` | fmt, clippy, `--locked` check, `cargo deny`, gitleaks |
| `just test` | `cargo test --workspace --all-features --locked` |
| `just proof` | Minimum-spec conformance (P1–P4b) once the slice exists |
| `just clean` | Remove `target/` and bootstrap evidence |
| `just secrets-init` | Create a local age key if absent, print its public key |
| `just secrets-edit [profile]` | Edit an encrypted profile via SOPS |
| `just secrets-check [profile]` | Decrypt to a pipe, validate names, redact values |
| `just secrets-rekey` | Update encrypted files after recipient changes |
| `just integration <name>` | Validate and run a declared API/MCP integration |

CI invokes the same recipes via `devbox run -- just ...` (`.github/workflows/ci.yml`).

## Agent context and handoff

Repository-local agent memory lives under `.agents/`. `CURRENT_STATUS.md` is the
resumable handoff; `OBSERVED_DEBT.md` holds out-of-scope gaps; `LESSONS.md` holds
only durable project-specific learning; and `OPEN_QUESTIONS.md` holds decisions
that research cannot resolve.

`just context-check` validates the required handoff sections and fails when
project files change without a corresponding status update. The script is plain
POSIX shell and accepts an optional `CONTEXT_BASE_REF`, so local tools and any CI
provider can use the same contract.

## Secrets (SOPS + age)

API and MCP credentials are encrypted at rest with [SOPS](https://github.com/getsops/sops)
and [age](https://github.com/FiloSottile/age). Per Shell-SPEC §8:

- Only `*.enc.env` files live under `secrets/` and are tracked. Plaintext is
  gitignored and never committed.
- The public age recipient lives in `.sops.yaml`; the **private key lives
  outside the repository** at `$SOPS_AGE_KEY_FILE` (default
  `~/.config/sops/key.txt`).
- `.envrc` decrypts the active profile (`$SEA_ENV`, default `dev`) only when a
  key is present. With no key it emits a redacted warning and leaves secret
  variables unset, so offline gates keep working (§8.2, §9).
- Secret-dependent recipes validate required `SEA_*` names before launch and
  never forward undeclared credential variables (§7.2).

```sh
just secrets-init           # first time: creates ~/.config/sops/key.txt
# add the printed age1... recipient to .sops.yaml, then:
just secrets-edit dev       # edit secrets/dev.enc.env
just secrets-check dev      # verify it decrypts (values redacted)
```

### Key loss

Lost age private keys are **not recoverable** from this repository. Generate a
new key with `just secrets-init`, add its recipient to `.sops.yaml`, and
re-encrypt with `just secrets-rekey` (Shell-SPEC §13).

## Offline work

Builds, formatting, clippy, unit tests, `cargo deny` (licenses/bans/sources),
and gitleaks run without credentials or network. Only `cargo deny check
advisories` fetches the RustSec database. Missing secrets block declared
integrations only — never the foundation gates (§4, §10.3).

## Diagnostics and observability

The CLI emits newline-delimited JSON diagnostics to stderr through Rust's
`tracing` ecosystem. Set `RUST_LOG` to control filtering; the default is `info`.

```sh
RUST_LOG=debug cargo run -p sea-forge-cli
```

Runtime diagnostics use stable event names and include `run_id`, `component`,
and `error_class`. They are distinct from the governed lifecycle events that the
minimum kernel will persist in `.sea-forge/runs/<run_id>/trace.jsonl`.

The one-shot minimum CLI intentionally has no aggregate metrics or external
telemetry exporter. Metrics and cross-process tracing belong at the full spec's
M3 server/integration boundary; they do not require Docker, but they do require
an explicit monitoring consumer.

## Project layout

```text
.agents/specs/       # normative specifications (Shell, minimum, full)
crates/
  sea-forge-core/    # kernel types and lifecycle (minimum spec)
  sea-forge-cli/     # one-shot CLI: run, validate, inspect, recall
justfile             # canonical command surface
devbox.json          # pinned system tools
rust-toolchain.toml  # pinned Rust channel + components
deny.toml            # cargo-deny policy
```

Runtime output goes under `.sea-forge/` (gitignored). Bootstrap evidence goes
under `target/bootstrap-evidence/` (gitignored). The two stores are never
conflated (Shell-SPEC §3.1).

## Specifications

- `.agents/specs/Shell-SPEC.md` — development shell, toolchain, secrets, CI.
- `.agents/specs/spec-minimum.md` — minimum governed kernel.
- `.agents/specs/spec-full.md` — additive M0–M8 evolution.

See `ARCHITECTURE.md` for the system map and how the layers fit together.
