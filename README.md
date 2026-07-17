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
  --intent "Generate a .sea model"
cargo run -p sea-forge-cli -- recall generate
```

Every invocation records its case, plan, authority decisions, trace, evidence,
settlement, and semantic envelope under `.sea-forge/`. Denied and failed work is
recorded too; only an accepted settlement exits zero.

Known limitation: `LocalWorkspaceSandbox` is a process-level sandbox — a
malicious child can write outside the workspace because there is no OS jail.
The minimum kernel therefore only allow-lists its trusted self-validator. An OS
jail is required before any untrusted command is allow-listed.

The minimum demo's `model.sea` is a JSON stub checked by SEA Forge's trusted
self-validator. It proves the governed lifecycle, artifact identity, and false-
success handling; it is not DomainForge SEA syntax and does not claim
DomainForge compatibility.

## DomainForge boundary

Full SEA Forge governs work in a world defined by `.sea`. DomainForge owns that
world's syntax, semantic graph, concept identities, validation, policy
evaluation, and deterministic projections. SEA Forge owns authorization,
isolation, side effects, evidence, and settlement.

The draft full-system roadmap introduces a first-party
`sea-forge-domainforge` crate at M0. It will call the `domainforge-core` Rust
library directly to load and validate `.sea` and to normalize DomainForge
authority results as candidate verdicts. M2 binds plans to hash-pinned semantic
models and canonical concept IDs. M5 adds governed `.sea` synthesis and
in-memory DomainForge projections; SEA Forge remains the only component that
materializes projection artifacts.

See the [DomainForge semantic-boundary decision](docs/decisions/ADR-001-domainforge-semantic-boundary.md)
and `.agents/specs/spec-full.md` §§7.0a and 10.4a.

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
just hooks-install    # install the checked-in git hooks (.githooks/)
just doctor           # machine-readable environment check
just check            # fmt + clippy + check --locked + cargo deny + gitleaks
just test             # cargo test --workspace --all-features --locked
```

A fresh clone is foundation-ready when `just doctor`, `just check`, and
`just test` all exit zero (Shell-SPEC §3.2). `just proof` runs the implemented
minimum kernel's conformance commands from `spec-minimum.md` §12.2.

## Contributor happy path

```sh
git switch main
git pull --ff-only
git switch -c feat/short-description

# make changes; update .agents/CURRENT_STATUS.md if you touched tracked files
just check-fast       # what the pre-commit hook runs
git add ...
git commit            # pre-commit hook runs `just pre-commit`

just pre-push         # what the pre-push hook runs (~20s warm, measured; longer cold)
git push -u origin HEAD

just pr               # verify + push + open a PR via gh; refuses from main
```

The PR title must be Conventional Commit-shaped (`feat(scope): ...`,
`fix(scope): ...`, etc.) so Release Please can derive the next release from
the squash-commit title. The full happy path, common failures, and recovery
procedures are in [`CONTRIBUTING.md`](CONTRIBUTING.md); the CI/CD architecture
is in [`docs/ci-cd-architecture.md`](docs/ci-cd-architecture.md); the release
runbook is in [`docs/skills/release-management.md`](docs/skills/release-management.md).

## Commands

`just` with no arguments prints the grouped recipe list. The full command
surface is documented in `CONTRIBUTING.md` and `docs/ci-cd-architecture.md`
§2. Required recipes (Shell-SPEC §10.2):

| Recipe | Purpose |
| --- | --- |
| `just setup` | Converge on the pinned toolchain and dependencies |
| `just sync` | Re-converge after a pull that touched Cargo.toml / rust-toolchain.toml |
| `just doctor` | Machine-readable environment check → `target/bootstrap-evidence/doctor.jsonl` |
| `just hooks-install` | Set `core.hooksPath=.githooks` (pre-commit, pre-push, post-checkout) |
| `just check-fast` | Context + fmt + typecheck — what the pre-commit hook runs |
| `just fmt` / `just fmt-check` | Apply / verify rustfmt |
| `just lint` | clippy with `-D warnings` |
| `just typecheck` | `cargo check --workspace --all-targets --locked` |
| `just security` | `cargo deny check` + `gitleaks detect` |
| `just test` | `cargo test --workspace --all-features --locked` |
| `just build` | `cargo build --workspace --all-targets --locked` |
| `just check` | context + fmt + clippy + typecheck + cargo deny + gitleaks |
| `just ci` | Canonical CI verification (union of all required CI jobs) |
| `just pr` | Verify + push + open a PR via `gh`; refuses from `main` |
| `just release-check [tag]` | Verify workspace/core/cli versions agree (and match `tag` if given) |
| `just publish-bootstrap <crate>` | One-time manual crates.io publish (trusted-publishing bootstrap) |
| `just proof` | Run the implemented minimum-spec conformance proofs (P1–P4b) |
| `just clean` | Remove `target/` and bootstrap evidence |
| `just secrets-init` | Create a local age key if absent, print its public key |
| `just secrets-edit [profile]` | Edit an encrypted profile via SOPS |
| `just secrets-check [profile]` | Decrypt to a pipe, validate names, redact values |
| `just secrets-rekey` | Update encrypted files after recipient changes |
| `just integration <name>` | Validate and run a declared API/MCP integration |

CI invokes the same recipes via `devbox run -- just ...`. The required check
name on a pull request is `CI / gate`; the PR-title check is
`PR title / conventional-commit`. See
[`docs/ci-cd-architecture.md`](docs/ci-cd-architecture.md) for the full
local-to-remote flow and the release causal chain.

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
minimum kernel persists in `.sea-forge/runs/<run_id>/trace.jsonl`.

The one-shot minimum CLI intentionally has no aggregate metrics or external
telemetry exporter. Metrics and cross-process tracing belong at the full spec's
M3 server/integration boundary; they do not require Docker, but they do require
an explicit monitoring consumer.

## Project layout

```text
.agents/specs/       # normative specifications (Shell, minimum, full)
docs/decisions/      # architecture decision records
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
- `docs/decisions/ADR-001-domainforge-semantic-boundary.md` — DomainForge owns
  `.sea` semantics; SEA Forge owns governed execution and final authority.

See `ARCHITECTURE.md` for the system map and how the layers fit together.

## License and commercial use

SEA Forge is source-available under the
[SEA-Forge Sustainable Use License](LICENSE). Separate
[commercial licensing](COMMERCIAL-LICENSE.md) is available for hosted,
embedded, redistributed, white-labeled, client-facing, and enterprise use.

SEA Forge follows the **Photoshop Principle**: you may own and commercialize
independent outputs you create with the tool, but you do not own or resell the
tool itself.

- **You may:** use and modify SEA Forge for permitted internal, personal,
  educational, research, evaluation, and non-commercial purposes.
- **You may also:** own and commercialize independent outputs generated with
  SEA Forge, provided they do not include SEA Forge itself, repository-generated
  materials, or enterprise-only components.
- **You may not without a commercial license:** resell or redistribute SEA
  Forge, offer it as a hosted or managed service, embed it in a paid product,
  white-label it, provide SEA-Forge-powered services to clients, or use
  enterprise-only components.

Enterprise-only files are identified by the rules in
[LICENSE_EE.md](LICENSE_EE.md). Third-party components remain subject to their
own licenses.

For production agents, client-facing deployments, or enterprise rights,
contact [licensing@godspeedai.com](mailto:licensing@godspeedai.com).
