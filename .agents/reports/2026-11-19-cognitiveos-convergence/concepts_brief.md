# GodSpeed CognitiveOS Convergence

* **Author:** EGM design agent, for the GodSpeed AI owner
* **Date:** 2026-11-19
* **Mode:** BRAINSTORM (concept brief preceding `/egm-draft`)

## Overview

Seven GodSpeed products already solve different parts of one problem: given an uncertain
situation, decide what can actually be done now, do one affordable and authorized thing,
observe what happened, and let that evidence change the next move. Today they do not share
the mechanisms that would let one product hand meaning to another without losing authority,
provenance, or the difference between "finished" and "settled".

The concept under review is narrower than "unify the ecosystem". It is: **make Cognate the
home of one recursive-execution capability, expressed as an ordinary Cognate run, and make
Workbench's existing Experts, Teams and Assignments the only agent hierarchy that uses it.**
Prime Agent becomes an optional provider adapter. Gauntlet stays an optional controller for
work that needs a frozen reference and independent verification. Horizon keeps navigating
cases and calls Workbench through the seam it already uses.

Target user: a developer or forward-deployed engineer who installs Cognate Workbench alone,
gets a working Expert/Team harness, and later connects Horizon, Gauntlet, DomainForge or
SEA-Forge without reinstalling anything or learning what an RLM is.

## Analysis

The strongest finding is subtractive. Read against the actual source, most of the required
behaviour already exists; what is missing is one execution seam and a handful of honest
labels. The critique below is grouped by what it changes.

**What the code already gives us (so it must not be rebuilt).** An active Team provably has
exactly one Orchestrator, enforced in the versioned mutation path, not in a prompt
(`cognate-workbench/src/experts/state.ts:39-47`). Delegation already narrows authority by
intersection and removes `team_delegate` from Specialists
(`src/experts/service.ts:181-216`); children inherit the parent's tool authority snapshot
(`:424-430`). A delegated child is a durable Assignment, the parent suspends on a native
continuation, and completion is reconciled on boot
(`src/agents/agentCoding.ts:781-816`, `src/experts/bridge.ts:13-104`). Model spend is
reserved before dispatch, unknown prices stay unknown, and a crash blocks the same logical
call instead of repeating it (`src/experts/billing.ts:1-9`). Consequential tool effects go
through a commitment boundary that fails into `reconciliation-required`
(`src/capabilities/expertEffect.ts:1-45`). Local acceptance and SEA-Forge settlement are
already two different event families with different producers
(`src/capabilities/cognitive.ts:253-269`, `src/settlement/orchestrator.ts:22-48`). Horizon
already reaches Workbench over Connect as a generic capability call
(`Horizon/src/adapters/workshop.ts:174`), and WP-23 already binds a run to a case assignment
(`cognate-workbench@feat/wp23-assignment-binding (668f686):src/assignments/assignmentBinding.ts:38-42`).

**What is genuinely absent.** There is no RLM, recursion, or context-decomposition contract
anywhere in Cognate, CEP, DomainForge, SEA-Forge or RealityTrace; the searches are recorded
in the recon reports. Cognate's execution port is buffered — it returns after exit, with no
stdin and no streaming (`cognate/packages/execution/src/types.ts:64-66`), so it cannot host
a live RPC harness. Workbench's governed-spawn wrapper for long-lived stdio processes exists
only on an unmerged branch (`cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/capabilities/execSpawn.ts:1-20`). There is no

**Where the assignment's premises are contradicted.** Three matter.

1. "Workbench's harness refuses governed sessions" is true and is one line
   (`src/capabilities/harness.ts:89`), but the refusal is not a property of external CLIs in
   general — it is a property of a CLI whose tool calls are not intercepted. Gauntlet's Prime
   adapter already demonstrates the interception pattern (a generated per-invocation MCP
   bridge with a fixed caller and a depth binding,
   `gauntlet/crates/adapters/gauntlet-adapter-agent-prime/src/mcp_bridge.rs:1-30`). The
   refusal should therefore become conditional on an attested tool policy, not deleted and
   not extended to the new path by default.
2. "Safe-mode machinery might restore the Expert baseline" is false on `main`. Safe mode
   exists only on `feat/capability-runtime-t4a` (`bbe6c73`), and even there it exists to
   exclude faulty *extensions* from mounting (`cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/workbench/safeMode.ts:1-7`).
   It never re-seeds data. Protecting the baseline needs a small restore command, not safe
   mode.
3. "Reuse Gauntlet's Prime integration" has a hidden fork: Gauntlet contains two independent
   Prime integrations — the Rust adapter (`.../runner.rs:639` over `AgentRunner`) and a
   TypeScript host (`gauntlet/workbench/src/real-prime-host.ts:1-40`). Extracting the
   TypeScript one is the cheap route for Cognate; bridging the Rust one would require a new
   sidecar protocol and adds a second process boundary for no behavioural gain.

**What the plan costs.** One new optional package (the Prime adapter), one new contract module
inside an existing package (the runner port), one extraction of a mechanism Workbench already
has on a branch, and three small honesty fixes. The expected net code footprint is close to
zero or negative once the duplicated Prime host and the legacy plugin registry are removed.

