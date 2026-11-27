# D — Minimal executable contracts (revision 4, port governance)

Revision 2 superseded revision 1 after the design-critic review. Revision 4 adds **port governance**
as a first-class admission route (D8) after the owner observed that a provider can be governed by
governing the port rather than its tool list. Verified mechanically on this host: bubblewrap 0.13.1
holds a live stdio process under `--unshare-all` (`PORT_OK`/`GOT:hello`), denies the network
(`Network is unreachable`), and can bind a host unix socket into the sandbox and reach it
(`PROXY_OK`). Changes are marked **[R2]** or **[R4]**.

## D0. What changed in revision 2 and why

| Rev 1 claim | Rev 2 correction | Reason |
|---|---|---|
| a `native` provider = the in-process Expert loop | **no `native` provider**; `runner` is `none` or `provider(id)` | The Expert loop lives in Workbench, not Cognate; a Workbench Assignment is bound to one run (`src/experts/service.ts:119-122`); Cognate has no `agent.expert` |
| `rlm.invoke` "starts a child run of an existing Assignment" | starts a **child run `agent.rlm` for** the Assignment's execution; the Assignment's `runId` stays the parent | One-run-per-assignment is enforced at both layers (`service.ts:119-122`, `cognate/packages/runtime-bun/src/service.ts:145`) |
| "authority intersection at the kernel" | authority/depth/budget are **caller-attested and re-checked at the seam**; the real grant is Workbench's kernel-enforced one | Cognate has no tool-grant, depth or USD reservation vocabulary (review B3) |
| `process.spawn.durable` extracted into Cognate with its stores | `StdioSpawnPort` **declared** in `runtime-api`, **implemented by the host app** | The spawn wrapper needs six app-owned dependencies (review M7) |
| providers admitted only with a tool-list attestation | **[R4]** two admission routes: a **governed port** (reach matrix + egress + evidence) or a **tool-list restriction** | Governing the port enforces the authority property directly; the tool list is the fallback for a provider that cannot run in a governed port |
| `requestedBy` mismatch refused | **`assertedBy`**, recorded, never a refusal | Horizon always presents the operator's own session (review M4) |
| `caseContext` trusted | `worldCorroboration: "verified" | "asserted"` recorded | CEP identity must not be pretended (review M5) |

## D1. Public capabilities (Cognate; callable through the normal Invoker)

### `rlm.providers` (read)
```ts
input:  {}
output: {
  providers: Array<{
    id: string;                     // operator-configured; no vendor name in core
    label: string;
    capabilities: {
      maxDepth: number;
      contextDecomposition: boolean;
      providerInternalSubagents: boolean;
      streaming: boolean;
      isolation: "none" | "process" | "container" | "vm";   // [R2] adopted from Gauntlet
      resultDelivery: "file" | "mailbox" | "return";        // [R2] adopted from Gauntlet
      cancellation: "process" | "none";
      paymentMetering: "reported" | "unreported";
    };
    toolPolicy: "bridge-only" | "contained" | "unattested";
    portGovernance?: PortGovernance;   // [R4] see D8; the second admission route
    // `bridge-only` requires positive evidence from the provider build (a CLI flag or a
    // settings key that removes built-in tools). For Prime 0.10.2 there is none: no
    // `--tools`/`--no-builtin-tools`/`--exclude-tools` in help, no `defaultTools` in settings,
    // zero hits in the binary. Its adapter declares `unattested` and is refused wherever a
    // run is governed or its authority is narrower than the operator's full grant.
    executable: { version: string; sha256: string } | null;
    status: "available" | "unavailable";
    reason?: string;                // required when unavailable, from a closed set
  }>;
  admission: {                      // [R2] the honest table, not prose
    governance: "on" | "off";
    posture: "contained" | "uncontained-local";
    externalProvider: "admitted" | "refused";
    refusalCode?: string;
  };
}
```
Provider registration is **configuration**, not code: an operator adds
`{id, label, executable, args, versionRange}` to the Workbench settings file and enables the
adapter package for it. The registration surface is public; the adapter internals are not (D7).
When nothing is registered the array is empty. **There is no built-in provider entry**: absence is
reported as absence, not as a weaker provider.



