# C — Target architecture

Seven diagrams, one per required view. Solid arrows are required at runtime; dashed arrows are
optional and their absence must not break the source side.

## C1. Component ownership and optional dependencies

```mermaid
graph TD
  subgraph Cognate["Cognate (kernel + runtime)"]
    RUN["runs · continuations · shared state · projections"]
    AUTH["authority + governance (Invoker/Governor)"]
    RLM["rlm.* capabilities<br/>agent.rlm · provider registry"]
    SPAWN["StdioSpawnPort<br/>(declared in runtime-api,
 implemented by the host app)"]
    RUN --> RLM
    RLM --> SPAWN
    AUTH --> RLM
  end
  subgraph WB["Cognate Workbench"]
    EXP["Experts · Teams · Assignments"]
    CASE["local case · world lineage · verification"]
    SETTLE["governed settlement client"]
    BUNDLES["capability bundles"]
    EXP --> RLM
    CASE --> RLM
    EXP --> SETTLE
  end
  PRIME["@godspeedai/cognate-rlm-prime<br/>(optional package)"]
  GNT["Gauntlet CLI<br/>(optional, frozen reference)"]
  HZ["Horizon<br/>(optional navigation)"]
  DF["DomainForge<br/>(optional semantics)"]
  SEA["SEA-Forge<br/>(optional authority + settlement)"]
  SXR["RealityTrace (sxr)<br/>(evidence ledger)"]

  SPAWN -.-> PRIME
  PRIME -.-> RLM
  NOTE["Governed provider route:<br/>contained streaming port (I3b)<br/>+ declared egress"]
  PRIME -.- NOTE
  EXP -.-> GNT
  HZ -.->|"expert.converse (Connect)"| EXP
  HZ -.->|"case verbs (NDJSON)"| SEA
  DF -.->|"CEP envelope (CLI)"| SEA
  SETTLE -.->|"authority_request / settle"| SEA
  SEA -.->|"evidence_packet"| SXR
```

**Rule.** Removing any dashed box leaves every solid path working. When no provider is
installed `rlm.providers` returns an empty array and Experts use their ordinary loop; there is no
`native` provider to mistake for one. No dashed box is a build
dependency of Cognate or Workbench except `cognate-rlm-prime`, which Workbench lists as an
*optional* dependency loaded only when an operator enables a provider.

## C2. Standalone Workbench execution (no optional services)

```mermaid
sequenceDiagram
  participant U as Operator
  participant W as Workbench
  participant C as Cognate runtime
  participant A as agent.expert
  participant M as model provider
  U->>W: open Team conversation, send objective
  W->>C: team.converse (validated command)
  C-->>W: Assignment queued (authority frozen, depth 0)
  W->>C: startRun(agent.expert, idempotency expert-run:...)
  C->>A: run admitted, principal wb-agent:product-manager
  A->>M: expert.model.generate (budget reserved first)
  M-->>A: text / tool calls
  A->>A: tools via expert.tool.invoke (commitment boundary)
  A-->>C: handoff (result + evidence + uncertainties)
  C-->>W: run completed
  W->>W: Assignment -> reconciliation-required
  U->>W: review evidence, accept
  W->>C: world.settle (human only) => settlementScope "local"
```

Verified substrate: `src/experts/bridge.ts:13-104`, `src/agents/agentCoding.ts:781-816`,
`src/capabilities/expertEffect.ts:1-45`, `src/capabilities/cognitive.ts:253-269`.
Nothing in this path consults Horizon, Gauntlet, SEA-Forge, DomainForge, RealityTrace or Prime.

## C4. Workbench → Gauntlet delegation (optional, explicit escalation)

```mermaid
sequenceDiagram
  participant O as Operator
  participant W as Workbench
  participant P as StdioSpawnPort (host)
  participant G as gauntlet run
  participant X as Gauntlet report
  O->>W: case.escalate (criteria already frozen)
  W->>W: project reference: .wayfinder/reference.yaml + answer-key.md
  W->>W: criterion.freeze / world.freeze => frozen hashes
  W->>P: spawn intent recorded (argv, cwd, posture, version)
  P->>G: gauntlet run --config <project config>
  G->>X: exit code 0/1/2 + stdout report (six StatusCategory buckets)
  X-->>W: archive stdout as reportRef; read terminal_state
  W->>W: record escalation, gauntletTerminal = terminal_state
  Note over W: escalation never writes world.settled and never a receipt
  O->>W: local acceptance still a separate human act
```

Gate criteria are an explicit list, not a score: irreversible effect, need for an independent
verifier, need for frozen criteria, need for recovery across restarts, coordination beyond one
Team's depth limit, or an operator's explicit request. Two preconditions are refused rather than
assumed: the operator must name Gauntlet's `runners.default`, and the host posture must admit a
host-process spawn (so governance mode refuses). Each is visible in the escalation record
with the value that triggered it.