**Where it can fail.** If the installed Cognate versions are not aligned first, none of the
rest is observable: both consumers currently have 0.8.0 in `node_modules` while their
manifests ask for 0.9.2 / 0.10.0. If Prime cannot be launched with only bridged tools, the
governed path must refuse, and the product falls back to the ordinary Expert loop — which must be
labelled as an ordinary loop, not sold as context decomposition.

## Goals vs. Non-Goals

**In-Scope Goals**

* One provider-independent RLM port in Cognate, delivered as ordinary Cognate runs with
  durable correlation, cancellation, budgets, evidence and recovery.
* Workbench Experts and Teams consume it transparently; exactly one Orchestrator per active
  Team remains an enforced invariant; standalone Workbench is unchanged when no provider is
  installed.
* A recoverable built-in Expert/Team baseline under explicit operator control.
* Gauntlet remains optional and is reached through one governed-spawn seam with a frozen
  reference.
* Horizon reuses `expert.converse` and carries case/world identity on the request instead of
  a natural-language summary.
* Local acceptance and externally governed settlement stay distinguishable in events,
  projections and downstream observations.
* A deletion/consolidation plan for duplicated registries, harnesses and Prime protocol code.

**Out-of-Scope Non-Goals**

* No new agent framework, orchestrator service, case engine, or universal envelope schema.
* No DomainForge schema change, no CEP profile addition, no SEA-Forge authority change.
* No mandatory Prime Agent, Gauntlet, Horizon or SEA-Forge dependency for Workbench.
* No Windows/macOS containment implementation in this change set; unsupported containment
  must refuse, not silently run uncontained.
* No backwards-compatibility work for superseded internal interfaces.
* No implementation in this phase.

## Alternatives Weighed

| Alternative | Why it was rejected |
|---|---|
| Put the RLM controller in Workbench | Creates the second hierarchy the assignment forbids; Horizon and Gauntlet would each need their own copy; Workbench would stop being a harness and become a platform. |
| Put the RLM controller in Gauntlet and call it from Workbench | Makes Gauntlet mandatory for ordinary recursive work and inverts the dependency the owner wants (Gauntlet optional). |
| A new `rlm-service` daemon with its own scheduler, store and identity | Duplicates Cognate runs, continuations, billing and authority; four new failure modes for one feature. |
| Bridge the Rust `AgentRunner` across an FFI/sidecar boundary | Needs a new wire protocol plus packaging per platform; the same protocol already exists in TypeScript in Gauntlet's workbench. |
| A new universal "cognitive envelope" covering case, authority, evidence and settlement | CEP already defines seven profiles at 1.0.0; a new one would be a second source of semantic truth. |
| Replace Workbench's Expert/Team assignment model with the PRIME subagent model | Loses one-Orchestrator enforcement, authority narrowing, budgets, review independence and durable returns that are already implemented and tested. |
| Implement RLM inside `agent.expert` with no shared contract | Every consumer (Horizon-triggered runs, Gauntlet, future agents) would re-implement provider discovery, cancellation and recovery. |
| Make the default Expert set immutable | The owner wants customization; recoverability is sufficient and cheaper. |
| Route every Workbench task through Gauntlet | Conflates complexity with duration and makes the common case heavy. |

restore path for the five built-in Experts: `seedExperts` short-circuits on a permanent
receipt (`src/experts/state.ts:57-76`). And Workbench's local `worldHash` is its own manifest
hash, not a CEP `world_ref`; nothing in Workbench emits or consumes a CEP profile today.


## Key Decisions & Staging for Next Steps

1. **RLM is a Cognate capability expressed as an ordinary run**, selected by a runner port;
   the port contract lives in `@godspeedai/cognate-runtime-api`, the child-run agent and registry in
   `runtime-bun`, and the Prime provider in an optional package. There is no built-in "native"
   provider; absence is reported as absence. (Decision 1–3.)
2. **Workbench's Assignment is the only recursion identity.** A runner executes an
   Assignment; it never creates a parallel hierarchy. Provider-internal subagents are opaque,
   attributed to the Specialist's Assignment, and cannot exceed the process's granted tools.
   (Decision 5–6.)
3. **The long-lived stdio spawn capability is extracted, not invented.** Workbench's T3/T4a
   `execSpawn` plus its MCP `StdioChannel` are the source; the RLM adapter and Workbench's MCP
   client become its two consumers. (Decision 4, 12.)
4. **Gauntlet is reached by governed spawn of its CLI with a frozen reference**, because
   `gauntlet run` is the controller and `fabric serve` deliberately does not freeze or settle.
   (Decision 8–9.)
5. **Horizon keeps `expert.converse` and adds case context**, reusing Cognate's existing
   `CaseContext` and WP-23's assignment binding. (Decision 10.)
6. **Baseline protection is an operator-only restore command**, not immutability and not safe
   mode. (Decision 7.)
7. **Settlement vocabulary is fixed at the label level**: local acceptance is tagged local;
   governed settlement keeps SEA-Forge's status and `final`. (Decision 11.)
8. **First slice is Cognate-only**: the contract plus a test provider, proven by tests
   that need no Workbench, no Prime and no Gauntlet. (Staging, `F`.)

Carried forward to `/egm-draft`: the eight decisions above, the three premise corrections, and
the four measurements the design must state as unverified (Prime tool policy, CEP profile
coverage in Workbench, installed-version skew, Gauntlet `fde/integration` state).