### `rlm.invoke` (write; one child run per invocation)
```ts
input: {
  invocationId: string;             // stable across retries
  providerId: string;
  parent: {                         // [R2] caller-attested lineage, recorded as declared
    runId: string;                  // must equal ctx.runId
    assignmentId: string | null;    // Workbench Assignment, when the caller has one
    attempt: number;
    depth: number;                  // parent depth; child is depth+1
    maxDepth: number;               // governing limit (Team maxDepth, or 1 for solo)
    authorityTools: string[];       // the parent's frozen grant, as the caller knows it
  };
  task: { purpose: string; acceptance: string[]; contextRefs: string[]; contextDigest: string };
  authority: { tools: string[]; mode: "read-only" | "workspace-write" };
  budget: { currency: "time"|"attention"|"money"|"risk"|"recovery";
            limit: number | "unknown"; reservationRef?: string };   // [R2]
  // [R2] a numeric `limit` without `reservationRef` is refused (the seam cannot consult it).
  caseContext?: {
    // [R2] mirrors the published CaseContext exactly; no new field is added to it.
    worldRef: string; semanticRef: string; caseId: string; planItemId: string;
    role?: string;                                  // [R2] optional: Workbench has no SOP roles
    kind: "evaluated" | "exploratory";
    episodeRef?: string; assignmentId?: string; expectationRefs?: readonly string[];
  };
  // `worldCorroboration` is NOT part of CaseContext (a published type). It is recorded in the
  // run's evidence as `worldCorroboration: "verified" | "asserted"` by the caller.
}
output: { invocationId: string; runId: string; status: "admitted" }
```
Rules: `parent.runId` must equal the handler's `ctx.runId`, so a top-level invocation is
impossible. `parent.authorityTools` is intersected, never trusted as a ceiling by itself — the
caller owns the real grant and this seam re-checks only that the request is a subset. `budget`
uses Cognate's `Currency` vocabulary (`cognate/packages/runtime-api/src/assignment.ts:19-27`),
which removes the three-currency ambiguity in revision 1.

### `rlm.inspect` (read; terminal-stable)
```ts
input:  { invocationId: string }
output: {
  status: "admitted" | "running" | "completed" | "failed" | "timedOut" | "cancelled" | "unknown";
  resultRef?: string;
  evidence: string[];
  payment: { currency: string; spent: number | "unknown"; basis: string };
  providerId: string;
  isolation: "none" | "process" | "container" | "vm";   // what was actually used
  lastEventAt: string;
}
```

### `rlm.cancel` (write; idempotent)
```ts
input:  { invocationId: string; reason: string }
output: { status: "cancelled" | "already-terminal" }
```

### No `rlm.reconcile` [R2]
Revision 1 invented one. Cognate already has the mechanism: `run.invocation.ambiguous` + an
`ambiguous-invocation` continuation resolved by `run.invocation.reconcile`
(`src/capabilities/ambiguousStep.ts:6-9`). `rlm` reuses it: an `unknown` invocation opens that
continuation, and the operator resolves it with the existing capability. Reuse removes a duplicate
authority surface and inherits the existing operator guard.

## D2. Internal provider port (never a cross-product contract)

```ts
interface ProviderFacts {
  capabilities: { maxDepth: number; contextDecomposition: boolean; providerInternalSubagents: boolean;
                  streaming: boolean; isolation: "none"|"process"|"container"|"vm";
                  resultDelivery: "file"|"mailbox"|"return"; cancellation: "process"|"none";
                  paymentMetering: "reported"|"unreported" };
  toolPolicy: "bridge-only"|"contained"|"unattested";
  executable: { version: string; sha256: string } | null;
}

interface RlmProvider {
  readonly id: string;
  probe(): Promise<ProviderFacts>;            // capabilities + executable + toolPolicy + isolation
  invoke(req: InvokeRequest, spawn: StdioSpawnPort, ctx: RunContext): Promise<{ runId: string }>;
  inspect(runId: string): Promise<InvocationView>;
  cancel(runId: string, reason: string): Promise<void>;
  reconcile?(runId: string): Promise<InvocationOutcome | "indeterminate">;
}
```

**[R2] `StdioSpawnPort` is declared in `runtime-api` and implemented by the host application**,
not by Cognate:

```ts
interface StdioSpawnPort {
  spawn(spec: { argv: string[]; cwd: string; env: Readonly<Record<string,string>>;
                restrictions: { filesystem?: string[]; network?: {outbound: string[]};
                               caps?: {memoryBytes?: number; cpuSeconds?: number};
                               subprocess?: boolean } },
        ctx: { intentId: string; runId: string }): Promise<{ id: string }>;
  write(id: string, line: string): Promise<void>;
  readLines(id: string, sink: (line: string) => void): Promise<void>;
  kill(id: string, reason: string): Promise<void>;
}
```
Workbench implements it over its existing governed spawn wrapper
(`cognate-workbench@feat/capability-runtime-t4a:src/capabilities/execSpawn.ts`); a standalone
Cognate host may supply none, in which case every external provider reports `unavailable`.

