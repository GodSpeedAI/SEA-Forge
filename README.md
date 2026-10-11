# SEA-Forge

### Put authority between agents and the systems they can change.

AI coding agents can already write files, run commands, call APIs, open pull requests, use credentials, and modify infrastructure.

That creates a new problem before it creates an enterprise architecture problem.

When an agent decides to run:

```text
rm file
git push
POST /deploy
merge pull request
read secret
call external model
```

where does permission actually get decided?

A prompt can tell the agent what it should do. Repository permissions can stop some operations. A sandbox can limit where code runs. Human review can catch mistakes afterward.

Those controls matter. They answer different questions.

SEA-Forge answers this one before the side effect:

> **Is this actor allowed to perform this operation on this resource under these conditions?**

**SEA-Forge is a runtime authority and evidence layer for autonomous systems. It evaluates proposed actions against declared domain meaning and policy before execution, returns allow, deny, or escalate decisions, and records evidence of what governed execution actually did.**

The basic path is:

```text
intent
  ↓
proposed operation
  ↓
authority decision
  ├── deny
  ├── escalate
  └── allow
        ↓
  bounded execution
        ↓
     evidence
        ↓
operational settlement
        ↓
 tamper-evident record
```

The agent can still choose what it wants to try.

It does not get to decide whether it has authority to do it.

---

## The gap appears as soon as agents get useful

Suppose you ask an agent:

> Update the deployment configuration, run the tests, and push the fix.

That sounds like one task.

Operationally, it may involve several different acts:

```text
read repository files
write configuration
execute a binary
access an environment variable
connect to a service
create a commit
push to a remote
```

Those acts do not necessarily deserve the same authority.

Maybe the agent may edit `deploy/staging/**` but not `deploy/prod/**`.

Maybe it may run `cargo test` but not arbitrary shell commands.

Maybe network access is normally denied.

Maybe a production deployment requires a human approval.

Maybe an agent may prepare a commit but never push it.

A prompt is a poor place to encode those distinctions because the same system interpreting the instruction is also choosing the action.

By the time a log tells you the wrong operation happened, the decision is already behind you.

SEA-Forge moves that decision in front of the side effect.

---

## Authority, containment, evidence, and acceptance are different controls

These are easy to collapse when everything lives inside one agent loop.

They should remain separate.

| Control                    | Question it answers                                                 |
| -------------------------- | ------------------------------------------------------------------- |
| **Prompt / task**          | What are we asking the agent to accomplish?                         |
| **Authority**              | May this actor perform this specific operation here?                |
| **Sandbox**                | Where and how may an allowed operation execute?                     |
| **Evidence**               | What actually happened?                                             |
| **Operational settlement** | Did the observed result satisfy the criteria declared for this run? |
| **Ledger**                 | Can we inspect and verify the record later?                         |

A container does not grant permission.

A successful process does not prove the requested artifact exists.

A log entry does not establish that an action should have happened.

An agent saying “done” is useful narration. It is not an independent acceptance criterion.

SEA-Forge keeps those judgments separate.

---

## Start with one operation you would rather not discover after the fact

You do not need to model an entire organization to test SEA-Forge.

Start with one action where the cost of a wrong decision is already obvious:

* an agent writing outside an approved directory
* a shell command that should require explicit permission
* an external API call
* access to a credential
* a Git commit or merge
* a deployment
* a delegated agent task

Give SEA-Forge a policy and submit a run:

```sh
sea-forge run \
  --policy sea-forge-policy.yaml \
  --intent "Generate a .sea model"
```

SEA-Forge records the run under `.sea-forge/`.

Inspect it:

```sh
sea-forge inspect <run_id>
```

If something was denied:

```sh
sea-forge ask ask_why_denied <run_id>
```

Re-check the evidence later:

```sh
sea-forge validate <run_id>
```

Search previous results:

```sh
sea-forge recall generate
sea-forge recall --result accepted --limit 10
```

The useful experiment is not “can SEA-Forge run an agent?”

It is:

> Can I put one consequential operation behind an independent authority decision and still understand exactly what happened afterward?

If that is valuable, expand from there.

---

## A run can finish without being accepted

Agent systems often collapse several states into one:

```text
command returned 0
        ↓
task succeeded
        ↓
agent can do this
```

