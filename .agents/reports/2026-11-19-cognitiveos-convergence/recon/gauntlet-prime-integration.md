# Gauntlet — Prime Agent integration and execution/verification/settlement control loop

Recon target: `/home/sprime01/projects/gauntlet`, branch `main`, HEAD `6a7fe6a`
(`fix(packaging): absolute --previous-dir for the upgrade leg; release gauntlet 0.1.3 (#30)`).
Read-only survey. All paths below are absolute.

---

## 1. `crates/adapters/gauntlet-adapter-agent-prime/` — contents

**Files** (11 src + 3 test targets, ~8.4k lines):

```
src/{lib,program,transport,runner,probe,capability,config,clock,managed,mcp_bridge,test_support}.rs
tests/{contract_suite,live_probe,mcp_bridge_live}.rs
```

- Dependency direction is enforced in the manifest: only `gauntlet-domain`, `gauntlet-ports`, `sha2` — the comment states adapters "depend on domain and ports only — never on gauntlet-app or gauntlet-cli" (`/home/sprime01/projects/gauntlet/crates/adapters/gauntlet-adapter-agent-prime/Cargo.toml:10-22`).

### Public types / exports

`/home/sprime01/projects/gauntlet/crates/adapters/gauntlet-adapter-agent-prime/src/lib.rs:49-72`

| Export | Kind |
|---|---|
| `CapabilityFinding, CapabilityProbeConfig, CapabilityReport, SeamStatus, probe_capabilities` | behavioral capability probe (`capability.rs`) |
| `SystemClock` | production `Clock` binding (`clock.rs`) |
| `AGENT_DIR_ENV, PrimeAdapterConfig` | typed config (`config.rs`) |
| `McpBridge` | per-invocation MCP bridge (`mcp_bridge.rs`) |
| `ProbeError, ProbeReport, VersionRange, is_supported, probe` | `--version` probe (`probe.rs`) |
| `InvocationForensics, PrimeAgentRunner` | the runner (`runner.rs`) |
| `AdmissionHandle, ControlError, HarnessControl, KernelSubmission, KernelTransport, NullParentHandle, ParentHandle, ProcessHarnessControl, ProcessKernelTransport, RealParentHandle, SubagentRecord, TransportError, parse_listing, read_admission` | the two harness seams (`transport.rs`) |
| `pub const RUNNER_TOKEN: &str = "prime-agent"` | composition token (`lib.rs:67`) |
| `pub use PrimeAgentRunner as RecursiveSubagentRunner` | capability-named alias (`lib.rs:72`) |

Other module-level public items: `SpawnDialect` (`config.rs:36-42`), `DEFAULT_BINARY_NAME = "prime-agent"` (`config.rs:65`), `DEFAULT_MAX_DEPTH = 1` (`config.rs:69`), `DEFAULT_ADMISSION_TIMEOUT_SECONDS = 600` (`config.rs:74`), `AGENT_DIR_ENV = "PRIME_AGENT_CODING_AGENT_DIR"` (`config.rs:80`), `BINARY_ENV = "PRIME_AGENT_BIN"` (`managed.rs:40`), `ChildTelemetry`/`ParentTelemetry` (`transport.rs:162,185`), `KernelProgramSpec` + `render`/`compose_child_prompt` (`program.rs:58,85,149`).

### Port traits it implements

- `impl AgentRunner for PrimeAgentRunner<T, C>` — `/home/sprime01/projects/gauntlet/crates/adapters/gauntlet-adapter-agent-prime/src/runner.rs:639` (`invoke` 640, `poll` 790, `collect` 947, `cancel` 967, `capabilities` 1001; `lifecycle_samples` left to the port default at `gauntlet-ports/src/agent_runner.rs:473`).
- `impl KernelTransport for ProcessKernelTransport` — `transport.rs:947` (adapter-local seam, not a port).
- `impl HarnessControl for ProcessHarnessControl` — `transport.rs:1252` (adapter-local seam).
- `impl ParentHandle for RealParentHandle` / `NullParentHandle` — `transport.rs:904,355`.
- `Clock` via `SystemClock` (`clock.rs`, re-exported `lib.rs:52`).


### How it spawns / drives the executable

```rust
// transport.rs:444-451
fn rpc_command(binary: &Path, workspace: &Path, agent_dir: Option<&Path>) -> std::process::Command {
    let mut command = std::process::Command::new(binary);
    command.arg("--mode").arg("rpc").current_dir(workspace);
    if let Some(dir) = agent_dir { command.env(crate::config::AGENT_DIR_ENV, dir); }
```

- CLI args: `<binary> --mode rpc` only. cwd is the **per-invocation** workspace `<workspace_root>/<spec.workspace>/`, validated as a single key-class component before any spawn (`runner.rs:496-501`, called at `runner.rs:662`; mirrors `gauntlet_adapter_agent_cli::runner::execution_root`).
- stdin framing: **one** newline-terminated JSON line `{"id":"gauntlet-admit","type":"prompt","message":"…"}` (`transport.rs:486`, `873-880`). stdin stays **open**; closing it is the shutdown signal (`transport.rs:988-993`, `637-651`).
- Program carriage: prose instruction + the program between `<<<GAUNTLET_PROGRAM_BEGIN>>>` / `<<<GAUNTLET_PROGRAM_END>>>` (`transport.rs:547-557`).
- stdout framing: **newline-delimited JSON events** (JSONL, not JSON-RPC), parsed by a background reader thread into `ParentTelemetry` — `response`, `message_end`, `agent_end`, `tool_execution_start`, `rlm_child_update` (`transport.rs:726-835`, reader `840-855`); lines >4 MiB are skipped whole (`transport.rs:475-478`).
- stderr: drained into a bounded 64 KiB buffer, diagnostics only (`transport.rs:859-869`, `482`).
- Admission is a **file watch**, not a protocol reply: `submit` polls `handle_path.is_file()` under `deadline`/`admission_timeout`, strict-parses it, and returns `(AdmissionHandle, Box<dyn ParentHandle>)` (`transport.rs:1013-1121`, success `1132-1154`).
- Control subcommands: `prime-agent stop <child>` and `prime-agent list --json`, both `Deadline`-bounded (`transport.rs:1259,1285`); the listing must be exactly `{"sessions":[…]}` or it is a typed error (`transport.rs:1301-1337`).
- Version gate: `prime-agent --version`, output accepted on **either** stream (the real harness prints the version to stderr — `probe.rs:18-19`), compared against an **injected** `VersionRange` (`probe.rs:45,139-141`).

