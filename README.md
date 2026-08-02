# SEA Forge

**Authority before action. Proof before acceptance.**

AI agents no longer just produce text. They write files, run commands, call
APIs, open pull requests, and modify infrastructure. The controls around them
have not kept up. A prompt says what you want; it does not enforce what is
allowed. An exit code says a process ran; it does not say the job was done.
And an agent's own summary of its work is not an inspection of it.

SEA Forge is the missing control layer. Before an agent touches a protected
surface, SEA Forge decides whether that specific action, by that actor, is
authorized. Authorized work runs inside a bounded sandbox. Everything that
happens is captured as structured evidence. Then — independently of the agent
and its exit code — SEA Forge checks the evidence against the outcome you
declared. Only if the evidence holds is the work accepted.

> Serious work has two boundaries: permission before action and proof after.

No building department lets a contractor issue their own permit, perform the
work, conduct their own inspection, and declare the building safe. Yet that is
the standard operating model for autonomous agents: interpret a prompt as
permission, perform consequential work, cite a successful exit as proof. SEA
Forge ends that arrangement.

```text
Intent → authority decision → bounded execution → evidence → independent acceptance → permanent record
```

Denied and failed work leaves the same complete record as accepted work.
[See it work](#see-it-work), or [install and run a governed
lifecycle](#installation-and-quick-start).

## The failure pattern

Agents can now write and delete files, run shell commands, call internal and
external APIs, open and merge pull requests, modify infrastructure, use
credentials, and delegate work to other agents. Most teams govern that
activity with some mix of prompts, sandbox configuration, repository
permissions, logs, human review, and the agent's own completion summary. Each
is a real control. None of them is the control it is being asked to be:

- **A prompt expresses intent. It does not enforce authority.** "Do not touch
  that directory" is a behavioral instruction, interpreted by the same system
  that decides how to act on it. Nothing independent stands between the
  instruction and the write.
- **A sandbox limits where work can happen. It does not decide whether the
  work should happen.** Containment shrinks the blast radius of an action; it
  grants no permission for it.
- **A log records activity. It does not prove an outcome.** Logs can show a
  busy afternoon without answering what was declared, what was authorized, or
  whether the result was accepted.
- **A zero exit code proves a process ran successfully. It does not prove the
  job was completed correctly.** The wrong artifact exits zero just as cleanly
  as the right one.
- **An agent's completion claim cannot be its own acceptance test.** The actor
  doing the work is also declaring whether the work succeeded. Useful context;
  not proof.

If any of these sound familiar, you have the gap:

- The agent edited the wrong file, then reported success — and the report is
  what got reviewed.
- The prompt said not to access something. Nothing enforced it, and you found
  out after the side effect.
- The process exited zero while the artifact the job actually required was
  missing, wrong, or never validated.
- An agent delegated work to another agent, and afterward nobody could say
  exactly which actor had been allowed to do what.
- The logs show everything that happened and nothing about whether it should
  have happened, or whether it was accepted.

These are not agent failures, and the fix is not a more careful prompt. They
are infrastructure failures: no authoritative decision before the action, and
no independent judgment after it.

## What SEA Forge changes

Before:

```text
Prompt → agent acts → agent says it worked → logs accumulate
```

With SEA Forge:

```text
Intent → authority decision → bounded execution → evidence → independent acceptance → permanent record
```

The decision before the action is **authority**: every operation the work will
require is evaluated against policy, at a single decision point, before any
side effect runs. The judgment after the action is **settlement**: the
independent decision that the recorded evidence satisfies the declared
outcome. Not the process's exit code. Not the agent's narration. The evidence,
measured against criteria declared before the work ran.

The analogy maps directly, and only where it fits: the permit is the authority
decision; the controlled worksite is the sandbox; the inspection is the
evidence — artifact hashes, validators, traces; acceptance against
requirements is settlement; the permanent record is the integrity ledger;
subcontractors are delegated agents, governed by the same rules.

The practical result is bounded autonomy: routine actions proceed under
declared authority, risky actions escalate to a human with the evidence
attached, and every result stays inspectable.

Put together, that composition is what we mean by a **governed
capability-execution kernel** — one lifecycle that authorizes, executes,
evidences, settles, and records consequential work. The name is a mouthful.
The mechanism is the point.

## See it work

The smallest credible demonstration: submit one intent, watch one operation
get allowed, denied, or escalated, then inspect the evidence and the
settlement. Everything below uses the CLI; install takes a few commands
([quick start](#installation-and-quick-start)).

Submit an intent, with a policy file that declares what the run may do:

```sh
sea-forge run \
  --policy sea-forge-policy.yaml \
  --intent "Generate a .sea model"
```

Every invocation records its case, plan, authority decisions, trace, evidence,
settlement, and capability envelope under `.sea-forge/`. Denied and failed
work is recorded too. The process exit code is the settlement, not the
executor's self-report:

| Code | Meaning |
| ---- | ------- |
| 0 | Settlement `accepted` |
| 1 | Internal error |
| 2 | Input / usage error |
| 3 | Settlement `rejected` |
| 4 | Settlement `escalated` or governed denial |
| 5 | Awaiting approval / parked active (resumable) |

Inspect what actually happened, and why:

```sh
sea-forge inspect <run_id>                     # show all six records of a run
sea-forge ask ask_why_denied <run_id>          # explain a denial
```

Re-check the evidence later, and search what has been accepted before:

```sh
sea-forge validate <run_id>                    # re-validate a past run's evidence
sea-forge recall generate                      # search capability memory
sea-forge recall --result accepted --limit 10  # filter by outcome
```

What you should see:

- **Accepted work exits 0.** `inspect` shows the declared criteria and the
  evidence that satisfied them.
- **Denied work exits 4** and never produces a side effect. The denial is
  still a complete record; `ask_why_denied` explains it.
- **False success exits 3.** A step can return zero while the required
  artifact is missing or wrong; settlement rejects the run because the
  evidence does not hold.
- **Escalated work exits 4; parked work exits 5.** A run awaiting human
  approval is resumable, not lost — `sea-forge approve <run_id>` in server
  mode continues it.

## Core guarantees

Each guarantee names the failure it removes and the record that proves it. The
conformance suite exercises all of them — `just proof` runs it.

### Default deny

An unknown operation does not inherit permission from a vague prompt. It stops
before any side effect, and the denial — with its reason — becomes part of the
record. A policy that fails to load denies everything; there is no fail-open
path.

### Authority before side effects

Governance that arrives after the action is documentation. Every operation is
decided before it runs, at one decision point spanning every policy surface —
files, shell commands, external APIs, commits, merges, secrets, deployments,
and more. The decision record binds actor, operation, resource, and verdict.
Missing required evidence denies.

### Sandboxing is a separate control

"It ran in a container" is not "it was allowed." Authority decides whether
work may happen; the sandbox constrains how and where allowed work happens —
OS-level jails (Landlock on Linux, Seatbelt on macOS), safe-join path
validation, and no network unless policy grants it. Neither control
substitutes for the other: a disallowed action inside a jail is still denied.

### Evidence is collected, not asserted

A completion claim that cannot be inspected is a confident rumor. Every run
produces SHA-256 content hashes, artifact descriptors with deterministic
identity (`ifl:hash:<sha256>`), execution results, and a structured trace.
`sea-forge inspect` shows them; `sea-forge validate` re-checks them later.

### Settlement is independent of the exit code

A process can return zero and still fail the job. Settlement evaluates the
declared outcome and its criteria, not merely whether the command ran — so
exit-zero-with-wrong-artifacts is rejected, and only an accepted settlement
exits 0. Validation checks artifacts; evidence supports claims; settlement
decides. The agent's narration has no standing in the decision.

### Every path leaves a complete record

The runs you most need to understand are usually the ones that left nothing
behind. Denied, failed, cancelled, escalated, rejected, and accepted work all
produce the full record set — case, plan, authority decisions, trace,
evidence, settlement, and the capability envelope appended to the ledger.

### Delegation keeps the same governance

Agent chains diffuse responsibility faster than they create accountability —
unless delegated work stays ordinary governed work. A delegated agent task is
authority-checked, turn-capped, cancellable, transcript-evidenced, and
independently settled. Subcontractors do not get to grade their own work
either.

### Records are tamper-evident and re-verifiable

An audit trail you can quietly edit is a claim, not evidence. The ledger is
append-only JSONL with domain-separated SHA-256 hashes over canonical
encoding, chain and Merkle Mountain Range construction for tamper evidence,
Ed25519-signed checkpoints, and independent witness receipts as the assurance
level rises. Inclusion and consistency proofs let you verify records without
trusting the store.

## How it works

A governed run starts from what you declared, not from what the agent decides
to do first. You state an intent and a policy; SEA Forge normalizes the intent
into typed operations, decides each operation before anything executes, runs
allowed work inside a bounded environment, collects evidence as it goes, then
settles the outcome against the criteria you declared.

```text
Intent → Case → Plan → Authority → Sandbox → Execute → Evidence → Settlement → Envelope
```

1. **Intake** — Normalize operator intent into a typed `Case` with classified
   operations.
2. **Plan** — Generate a `CasePlan` with stages, sentries, milestones, and
   plan items. Plans may use versioned templates (ADLC, ODI,
   sequential/concurrent agents) and bind to hash-pinned semantic models.
3. **Authorize** — Evaluate every operation against the authority fabric.
   Default deny. Single decision point across all policy surfaces.
4. **Sandbox** — Prepare an isolated workspace with OS-level jails (Landlock
   on Linux, Seatbelt on macOS). Safe-join path validation prevents traversal.
5. **Execute** — Run allowed steps via argv-based process execution (no
   shell). Minimal explicit environment. Enforced timeouts.
6. **Evidence** — Collect SHA-256 content hashes, artifact descriptors with
   deterministic pre-mint identity (`ifl:hash:<sha256>`), and execution
   results.
7. **Settle** — Evaluate evidence against declared criteria. Detect false
   success (exit 0 but wrong/missing artifacts → rejected).
8. **Record** — Append a semantic capability envelope — the durable record of
   what capability was exercised, under what authority, with what outcome — to
   the integrity ledger. Update capability memory and feed the Thoth knowledge
   graph.

Four distinctions do most of the work:

- **Authority** decides whether work may happen.
- **Sandboxing** limits how and where allowed work happens.
- **Evidence** records what happened.
- **Settlement** decides whether what happened satisfies the declared outcome.

One honest boundary: SEA Forge governs actions routed through its declared
execution and authority surfaces. A side effect that occurs through an
unmediated path — a shell opened outside the governed run — is outside that
boundary. Govern the paths that matter rather than assume universal
interception.

On status: the CLI lifecycle above — run, inspect, validate, recall — is
implemented and covered by the conformance suite. Broader capabilities
described in the reference below (the ADLC lifecycle, the Genesis self-model,
orchestration templates, artifact IP) are built milestone by milestone against
the normative specifications. `just proof` runs P1–P4b plus the milestone
gates; the suite, not this page, is the authority on what is proven.

## Where it gets used

- **Coding agents that write files and open pull requests.** Policy decides
  which paths, commands, commits, and merges an agent may touch; settlement
  decides whether the required artifact — code, tests, proof commands —
  actually exists and passes.
- **Agents that call external APIs.** Provider calls are gated by exact grants
  binding endpoint, config hash, destination, model, limits, and credential
  reference. Credentials resolve by indirection only after authority grants.
- **Infrastructure and deployment changes.** Deployment is a first-class
  authority surface. Risky changes escalate for human approval instead of
  being silently allowed or permanently blocked.
- **Multi-agent delegation.** Sequential and concurrent templates coordinate
  agent chains while each delegated task remains authority-checked,
  turn-capped, cancellable, and independently settled.
- **Governed spec-to-code pipelines.** Requirements and design artifacts flow
  through a generation chain with governed records at each stage; generated
  zones reject hand-edits.
- **Regulated or auditable automation.** Every run answers what was requested,
  what was authorized, what ran, what evidence was produced, and how the
  outcome was decided — at execution time, not assembled afterward for the
  audit.

SEA Forge governs the work, not the worker: argv processes, sandboxed tasks,
and external agents behind the provider seam are all replaceable executors
inside the same lifecycle.

## Installation and quick start

### Prerequisites

Only three host tools are required before entering the shell:

- [Git](https://git-scm.com/)
- [Devbox](https://www.jetify.com/devbox)
- [direnv](https://direnv.net/)

Everything else (Rust toolchain, `just`, `sops`, `age`, `gitleaks`,
`cargo-deny`) is pinned by Devbox and `rust-toolchain.toml`, so the
environment you verify is the environment CI verifies.

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
exit zero. `just proof` runs the full conformance suite — the executable
statement of every guarantee above.

How to recognize trouble:

- `just doctor` is machine-readable and names what is missing.
- Without an age key, `direnv` warns and leaves secret variables unset; the
  foundation gates still work offline. Missing secrets block declared
  integrations only — never the core gates.
- Runtime records appear under `.sea-forge/` (gitignored). Bootstrap evidence
  goes under `target/bootstrap-evidence/` (also gitignored). The two stores
  are never conflated.

## Technical reference

Depth for implementers and evaluators. None of this is required to run the
quick start; all of it is behavior the specifications define.

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
policy hot-reload between dispatches. All runs use the same governed lifecycle
as the CLI.

### Agent delegation

Delegate governed tasks to external agents:

```sh
sea-forge agent probe <endpoint>              # one-call diagnostic
sea-forge agent list                          # registered agent endpoints
sea-forge run cancel <run_id>                 # cancel an in-flight delegation
sea-forge case manage <case_id> --iterations 5  # Thoth manager loop
```

Agent tasks are ordinary governed runs whose executor is an agent dialogue
instead of an argv command. Every delegation is authority-checked,
turn-capped, cancellable, and transcript-evidenced. Agent output is untrusted
input — settlement evaluates criteria, not agent narration.

### Self-model and Thoth

Thoth is SEA Forge's self-knowledge protocol: it answers questions about the
system's own capabilities with evidence-linked, disclosure-controlled claims.
Knowledge is never authority.

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

### Authority model

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
denies. Unresolvable policy engine conflicts produce an `OpaqueConstraint`
that halts with `escalate`.

Identity is resolved via configurable identity maps with RBAC and
separation-of-duty rules. Built-in roles include operator, agent, system,
lifecycle controller, settlement authority, and self-model actor.

### Case engine

Cases are the primary unit of work, modeled on CMMN concepts — stages,
sentries, milestones, and plan items. A `CasePlanModel` contains:

- **Stages** with entry/exit sentries evaluated over the trace and case-file
  ledger
- **Plan items**: sandboxed tasks, agent tasks, human tasks (approvals),
  milestones, timer listeners, user event listeners
- **Discretionary items** for governed runtime plan mutation
- **Sentries** as event-condition rules that drive stage activation,
  reactivation, and milestone achievement

Cases auto-complete when all required items are completed. Reactivation
sentries enable non-waterfall behavior — a rejected simulation can re-activate
the design phase.

### Integrity ledger

Every record is a `LedgerEntry` with:

- ULID-based entry ordering with append ordinals
- Domain-separated SHA-256 hashes over JCS-canonical encoding
- Chain and Merkle Mountain Range construction for tamper evidence
- Ed25519-signed checkpoints at configurable cadence
- Independent witness receipts for external verification
- Inclusion and consistency proofs

Assurance levels: `legacy_digest_only` → `local_tamper_evident` →
`checkpoint_signed` → `externally_verified`.

### Sandbox and isolation

SEA Forge provides defense-in-depth isolation:

- **OS-level jails**: Landlock (Linux) / Seatbelt (macOS) restrict filesystem,
  network, and system call access. Untrusted commands must specify `jail` or
  `microvm` sandbox class in policy.
- **Workspace isolation**: Safe-join path validation prevents directory
  traversal, symlink escapes, and charset violations.
- **Agent cells**: Isolated execution contexts with scoped filesystem,
  environment, resource limits, and capability-restricted API access.
- **Network deny by default**: Jailed commands have no network unless policy
  explicitly grants `network: true`.

### DomainForge boundary

DomainForge owns `.sea` syntax, semantic graph, concept identities,
validation, and deterministic projections. SEA Forge owns authorization,
isolation, side effects, evidence, and settlement.

The `sea-forge-domainforge` crate calls the `domainforge-core` Rust library
directly to:

- Load and validate `.sea` files
- Normalize DomainForge governance verdicts as candidate authority verdicts
- Bind plans to hash-pinned semantic models with canonical concept IDs
- Execute governed `.sea` synthesis and projections (CALM, RDF, SBVR, SHACL,
  KG events, manifests, generated contracts)

SEA Forge remains the only component that materializes projection artifacts.
See the
[DomainForge semantic-boundary decision](docs/decisions/ADR-001-domainforge-semantic-boundary.md).

### Spec-to-code pipeline

The `sea-forge-spec-pipeline` crate drives a governed generation chain:

```
ADR → PRD → SDS → SEA → AST → IR → manifest → codegen → last-mile
```

Each stage produces governed records with deterministic replay. Generated
zones are guarded — hand-edits are rejected; change the source or generator
and regenerate.

<details>
<summary><strong>Agent orchestration: provider seam, topologies, and the Thoth manager loop</strong></summary>

The agent orchestration layer is a governed delegation framework — not an
agent runtime. Every agent side effect is authority-gated, evidenced,
cancellable, and settlement-judged.

The `sea-forge-agent` crate provides the `AgentProvider` trait with built-in
implementations: OpenAI-compatible HTTP, Anthropic HTTP, and ACP (Agent
Communication Protocol) for CLI agents (Claude Code, Codex, SWE_SEED-harnessed
hosts). All provider calls are gated by exact grants binding endpoint ID,
config hash, destination, model, limits, and credential reference. Credentials
resolve via indirection only after authority grants. No provider fallback;
endpoint failure is recorded, not routed around.

Built-in templates coordinate multi-agent work: `sequential_agents` chains
tasks through source-bound sentries; `concurrent_agents` runs tasks in
parallel with an all-success rollup milestone. Templates instantiate to plain
case plans — the case engine (sentries, milestones, discretionary items) is
the coordination substrate, not an actor runtime.

Thoth can also orchestrate development cycles through bounded iterations: read
case state and ledger, propose discretionary `agent_task` items via the
planner path, await settlement, then evaluate `satisfied | progressing |
stalled | blocked`. The manager never settles or promotes what it proposed
(separation of duty). The iteration cap triggers case parking and escalation.

</details>

<details>
<summary><strong>ADLC, ODI, and the Genesis self-model</strong></summary>

ADLC (Agentic Development Lifecycle) models software development as a governed
lifecycle with four containment stages, each a SEA Forge capability-execution
cycle:

| Stage | Activities | Exit Milestone |
| ----- | ---------- | -------------- |
| **Frame** | Preparation, hypothesis, scope | `ProblemFramed` |
| **Form** | Design, simulation | `DevelopmentAuthorized` |
| **Build** | Implementation, continuous evaluation | `ReleaseCandidateAccepted` |
| **Activate** | Controlled deployment, production observation | `ActivationSettled` |

ODI (Outcome-Driven Innovation) extends ADLC with outcome measurement.
Settlement criteria bind to desired outcomes via DomainForge concept
references, so every criterion traces back to *why* it exists. ODI analysis
(importance/satisfaction ranking) runs as sandboxed plan items.

SEA Forge ships a versioned canonical `.sea` self-model
(`godspeed.seaforge.system`) so no installation starts semantically empty. The
self-model enforces a strict three-view separation:

| View | Contents |
| ---- | -------- |
| **Declared** | Canonical ontology + release realization |
| **Observed** | Installation-specific: extensions, toolchains, environment contracts |
| **Demonstrated** | Evidence-backed capability projection from ledgered records |

A declared capability is never reported as demonstrated. The system represents
absence and uncertainty with a rich status vocabulary from `unsupported`
through `proven_narrow_variation` to `deprecated` and `quarantined`.

</details>

<details>
<summary><strong>Artifact identity and IP (IFL)</strong></summary>

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

</details>

### Project layout

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

### Commands

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

### Secrets (SOPS + age)

API and MCP credentials are encrypted at rest with
[SOPS](https://github.com/getsops/sops) and
[age](https://github.com/FiloSottile/age):

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

Lost age private keys are **not recoverable**. Generate a new key with `just
secrets-init`, add its recipient to `.sops.yaml`, and re-encrypt with `just
secrets-rekey`.

### Offline work

Builds, formatting, clippy, unit tests, `cargo deny` (licenses/bans/sources),
and gitleaks run without credentials or network. Only `cargo deny check
advisories` fetches the RustSec database. Missing secrets block declared
integrations only — never the foundation gates.

### Diagnostics and observability

The CLI emits newline-delimited JSON diagnostics to stderr through Rust's
`tracing` ecosystem. Set `RUST_LOG` to control filtering (default `info`):

```sh
RUST_LOG=debug sea-forge run --intent "..."
```

Runtime diagnostics use stable event names and include `run_id`, `component`,
and `error_class`. They are distinct from the governed lifecycle events
persisted in `.sea-forge/` run directories.

The server exposes an event subscription stream for external monitoring
consumers and supports OpenTelemetry-compatible trace export.

### Specifications

The specifications are normative: they define the behavior the conformance
suite checks.

| Spec | Scope |
| ---- | ----- |
| [Shell-SPEC.md](.agents/specs/Shell-SPEC.md) | Development shell, toolchain, secrets, CI |
| [spec-minimum.md](.agents/specs/spec-minimum.md) | Minimum governed kernel (M-slice) |
| [spec-full.md](.agents/specs/spec-full.md) | Full system milestones M0–M8 |
| [spec-adlc-thoth-minimum.md](.agents/specs/spec-adlc-thoth-minimum.md) | ADLC/ODI lifecycle, Genesis self-model, Thoth (M9–M11) |
| [spec-agent-orchestration.md](.agents/specs/spec-agent-orchestration.md) | Agent delegation, orchestration topologies, ACP (M12–M16) |
| [ADR-001](docs/decisions/ADR-001-domainforge-semantic-boundary.md) | DomainForge semantic boundary |

See `ARCHITECTURE.md` for the system map and how the layers fit together.

### Contributing

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
`fix(scope): ...`, etc.) so Release Please can derive the next release. The
full happy path and recovery procedures are in
[`CONTRIBUTING.md`](CONTRIBUTING.md); the CI/CD architecture is in
[`docs/explanations-and-references/ci-cd-architecture.md`](docs/explanations-and-references/ci-cd-architecture.md).

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
  SEA Forge, provided they do not include SEA Forge itself,
  repository-generated materials, or enterprise-only components.
- **You may not without a commercial license:** resell or redistribute SEA
  Forge, offer it as a hosted or managed service, embed it in a paid product,
  white-label it, provide SEA-Forge-powered services to clients, or use
  enterprise-only components.

Enterprise-only files are identified by the rules in
[LICENSE_EE.md](LICENSE_EE.md). Third-party components remain subject to their
own licenses.

For production agents, client-facing deployments, or enterprise rights,
contact [licensing@godspeedai.com](mailto:licensing@godspeedai.com).