Those claims are different.

SEA-Forge distinguishes execution from operational settlement.

A command may exit successfully while:

* the required artifact is missing
* the wrong file was changed
* a validator fails
* an expected hash does not match
* required evidence was never produced

SEA-Forge can reject that run even though the underlying process returned `0`.

The CLI exposes the resulting state directly:

| Exit code | Meaning                                   |
| --------: | ----------------------------------------- |
|       `0` | Settlement `accepted`                     |
|       `1` | Internal error                            |
|       `2` | Input or usage error                      |
|       `3` | Settlement `rejected`                     |
|       `4` | Settlement `escalated` or governed denial |
|       `5` | Awaiting approval / parked active         |

That distinction is intentionally narrow.

An accepted SEA-Forge run means the declared criteria for that governed run were satisfied by its evidence.

It does **not** mean the agent has acquired a durable capability. Repeated performance under variation, recovery, and developmental promotion are stronger claims handled elsewhere.

---

## Denial is a result, not a missing run

The runs that matter most during an incident are often the ones that traditional automation records least clearly.

SEA-Forge preserves denied, failed, cancelled, escalated, rejected, and accepted paths.

A governed run can leave records for:

```text
case
plan
authority decisions
execution trace
evidence
settlement
capability envelope
```

That makes questions such as these answerable later:

```text
What did the operator request?
Which actor attempted the operation?
Which resource was targeted?
Which policy applied?
Was the action allowed, denied, or escalated?
What actually executed?
What evidence was produced?
Why was the result accepted or rejected?
```

The record exists because the decision happened as part of the work, not because someone reconstructed the story afterward.

---

## Default deny makes missing knowledge visible

An authority system has to decide what happens when it cannot establish permission.

SEA-Forge defaults to denial.

An unknown operation does not inherit authority from a broad instruction such as:

```text
"Fix the repository."
```

A policy that cannot load does not quietly disappear.

Missing required authority evidence does not become an implicit allow.

Unresolvable policy conflicts do not get averaged into a best guess.

Depending on the condition, the operation is denied or escalated before the side effect.

That can feel stricter than an agent that simply tries things until something works.

That is the point.

A blocked operation tells you which permission, representation, or approval is missing. A silently permitted one removes that information along with the protection.

---

## Sandboxing answers a different question

SEA-Forge can execute allowed work inside bounded environments using controls including:

* Landlock on Linux
* Seatbelt on macOS
* safe-join path validation
* scoped workspaces
* explicit environment variables
* execution timeouts
* network deny by default
* agent cells with constrained resources and APIs

But sandboxing comes **after** authority.

Consider these two questions:

```text
May this agent write deploy/prod/config.yaml?

If allowed, what filesystem and network access
should the process receive while doing it?
```

The first is authority.

The second is containment.

Running a disallowed operation safely inside a sandbox would still be the wrong operation.

---

## Delegation does not erase the authority boundary

The accountability problem becomes harder when one agent delegates to another.

Without a shared authority boundary, it is easy to end up with:

```text
operator
  ↓
agent A
  ↓
agent B
  ↓
tool
  ↓
side effect
```

and no clean answer to which actor was allowed to do what.

SEA-Forge treats delegated agent work as governed work.

Delegated tasks can be:

* authority-checked
* turn-capped
* cancellable
* transcript-evidenced
* independently settled

The executor can change without changing the authority model.

That means an argv process, a sandboxed task, an OpenAI-compatible agent, an Anthropic-backed agent, an ACP-connected CLI agent, or another supported executor can operate behind the same governed lifecycle.

SEA-Forge governs the work path rather than trusting a particular model.

---

## Domain meaning can participate in authority

File paths and command names are useful policy surfaces.

They are not always enough.

A rule may depend on domain meaning:

```text
this actor is a Reviewer

this operation changes a ProductionDeployment

this resource belongs to Payments

this flow carries CustomerData

this action affects a regulated approval boundary
```

That is where DomainForge connects.

`.sea` is the **Semantic Executable Abstraction** source language used by DomainForge to represent the consequential structure of a purposeful system.

DomainForge compiles that source into a canonical semantic model.

```text
domain knowledge
      ↓
   .sea source
      ↓
  DomainForge
      ↓
canonical semantic model
      ↓
concept identities + validated meaning
```