### What the rendered program does (byte-pinned)

`program.rs:85-137` renders one Python cell:

```python
envelope = pathlib.Path(<envelope>).read_text()
child_input.write_text(envelope)
handle = await rlm.spawn(   # or  await rlm(  for SpawnDialect::Callable08
    <role prompt + contract instruction>, name=<invocation name>, model=<hint>)
pathlib.Path(<handle_path>).write_text(json.dumps({ 'rlm_child_id': str(...), ... }))
```

Dialect: `SpawnDialect::Spawn09` (`rlm.spawn(`, default) vs `Callable08` (`rlm(`) — `config.rs:36-51`. No linger, no poll loop, no `time.sleep` (`program.rs:18-37`); byte shape pinned at `program.rs:84`.

### What it persists (correlation / recovery)

Under the invocation directory (`runner.rs:374-412`):

```
<state_dir>/runs/<run_id>/inv/<invocation_id>/
  <result_contract_token>          # the contract file the child writes
  <token>.envelope.json            # runtime-written context envelope
  <token>.child-input.json         # written by the rendered program
  <token>.handle.json              # the 4-field admission handle
  agent/settings.json (+symlinks)  # only when an MCP bridge is provisioned
```

- `invocation_dir` refuses any contract token that is not a single leaf component and re-checks `dir.starts_with(state_dir)` (`runner.rs:379-404`).
- Deterministic `rlm(name=)` = `"<role>-<run_short>-<inv_short>"` (`runner.rs:418-429`) — the name the orphan sweep searches (`runner.rs:600-631`).
- `session_ref` is recorded losslessly as `session-dir:<session_dir>`, forensics only, never control state (`runner.rs:751-765`).
- In-memory `PrimeEntry` holds `spec, contract_file, child (rlm_child_id), agent_dir, invocation_name, session_ref, admission_at, parent, terminal, result, control_note` (`runner.rs:769-784`); the live `Box<dyn ParentHandle>` is owned there for the invocation's lifetime.
- Durable correlation is NOT in the adapter: `gauntlet_app::compare::recording::RecordingAgentRunner` refuses to expose a handle for polling until the admitted `RoleInvocation` is durably recorded (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/compare/recording.rs:1-40`), via `StateStore::record_role_invocation` (`/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/state_store.rs:461-470`).

---

## 2. The agent-runner port (`crates/gauntlet-ports/src/agent_runner.rs`)

Spec base: `/home/sprime01/projects/gauntlet/.agents/specs/Core-spec.md:408-434` (§6.1).

```rust
// agent_runner.rs:441-484
pub trait AgentRunner {
    fn invoke(&self, spec: RoleInvocationSpec) -> Result<InvocationHandle, AgentError>;
    fn poll(&self, handle: &InvocationHandle) -> Result<InvocationStatus, AgentError>;
    fn collect(&self, handle: &InvocationHandle, deadline: Deadline) -> Result<RoleResult, AgentError>;
    fn cancel(&self, handle: &InvocationHandle, reason: CancelReason, deadline: Deadline) -> Result<(), AgentError>;
    fn lifecycle_samples(&self, handle: &InvocationHandle) -> Result<Vec<HarnessLifecycleSample>, AgentError> { Ok(Vec::new()) }
    fn capabilities(&self) -> RunnerCapabilities;
}
```

- Lifecycle invariant: `invoke` MUST return promptly after admission and never returns the role result; results arrive out-of-band through the declared result contract (`agent_runner.rs:1-22`, `442-444`).
- `poll` is a non-blocking peek and must be **terminal-stable**: once a terminal status is reported, every later poll returns the SAME status (`agent_runner.rs:425-435`).
- `cancel` is idempotent — later calls `Ok(())` or `AlreadyTerminal`, never a second effect (`agent_runner.rs:433-435`).
- Ambiguity handling is prescribed, not left to adapters: "Communication with the runner failed. The invocation's fate may be unknown — recovery re-probes result contracts (§18), never assumes" (`agent_runner.rs:46-48`).
- `lifecycle_samples` is a default-no-op forensics input — readings "carry no sequence, never change the invocation's status, and never complete or fail it" (`agent_runner.rs:468-479`).

| Type | Location | Notes |
|---|---|---|
| `AgentError` (7 variants) | `agent_runner.rs:40-74` | `AdmissionRefused, UnknownHandle, Transport, ResultNotReady, CollectionFailed, AlreadyTerminal, CancelFailed` |
| `InvocationFailure` | `agent_runner.rs:133-139` | `MalformedResult, ContractMismatch, OrphanedByRestart, Execution, MissingResult` (wire tokens `141-152`) |
| `InvocationStatus` | `agent_runner.rs:162-176` | `Admitted \| Running{last_event_at, payment_so_far} \| Completed{result_ref} \| Failed{class} \| TimedOut \| Cancelled` |
| `RunnerCapabilities` | `agent_runner.rs:203-211` | `supports_native_subagents, supports_streaming_events, supports_model_selection, isolation, result_delivery, max_depth: u8` |
| `Isolation` / `ResultDelivery` | `agent_runner.rs:217,229` | `None<Process<Container<Vm`; `File\|Mailbox\|ReturnValue` |
| `ContextEnvelope` | `agent_runner.rs:241-259` | opaque `body: String` + `digest: Digest`; the runner never computes the digest |
| `RoleInvocationSpec` | `agent_runner.rs:268-285` | the 10 §6.1 fields **plus** `invocation_id, run_id, iteration_no` so §10.3 rule-6 can be checked |
| `InvocationHandle` | `agent_runner.rs:292-300` | `{invocation_id, session_ref: Option<SessionRef>}` |
| `CancelReason` | `agent_runner.rs:319-353` | opaque non-empty token; empty ⇒ typed `CancelReasonInvalid` |

Deadlines: `collect`/`cancel` carry one because they are blocking IO per the §6 preamble; `poll` does not because it must not block (`agent_runner.rs:8-18`).


---

## 3. `workbench/src/prime-context-envelope.ts`

`/home/sprime01/projects/gauntlet/workbench/src/prime-context-envelope.ts` (548 lines).

- Schema/version token: `export const PRIME_CONTEXT_ENVELOPE_SCHEMA = "gauntlet.workbench.prime-context-envelope/1"` (`:38`). Versioning is a **single string literal in the schema field** — no negotiation, no `v1|v2` union.
- Fields (`:110-134`): `schema, envelope_id, kind ("bounded"|"whole_case_disclosed"), horizon_id (string|null), horizon_generation, root_focus, items: EnvelopeItem[], role_instructions: string[], declared_dependencies: FocusRef[], disclosures: string[], budget: EnvelopeBudgetAccounting|null, constructed_at_ms`.
- `EnvelopeItem` = `{focus, role, reason, mandatory}` with `role ∈ {horizon_root, relation, mandatory_reference, mandatory_governance, mandatory_evidence, declared_dependency}` (`:41-47, 96-103`).
- Budget: `EnvelopeRoleBudget {relation, declared_dependency}`; `BUDGET_EXEMPT_ROLES` keeps the mandatory closure + root exempt (`:59-93`). Exceeding a budget is a typed `EnvelopeConstructionError`, "never silently re-kinded" (`:49-64`).
- Constructors: `constructBoundedContextEnvelope` (`:226`) — requires a Horizon and a plan naming that same horizon; `constructWholeCaseEnvelope` (`:335`) — the only unbounded path, requires a recorded disclosure; `envelopeToJSON` (`:372`); `envelopeFromJSON` (`:381-548`) — re-admits untrusted JSON, refusing a schema mismatch (`:392`) and re-resolving FocusRefs "so deserialization can never fork identity" (`:25-26`).
- Explicit non-capability: critic envelopes are not built here — "this module deliberately has no critic-envelope capability at all" (`:27-30`).

Producers / consumers:
- `/home/sprime01/projects/gauntlet/workbench/src/workbench-host.ts:22` imports it; `:542-554` builds a bounded envelope from `model.planForFocus(focus)` at Prime launch and refuses the launch on construction failure.
- `/home/sprime01/projects/gauntlet/workbench/src/prime-recursion.ts:47`; `:323` embeds `JSON.parse(envelopeToJSON(this.context.envelope))` in the delegation request; `:425` re-validates `request.envelope.schema !== PRIME_CONTEXT_ENVELOPE_SCHEMA`.
- Re-exported from the barrel `/home/sprime01/projects/gauntlet/workbench/src/index.ts:27`; exercised by `prime-context-envelope.test.ts`, `prime-delegation-real.test.ts`, `prime-recursion.test.ts`, `prime-fresh-target.test.ts`, `security-boundaries.test.ts`, `secret-leakage.test.ts`.

**CEP profile: none.** `grep -ni 'cep|profile' workbench/src/prime-context-envelope.ts` returns no match; the module's inputs are Horizon + planMaterialization + declared dependencies and its only protocol token is its own schema string. (CEP appears on the DomainForge path instead — §4.)

Cross-language note: this type is **not** the Rust `ContextEnvelope` port type (`agent_runner.rs:241`); Rust takes an opaque `body` + `digest` and writes it verbatim to `<token>.envelope.json` (`runner.rs:670-676`).


---

## 4. `workbench/src/domainforge-adapter.ts`

`/home/sprime01/projects/gauntlet/workbench/src/domainforge-adapter.ts` (1501 lines).

What it does: parses the **pinned DomainForge canonical semantic envelope** and projects it into FocusRef/Relation/horizon seams "WITHOUT creating a new identity domain or a new truth store" (`:1-7`). Three branded identities never collapse (Bun cache hash / file content hash / semantic fingerprint — `:225-328`); Unit bindings are by exact producer declaration identity ("no name-based joins" `:42-44`); and "The adapter never spawns a process: producer output arrives out of band" (`:48-50`).

Wire format / version pins (`:69-80`):

```ts
export const PINNED_ENVELOPE_SCHEMA_VERSION = "domainforge-semantic-envelope/v1";
export const PINNED_PRODUCER_NAME = "domainforge-core";
export const PINNED_PRODUCER_VERSION = "0.16.0";
export const PINNED_LANGUAGE_SCHEMA_VERSION = "domainforge-ast/v3";
export const PINNED_CANONICALIZATION_VERSION = "domainforge-canonicalization/v1";
export const PINNED_INTERPRETATION_VERSION = "domainforge-interpretation/v1";
```

Two parsers: `parseSemanticEnvelopeDocument` (bare D — exact-key checks `:858-896`, `schema_version` must equal the pin `:846-855`, producer **name** must equal the pin `:867-872`; the producer **version is extracted but never compared** at `:866`) and `parseProducerEnvelopeOutput` for the `--emit both` carrier `{"representation": D, "cep_envelope": E}` (`:944-1010`).

### Recorded incompatibilities (explicit and machine-readable)

`PROTOCOL_COMPATIBILITY` (`:148-194`), statuses `pinned | incompatible | unqualified | unknown` (`:134-146`):

| pair | producer × consumer | status |
|---|---|---|
| P1 | `domainforge release 0.16.0 envelope <entry.sea>` (bare D) × workbench T14 adapter | `pinned`, locally_verified |
| P2 | `domainforge installed 0.16.0 (~/.cargo/bin)` × `sxr-df CliDomainForgeAdapter (Slice 0B)`, argv `envelope --emit both <entry>` | **`incompatible`** — "binary has no `envelope` subcommand at all (exit 2: unrecognized subcommand)" (`:160-171`) |
| P3 | `domainforge release 0.16.0 (target/release)` × same SXR consumer | **`incompatible`** — "release build rejects --emit (exit 2: unexpected argument '--emit')" (`:172-181`) |
| P4 | working-tree build (`target/debug`) × same SXR consumer | `unqualified` — invocation succeeds locally but "deliberately NOT selected" (`:182-193`) |

Also recorded: three distinct `domainforge 0.16.0` builds on the machine, only some of which emit the envelope document (`:12-25`); DEC-005's consequence stated as "the main-CLI/SXR-0B pair stays INCOMPATIBLE until qualified (T14B)" (`:26-30`); `selectProtocolPairs` refuses to fall back on unknown shapes (`:196-210`).

CEP on this path: the adapter **recognizes and types** a CEP-0008 semantic-snapshot wrapper (`cep_version` / `envelope_kind` keys) and forces `model_valid:false` when `extensions.domainforge.diagnostics` is non-empty, so "resolved output carrying diagnostics MUST NOT appear conformant" (`:794-843`, `:992-1010`). Captured CEP instance in tests: `cep_version: "0.1.0"`, `envelope_kind: "semantic_snapshot"`, `envelope_version: "1.0.0"` (`/home/sprime01/projects/gauntlet/workbench/src/domainforge-adapter.test.ts:50-53`). It does **not** consume a CEP *profile* for the Prime envelope.

RealityTrace/SXR side (separate file): `/home/sprime01/projects/gauntlet/workbench/src/sxr-adapter.ts:1-50` — Reality Trace evidence lens over `sxr_output_version: "1"` and the raw `ledger export` array, with SXR identity/settlement kept distinct from Gauntlet's; the DEC-005 requalification and the both-mode **object** carrier (`cep_envelope.representations[0].inline_content`) are recorded in `/home/sprime01/projects/gauntlet/.agents/reports/gauntlet-tui-prime-workbench/evidence/T14B/sxr-evidence.md:12-40`.

Post-pin captures that still parse: 0.18.2 and 0.19.0 documents parse with recomputed digests and an identical `semantic_closure_hash` (`workbench/src/domainforge-0-18-2-capture.test.ts:1-50`, `workbench/src/domainforge-0-19-0-capture.test.ts:1-40`) — the pin gates schema, not producer version.


---

## 5. Reference freeze / execution / verification / recovery / settlement

### Reference (freeze + I1 tripwire)

- Port: `ReferenceProvider` with `resolve / normalize / digest / freeze / verify_unchanged`, every method carrying a `Deadline` (`/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/reference_provider.rs:105-147`). `freeze` is documented as §5.3.3: "a `Run` has exactly one frozen `reference_version` for its whole life" (`:130-138`); `verify_unchanged` is "The I1 tripwire … re-derives the current digest and compares" (`:140-147`).
- Freeze is re-verified at **every** iteration boundary (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/control/controller.rs:30-31`); `ReferenceProviderError::ReferenceMutated` is terminal `aborted{reference_mutated}` with both digests persisted as pinned evidence and no retry (`controller.rs:40-48`).
- `.gauntlet/reference.lock`: `verify_lock` refuses a mutated answer key or manifest naming both digests (`/home/sprime01/projects/gauntlet/crates/gauntlet-cli/src/reference_lock.rs:1,17-49,68-70`); CLI surface `reference digest|validate|form|freeze|show|diff|verify` (`crates/gauntlet-cli/src/lib.rs:112-118`).
- I1 anchor: the answer key must live outside every builder-writable workspace; `execution_root` refuses to fall back to the project root precisely because the key lives there (`runner.rs:480-495`); G10 proves both the prevented write attempt and the digest-tripwire detection (`/home/sprime01/projects/gauntlet/crates/gauntlet-cli/tests/g10_builder_writes_answer_key.rs:1-25`).

