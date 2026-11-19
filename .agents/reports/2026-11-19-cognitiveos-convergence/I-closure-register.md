# I — Closure register (revision 5)

Purpose: leave **no open question**. Every item ends in a decision, a named guard, and an accepted
residual risk. Evidence was gathered by running the mechanism, not by reading about it.

## I1. What was tested, and what it settles

| # | Test | Result | What it settles |
|---|---|---|---|
| 1 | `bwrap --unshare-all … sh -c 'echo PORT_OK; read x; echo GOT:$x'` | `PORT_OK` / `GOT:hello` | a contained world can hold a **live stdio** process |
| 2 | socket connect to `1.1.1.1:443` inside that world | `OSError: [Errno 101] Network is unreachable` | the world has no network by default |
| 3 | host unix listener + `--bind x.sock /egress.sock` + connect inside | `PROXY_OK` | a single declared egress channel is possible |
| 4 | listener + client on `127.0.0.1` inside the same world | `LOOPBACK_OK` | an in-sandbox bridge can front the egress socket |
| 5 | `systemd-run --user --scope -p TasksMax=64` / `-p MemoryMax=256M` | exit 0 | a pid/memory ceiling is available without root |
| 6 | `bwrap --seccomp FD` in `--help` | present | a syscall filter is available but not required |
| 7 | 15× spawn timing, plain vs bwrap | 0.9 ms vs 8.1 ms (**+7.2 ms**) | port cost per child is negligible beside a model call |
| 8 | `prime-agent --mode acp` + ACP `initialize` | valid response; `agentInfo.version 0.10.2`; `mcpCapabilities.http`; `sessionCapabilities.close` | Prime's ACP transport is real and current |
| 9 | ACP `session/new {cwd, mcpServers}` | `sessionId` + `configOptions` | the client drives session creation, per invocation |
| 10 | Prime `models.json` | `providers.<name>` with `baseUrl`, `api`, `apiKey`, `compat`, `models` | a per-invocation provider can point at a loopback proxy with a **dummy** key |
| 11 | Pi upstream `docs/security.md` | "it does not ask for approval before every tool call"; "Safety comes from limiting the files, credentials, processes, and network services" | tool-level approval is **not** the intended boundary |
| 12 | Pi upstream `docs/containerization.md` | "Provider credentials remain on the host and are substituted by the proxy" | host-side credential substitution is the supported pattern |

## I2. The port-governance gaps, each closed

### G-I1. Per-tool policy ("read but not edit")
**Decision.** Do not attempt per-tool policy for opaque binaries. Map the parent's grant to a
filesystem mode; use ACP observation + cancellation where the provider supports it (I3).
**Why.** Upstream states approval-per-call is not a boundary. A gate we cannot enforce inside the
child would be theatre.
**Guard.** A grant that cannot be expressed as a filesystem mode (`bash` without subprocess, `edit`
without `bash`) is refused `rlm-containment-unavailable`, never approximated.
**Residual.** The whole workspace is reachable at the chosen mode. Accepted: the workspace is the
unit of work and credentials live in the secret store, not the workspace.

### G-I2. Exfiltration through the model channel
**Decision.** Under governance the only network is `proxied`, to one declared destination set, via a
host proxy that forwards only there, injects the real credential (sandbox holds a dummy), records
destination + byte size + body digest per request, and caps body size.
**Why.** Content must reach a model or the provider cannot work; the honest controls are *where*,
*with which credential*, and *what is recorded*.
**Guard.** `network:"open"` refused under governance; no declared destination →
`rlm-egress-uncontrolled`.
**Residual.** A proxied provider can still send workspace content to its allowed model provider.
Same channel the ordinary Expert loop already uses. Recorded, not claimed closed.

### G-I3. Subprocess counting and resource ceilings
**Decision.** Launch the provider as a `systemd-run --user --scope` unit with `TasksMax` and
`MemoryMax` when a user manager exists (verified); otherwise record `caps:"partial"` and rely on the
process group. Never claim counting that did not happen.
**Guard.** `reach` reports the level observed at launch, not the level requested.

### G-I4. Approval inside the child
**Decision.** Impossible by construction. Any operation needing advance approval becomes a
`case.escalate` to Gauntlet, where the reference is frozen and the control loop is ours.
**Guard.** Such an assignment is refused with a message naming escalation. A provider run is
accepted at assignment granularity only.
**Residual.** Post-hoc review (fs diff) is the fallback for work that did not escalate.

### G-I5. Hardening depth (seccomp, userns)
**Decision.** Require `--unshare-all --unshare-user --disable-userns --assert-userns-disabled
--new-session --die-with-parent --cap-drop ALL`. Seccomp optional, declared `partial` when absent.
No filter in this change set.
**Guard.** The matrix records `seccomp: "absent" | "filter"`.

### G-I6. Evidence of what the child did
**Decision.** Reuse `worldManifest`/`worldHash` (`src/cognitive/world.ts`) for before/after
digests, plus `git status --porcelain` when the workspace is a repo, plus ACP `tool_call` records
when the provider speaks ACP. Store the manifest diff as evidence.
**Why.** Already implemented and already used for case lineage; a second evidence mechanism would
duplicate it.
**Guard.** If the diff cannot be computed, the run records `evidence:"none"` and cannot be accepted
as verified.