Provider-specific flags, env names, file names, framing and version ranges live only in the
provider's own package. `runtime-api` must not mention any of them; a test asserts the token
`prime` does not appear in `packages/runtime-api/src` or in any capability id, and that the
capability-fact vocabulary (`isolation`, `resultDelivery`, statuses) matches Gauntlet's port
(`crates/gauntlet-ports/src/agent_runner.rs:162-229`) so the two cannot drift.

## D3. Workbench-facing extensions

| Capability | Change | Refusal conditions |
|---|---|---|
| `expert.converse`, `team.converse` | `runner` frozen from Team/Expert config into `ExpertExecution`; optional `caseContext`; optional `assertedBy` (recorded, never refused) | `caseContext.kind === "evaluated"` without `episodeRef` (the published type's own rule, `types.ts:28-30`); runner `provider` unavailable |
| `team.delegate` | unchanged; the child inherits `runner` and the same Team | depth > Team `maxDepth`; caller not the depth-0 Orchestrator |
| `experts.restore` | new, operator-only; `{ includeArchived?: boolean }` → `{ created: string[]; reactivated: string[]; kept: string[] }` | called from inside a run; non-operator actor |
| `case.escalate` | new; `{ caseId, assignmentId, criteria: string[], runner: string, budget }` → `{ escalationId, exitCode, stdoutRef, stderrRef, gauntletTerminal }` | no frozen criterion; posture does not admit a host spawn; operator has not named a Gauntlet runner |
| `affordance.field` | new read; `{ caseId }` → `AffordanceField` | unknown case; no frozen criterion when an evaluated move is requested |

**`experts.restore` semantics [R2]**, stated so the two clauses cannot contradict:
* creates baseline ids that are **absent**;
* with `includeArchived:true`, reactivates baseline ids the operator archived (this deliberately
  reverses an operator choice, so it is never automatic);
* never modifies an active baseline id and never touches any custom id;
* returns which ids were created, reactivated and left alone.

**`case.escalate` result [R2]** is Gauntlet's actual surface, not invented words:
`gauntlet run --config <path> <intent>` — the `<intent>` positional is required (absence exits 2).
Exit code is authoritative and binary: 0 only for `settled`, 1 otherwise, 2 for usage. The six
terminal states are `settled | blocked | budget_exhausted | escalated | aborted | inconclusive`
(`crates/gauntlet-domain/src/values.rs:266-268`), reported in the stdout interleave, not as a JSON
field. Workbench archives stdout as `stdoutRef` and stderr as `stderrRef`, and sets
`gauntletTerminal` from the parsed report; a refusal that never produced a report is surfaced from
`stderrRef`. Workbench never calls that outcome "settled" in its own vocabulary — the UI says
which system settled what.


## D4. Lifecycle and identity rules (normative)

1. **One invocation, one child run.** `rlm.invoke` creates exactly one run of agent `agent.rlm`
   with `idempotencyKey = "rlm:" + invocationId` and
   `correlationId = "rlm:" + parent.runId + ":" + invocationId`. The parent Assignment's `runId`
   does not change.
2. **No new identity.** The child run's principal is derived from the parent run's starter, the
   way `subagent.start` already derives it (`src/capabilities/subagentStart.ts:43-57`). A
   provider never supplies a principal. The `invocationId → runId` mapping is written to the
   **parent run state** with the existing `updateState`, so `rlm.inspect` survives restart by
   reading that mapping and then the child run; no new store is introduced.
3. **Authority is caller-attested and re-checked.** `authority.tools` must be a **subset** of
   `parent.authorityTools`; a request outside it is refused (`rlm-authority-exceeds-parent`), not
   silently narrowed, so a caller learns its grant is wrong. The recorded set is the intersection.
   The
   *real* grant is Workbench's, enforced at `authorizeCapabilityOp`
   (`src/workbench/authoritySelection.ts:86-129`); a caller that bypasses Workbench policy is out
   of scope and named as such in the NFRs.
4. **Depth is caller-attested and bounded.** `depth+1 > maxDepth` refuses. Cognate does not store
   depth; the Workbench Assignment and Team do (`src/experts/service.ts:190`).
5. **Budget.** `limit:"unknown"` admits and records `spent:"unknown"`. A numeric `limit`
   requires `reservationRef`; without it the call is refused with `rlm-budget-unverifiable` (the
   seam cannot consult a reservation it does not have). Workbench passes its existing reservation
   (`src/experts/billing.ts:1-9`).
6. **Evidence.** Every terminal outcome records: run id, provider id and executable digest,
   context digest, result ref, isolation actually used, and payment basis.
7. **Terminal stability.** `completed | failed | timedOut | cancelled` never change; `unknown` is
   not terminal. An `unknown` invocation is surfaced through the **existing** ambiguous-invocation
   machinery (`run.invocation.ambiguous` + an `ambiguous-invocation` continuation, resolved by
   `run.invocation.reconcile` — `src/capabilities/ambiguousStep.ts:6-9`), and a retry is refused
   until that continuation is resolved. Cognate gains no second reconciliation surface.
8. **No fallback.** An unavailable provider fails with a typed code. Workbench may *offer* to
   clear the runner, but never switches on its own. A retry after a terminal failure is a **new**
   invocation: a new `invocationId` derived with the attempt number, therefore a new idempotency
   key. Reusing an `invocationId` returns the existing run and never starts a second one.
9. **[R4] Admission under governance is a port property.** A provider is admitted under
   governance only if its declared `portGovernance` covers the parent's grant and the egress
   channel (D8). A provider that can run only on a `local`, uncontained world is refused with
   `rlm-posture-refused`.

   Revision 2 refused every provider under governance because the *current* Workbench wrapper
   holds stdio only on a `local` world under an `uncontained-local` posture
   (`cognate-workbench@feat/capability-runtime-t4a:src/capabilities/execSpawn.ts:195-196`) and
   governance mandates `contained` (`src/server.ts:510-511`). That is a limitation of the wrapper,
   not of the mechanism: bubblewrap 0.13.1 on this host holds a live stdio process under
   `--unshare-all`, denies the network, and accepts a bind-mounted unix socket (D8 evidence). The
   missing piece is a contained **streaming** world, delivered by I3b.
10. **Context integrity.** `contextDigest` is sha256 over the canonical *resolved* artifacts, not
    over their ids, and is re-computed at child start; a mismatch refuses.

## D5. Failure codes

| Code | Meaning |
|---|---|
| `rlm-provider-unknown` | `providerId` not listed |
| `rlm-provider-unavailable` | listed but binary, version or posture unavailable |
| `rlm-posture-refused` | governance or posture forbids an external provider on this host |
| `rlm-authority-exceeds-parent` | requested tools are not a subset of `parent.authorityTools` |
| `rlm-depth-exceeded` | `parent.depth + 1 > parent.maxDepth` |
| `rlm-budget-unverifiable` | a numeric limit was supplied without a reservation the seam can consult |
| `rlm-containment-unavailable` | required restrictions cannot be enforced in this world |
| `rlm-port-unavailable` | no contained streaming world is registered; this provider needs a governed port |
| `rlm-egress-uncontrolled` | the provider needs a model endpoint and no host-mediated or proxied egress channel is declared |
| `rlm-context-digest-mismatch` | resolved context does not hash to `contextDigest` |
| `rlm-invocation-unknown` | unknown `invocationId` |
| `rlm-invocation-ambiguous` | invocation is `unknown`; an `ambiguous-invocation` continuation is open and retry is refused until resolved |
| `rlm-top-level-invocation` | `parent.runId` does not equal the handler's run |

## D6. Compatibility rules

* **CEP**: no profile is added or changed. Crossings use the seven existing `godspeed.*` profiles
  at `1.0.0` with `scope.world_ref` = `world:<name>@sha256:<64hex>` ‡. An unknown profile or
  version is refused, never translated. `worldCorroboration` records whether the world was
  checked against a registry or only asserted.
* **Provider executables**: an adapter declares a supported version range and the executable's
  digest; outside the range yields `rlm-provider-unavailable` with the observed version.

## D8. Port governance — the second admission route [R4]

A provider can be governed two ways. Route A restricts *what the child can do with tools*. Route B
restricts *what the child can reach at all*. Route B is stronger for an opaque binary, because it
does not depend on the provider cooperating.

### D8.1 The declaration

```ts
portGovernance?: {
  world: "contained-streaming" | "local";
  reach: {
    filesystem: "workspace-ro" | "workspace-rw" | "none";
    network: "none" | "proxied" | "host-mediated" | "open";
    process: "group-kill" | "uncontrolled";
    lifetime: "parent-death";
  };
  evidence: "fs-diff" | "process-ledger" | "none";
  egress?: { via: "unix-socket" | "loopback-proxy"; destinations: string[] };  // required when network != "none"
}
```

### D8.2 The admission rule

A provider is admitted for a run when **either**:

* **Route A (tool route):** `toolPolicy:"bridge-only"` and the bridge is the only tool path; or
* **Route B (port route, [R5] primary):** `portGovernance.world === "contained-streaming"`, and for
  every restriction the parent's grant implies, the port's reach is equal or narrower:

**[R5] Route C** is not an admission route. A provider that speaks ACP (verified for Prime 0.10.2)
may additionally be driven through `initialize` / `session/new` / `session/prompt`, in which case
`session/update` `tool_call` events (`kind`, `name`, `rawInput`) are recorded as **claims** and the
session can be cancelled on a policy violation. It buys observation and a kill switch, never
pre-approval; the port remains the boundary.

| Parent grant | Required port reach |
|---|---|
| read-only | `filesystem: "workspace-ro"`; `network` ∈ {`none`, `proxied`, `host-mediated`}; `process: "group-kill"` |
| workspace-write | `filesystem: "workspace-rw"`; as above |
| full-access | any declared reach, but `network:"open"` still requires operator authorisation |

`network:"proxied"` requires `egress.destinations` to be non-empty and the proxy to be the only
reachable endpoint. `network:"host-mediated"` means the provider asks the host for model calls and
holds no credentials (not available for Prime 0.10.2; kept because it is the best route when a
provider supports it).

A run governed by SEA-Forge requires Route B with `network` ∈ {`proxied`, `host-mediated`} and
`evidence !== "none"`. Route A alone is insufficient under governance, because an opaque binary with
network access can disclose workspace content through its model calls.

### D8.3 Why this is truthful — verified mechanics on this host

| Fact | Command | Result |
|---|---|---|
| a contained world can hold a live stdio process | `bwrap --unshare-all … sh -c 'echo PORT_OK; read line; echo GOT:$line'` | `PORT_OK` / `GOT:hello` |
| the same world has no network | `python3 -c "socket().connect(('1.1.1.1',443))"` inside it | `OSError: [Errno 101] Network is unreachable` |
| a host socket can be the only egress | host unix listener + `--bind /tmp/x.sock /egress.sock` + connect inside | `PROXY_OK` |
| the workspace bind is the only writable project path | `--bind /tmp/ws /ws` + write outside | write inside succeeded; `/tmp` is a private tmpfs |

| a user-systemd pid/memory ceiling is available | `systemd-run --user --scope -p TasksMax=64 true`; `-p MemoryMax=256M` | exit 0 |
| the port's per-child cost | 15× spawn timing, plain vs bwrap | 0.9 ms vs 8.1 ms (**+7.2 ms**) |
| Prime speaks ACP | `prime-agent --mode acp` + ACP `initialize` / `session/new` | valid response, `version 0.10.2`, live `sessionId` |
| a per-invocation provider can use a loopback proxy | `~/.prime/agent/models.json` provider keys | `baseUrl`, `api`, `apiKey` — dummy key in the sandbox, real key host-side |

bubblewrap 0.13.1, `systemd 259`. These are smoke tests, not a security audit: they establish
feasibility, not guarantees (no seccomp profile; cgroup ceiling depends on a user manager). Where a
probe fails, the reach declaration reports the weaker level rather than the requested one.

### D8.4 What Route B does **not** give you

* **Per-tool policy.** The port cannot say "read but not edit" at the tool level. It maps the grant
  to a filesystem mode instead (`workspace-ro` vs `workspace-rw`). That is coarser, and it is the
  honest resolution.
* **Exfiltration control.** A provider with any model egress can send workspace content to its model
  provider. This is the same channel the ordinary Expert loop already uses; the difference is that
  the port can name the destination and record it. The design records it and refuses `network:"open"`
  under governance rather than pretending the risk is gone.
* **Subprocess counting.** The current matrix reports `subprocess: "not-enforced"` for every world
  kind, including bubblewrap. Reach is bounded; the number of processes is not. Evidence comes from
  the process group and the filesystem diff, not from a count.
* **Approval inside the child.** A consequential operation cannot be gated inside a black-box
  process. The gate is before spawn (authority) or after (fs-diff review). That is why the escalation
  path, not the provider path, carries per-operation approval.

* **Gauntlet**: the projector pins the Gauntlet version and requires the operator to name a
  `runners.default`; Workbench never chooses `prime-agent` on the operator's behalf.
* **Cognate packages**: one version line for Workbench and Horizon (I0); mixing is a refusal at
  boot, not a warning.

## D7. Explicitly not public

`RlmProvider`, `StdioSpawnPort` implementations, provider ids *in code* (they are configuration), the bridge/settings file
format, the spawn ledger schema, Workbench's `worldHash` algorithm, Gauntlet's
`RoleInvocation`/`StatusCategory` types, and any Prime env var or CLI flag.