### Execution

- The loop: checkpoint → boundary re-verify → tick (re-read key) → plan → budget re-check → dispatch → drain → per result: charge → observe → bind (gated) → compare → produce evidence (gated) → verify → dispose → select (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/control/controller.rs:24-38`).
- Ports used: `AgentRunner`; `UnitScheduler` + the §11.5 `disposition` table (`controller.rs:163-172`); `WorkspaceProvider` (`create/cleanup/hooks/promote/check_containment` — `/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/workspace_provider.rs:162-201`); `BudgetMeter`; `Observer`; `Clock`; `Deadline`.
- Authority precedes execution: governance is "authority decides WHETHER, containment decides HOW CONTAINED … Both checks precede builder process start, in that order" (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/govern/mod.rs:1-13`).
- Capability honesty: `PrimeAgentRunner::capabilities` reports `isolation: Isolation::None`, `result_delivery: File`, `supports_streaming_events: false` (`runner.rs:1001-1014`), matching Annex C.2 and §6.1's "a runner reporting `isolation: none` MUST NOT be selected for a tier requiring containment".

### Verification

- Port: `Verifier::verify(&VerificationRequest, Deadline) -> VerificationRecord`, plus `is_deterministic`, `instrument`, `supports` (`/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/verifier.rs:87-121`). Missing evidence is typed `EvidenceMissing` at the instrument layer and INCONCLUSIVE at the consequential layer — "the instrument may not pretend to have decided, and the loop may not treat absence as silence or success" (`verifier.rs:15-31`).
- Ladder: deterministic-first is structural — positions 1–4 deterministic (`FileDigest`, `SchemaCheck`, `CommandExitCode`, `TestReport`), position 10 `ModelJudgment`; "a model record claiming `instrument.deterministic: true` is refused" (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/verify/ladder.rs:1-36`).
- Record codec: `verification_record@v1` with a closed 13-field key set and strict decode (`/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/verification_record.rs:63-88`); the settlement key's verification component is moored in the durable record (`:11-22`).
- Governed chain: `VerificationAuthority::for_run` → `GovernedVerification::verify_claim`; "Nothing in this module compares two values the same caller supplied" (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/verify/authority.rs:1-10`).