### G-I7. Where the pieces live
**Decision.** `contained-streaming` world provider + reach matrix in
`@godspeedai/cognate-execution`. Egress proxy, per-invocation `models.json` generation and the
operator's destination allowlist in Workbench. Both reach the adapter through `StdioSpawnPort` only.
**Guard.** A dependency-graph test asserts the adapter imports no Workbench module.


## I3. The three admission routes, and which one is primary

| Route | Mechanism | Gives | Does not give | Status |
|---|---|---|---|---|
| **B — port** (primary) | contained streaming world: fs mode, no network except one proxied socket, process group, manifest-diff evidence | reach no wider than the parent's grant, independent of provider cooperation | per-tool policy, in-child approval | **adopt**; I3b |
| **C — ACP mediation** | host acts as ACP client (`initialize`, `session/new`, `session/prompt`), receives `session/update` `tool_call` events with `kind` and `rawInput`, can `session/cancel` | per-call **observation** and a kill switch; session-scoped model selection | pre-approval (Prime does not request permission per call; upstream says it will not) | **adopt as observation**; same package |
| **A — tool list** | CLI/settings allowlist (`--tools`, `--no-builtin-tools`, `defaultTools`) | true pre-restriction where offered | anything for Prime 0.10.2 (no flags, no key) | **secondary**, provider-declared |

**Decision.** Route B governs. Route C is an evidence and kill-switch channel, **not** an
authorization gate — say so in the UI and docs, because calling observation "approval" is the
dishonesty this design exists to avoid. Route A stays in the contract for providers that can attest
it; Prime cannot.

**Consequence for the adapter (supersedes the extraction plan).** The Cognate adapter speaks **ACP**
instead of extracting Gauntlet's TypeScript host. ACP is a public JSON-RPC spec with per-call tool
events and cancellation; the handshake is verified against Prime 0.10.2; and it removes coupling
(no Gauntlet-internal `focus`/`topology`/`affordance` imports). Gauntlet's host stays unchanged. The
reuse claim becomes: reuse the *spawn validation and protocol knowledge*, adopt a standard instead of
an internal dialect.

## I4. Every remaining decision, closed

| # | Question | Decision | Guard if wrong |
|---|---|---|---|
| O1 | Package or sidecar for the adapter | **Package** `@godspeedai/cognate-rlm-prime`, speaking ACP | If Prime drops ACP, fall back to `--mode rpc` using Gauntlet's fixture; the package remains correct |
| O2 | Is I3b in scope | **Yes.** Without it the capability is local-only and governance cannot use it | If the proxy proves unworkable, ship B without network and let providers run offline-only; governed providers then refuse, as today |
| O3 | Who may call `case.escalate` | **Operator only** | If Teams need it, add an orchestrator-scoped variant with a recorded approval |
| O4 | Local `worldHash` as CEP `world_ref` | **Never**; local cases are `scope:"local"` | none — the grammar makes it unrepresentable, not merely disallowed |
| O5 | Restore verbatim or refresh | **Verbatim** | If operators want upgrades, add a separate `experts.refresh` command |
| O6 | Provider misreports ACP tool events | Treat ACP events as **claims**, corroborated by the fs diff; a mismatch records `evidence-discrepancy` and blocks acceptance | If discrepancies are common, drop ACP observation and rely on the port alone |
| O7 | Platform scope | Linux only for Route B; macOS/Windows run the ordinary loop and refuse providers under governance | A future seatbelt/job-object implementation extends the matrix; nothing else changes |
| O8 | Secret committed in the workspace | Out of scope: the port exposes the workspace | Add a pre-spawn secret scan (existing gitleaks tooling) as an optional gate |

## I5. The five "needs running" items, converted to decision trees

| Item | If the check passes | If it fails |
|---|---|---|
| `CaseContext` in 0.9.2 | proceed with I0 | pin 0.10.0; if 0.10.0 breaks Workbench, define the field locally, pass it through `state`, open the Cognate PR — do not fork the semantics |
| DomainForge → SEA-Forge `evidence_packet` | J14 passes | if a field is rejected, the handoff carries only profile fields; if a required datum has no slot, **refuse the handoff** and file a CEP change proposal |
| A real Prime invocation under ACP | J4 with provider passes | if ACP framing differs, correct the adapter from a recorded transcript; if Prime cannot run non-interactively, ship contract + test provider and **do not** claim RLM |
| Gauntlet escalation | J7 passes | if it needs more than one config + reference projection, **drop I6**; it is optional |
| Runtime behaviour of any journey | TDD loop | report the journey failing; no design change unless a guard is proven wrong |

## I6. Residual risks that remain accepted (not open)

1. A proxied provider can disclose workspace content to its model provider. Recorded per request;
   accepted as the same class as the ordinary loop's context upload.
2. The whole workspace is readable at `workspace-ro`. Accepted; secrets belong in the store.
3. Subprocess count is unenforced without a user systemd manager. Reach stays bounded.
4. No seccomp filter. Declared `partial`.
5. ACP tool events are claims; the fs diff corroborates.
6. Attribution remains bearer-key based in Horizon. Unchanged from current behaviour.

## I7. Statement of closure

Nothing here is waiting on a decision. Two items wait on **work** — I3b's implementation and the
acceptance evidence in `F`/`E` — and those are plan items, not uncertainties. If a guard fails, the
pre-agreed response is written next to it; the design does not need reopening to know what to do.