## C5. Horizon → Workbench / RLM handoff

```mermaid
sequenceDiagram
  participant H as Horizon desk
  participant K as SEA-Forge case keeper
  participant W as Workbench
  participant R as rlm.*
  H->>H: draft -> handoff item (world_ref, semantic_refs, acceptance, uncertainties)
  H->>K: case_preflight (dry run, digest pin)
  K-->>H: preconditions accepted
  H->>K: case_add_item / advance
  H->>W: expert.converse {caseContext, assertedBy, workspaceId, message}
  W->>W: validate: worldRef present when evaluated; record worldCorroboration
  W->>R: runner from Team config: none, or a provider via rlm.invoke
  R-->>W: handoff + evidence
  W-->>H: attribute to caseId + planItemId (read from Workbench projections)
  H->>K: human_task_complete (evidence refs)
```

Horizon keeps one Workbench call (`expert.converse`) and one SEA-Forge case transport. It does
not gain an RLM client, an agent runtime, or a second case store.

## C6. CEP authority, evidence and settlement lifecycle

```mermaid
stateDiagram-v2
  [*] --> Represented
  Represented --> Authorized: authority_request -> decision
  Authorized --> Attempted: run starts (CaseContext frozen)
  Attempted --> Observed: tool/run events
  Observed --> Evidenced: RealityTrace ledger rows
  Evidenced --> SettledGoverned: SEA-Forge accepts evidence_packet
  Evidenced --> LocallyAccepted: human world.settle (settlementScope=local)
  Attempted --> Unknown: transport/tool lost
  Unknown --> Attempted: reconciled, evidence recorded
  Unknown --> Failed: reconciliation proves failure
  SettledGoverned --> [*]
  LocallyAccepted --> [*]
```

Distinctions preserved: `represented ≠ authorized ≠ attempted ≠ observed ≠ evidenced ≠ settled`.
`LocallyAccepted` is terminal for a local case only and carries `receiptRef: null`;
`SettledGoverned` carries SEA-Forge's receipt id and its own `final` flag.

## C7. Interrupted external execution and recovery

```mermaid
sequenceDiagram
  participant W as Workbench assignment
  participant C as Cognate run
  participant P as provider process
  participant L as ledger (events)
  W->>C: run admitted (attempt n)
  C->>P: invoke (durable intent recorded first)
  Note over P: process dies / host restarts
  C->>L: read: intent without terminal record
  C->>C: status = unknown (never failed, never success)
  C->>W: Assignment -> reconciliation-required
  W->>W: boot sweep finds the unfinished assignment
  W->>P: re-probe result contract / result file (no re-run)
  alt result contract proves completion
    P-->>W: terminal result + evidence
    W->>C: run.invocation.reconcile(mark-completed, evidence)
  else result contract proves nothing ran
    W->>C: run.invocation.reconcile(reexecute) then a NEW attempt + new invocationId
  else indeterminate
    W->>W: keep reconciliation-required for the operator
  end
```

The rule is Gauntlet's, adopted rather than invented: "the invocation's fate may be unknown —
recovery re-probes result contracts, never assumes" (`agent_runner.rs:46-48`).


## C3. Team Orchestrator and recursive Specialist execution

```mermaid
graph LR
  PM["Product Manager<br/>(Orchestrator, depth 0)"]
  B["Builder<br/>(Specialist, depth 1)"]
  V["Verifier<br/>(Specialist, depth 1)"]
  R["Reviewer<br/>(Specialist, depth 1)"]
  PM -->|"team.delegate (durable assignment)"| B
  PM -->|"team.delegate"| V
  PM -->|"team.delegate (reviews=B)"| R
  B -->|"rlm.invoke (runner=prime)<br/>same Assignment, same evidence line"| RC["RLM child run<br/>opaque provider-internal children"]
  V -->|"read-only, no builder context"| E["independent observed checks"]
  R -->|"accept requires independent checks"| S["human settlement"]
```

Enforced, not prompt-level:
* exactly one Orchestrator per active Team (`src/experts/state.ts:39-47`);
* `team.delegate` only from a depth-0 Orchestrator in a shared Team conversation
  (`src/experts/service.ts:424-430`);
* authority = intersection, `team_delegate` removed below the Orchestrator (`:195-199`);
* child depth bounded by Team `maxDepth` (`:190`);
* review acceptance needs an independent Expert's observed checks (`:316-325`);
* an RLM child is a *run of the same Assignment*, so the lineage is
  `Assignment → run → provider invocation`; the provider cannot mint identities.