### Recovery

- Durable checkpoints: `StateStore::checkpoint` / `resume` — "Critical run state is never reconstructed from conversational memory (I11)" (`/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/state_store.rs:446-457`); `RunController::resume_run` (`controller.rs:1154`).
- Delegated-invocation reconciliation: only `AgentError::UnknownHandle` is authoritative loss evidence, every other runner error leaves the graph untouched "so the next recovery asks again"; restart loss is `orphaned_by_restart` (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/delegation/reconcile.rs:1-22,47-48`).
- Adapter-level: handle-absent/undecodable ⇒ `AdmissionRefused` **with** an orphan sweep by deterministic invocation name (`runner.rs:730-749`); parent reap is forensics only and never a completion/failure signal (`runner.rs:801-809`); a late identity-matching writer never resurrects a `TimedOut` invocation (`tests/contract_suite.rs:777-847`).
- Settlement replay after crash: `SettlementService::complete_pending_after_checkpoint` (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/settle/mod.rs:85-120`).
- Single-writer guard: `crates/gauntlet-cli/src/writer_lock.rs` (documented `run_cmd.rs:13-17`); P7 proves commit ordering and all three §16.3 crash windows (`/home/sprime01/projects/gauntlet/crates/gauntlet-cli/tests/p7_recovery.rs:1-25`).

### Settlement

`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/settle/mod.rs` is "the ONLY settle path in the application layer" (`:1-2`) and enforces, in order (`:19-51`):