SEA-Forge can then bind runtime decisions to that declared meaning.

```text
actor + operation + resource + context
              ↓
       canonical domain meaning
              +
           policy
              ↓
      allow / deny / escalate
```

The boundary is deliberate:

**DomainForge owns semantic compilation.**

It parses `.sea`, resolves concepts, validates the model, and computes deterministic projections.

**SEA-Forge owns runtime authority and governed side effects.**

It decides whether an operation may proceed, constrains execution, collects evidence, settles the run, and records what happened.

When a DomainForge projection is invoked inside a governed SEA-Forge run, DomainForge can compute the projection while SEA-Forge governs whether and where the resulting artifact may be materialized.

Meaning and authority remain separate.

---

## Policies cover more than file writes

SEA-Forge's authority fabric evaluates multiple first-class operational surfaces through one policy layer.

Current surfaces include:

```text
file
shell_cmd
external_api
git_commit
pr_merge
prompt_risk
memory_recall
spec_pipeline
artifact_transition
attestation
deployment
secret_access
policy_mutation
evidence_mutation
extension_install
projection_execute
identity_binding
self_disclosure
```

The useful property is not the length of that list.

It is that a run does not need one permission system for shell commands, another for provider calls, another for Git, and another for deployments while hoping their decisions remain coherent.

Policy resolution is deterministic:

```text
any deny
    ↓
deny

otherwise any escalate
    ↓
escalate

otherwise intersect applicable boundaries
    ↓
allow
```

Missing required evidence denies.

Policy conflicts that cannot be resolved become an explicit blocking condition rather than an invented permission.

---

## Evidence can be checked again later

A normal application log is usually optimized for debugging.

SEA-Forge's governed records have a stronger job: preserve enough information for later verification.

Evidence can include:

* execution results
* artifact descriptors
* SHA-256 content hashes
* structured trace records
* deterministic artifact identities

Past evidence can be revalidated:

```sh
sea-forge validate <run_id>
```

The ledger itself supports progressively stronger tamper evidence through:

```text
legacy_digest_only
      ↓
local_tamper_evident
      ↓
checkpoint_signed
      ↓
externally_verified
```

The implementation includes canonical encoding, hash chaining, Merkle Mountain Range construction, Ed25519-signed checkpoints, witness receipts, and inclusion / consistency proofs.

The purpose is not to make a database magically truthful.

It is to make silent alteration of the recorded evidence detectable at the assurance level being used.

---

## One governed lifecycle

Underneath the CLI, SEA-Forge carries a run through a structured lifecycle:

```text
Intent
  ↓
Case
  ↓
Plan
  ↓
Authority
  ↓
Sandbox
  ↓
Execute
  ↓
Evidence
  ↓
Settlement
  ↓
Record
```

### Intake

Operator intent becomes a typed `Case` with classified operations.

### Plan

The case is represented as stages, sentries, milestones, and plan items.

### Authority

Each consequential operation passes through the authority fabric before execution.

### Sandbox

Allowed work receives an execution environment constrained by the applicable policy.

### Execute

The selected executor performs the operation with explicit environment and timeout boundaries.

### Evidence

SEA-Forge records execution results, traces, hashes, artifacts, and other required evidence.

### Settlement

The evidence is evaluated against the criteria declared for the run.

### Record

The resulting governed history is appended to the integrity ledger and made available for inspection and recall.

This is deliberately more structured than:

```text
prompt
  ↓
model
  ↓
tools
  ↓
"done"
```

The extra structure exists because side effects create consequences that text generation does not.

---

## The boundary SEA-Forge cannot cross for you

SEA-Forge only governs operations routed through its authority and execution surfaces.

If somebody opens an unrelated shell and changes the same system outside SEA-Forge, SEA-Forge did not intercept that action.

Likewise, connecting a provider to SEA-Forge does not mean every possible side effect the provider can cause elsewhere is automatically mediated.

The security boundary is therefore concrete:

> Route the consequential paths you want governed through the authority layer.

Do not treat installation as universal interception.

That boundary is easier to reason about, test, and improve than a claim that every possible action is somehow under control.

---

## Where SEA-Forge earns its keep

### Coding agents

Allow agents to modify approved repository surfaces while treating sensitive paths, commands, commits, or merges differently.

