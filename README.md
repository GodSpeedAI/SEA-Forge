# SEA Forge

A governed capability-execution kernel. SEA Forge normalizes operator intent
into typed operations, decides authority **before** any side effect, executes
allowed work in an isolated sandbox, records structured trace and evidence,
settles the declared outcome against criteria, and appends a semantic capability
envelope to an integrity-protected ledger.

Every side effect is authorized, then traced, evidenced, settled, and recorded.
A successful process is not a successful run unless settlement accepts the
declared outcome.

## Highlights

- **Default deny.** Unknown operations are denied. Policy load failure denies
  all. No fail-open path exists.
- **Authority before execution.** All operations are decided before any side
  effect runs. Authority and sandboxing are distinct controls that never weaken
  each other.
- **Settlement ≠ exit code.** A zero exit code does not mean accepted. Settlement
  evaluates evidence — artifact hashes, validators, criteria — against the
  declared outcome.
- **Complete records on every path.** Denied, failed, cancelled, and accepted
  runs all produce trace, evidence, settlement, and semantic envelopes.
- **Integrity ledger.** Append-only JSONL truth store with domain-separated
  SHA-256 hashes, Ed25519-signed checkpoints, and independent witness receipts.
- **Multi-agent governance.** Delegated agent tasks are ordinary governed runs —
  authority-checked, turn-capped, cancellable, transcript-evidenced, and
  independently settled on declared criteria.
- **Self-knowledge protocol.** Thoth answers questions about SEA Forge's own
  capabilities with evidence-linked, disclosure-controlled claims. Knowledge is
  never authority.

## Getting started

### Prerequisites

Before entering the shell you need only three host tools:

- [Git](https://git-scm.com/)
- [Devbox](https://www.jetify.com/devbox)
- [direnv](https://direnv.net/)

Everything else (Rust toolchain, `just`, `sops`, `age`, `gitleaks`,
`cargo-deny`) is pinned by Devbox and `rust-toolchain.toml`.

### Quick start

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

A fresh clone is ready when `just doctor`, `just check`, and `just test` all
exit zero. `just proof` runs the full conformance suite.

## Usage

### Run the governed lifecycle

```sh
sea-forge run \
  --policy sea-forge-policy.yaml \
  --intent "Generate a .sea model"
```

Every invocation records its case, plan, authority decisions, trace, evidence,
settlement, and semantic envelope under `.sea-forge/`. Denied and failed work is
recorded too; only an accepted settlement exits zero.

### Inspect and recall

```sh
sea-forge inspect <run_id>                     # show all six records of a run
sea-forge recall generate                      # search capability memory
sea-forge recall --result accepted --limit 10  # filter by outcome
```

### Validate artifacts

```sh
sea-forge validate <run_id>    # re-validate a past run's evidence
```

### Server mode

For long-running and concurrent workloads, run the governed daemon:

```sh
sea-forge server start                        # Unix-socket daemon
sea-forge submit --intent "Build the module"  # submit a case
sea-forge status <case_id>                    # check case status
sea-forge approve <run_id>                    # approve an escalated operation
sea-forge subscribe                           # stream lifecycle events
```

The server dispatches runs concurrently (configurable `max_concurrent_runs`,
default 4), manages approval workflows with configurable TTL, and supports
policy hot-reload between dispatches. All runs use the same governed lifecycle as
the CLI.

### Self-model and Thoth

Query SEA Forge about its own capabilities:

```sh
sea-forge self-model validate                 # verify bundled .sea self-model
sea-forge self-model rebuild --probe          # rebuild with environment probes
sea-forge self-model show --json              # composed self-model view

sea-forge ask ask_capability sea-forge.settlement   # query a capability
sea-forge ask ask_environment_status sandbox.jail   # query environment status
sea-forge ask ask_why_denied <run_id>               # explain a denial
```

Thoth answers with grounded claims backed by evidence references, not
free-text opinions. Disclosure is decided before retrieval — Thoth never
retrieves broadly and then redacts. A Thoth answer is never permission to
perform an operation.

### Agent delegation

Delegate governed tasks to external agents:

```sh
sea-forge agent probe <endpoint>              # one-call diagnostic
sea-forge agent list                          # registered agent endpoints
sea-forge run cancel <run_id>                 # cancel an in-flight delegation
sea-forge case manage <case_id> --iterations 5  # Thoth manager loop
```

Agent tasks are ordinary governed runs whose executor is an agent dialogue
instead of an argv command. Every delegation is authority-checked, turn-capped,
cancellable, and transcript-evidenced. Agent output is untrusted input —
settlement evaluates criteria, not agent narration.

### Exit codes

| Code | Meaning |
| ---- | ------- |
| 0 | Settlement `accepted` |
| 1 | Internal error |
| 2 | Input / usage error |
| 3 | Settlement `rejected` |
| 4 | Settlement `escalated` or governed denial |
| 5 | Awaiting approval / parked active (resumable) |

## Governed lifecycle

Every invocation follows the same eight-phase lifecycle:

```
Intent → Case → Plan → Authority → Sandbox → Execute → Evidence → Settlement → Envelope
```

1. **Intake** — Normalize operator intent into a typed `Case` with classified
   operations.
2. **Plan** — Generate a `CasePlan` with stages, sentries, milestones, and
   plan items. Plans may use versioned templates (ADLC, ODI, sequential/concurrent
   agents) and bind to hash-pinned semantic models.
3. **Authorize** — Evaluate every operation against the authority fabric.
   Default deny. Single decision point across all policy surfaces.
4. **Sandbox** — Prepare an isolated workspace with OS-level jails (Landlock on
   Linux, Seatbelt on macOS). Safe-join path validation prevents traversal.
5. **Execute** — Run allowed steps via argv-based process execution (no shell).
   Minimal explicit environment. Enforced timeouts.
6. **Evidence** — Collect SHA-256 content hashes, artifact descriptors with
   deterministic pre-mint identity (`ifl:hash:<sha256>`), and execution results.
7. **Settle** — Evaluate evidence against declared criteria. Detect false
   success (exit 0 but wrong/missing artifacts → rejected).
8. **Record** — Append semantic capability envelope to the integrity ledger.
   Update capability memory and feed Thoth knowledge graph.

## Authority model

The authority fabric is a single unified policy evaluation layer — not
per-surface gates. Policy bundles are hash-addressed YAML with 18+ first-class
surfaces:

```
file · shell_cmd · external_api · git_commit · pr_merge · prompt_risk ·
memory_recall · spec_pipeline · artifact_transition · attestation · deployment ·
secret_access · policy_mutation · evidence_mutation · extension_install ·
projection_execute · identity_binding · self_disclosure
```

Verdict resolution is deterministic and commutative: any `deny` blocks → any
`escalate` blocks → boundaries intersect → `allow`. Missing required evidence
denies. Unresolvable policy engine conflicts produce an `OpaqueConstraint` that
halts with `escalate`.

Identity is resolved via configurable identity maps with RBAC and
separation-of-duty rules. Built-in roles include operator, agent, system,
lifecycle controller, settlement authority, and self-model actor.

## CMMN case engine

Cases are the primary unit of work. A `CasePlanModel` contains:

- **Stages** with entry/exit sentries evaluated over the trace and case-file
  ledger
- **Plan items**: sandboxed tasks, agent tasks, human tasks (approvals),
  milestones, timer listeners, user event listeners
- **Discretionary items** for governed runtime plan mutation
- **Sentries** as event-condition rules that drive stage activation,
  reactivation, and milestone achievement

Cases auto-complete when all required items are completed. Reactivation sentries
enable non-waterfall behavior — a rejected simulation can re-activate the design
phase.

## Integrity ledger

Every record is a `LedgerEntry` with:

- ULID-based entry ordering with append ordinals
- Domain-separated SHA-256 hashes over JCS-canonical encoding
- Chain and Merkle Mountain Range construction for tamper evidence
- Ed25519-signed checkpoints at configurable cadence
- Independent witness receipts for external verification
- Inclusion and consistency proofs

Assurance levels: `legacy_digest_only` → `local_tamper_evident` →
`checkpoint_signed` → `externally_verified`.

## Sandbox and isolation

SEA Forge provides defense-in-depth isolation:

- **OS-level jails**: Landlock (Linux) / Seatbelt (macOS) restrict filesystem,
  network, and system call access. Untrusted commands must specify `jail` or
  `microvm` sandbox class in policy.
- **Workspace isolation**: Safe-join path validation prevents directory traversal,
  symlink escapes, and charset violations.
- **Agent cells**: Isolated execution contexts with scoped filesystem, environment,
  resource limits, and capability-restricted API access.
- **Network deny by default**: Jailed commands have no network unless policy
  explicitly grants `network: true`.

## DomainForge boundary

DomainForge owns `.sea` syntax, semantic graph, concept identities, validation,
and deterministic projections. SEA Forge owns authorization, isolation, side
effects, evidence, and settlement.

The `sea-forge-domainforge` crate calls the `domainforge-core` Rust library
directly to:

- Load and validate `.sea` files
- Normalize DomainForge governance verdicts as candidate authority verdicts
- Bind plans to hash-pinned semantic models with canonical concept IDs
- Execute governed `.sea` synthesis and projections (CALM, RDF, SBVR, SHACL,
  KG events, manifests, generated contracts)

SEA Forge remains the only component that materializes projection artifacts.
See the [DomainForge semantic-boundary decision](docs/decisions/ADR-001-domainforge-semantic-boundary.md).

## ADLC — Agentic Development Lifecycle

ADLC models software development as a governed lifecycle with four containment
stages, each a SEA Forge capability-execution cycle:

| Stage | Activities | Exit Milestone |
| ----- | ---------- | -------------- |
| **Frame** | Preparation, hypothesis, scope | `ProblemFramed` |
| **Form** | Design, simulation | `DevelopmentAuthorized` |
| **Build** | Implementation, continuous evaluation | `ReleaseCandidateAccepted` |
| **Activate** | Controlled deployment, production observation | `ActivationSettled` |

### ODI (Outcome-Driven Innovation)

ODI extends ADLC with outcome measurement. Settlement criteria bind to desired
outcomes via DomainForge concept references, so every criterion traces back to
*why* it exists. ODI analysis (importance/satisfaction ranking) runs as sandboxed
plan items.

### Self-model and Genesis

SEA Forge ships a versioned canonical `.sea` self-model (`godspeed.seaforge.system`)
so no installation starts semantically empty. The self-model enforces a strict
three-view separation:

| View | Contents |
| ---- | -------- |
| **Declared** | Canonical ontology + release realization |
| **Observed** | Installation-specific: extensions, toolchains, environment contracts |
| **Demonstrated** | Evidence-backed capability projection from ledgered records |

A declared capability is never reported as demonstrated. The system represents
absence and uncertainty with a rich status vocabulary from `unsupported` through
`proven_narrow_variation` to `deprecated` and `quarantined`.

## Agent orchestration

The agent orchestration layer is a governed delegation framework — not an agent
runtime. Every agent side effect is authority-gated, evidenced, cancellable, and
settlement-judged.

### Provider seam

The `sea-forge-agent` crate provides the `AgentProvider` trait with two built-in
implementations:

- **OpenAI-compatible** — HTTP API with pinned request shapes
- **Anthropic** — HTTP API with pinned request shapes
- **ACP (Agent Communication Protocol)** — for CLI agents (Claude Code, Codex,
  SWE_SEED-harnessed hosts)

All provider calls are gated by exact grants binding endpoint ID, config hash,
destination, model, limits, and credential reference. Credentials resolve via
indirection only after authority grants.

### Delegation model

Agent delegations are ordinary governed runs:

- Authority-checked against the `external_api` surface (deny-by-default)
- Turn-capped (`max_turns` enforced inside the dialogue loop)
- Cancellable without affecting sibling runs
- Transcript-evidenced with SHA-256 over JCS-canonical JSONL
- Independently settled on declared criteria

Agent output is **untrusted input**. Agent narration has no standing in
settlement — only criteria evaluation matters. No provider fallback; endpoint
failure is recorded, not routed around.

### Orchestration topologies

Built-in templates coordinate multi-agent work:

- **`sequential_agents`** — chain of tasks linked by source-bound sentries
- **`concurrent_agents`** — parallel tasks with all-success rollup milestone

Templates instantiate to plain case plans. The case engine (sentries,
milestones, discretionary items) is the coordination substrate — no actor
runtime.

### Thoth manager loop

Thoth can orchestrate development cycles through bounded iterations:

1. Read case state and ledger
2. Propose discretionary `agent_task` items via the planner path
3. Await settlement of proposed work
4. Evaluate: `satisfied | progressing | stalled | blocked`

The manager never settles or promotes what it proposed (separation of duty).
Iteration cap triggers case parking and escalation.

## Spec-to-code pipeline

The `sea-forge-spec-pipeline` crate drives a governed generation chain:

```
ADR → PRD → SDS → SEA → AST → IR → manifest → codegen → last-mile
```

Each stage produces governed records with deterministic replay. Generated zones
are guarded — hand-edits are rejected; change the source or generator and
regenerate.

## Artifact identity and IP

The `sea-forge-artifact-ip` crate tracks work products through their full
lifecycle:

- **Pre-mint identity**: Deterministic `ifl:hash:<sha256>` computed from
  canonical content properties (no timestamps, paths, or hostnames)
- **Registration**: Artifact registration with type, stage, producer, owner,
  license, and review status
- **Lineage DAG**: Track derivation chains and transformation provenance
- **Stage progression**: Cognitive → Intellectual → Product → Capital
- **Transition tokens**: Governed promote/derive operations with authority
- **IFL attestation**: From content-hash identity to registered intellectual
  property token
- **Capital projection**: Aggregate IP portfolio views

## Project layout

```text
.agents/specs/               # normative specifications
docs/decisions/              # architecture decision records
crates/
  sea-forge-core/            # kernel types, IDs, lifecycle, errors
  sea-forge-ledger/          # append-only JSONL truth store, integrity proofs
  sea-forge-domain/          # intent vocabulary, domain primitives
  sea-forge-domainforge/     # DomainForge semantic adapter
  sea-forge-authority/       # policy engine, authority fabric, RBAC
  sea-forge-planner/         # case plans, stages, sentries, templates
  sea-forge-sandbox/         # workspace isolation, jail backends
  sea-forge-runtime/         # process execution, timeout enforcement
  sea-forge-trace/           # structured trace events, EventSink trait
  sea-forge-evidence/        # evidence collection, hash verification
  sea-forge-settlement/      # settlement evaluation, criteria, authorities
  sea-forge-capability/      # capability envelopes, memory, semantic recall
  sea-forge-extension/       # extension registry, projection adapter ABI
  sea-forge-server/          # Tokio daemon, Unix-socket API, event stream
  sea-forge-cli/             # CLI frontend
  sea-forge-spec-pipeline/   # spec-to-code generation pipeline
  sea-forge-cell/            # cell-based isolation and federation
  sea-forge-artifact-ip/     # artifact identity, IP lifecycle, attestation
  sea-forge-self-model/      # Genesis self-model, self-knowledge primitives
models/                      # bundled .sea self-model and ontology seeds
justfile                     # canonical command surface
devbox.json                  # pinned system tools
rust-toolchain.toml          # pinned Rust channel + components
deny.toml                    # cargo-deny policy
```

Runtime output goes under `.sea-forge/` (gitignored). Bootstrap evidence goes
under `target/bootstrap-evidence/` (gitignored). The two stores are never
conflated.

## Contributor happy path

```sh
git switch main
git pull --ff-only
git switch -c feat/short-description

# make changes
just check-fast       # what the pre-commit hook runs
git add ...
git commit            # pre-commit hook runs `just pre-commit`

just pre-push         # what the pre-push hook runs
git push -u origin HEAD

just pr               # verify + push + open a PR via gh; refuses from main
```

The PR title must be Conventional Commit-shaped (`feat(scope): ...`,
`fix(scope): ...`, etc.) so Release Please can derive the next release. The full
happy path and recovery procedures are in [`CONTRIBUTING.md`](CONTRIBUTING.md);
the CI/CD architecture is in [`docs/ci-cd-architecture.md`](docs/ci-cd-architecture.md).

## Commands

`just` with no arguments prints the grouped recipe list. Key recipes:

| Recipe | Purpose |
| --- | --- |
| `just setup` | Converge on the pinned toolchain and dependencies |
| `just sync` | Re-converge after a pull that touched Cargo.toml / rust-toolchain.toml |
| `just doctor` | Machine-readable environment check |
| `just hooks-install` | Set `core.hooksPath=.githooks` |
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
| `just proof` | Run the full conformance suite (P1–P4b + milestone gates) |
| `just release-check [tag]` | Verify workspace/core/cli versions agree |
| `just clean` | Remove `target/` and bootstrap evidence |
| `just secrets-init` | Create a local age key if absent |
| `just secrets-edit [profile]` | Edit an encrypted secrets profile via SOPS |
| `just secrets-check [profile]` | Verify decryption (values redacted) |

CI invokes the same recipes via `devbox run -- just ...`. The required check
name on a pull request is `CI / gate`.

## Secrets (SOPS + age)

API and MCP credentials are encrypted at rest with [SOPS](https://github.com/getsops/sops)
and [age](https://github.com/FiloSottile/age):

- Only `*.enc.env` files live under `secrets/` and are tracked. Plaintext is
  gitignored and never committed.
- The private key lives **outside the repository** at `$SOPS_AGE_KEY_FILE`
  (default `~/.config/sops/key.txt`).
- `.envrc` decrypts the active profile (`$SEA_ENV`, default `dev`) only when a
  key is present. With no key it emits a warning and leaves secret variables
  unset, so offline gates keep working.

```sh
just secrets-init           # first time: creates ~/.config/sops/key.txt
# add the printed age1... recipient to .sops.yaml, then:
just secrets-edit dev       # edit secrets/dev.enc.env
just secrets-check dev      # verify it decrypts (values redacted)
```

Lost age private keys are **not recoverable**. Generate a new key with
`just secrets-init`, add its recipient to `.sops.yaml`, and re-encrypt with
`just secrets-rekey`.

## Offline work

Builds, formatting, clippy, unit tests, `cargo deny` (licenses/bans/sources),
and gitleaks run without credentials or network. Only `cargo deny check
advisories` fetches the RustSec database. Missing secrets block declared
integrations only — never the foundation gates.

## Diagnostics and observability

The CLI emits newline-delimited JSON diagnostics to stderr through Rust's
`tracing` ecosystem. Set `RUST_LOG` to control filtering (default `info`):

```sh
RUST_LOG=debug sea-forge run --intent "..."
```

Runtime diagnostics use stable event names and include `run_id`, `component`,
and `error_class`. They are distinct from the governed lifecycle events persisted
in `.sea-forge/` run directories.

The server exposes an event subscription stream for external monitoring
consumers and supports OpenTelemetry-compatible trace export.

## Specifications

| Spec | Scope |
| ---- | ----- |
| [Shell-SPEC.md](.agents/specs/Shell-SPEC.md) | Development shell, toolchain, secrets, CI |
| [spec-minimum.md](.agents/specs/spec-minimum.md) | Minimum governed kernel (M-slice) |
| [spec-full.md](.agents/specs/spec-full.md) | Full system milestones M0–M8 |
| [spec-adlc-thoth-minimum.md](.agents/specs/spec-adlc-thoth-minimum.md) | ADLC/ODI lifecycle, Genesis self-model, Thoth (M9–M11) |
| [spec-agent-orchestration.md](.agents/specs/spec-agent-orchestration.md) | Agent delegation, orchestration topologies, ACP (M12–M16) |
| [ADR-001](docs/decisions/ADR-001-domainforge-semantic-boundary.md) | DomainForge semantic boundary |

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