1. condition 3 — authority chain over the **frozen** unit's `authority` (`critic` → automatic; `lead_agent` → orchestrator + recorded rationale; `human` → operator identity); an automatic closure the runtime cannot exercise REFUSES and escalates;
2. condition 4 — no blocking residual, checked against the tracked view AND the durable projection; a durable `blocked` the tracked view contradicts is `SettleError::DurableBlockDiverged` — "the DURABLE state wins" (`:66-76`);
3. condition 5 — a settle-time `verify_unchanged` against the reference **provider's own file-derived** answer, "NEVER a comparison against a digest the controller froze itself" (`:30-37`);
4. the commit — idempotent on `(run, unit, verification)`; `SettlementCommit` is admitted by `SettlementCommitAuthority` against durable PASS records and committed through the store's §16.3 sequence (fsync'd event append → keyed row) (`:38-46`; `state_store.rs:361`, and a plain `SettlementCommitted` event append is refused at `state_store.rs:40-67`);
5. "the in-memory advance is LAST" (`:47-51`).

Spec anchor: `/home/sprime01/projects/gauntlet/.agents/specs/Core-spec.md:1018-1110` (§16.1, §18.3).

---

## 6. CLI surface, machine-readable output, exit codes

Dispatch: `gauntlet_cli::run_cli(&args) -> ExitCode` (`/home/sprime01/projects/gauntlet/crates/gauntlet-cli/src/lib.rs:56-102`); the binary is a 12-line wrapper (`crates/gauntlet-cli/src/bin/gauntlet.rs:9-12`). The crate is a **library with a thin binary**; the docs state tests "may call this directly" (`lib.rs:4-9,54-55`).

Subcommands (`lib.rs:74-91`): `doctor`, `agent` (`doctor`/`setup`), `init`, `reference` (`digest|validate|form|freeze|show|diff|verify`), `run`, `status`, `inspect`, `verify`, `resume`, `escalations` (`list|resolve|retry`), `roles` (`install|verify|list`), `skills` (`install|verify|list`), `improve` (`propose|evaluate|baseline|promote|rollback`), `export`, `deliver`, `fabric` (`admit-root|serve`), plus feature-gated `serve` (absent from the default build — `Cargo.toml:10-17`). Root `--help`/`--version` are handled before dispatch (`lib.rs:62-73`).

Exit codes: `0` success, `1` (`ExitCode::FAILURE`) refusal/failure, `2` usage error (`lib.rs:8-9,92-100`); `fabric_cmd.rs:119-120` states "`Usage` is a bad invocation (exit 2); `Refused` is a fail-closed refusal of a well-formed request (exit 1)". Run outcome maps `Settled => SUCCESS, _ => FAILURE` (`run_cmd.rs:454-459`).

Machine-readable output:
- `gauntlet run`/`resume` prints key=value lines (`run: run_id=…`, `run: terminal=…`) **and** the report JSON on stdout: `println!("{}", report.to_json())` (`run_cmd.rs:430-442`, `464-476`).
- `gauntlet doctor` writes `$GAUNTLET_REPORT_DIR/doctor.json`, prints a human summary and `report: <path>` (`crates/gauntlet-cli/src/doctor_cmd.rs:1-74`).
- `fabric serve` speaks **MCP-subset JSON-RPC 2.0 over stdin/stdout**, newline-delimited; "Stdout carries protocol frames only. Diagnostics go to stderr" (`fabric_cmd.rs:19-21`; `crates/adapters/gauntlet-adapter-surface-rpc/src/lib.rs:1-20`, `SUPPORTED_PROTOCOL_VERSIONS = ["2025-06-18","2025-03-26","2024-11-05"]` `:25`, `tools/list`/`tools/call` `:30-31`).

### Smallest programmatic seam for ONE bounded run