### External API calls

Bind provider calls to explicit endpoints, destinations, models, limits, configuration hashes, and credential references.

### Infrastructure changes

Allow routine operations while escalating higher-risk deployments or environment changes for approval.

### Multi-agent work

Keep delegated tasks inside the same authority, evidence, cancellation, and settlement model.

### Generated systems

Govern projection and spec-to-code operations so generated artifacts retain evidence about the source and process that produced them.

### Auditable automation

Preserve the request, authority decision, execution, evidence, and operational settlement as part of the work rather than manufacturing an audit trail later.

The common property is simple:

> The action matters enough that finding out afterward is too late.

---

## Install and verify the repository

### Prerequisites

The host needs:

* [Git](https://git-scm.com/)
* [Devbox](https://www.jetify.com/devbox)
* [direnv](https://direnv.net/)

The remaining development tools are pinned by the repository, including the Rust toolchain, `just`, `sops`, `age`, `gitleaks`, and `cargo-deny`.

### Bootstrap

```sh
git clone <repo> sea-rs
cd sea-rs

devbox shell
direnv allow

just setup
just hooks-install
just doctor
just check
just test
```

A fresh clone is in a good state when:

```text
just doctor
just check
just test
```

all exit zero.

Run the full conformance suite with:

```sh
just proof
```

The proof suite is the executable reference for the guarantees implemented by the repository.

If the README and the proof suite disagree, trust the proof suite.

### Missing secrets

Core offline gates do not require provider credentials.

Without an age key, `direnv` leaves secret-backed integrations unavailable while local foundation checks continue to work.

Runtime records live under:

```text
.sea-forge/
```

Bootstrap evidence lives under:

```text
target/bootstrap-evidence/
```

Both are gitignored and intentionally separate.

---

## Server mode

For longer-running or concurrent work:

```sh
sea-forge server start
sea-forge submit --intent "Build the module"
sea-forge status <case_id>
sea-forge approve <run_id>
sea-forge subscribe
```

The daemon manages concurrent runs, approval workflows, event streaming, and policy reload between dispatches while using the same governed lifecycle as the CLI.

The default maximum concurrent run count is configurable and currently defaults to `4`.

---

## Agent delegation

Inspect and use configured agent endpoints:

```sh
sea-forge agent probe <endpoint>
sea-forge agent list
sea-forge run cancel <run_id>
```

SEA-Forge supports a provider seam with implementations for:

* OpenAI-compatible HTTP
* Anthropic HTTP
* ACP-connected CLI agents

Provider access itself is governed.

Exact grants can bind details such as:

```text
endpoint
configuration hash
destination
model
limits
credential reference
```

Credential material is resolved only after authority permits the operation.

There is no silent provider fallback. A failed endpoint remains a failed recorded endpoint rather than becoming permission to route somewhere else.

---

## Case management

SEA-Forge represents work as cases rather than assuming every task is a fixed linear workflow.

A `CasePlanModel` can contain:

* stages
* entry and exit sentries
* milestones
* sandboxed tasks
* agent tasks
* human approval tasks
* timer listeners
* user-event listeners
* discretionary plan items

That allows work to react to what actually happens.

A rejected result can reopen a stage. An approval can unblock a waiting task. A timer or external event can change what becomes available next.

Cases complete when their required conditions are satisfied rather than because the agent reached the bottom of a checklist.

---

## Self-knowledge without self-authorization

SEA-Forge includes a versioned `.sea` self-model and the Thoth self-knowledge protocol.

Inspect it with:

```sh
sea-forge self-model validate
sea-forge self-model rebuild --probe
sea-forge self-model show --json
```

Ask evidence-grounded questions:

```sh
sea-forge ask ask_capability sea-forge.settlement
sea-forge ask ask_environment_status sandbox.jail
sea-forge ask ask_why_denied <run_id>
```

The self-model separates three kinds of claim:

| View             | Meaning                                   |
| ---------------- | ----------------------------------------- |
| **Declared**     | What the system specification says exists |
| **Observed**     | What this installation currently exposes  |
| **Demonstrated** | What prior evidence supports              |

A declared capability is not automatically demonstrated.

More importantly, knowledge is not authority.

A Thoth answer can explain that a capability or environment surface exists. It cannot grant permission to use it.

---

## Spec-to-code without hiding the source chain

SEA-Forge includes a governed generation pipeline:

```text
ADR
 ↓
PRD
 ↓
SDS
 ↓
.sea
 ↓
DomainForge semantic model
 ↓
generated representations
 ↓
code generation
 ↓
last-mile implementation
```

Each stage can produce governed records and participate in deterministic replay.

Generated zones can reject hand edits so a change is made at the source or generator rather than silently diverging downstream.

The important distinction is the same one used throughout SEA-Forge:

```text
generation
≠
authority to materialize
≠
evidence of correctness
≠
acceptance
```

Each needs its own judgment.

---

<details>
<summary><strong>Agent orchestration and bounded manager loops</strong></summary>

SEA-Forge's orchestration layer governs delegation. It is not the model itself and does not turn the authority layer into an autonomous actor.

The `sea-forge-agent` crate provides the `AgentProvider` trait.

Built-in orchestration templates include sequential and concurrent agent arrangements. Templates become ordinary case plans, so the case engine remains the coordination substrate.

Thoth can also operate bounded management iterations:

```text
read case state
      ↓
propose discretionary work
      ↓
await governed execution and settlement
      ↓
evaluate
satisfied | progressing | stalled | blocked
```

The manager cannot settle or promote the work it proposed.

Iteration limits prevent indefinite autonomous loops; exhausted loops park and escalate.

</details>

<details>
<summary><strong>ADLC, outcome measurement, and lifecycle models</strong></summary>

SEA-Forge includes an Agentic Development Lifecycle model with four stages:

| Stage        | Typical work                       | Exit milestone             |
| ------------ | ---------------------------------- | -------------------------- |
| **Frame**    | preparation, hypothesis, scope     | `ProblemFramed`            |
| **Form**     | design, simulation                 | `DevelopmentAuthorized`    |
| **Build**    | implementation, evaluation         | `ReleaseCandidateAccepted` |
| **Activate** | controlled deployment, observation | `ActivationSettled`        |

Outcome-Driven Innovation extensions can bind settlement criteria to desired outcomes through DomainForge concept references.

These lifecycle models are built and proven milestone by milestone. Check `just proof` and the relevant normative specifications before treating a broader lifecycle capability as proven.

</details>

<details>
<summary><strong>Artifact identity and lineage</strong></summary>

The `sea-forge-artifact-ip` crate supports artifact records including:

* deterministic pre-mint identity using `ifl:hash:<sha256>`
* artifact type, stage, producer, owner, license, and review status
* lineage DAGs
* governed stage transitions
* attestation
* portfolio projections

These features track work products and their derivation.

They do not turn generated content into legally protected intellectual property by themselves; legal status remains external to the technical record.

</details>

---

## Repository layout

```text
.agents/specs/               # normative specifications
docs/decisions/              # architecture decision records

crates/
  sea-forge-core/            # kernel types, IDs, lifecycle, errors
  sea-forge-ledger/          # append-only record store + integrity proofs
  sea-forge-domain/          # intent vocabulary and domain primitives
  sea-forge-domainforge/     # DomainForge semantic adapter
  sea-forge-authority/       # policy engine, authority fabric, RBAC
  sea-forge-planner/         # cases, stages, sentries, templates
  sea-forge-sandbox/         # workspace isolation and jail backends
  sea-forge-runtime/         # process execution and timeouts
  sea-forge-trace/           # structured trace events
  sea-forge-evidence/        # evidence collection and verification
  sea-forge-settlement/      # operational settlement
  sea-forge-capability/      # capability envelopes and recall
  sea-forge-extension/       # extension and projection interfaces
  sea-forge-server/          # long-running daemon
  sea-forge-cli/             # CLI
  sea-forge-spec-pipeline/   # governed generation pipeline
  sea-forge-cell/            # cell isolation and federation
  sea-forge-artifact-ip/     # artifact identity and lineage
  sea-forge-self-model/      # system self-model

models/                      # bundled .sea models
justfile                     # canonical developer commands
devbox.json                  # pinned host tooling
rust-toolchain.toml          # pinned Rust toolchain
deny.toml                    # dependency policy
```

---

## Developer commands

Run `just` with no arguments to see the complete command surface.

Common recipes:

| Command                    | Purpose                                       |
| -------------------------- | --------------------------------------------- |
| `just setup`               | Converge on pinned dependencies               |
| `just doctor`              | Check the local environment                   |
| `just check-fast`          | Fast formatting + type checks                 |
| `just fmt`                 | Apply Rust formatting                         |
| `just lint`                | Run clippy with warnings denied               |
| `just typecheck`           | Check the workspace                           |
| `just security`            | Run dependency and secret checks              |
| `just test`                | Run workspace tests                           |
| `just build`               | Build workspace targets                       |
| `just check`               | Run the standard verification set             |
| `just ci`                  | Run the canonical CI verification             |
| `just proof`               | Run the conformance and milestone proof suite |
| `just release-check [tag]` | Check release version consistency             |
| `just clean`               | Remove generated build/evidence output        |

CI invokes the same command surface through Devbox.

The required pull-request check is:

```text
CI / gate
```

---

## Secrets

SEA-Forge uses SOPS and age for encrypted repository-managed secret profiles.

Tracked files under `secrets/` remain encrypted.

The age private key stays outside the repository.

```sh
just secrets-init
just secrets-edit dev
just secrets-check dev
```

If the key is unavailable, secret-backed integrations remain unavailable rather than causing the authority foundation to fail open.

Lost age private keys cannot be recovered. Generate and register a replacement, then re-encrypt the affected profiles.

---

## Offline work

The core repository is designed so that formatting, builds, Clippy, unit tests, license/bans/source checks, and secret scanning can run without provider credentials.

Some dependency advisory checks may require network access to refresh their databases.

Missing external credentials should block the integration that needs them, not unrelated local proof.

---

## Diagnostics

SEA-Forge emits structured diagnostics through Rust's `tracing` ecosystem.

Increase runtime verbosity with:

```sh
RUST_LOG=debug sea-forge run --intent "..."
```

Diagnostics include stable fields such as:

```text
run_id
component
error_class
```

These diagnostics are separate from governed lifecycle events stored as run evidence.

Server mode also exposes an event subscription stream and supports OpenTelemetry-compatible trace export.

---

## Specifications are stronger than this README

The normative behavior lives in the repository specifications and their proof gates.

Key references include:

* [`Shell-SPEC.md`](.agents/specs/Shell-SPEC.md)
* [`spec-minimum.md`](.agents/specs/spec-minimum.md)
* [`spec-full.md`](.agents/specs/spec-full.md)
* [`spec-adlc-thoth-minimum.md`](.agents/specs/spec-adlc-thoth-minimum.md)
* [`spec-agent-orchestration.md`](.agents/specs/spec-agent-orchestration.md)
* [`ADR-001`](docs/decisions/ADR-001-domainforge-semantic-boundary.md)
* [`ARCHITECTURE.md`](ARCHITECTURE.md)

Use:

```sh
just proof
```

to check the executable evidence behind implemented claims.

A README can become stale.

A passing proof tied to a frozen specification is harder to hand-wave.

---

## Contributing

Create a branch from current `main`, make the change, and exercise the same gates CI will use:

```sh
git switch main
git pull --ff-only
git switch -c feat/short-description

just check-fast

git add ...
git commit

just pre-push
git push -u origin HEAD

just pr
```

Pull-request titles use Conventional Commit form so Release Please can determine version changes.

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the complete workflow and recovery procedures.

---

## License and commercial use

SEA-Forge is source-available under the [SEA-Forge Sustainable Use License](LICENSE).

Permitted uses include the internal, personal, educational, research, evaluation, and non-commercial uses described by the license.

Independent outputs produced with SEA-Forge can be owned and commercialized when they do not include SEA-Forge itself, restricted repository-generated materials, or enterprise-only components.

A commercial license is required for restricted operational uses such as hosted, embedded, redistributed, white-labeled, client-facing, managed-service, or other uses covered by the commercial terms.

See:

* [LICENSE](LICENSE)
* [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)
* [LICENSE_EE.md](LICENSE_EE.md)

for the actual legal terms.

The practical distinction is simple:

> You can own what you create with the tool. The tool itself remains subject to its license.

For production agents, client-facing deployment rights, enterprise components, or commercial licensing questions, contact:

[licensing@godspeedai.com](mailto:licensing@godspeedai.com)