1. **`gauntlet fabric serve` — one bounded call, no controller adoption.** One host process per caller; `tools/call` is bounded by `CALL_BOUND_SPAN = 60` s (`fabric_cmd.rs:77-78,308-312`); an unbound process has every `tools/call` refused before any service/graph/budget access (`fabric_cmd.rs:112-117`). The model-facing tool set is `agents.spawn / agents.status / agents.collect / agents.cancel / agents.list` (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/delegation/agents.rs:1-18`). It does not run freeze/verify/settle.
2. **`gauntlet fabric admit-root`** — admits exactly one root invocation into the durable invocation graph, prints `invocation_id=<id>`, idempotent on an idempotency key (`fabric_cmd.rs:4-7,106-108`).
3. **Library call (Rust embedder)**: `PrimeAgentRunner::new(config, state_dir, workspace_root, transport, control, clock)` + `invoke/poll/collect/cancel` (`compose.rs:231-262`, `runner.rs:535`), or `gauntlet_cli::run_cli(&["run","--dry-run",…])`.

`gauntlet run` itself is the whole controller (doctor → freeze → plan → dispatch → observe → compare → verify → select → settle → checkpoint → report — `run_cmd.rs:1-11`); adopting it means adopting the ruler.


---

## 7. Dependencies on SEA-Forge / CEP / DomainForge / RealityTrace

**No Rust dependency on any of them.** The workspace's entire non-path external list is `toml` (arch tests only) and `sha2` (`/home/sprime01/projects/gauntlet/Cargo.toml:100-108`); the other ~40 entries are path-local Gauntlet crates (`Cargo.toml:59-99`). `Cargo.lock` lists 126 packages and none is named `sea-*`, `domainforge`, `sxr`, or `realitytrace`. A repo-wide search for `sea-forge|sea_forge|seaforge` over `*.rs/*.toml/*.ts/*.md/*.json` returns only prose (`README.md:806-812,1227,1248`, `.agents/plans/task28-needle-navigation.md:4,9`, `workbench/src/world-ref-boundary.test.ts:16`), never a dependency edge.

The composition is therefore **wire/policy-level, not crate-level**:

| Neighbour | Relationship | Evidence |
|---|---|---|
| **SEA-Forge** | owns the runtime authority boundary *above* Gauntlet: `proposed work → SEA-Forge allow/deny/escalate → authorized work contract → Gauntlet constrained execution`. Gauntlet consumes decisions "through a `PolicyProvider`, approval surface, or other authority integration" | `/home/sprime01/projects/gauntlet/README.md:786-812`; port `PolicyProvider` at `/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/policy_provider.rs:1-5` |
| **DomainForge / `.sea`** | the reference *source*: "A `.sea` model can provide canonical domain meaning"; DomainForge compiles it into a canonical semantic model Gauntlet's Reference can start from | `README.md:813-860`; no Rust code path — the TS workbench adapter parses `domainforge-semantic-envelope/v1` (§4) |
| **CEP** | only as a *producer-output wrapper* on the DomainForge path (`cep_version` / `envelope_kind`), typed, diagnostics-preserving; no CEP profile in the Prime envelope | `workbench/src/domainforge-adapter.ts:794-843,992-1010`; absence of CEP in `prime-context-envelope.ts` (§3) |
| **RealityTrace (SXR)** | TS-only evidence lens over the `sxr` CLI JSON (`sxr_output_version: "1"`) + `ledger export`; "SXR identity and settlement stay source-typed and DISTINCT from Gauntlet's" | `workbench/src/sxr-adapter.ts:1-20`; `workbench/src/reality-trace-binding.ts:1-10` |
| **Prime Agent** | the one *executable* dependency: a bare `prime-agent` name resolved on `PATH` (`config.rs:62-65`), version-gated by an injected `VersionRange` (`probe.rs:39-45`), with an opt-in verified installer that never downloads (`managed.rs:1-20,40`) |

Workbench (`Bun`) deps: `@opentui/core|react 0.8.4`, `quickjs-wasi 3.6.2`, `react 19.3.1`, `react-reconciler 0.34.0` (`/home/sprime01/projects/gauntlet/workbench/package.json`). No SEA-Forge/DomainForge/SXR npm packages either — they are consumed as *process output*.

---

## 8. Reusability from a non-Rust (TypeScript/Bun) process

Nothing in the Rust crates is importable from TS. What crosses is **processes and JSON documents**, and Gauntlet already has two such precedents:

1. **`gauntlet fabric serve` — stdio MCP JSON-RPC.** A Bun process can spawn it and speak `initialize` / `tools/list` / `tools/call` over newline-delimited JSON (`/home/sprime01/projects/gauntlet/crates/adapters/gauntlet-adapter-surface-rpc/src/lib.rs:1-31`); the caller is fixed by `--caller` at process start and never model-supplied (`fabric_cmd.rs:8-21`). This is the intended external seam.
2. **Rust→Bun precedent with a normative protocol doc**: `gauntlet-adapter-tool-program-codemode` spawns the Bun host once per run and speaks NDJSON `{"v":1,"t":…}` frames; the protocol is normative in `/home/sprime01/projects/gauntlet/workbench/src/codemode/PROTOCOL.md:1-40`, and the Rust end is `crates/adapters/gauntlet-adapter-tool-program-codemode/src/protocol.rs:30-98`. This is the exact shape a Bun→Gauntlet client would mirror.
3. **CLI as sidecar**: `gauntlet run --dry-run` (zero writes, exit 0 — `run_cmd.rs:199-267`), `gauntlet run` (JSON report on stdout), `gauntlet doctor` (`doctor.json`), `gauntlet fabric admit-root` (`invocation_id=<id>`); stable exit codes 0/1/2 (§6).
4. **Wire documents** already strict and language-neutral: `role_result@v1`, `verification_record@v1` (`crates/gauntlet-ports/src/verification_record.rs:63-88`), `comparison_record@v1`, `settlement_authority_decision@v1` (`settle/mod.rs:53-65`), `AdmissionHandle`'s 4-field JSON (`transport.rs:71-139`), `{"sessions":[…]}` (`transport.rs:1301`).

Not reusable across the boundary: the port traits themselves (`AgentRunner`, `Verifier`, `ReferenceProvider`, `SettlementService`) — there is no FFI surface and no WASM target (`Cargo.toml`, `deny.toml`). The TS side has its **own duplicate** Prime integration (§9), not a binding to the Rust one.


---

## 9. Recursive language model execution?

Three distinct answers, because Gauntlet has three layers.

**(a) The Rust Prime adapter is single-run and explicitly non-recursive itself.**
`invoke` renders exactly ONE cell containing exactly ONE `await rlm(...)` / `await rlm.spawn(...)`, then returns at admission (`program.rs:100-137`; `program.rs:18-37`: "There is no linger, no result-contract poll, and no `time.sleep`/`while` loop anywhere in the rendered text"). Recursion is the *harness's* mechanism, inherited: "Delegation returns an admission handle only … nothing in this crate may await an answer from the delegation call" (`lib.rs:1-5`). There is no context decomposition — the envelope is passed through verbatim (`program.rs:108-114`) — and depth is only *reported* (`capabilities().max_depth = config.max_depth`, default 1: `runner.rs:1001-1014`, `config.rs:69`). Child inspection is banned as a state input: `rlm.list_subagents` is "liveness/forensics … not a completion signal" (`transport.rs:17-21,411-414`).

**(b) The application layer DOES have genuine bounded recursion — via delegation, not via the Prime adapter.**
- `InvocationGraph` models parent/child lineage with typed refusals `DepthLimit { max, would_be }`, `ChildLimit`, `BudgetExceeded`, `RootAlreadyExists` (`/home/sprime01/projects/gauntlet/crates/gauntlet-ports/src/invocation_graph.rs:140-166,234-360`), incl. `descendants`/`ancestors`/`cancel_subtree`.
- The model-facing affordance layer is recursive by design: `agents.spawn / status / collect / cancel / list`, where "The caller is fixed when the service is built for one invocation. A caller can never name a different parent, because spawn takes no parent argument", and a child gets `child_budget(slice, max_depth)` (`/home/sprime01/projects/gauntlet/crates/gauntlet-app/src/delegation/agents.rs:1-18,426,560-564`). Test: `agents_recursion_refuses_a_third_level_by_depth_limit_with_zero_effects` (`agents_tests.rs:731`).
- Depth is enforced *across processes* through the MCP bridge: the root's fabric server is launched with `--session-binding RLM_DEPTH=0`, so "a child process inherits the same settings, but its harness sets `RLM_DEPTH` to a deeper value, so its server is unbound and may not act on the invocation graph" (`/home/sprime01/projects/gauntlet/crates/adapters/gauntlet-adapter-agent-prime/src/mcp_bridge.rs:11-27,57-65`).

**(c) The workbench (TS) has a second, independent recursion implementation.**
`prime-recursion.ts` implements `RecursionPolicy` (max_depth, per-parent child cap) as a typed pre-delegation refusal, computes depth "by walking the adapter's real parent chains", and records via `adapterCapabilityFacts` that the installed Prime adapter "exposes NO native nested-delegation depth or child-budget capability … it is never presented as a Prime capability" (`/home/sprime01/projects/gauntlet/workbench/src/prime-recursion.ts:1-30,52-60`). Production child spawning is `RealPrimeHost` over `Bun.spawn` with NDJSON event parsing (`/home/sprime01/projects/gauntlet/workbench/src/real-prime-host.ts:1-50`).

Context decomposition is **absent** everywhere: the Rust `ContextEnvelope` is an opaque body+digest (`agent_runner.rs:241`) and the workbench envelope selects FocusRefs under a per-role budget (§3) — neither decomposes the task itself.


---

## 10. Tests that prove the above

### Prime adapter — `crates/adapters/gauntlet-adapter-agent-prime/tests/`

| Test | What it asserts |
|---|---|
| `contract_suite.rs:335` `the_prime_runner_passes_the_unmodified_shared_contract_suite` | the port's shared suite runs against this adapter **unmodified**, via in-memory doubles that execute the rendered program's semantics |
| `contract_suite.rs:405` `a_handle_absent_admission_sweeps_the_orphaned_child` | handle-absent ⇒ `AdmissionRefused` + orphan sweep (RES-D4) |
| `contract_suite.rs:431` `a_spawn_failure_is_the_transport_class` | spawn failure ⇒ `AgentError::Transport`, no sweep |
| `contract_suite.rs:518` `the_parent_cwd_is_the_per_unit_workspace_not_a_shared_root`; `:546` two invocations get different cwds; `:581` `a_hostile_workspace_ref_refuses_admission_with_zero_spawn` | the execution anchor + fail-closed workspace validation (R-1b/RC4) |
| `contract_suite.rs:623` `an_unknown_handle_is_refused_by_every_operation` | typed `UnknownHandle` on all four ops |
| `contract_suite.rs:688` `an_answer_shaped_handle_field_refuses_admission_end_to_end` | a 5th handle field is refused (unrepresentability device) |
| `contract_suite.rs:734` `cancel_issues_exactly_one_stop_ever` | idempotent cancel |
| `contract_suite.rs:777` `a_late_identity_matching_writer_never_resurrects_a_timed_out_invocation` | terminal stability after `TimedOut` |
| `contract_suite.rs:901` `invoke_does_not_dispose_of_the_parent_at_admission`; `:927` `cancel_terminates_the_parent_and_treats_an_already_gone_child_as_success` | parent ownership (RC1) |
| `live_probe.rs:91` `one_real_admission_completes_through_the_result_contract`; `:311` `a_long_running_child_still_lands_its_result` | opt-in (`live_probe_requested()`) real `prime-agent` run: admission → contract → completion |
| `mcp_bridge_live.rs:32` `prime_agent_mcp_list_sees_the_provisioned_fabric_server` | the generated `settings.json` is actually picked up by the real harness |
| `src/program.rs:84,381` `the_rendered_program_is_byte_pinned`, `the_child_prompt_carries_the_whole_result_contract` | exact program bytes; prompt carries path, `.tmp` protocol, `role_result@v1`, invocation id, envelope digest, `GAUNTLET_RESULT_PATH` mention |
| `src/transport.rs:1358-1420+` (in-module, fake `--mode rpc` binary) | prompt-line write, prompt refusal (`success:false`), **cell refusal** (`agent_end` with no handle), late `agent_end` after admission, stdin-EOF shutdown |
| `src/runner.rs:1228` `poll_reports_nonzero_measured_payment_so_far`; `:1259` `collect_installs_the_max_of_reported_and_measured_payment`; `:1369/:1414/:1462` bridged agent-dir tests | metering + per-invocation MCP isolation |
| `src/config.rs:121` `defaults_are_conservative_and_typed` | `max_depth=1`, 600 s admission, `Spawn09`, bridge off by default |

### Architecture tests

- `/home/sprime01/projects/gauntlet/crates/gauntlet-arch-tests/src/delegation.rs:1-70` — the await-ban: rule A (no decode call outside `runner.rs`), rule B (decode only inside `fn poll`/`fn collect`), rule C (`AdmissionHandle` is exactly `["rlm_child_id","name","session_dir","model"]`), **with planted negative controls** (`:358-390`); executed by `the_prime_adapter_derives_no_result_from_the_delegation_call` (`:317`).


### Control loop / proofs — `crates/gauntlet-cli/tests/`

`p1_reference_integrity.rs`, `p2_context_isolation.rs`, `p3_real_observation.rs`, `p4_corrective_loop.rs`, `p5_hard_gate.rs`, `p6_budget_control.rs`, `p7_recovery.rs` (commit order + three §16.3 crash windows through a real aborting child + fresh controller), `p8_adapter_substitution.rs`, `p9_self_improvement_containment.rs`, `p10_no_false_learning.rs`, `p11_critic_is_not_verifier.rs`, `p12_missing_is_not_passing.rs`, `g6_hide_failed_observations.rs`, `g7_tier_downgrade.rs`, `g10_builder_writes_answer_key.rs`, `e2e_full_loop_mock.rs`, `e2e_full_loop_real_runner.rs`, `e2e_recovery.rs`, `e2e_budget.rs`, `fabric_cli.rs`, `fabric_acceptance.rs`, `cli_surface.rs` (frozen `cli_help.txt`, `cli_help_surfaces.txt`), `tool_program_codemode.rs`, `antigaming.rs`, `operator_actions_no_bypass.rs`.

Honest posture note: the "real runner" E2E uses the **generic `cli-agent` adapter** with a deterministic fixture child — not a frontier model and not the Prime adapter (`/home/sprime01/projects/gauntlet/crates/gauntlet-cli/tests/e2e_full_loop_real_runner.rs:1-33`).

### Workbench — `workbench/src/*.test.ts`

`prime-context-envelope.test.ts`, `prime-kernel.test.ts`, `prime-host.test.ts`, `prime-topology.test.ts`, `prime-recursion.test.ts`, `prime-long-running.test.ts`, `prime-cross-process-lifecycle.test.ts`, `prime-delegation-real.test.ts`, `prime-fresh-target.test.ts`, plus gated "real battery" suites (`prime-delegation-real-battery.test.ts`, `prime-cross-process-lifecycle-real-battery.test.ts`, `prime-message-continuation-real-battery.test.ts`), `real-prime-host.test.ts`, `domainforge-adapter.test.ts`, `domainforge-0-18-2-capture.test.ts`, `domainforge-0-19-0-capture.test.ts`, `sxr-evidence-adapter.test.ts`, `semantic-protocol-compatibility.test.ts`, `reality-trace-binding.test.ts`, `secret-leakage.test.ts`, `security-boundaries.test.ts`.

---

## Observed gaps (evidence, not inference from silence)

- The Prime child prompt says the contract path is named by `GAUNTLET_RESULT_PATH` "when the harness supplies it" (`program.rs:162-164`), but the adapter never sets that variable: `grep -rn 'RESULT_PATH' crates/adapters/gauntlet-adapter-agent-prime/src` finds only prose plus a prompt assertion (`program.rs:395`). The generic CLI adapter does set it (`crates/adapters/gauntlet-adapter-agent-cli/src/lib.rs:92`, `transport.rs:433`), so the Prime path relies on the literal path in the prompt alone.
- `PINNED_PRODUCER_VERSION = "0.16.0"` is declared (`domainforge-adapter.ts:75`) but never compared with the document's `producer.version` (only the **name** is checked at `:867`); the 0.18.2/0.19.0 capture tests depend on that leniency.
- `AdmissionHandle::FIELDS` and the arch test's `HANDLE_FIELDS` are two independent copies of the same four names (`transport.rs:83`, `arch-tests/src/delegation.rs:84`).
- `capability.rs` (980 lines: `SeamStatus`, `CapabilityReport`, `probe_capabilities`) is reachable only from the doctor path (`crates/gauntlet-cli/src/context.rs:174`), not from the runner construction in `compose.rs:231-262`, which wires only transport + control — so the behavioral probe never gates a real dispatch.
- An existing in-repo synthesis of Gauntlet × CognitiveOS exists but is explicitly non-canonical and stale relative to HEAD: `.agents/reports/gauntlet-cognitiveos-observer-reference-capability-dossier-v0.1.md:1-60` states "Tasks 0 through 5 are closed … Task 6 is next", and its companion zone-identifier file is a two-line `[ZoneTransfer]` stub (`.agents/reports/gauntlet-cognitiveos-observer-reference-capability-dossier-v0.1-zone-identifier.txt`).


---

## VERIFIED / UNVERIFIED

### Verified — directly observed in source/config at HEAD `6a7fe6a`

- **Q1**: file inventory; every export in `lib.rs:49-72`; the `--mode rpc` argv and per-invocation cwd; the single prompt JSON line and its field names; NDJSON stdout event parsing and the five handled event types; handle-file admission watch with bounded retry; `stop`/`list --json` control commands and the exact listing shape; the rendered program bytes and both dialects; the `<state_dir>/runs/<run>/inv/<id>/` persistence layout; `RUNNER_TOKEN = "prime-agent"` and its use in `compose.rs`.
- **Q2**: all six trait method signatures and their pinned semantics (admission promptness, terminal stability, cancel idempotency, deadline placement, ambiguity/reread rule); the full definition of `AgentError`, `InvocationFailure`, `InvocationStatus`, `RunnerCapabilities`, `Isolation`, `ResultDelivery`, `ContextEnvelope`, `RoleInvocationSpec`, `InvocationHandle`, `CancelReason`.
- **Q3**: schema token, exact field list, role taxonomy, budget/exempt roles, all four constructors and `envelopeFromJSON` strictness; producer/consumer call sites (`workbench-host.ts`, `prime-recursion.ts`); and the **absence** of CEP in that file (grep for `cep`/`profile` returned nothing).
- **Q4**: both wire-format pins and the six version constants; both parsers' key/schema/producer checks; the four `PROTOCOL_COMPATIBILITY` rows with their verbatim `incompatible` evidence strings; CEP-0008 wrapper recognition and diagnostics forcing; producer-version non-comparison; the 0.18.2/0.19.0 capture tests; the T14B evidence note.
- **Q5**: `ReferenceProvider`, `Verifier`, `EvidenceStore`, `StateStore`, `WorkspaceProvider`, `HumanEscalation`, `InvocationGraph` trait surfaces as read; the controller's documented loop order; reference-mutation terminality; ladder determinism and model-flag refusal; `verification_record@v1` key set; reconcile authority rules; the five settlement conditions in order; commit-idempotency and in-memory-last ordering.
- **Q6**: full dispatch table and help text; the 0/1/2 exit-code convention and `Settled => SUCCESS` mapping; `report.to_json()` on stdout for run/resume; `doctor.json`; `fabric serve` MCP framing, protocol versions, and the 60 s per-call bound; the unbound-host refusal; `admit-root` idempotency and printed `invocation_id`.
- **Q7**: `Cargo.toml`/`Cargo.lock` contain no SEA-Forge/CEP/DomainForge/RealityTrace crate (and the repo-wide text search finds only prose); README states the SEA-Forge-authority / Gauntlet-execution split; `PolicyProvider` is the named integration port; workbench `package.json` lists no such npm package.
- **Q8**: surface-rpc framing and limits; the normative codemode NDJSON protocol (`PROTOCOL.md`) and its Rust encoder/decoder; CLI JSON/key=value output and exit codes; the strict wire documents available to a foreign process; absence of any FFI/WASM target in the manifests.
- **Q9**: exactly one `rlm.spawn`/`rlm` per rendered program; the explicit no-loop/no-linger statement; `list` restricted to forensics; `max_depth` reporting only; `InvocationGraph` depth/child refusals; `agents.spawn` recursion with `child_budget`; the `RLM_DEPTH` session-binding barrier; the workbench's separate `RecursionPolicy`.
- **Q10**: every listed test path exists; the quoted assertion intents come from the tests' own names/doc comments as read.

### Not verified — inferred, or requiring execution I did not perform

- That `rlm.spawn` (Spawn09, the default) is the correct call for the *installed* `prime-agent` build — the crate defaults to it and `probe.rs` gates by an injected version range, but I ran no `prime-agent --version` and no live probe.
- That a real Prime child writes its result contract at the prompt-declared literal path without `GAUNTLET_RESULT_PATH` being set — the adapter depends on that, `live_probe.rs` is the (opt-in, unrun) evidence, and no test asserts the env var is set for this adapter (it is a stated gap above).
- The full runtime tool list `fabric serve` exposes — I read `agents.*` and the catalog wiring (`fabric_cmd.rs:44-55,825-827`, `agents_wire.rs`) but did not enumerate the composed catalog.
- Behavior of `improve`, `deliver`, `roles`, `skills`, `export`, `escalations` beyond their dispatch lines — command bodies were not read.
- CEP-0008 profile semantics beyond the captured fixture (`cep_version 0.1.0`, `envelope_kind semantic_snapshot`, `envelope_version 1.0.0`); the CEP specification is not in this repository.
- Any `docs/**` subsystem page — not read; the authoritative sources used here were the Rust/TS sources, `.agents/specs/Core-spec.md`, `README.md`, and `.agents/` evidence reports.
- No test suite, binary, or live harness was executed in this recon; every claim is a reading of source, spec, or recorded evidence.

