# Recon: SEA-Forge authority → settlement chain, and RealityTrace

Repo state: `/home/sprime01/projects/sea-rs` at `2f35ad27` ("docs(egm): bootstrap contract READMEs across the code tree (#104)"), clean tree.
All citations are absolute path:line. Excerpts are ≤6 lines.

**Headline:** SEA-Forge implements the authority loop with *two distinct settlement notions*:

1. a **local run settlement** (`sea-forge-settlement`) that produces a `SettlementEvent` and persists `settlement.json` into a run directory — the "process exit is not success" gate; and
2. a **CEP settlement packet** (`sea-forge-authority::cep`) that settles an *operation* against committed evidence, idempotently, in the `cognate-authority` ledger.

There is **no type named `SettlementReceipt`** anywhere in `crates/`. `/home/sprime01/projects/RealityTrace` **does not exist** as a working tree — sea-rs consumes RealityTrace as (a) a pinned **git dependency** `sxr-core` and (b) a local evidence *consumer* crate. Details in §6.

---

## 1. SEA-Forge authority loop (`authority_request` → `authority_decision`)

### 1.1 Where it lives (crate-by-crate)

Initial locator: `rg -n "authority_request" crates -g '*.rs' -l` →
`sea-forge-case-runner/{src/case_ops/mediation.rs,src/case_ops/mod.rs}`, `sea-forge-cli/{src/plan_pipeline.rs,src/pipeline.rs,src/commands/resume.rs}`, `sea-forge-authority/{src/cep.rs,src/cep/*}`, `sea-forge-server/{src/lib.rs,src/cep_authority.rs,src/config.rs}`.

| Layer | Crate | File:line | Role |
|---|---|---|---|
| Profile constants | `sea-forge-authority` | `/home/sprime01/projects/sea-rs/crates/sea-forge-authority/src/cep.rs:41-48` | `REQUEST_PROFILE="godspeed.authority_request"`, `DECISION_PROFILE="godspeed.authority_decision"`, `PROFILE_VERSION="1.0.0"`, `MAX_ENVELOPE_BYTES=64 KiB`, `LEDGER_ID="cognate-authority"` |
| Request parse (fail-closed) | `sea-forge-authority` | `cep.rs:542` (`parse_request`) | typed `ParsedRequest` (`cep.rs:193`); world binding checked against caller's `WorldRegistry` |
| Decision (evaluate+commit+envelope) | `sea-forge-authority` | `cep.rs:1069` (`CepAuthorityService::decide`) | parses, evaluates, commits `authority_decision`, returns envelope |
| Envelope builder | `sea-forge-authority` | `cep.rs:3651` (`decision_envelope`) | emits the CEP `authority_decision` |
| Server verb dispatch | `sea-forge-server` (async edge) | `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/lib.rs:2841` | identity gate, then `spawn_blocking(cep_authority::respond)` |
| Verb body | `sea-forge-server` | `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/cep_authority.rs:224-236` (`respond`) | loads policy + worlds, calls `decide` |
| Service assembly | `sea-forge-server` | `cep_authority.rs:149-186` (`with_service`) | refuses `cep_authority_disabled` unless enabled; fresh engine per request so escalation constraints cannot leak |
| Request enum | `sea-forge-server` | `lib.rs:905-912` | `AuthorityRequest { envelope: serde_json::Value, request_id: Option<String> }` |
| Governed-verb classification | `sea-forge-server` | `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/identity.rs:1032-1054` (`is_protected`) | `AuthorityRequest` is protected ⇒ verified socket identity required; the match is wildcard-free on purpose (`identity.rs:1026-1031`) |

Adjacent verbs on the same loop (`lib.rs:2874-3051`): `AuthorityApproval`, `AuthorityApprovals`, `AuthorityTransitions`, `AuthorityEvidence` (`lib.rs:927`), `AuthoritySettle` (`lib.rs:933`), `AuthorityClosure` (`lib.rs:939`).

### 1.2 Response shape

The wire reply is a plain JSON object, not a CEP document (`cep_authority.rs:227-234`):
```rust
Ok(out) => json!({
    "ok": true,
    "envelope": out.envelope,
    "decision_id": out.decision.decision_id,
    "ledger_entry": out.committed.entry_ulid(),
}),
Err(e) => refusal(e.class(), e),
```
Refusal shape (`cep_authority.rs:16-18`): `{"error": msg, "error_class": class, "no_side_effect": true}`. Classes (`cep.rs:157-171`): `cep_request_oversize`, `cep_request_invalid`, `cep_world_refused`, `cep_approval_refused`, `cep_evidence_refused`, `cep_settlement_pending`, `cep_transition_refused`, `cep_authority_internal`. Missing identity ⇒ `error_class: "identity_required"` (`lib.rs:2866-2869`) — never a default/local operator (`lib.rs:2865`).

Nested `envelope` = CEP `authority_decision` (`cep.rs:3720-3761`): `envelope_id="env-authority_decision-<operation_id>"`, `envelope_kind="authority_decision"`, `created_by="sea-forge-authority"`, `completeness_status="partial"`, `extensions["cep.profile"]={profile_id:"godspeed.authority_decision",profile_version:"1.0.0"}`, and `extensions["godspeed.authority_decision"]={decision, operation_id, request_ref, decision_id, ledger_entry, policy_bundle_hash, …}`. Boundary/degraded also carry `policy_basis`, `constraints`, and (degraded only) `compensating_controls`; an incoherent one is downgraded to `"unknown"` instead of emitted (`cep.rs:3682-3709`).

### 1.3 Transport: Unix socket NDJSON

- Socket file name: `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/config.rs:316` → `pub const SOCKET_FILE_NAME: &str = "server.sock";`
- Root: `config.rs:320` (`.sea-forge` default), `SEA_FORGE_ROOT` override `config.rs:366-371`, `SEA_FORGE_SOCKET` override `config.rs:378-380` (documented single escape hatch for cross-cell sockets). Resolved path at `config.rs:603`.
- **Socket creation:** `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/lib.rs:1732-1743` — bind `"<socket>.binding"` staging path, apply publication mode, then `rename` onto the live path. Rationale `lib.rs:1723-1731`: `bind()` derives umask permissions, so mode-after-bind leaves a connect window; staging+rename publishes a `0600` socket from first reachability and atomically replaces a stale one. `deny(unsafe_code)` rules out `umask` juggling.
- Publication mode `0600` default, `0660`+group when `socket_gid` set (`config.rs:250-259`); changing it on reload is refused (`config.rs:641-645`, `SOCKET_GID_RELOAD_REQUIRES_RESTART`).
- Windows twin: `lib.rs:1747-1767` — named pipe from `sea_forge_core::pipe_name_for`, DACL via `sea_forge_os::PipeSecurity` (SYSTEM + this user + cell SIDs), first-instance-only so a squatter cannot be joined.
- Framing: NDJSON — `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/sfwp/mod.rs:5` ("SFWP is layered *additively* on the existing Unix-socket NDJSON server").
- Admission bounded to 64 connections (`lib.rs:1774-1775`), separate from governed run permits.

---

## 2. Settlement

Two independent receipts exist. **Neither is named `SettlementReceipt`** and neither is called `settlement_receipt`.

### 2.1 Local run settlement — `SettlementEvent`, produced by `settle()`

Type (`/home/sprime01/projects/sea-rs/crates/sea-forge-core/src/types.rs:935-946`):
```rust
pub struct SettlementEvent {
    pub version: String,
    pub settlement_id: String,
    pub run_id: String,
    pub status: SettlementStatus,   // Accepted | Rejected | Escalated
    pub basis: Vec<String>,
    pub review_required: bool,
    pub settled_at: String,
    pub criteria_ref: Option<String>,  // skip_serializing_if none
}
```
`SettlementStatus` enum at `types.rs:920-926`; `SettlementClaim` input at `types.rs:883-901` (`run_id`, `plan_item_id`, `criteria_ref`, `criteria`, `execution: Option<ExecutionResult>`, `authority_verdicts: Vec<Verdict>`, `evaluator_scores: BTreeMap<String,f64>`, `batch: Option<BatchEvaluationResult>`, `write_only: bool`).

Producer: `/home/sprime01/projects/sea-rs/crates/sea-forge-settlement/src/lib.rs:17` — `pub fn settle(claim: &SettlementClaim, workspace: &Path, run_dir: &Path) -> Result<SettlementEvent, ForgeError>`. It is a deterministic basis machine, not a status trust: `authority_escalate`→`Escalated`+review, `authority_deny`→`Rejected` (`lib.rs:24-31`); then execution status `spawn_failed` / `timed_out` / `jail_violation` / `suspected_jail_violation` (the last "suspected, never definite" per F-20, `lib.rs:60-79`), `exit_zero`/`exit_nonzero`, `required_artifact_<path>:present|missing`, `stdout_match`/`stdout_mismatch`, `evaluator_score:<name>=<score>`, `batch_pass_ratio`/`batch_threshold`/`batch_passed` (`lib.rs:80-151`). `write_only` items settle on materialized artifacts and record an honest `write_only` basis rather than fabricate a process result (F-10, `lib.rs:33-58`). Legacy unattributed criteria get a `legacy_unattributed_criteria` basis (`lib.rs:156-158`).

Id minting: `lib.rs:159-174` — `settlement_id: ids::random_id("set")?`, with an in-code history note that this site previously returned a literal `set_01` so two settlements were indistinguishable (`lib.rs:161-166`). Other mint sites: `crates/sea-forge-case-runner/src/lib.rs:193,228`, `crates/sea-forge-case-runner/src/case_ops/advance.rs:309`, `crates/sea-forge-cli/src/plan_pipeline.rs:630`, `crates/sea-forge-self-model/src/store.rs:403`.

**Persistence:** `settlement.json` in the run directory. Writers: `/home/sprime01/projects/sea-rs/crates/sea-forge-cli/src/plan_pipeline.rs:648,843`, `crates/sea-forge-cli/src/pipeline.rs:657`, `crates/sea-forge-cli/src/commands/resume.rs:587,1053`. Declared bundle member: `/home/sprime01/projects/sea-rs/crates/sea-forge-cell/src/bundle.rs:23` (also `crates/sea-forge-cell/tests/projection_bundle_boundary.rs:20`). Read back by `crates/sea-forge-cli/src/commands/artifact.rs:179`, `commands/inspect.rs:56`, `commands/runs.rs:31`.
Failing batch records go to `<run_dir>/quarantine/<plan_item_id>.jsonl`, with the id validated as a single safe path segment (F-17) at `crates/sea-forge-settlement/src/lib.rs:118-141`.

**Idempotency:** none on this type. `SettlementEvent` has no idempotency key and `settlement_id` is random per call, so re-running `settle` mints a new event. Idempotence for this path comes only from re-writing the same `settlement.json` path. This is a real asymmetry with §2.2.




### 2.2 Ledgered operation settlement — `SettlementOutcome`

Producer: `/home/sprime01/projects/sea-rs/crates/sea-forge-authority/src/cep.rs:1702` (`settle_with`), wrapper `cep.rs:1692` (`settle`). Result type `SettlementOutcome` (`cep.rs:1421-1429`):
```rust
pub struct SettlementOutcome {
    pub status: &'static str,        // settled | rejected | unsettled
    pub final_status: bool,
    pub envelope: Value,
    pub committed: CommittedRecordRef,
    pub replayed: bool,              // true => returned an earlier final settlement
}
```
Committed as ledger `record_kind = "settlement_packet"`; payload under `extensions["godspeed.settlement_packet"]` (`cep.rs:1838-1916`) with `settlement_id = "set-<operation_id>-<set_hash>-<sha_short(question)>"`, `settlement_status`, `transformation_ref`, `question_ref`, `evidence_refs`, `threshold_basis` (`min_supporting=…; min_reliability=…; criteria=<criteria_sha256>`), `missing_evidence`, `unresolved_contradictions`, `operation_id`, `authority_decision_ref`, `criteria_sha256`, `overall_status`, `final`, `declared_by`. Threshold logic (`cep.rs:1797-1837`): any `contradicts` ⇒ `rejected`; `supports` at rank ≥ `min_reliability`, count ≥ `min_supporting` ⇒ `settled`; else `unsettled` with explicit missing-evidence text.

**Idempotency keys, per record kind:**

| Record kind | Key | Site |
|---|---|---|
| `authority_decision` (normal / escalation) | `escalation_key(op)` = `cep:<operation_id>` | `cep.rs:757`, `cep.rs:1183-1189` |
| `authority_decision` (approval consumed) | `use_key(apr)` = `cep-approval-use:<approval_id>` via `commit_typed_new` ⇒ approval usable once | `cep.rs:764-766`, `cep.rs:1152-1162` |
| `evidence_packet` | `cep-evidence:<envelope_id>` | `cep.rs:1674-1680` |
| `settlement_packet` (final) | `cep-settle-final:<operation_id>` via `commit_typed_new` | `cep.rs:1919-1926` |
| `settlement_packet` (non-final) | `cep-settle:<operation_id>:<set_hash>` | `cep.rs:1950-1964` |
| settlement approval resolve | `cep-approval-resolve:<approval_id>` | `cep.rs:761-763` |

`set_hash` = first 16 hex of SHA-256 over `|`-joined evidence ids (`cep.rs:1804-1810`; `sha_short` at `cep.rs:1976-1979`). On a final-key conflict the code re-reads the winner and returns it with `replayed: true` (`cep.rs:1928-1947`).

Ledger primitives: `/home/sprime01/projects/sea-rs/crates/sea-forge-ledger/src/types.rs:1217-1256` (`commit_typed_once`: same key + same payload hash + same subject/authority refs ⇒ returns the existing `CommittedRecordRef`; any divergence ⇒ `ledger_integrity_error: idempotency key payload conflict`) and `types.rs:1260-1292` (`commit_typed_new`: a matching key always conflicts). `idempotency_key: Option<String>` lives on `LedgerEntry` (`types.rs:225`); `CommittedRecordRef` fields at `types.rs:905-914`.

---

## 3. World reference (`WorldRef`)

The type is **not defined in sea-rs**: re-exported from a pinned external crate at `/home/sprime01/projects/sea-rs/crates/sea-forge-domainforge/src/lib.rs:24` (`pub use domainforge_core::application::world::WorldRef;`), pinned `domainforge-core = { version = "=0.22.0", default-features = false }` at `crates/sea-forge-domainforge/Cargo.toml:13`. `WorldRef`'s textual grammar (`from_identity`, `parse`, `verify_identity`) therefore lives outside this repo.

Who can mint — only `WorldRegistry`, from host-owned configuration (`/home/sprime01/projects/sea-rs/crates/sea-forge-domainforge/src/world.rs`):
- `register_source_set` (`world.rs:123-138`) — resolves the semantic envelope via `resolve_semantic_envelope`, then `DomainModelIdentity::from_document`.
- `register_identity` (`world.rs:142-154`) — `WorldRef::from_identity(name.parse()?, &identity)`; refuses a canonical ref already registered as *prior* (`IdentityRole`).
- `register_prior_identity` (`world.rs:163-183`) — superseded identities; must differ from every current world; re-registering the same prior is idempotent. Doc comment `world.rs:156-162` is explicit that the digest check proves the document and its ref *agree*, **not** that the document is authentic.

What is hashed: identity digest fields `content_hash` and `semantic_closure_hash` (`world.rs:290-301`); and in the server loader, a cache key over the config JSON **plus** every world file's SHA-256, so an edited file can never reuse a stale registry (`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/cep_authority.rs:40-62`). Loaded worlds are logged with both configured name and minted ref (`cep_authority.rs:103-108`); aggregate source bytes capped by `MAX_AGGREGATE_BYTES` (`cep_authority.rs:56-61`).

Verification:
- `require(world_ref)` — unknown ref fails closed, `WorldRefError::UnknownWorld` (`world.rs:215-225`).
- `verify_snapshot(envelope)` — reads `/scope/world_ref`, refuses `model_validation_status == "invalid"`, re-parses the carried `domain_model_identity`, re-verifies and compares to the registered identity (`world.rs:229-255`).
- `bind` / `verify_model_ref` — SEA-Forge's own `DomainModelRef` must carry the canonical ref and matching `d_content_hash` / `semantic_closure_hash` (`world.rs:258-301`).
- Registry is append-only with one role per world (`world.rs:94-106`).
- Prior-identity documents are path-validated and size-bounded at 64 KiB (`cep_authority.rs:26`, `cep_authority.rs:120-134`).



---

## 4. Decision envelope

There is **no Rust type named `DecisionEnvelope`**. The decision has three representations.

### 4.1 Kernel decision object — `AuthorityDecision` (the real authority record)

`/home/sprime01/projects/sea-rs/crates/sea-forge-core/src/types.rs:510-547`: `version`, `decision_id`, `run_id`, `plan_item_id`, `action_id`, `correlation_id`, `operation: AuthorityAction`, `outcome: Verdict`, `verdict: Verdict`, `normalized_disposition: NormalizedDisposition`, `matched_rule`, `reason_codes`, `reason`, `policy_refs`, `required_next_steps`, `identity_binding`, `determinism`, `action_request: AuthorityRequest`, `audit_record`, `decided_at`, plus `#[serde(default)]` optionals: `candidate_verdicts`, `winning_source`, `precedence_reason`, `sandbox_class_granted`, `approval_request_id`, `opaque_constraint_id`, `boundary_constraints: BTreeMap<String,Vec<String>>`, `compensating_controls: Vec<String>`.

Disposition is deliberately not a boolean — `cep_disposition` (`/home/sprime01/projects/sea-rs/crates/sea-forge-authority/src/cep.rs:3639-3649`) yields `deny | escalate | boundary | degraded | allow`, and an `Allow` verdict carrying a Deny/Escalate normalization is reported as `unknown` instead of being allowed.

### 4.2 Wire envelope — CEP `authority_decision`

Built only by `decision_envelope` (`cep.rs:3651-3766`). Fields: `envelope_id = "env-authority_decision-<operation_id>"` (so a replay names the same envelope), `cep_version "1.0.0"`, `envelope_version "1.0"`, `envelope_kind "authority_decision"`, `created_at`, `created_by "sea-forge-authority"`, `scope` (inherited from the request, with `evaluation_context: "godspeed-authority_decision"`), `boundary_record` (with `known_omissions: ["policy evaluation detail is in the committed ledger record"]`, `classification: "internal"`), `completeness_status "partial"`, `omission_status "marked"`, `provenance_refs`, `validation_status "valid"`, `lineage_refs` (request envelope id + approval lineage), `provenance[]` (`production_method: "authority_evaluation"`), `transformations` (inherited), `authority[] = [{authority_id, subject_actor_ref}]`, `constraints[]` (boundary/degraded only), `extensions` (`cep.profile` + `godspeed.authority_decision`).

### 4.3 Persisted form

Committed to ledger id `cognate-authority` (`cep.rs:46`); the stream is opened at `cep.rs:796` as `LedgerStream::open(root, LEDGER_ID, "sea-forge-cep-authority")` — under the **cell root**, not a run dir.

### 4.4 Who consumes it

- **The server** returns it over the socket (`cep_authority.rs:227-234`) and re-reads it from the ledger for evidence, settlement, approval and transition queries (`cep.rs:791,873,982,1499,1724,1934`).
- **The governed execution boundary** consumes the *kernel* decision, not the wire envelope: `emit_authorized_invocation` (`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/governed_execution_boundary.rs:470-486`) emits nothing unless `verdict == Allow && normalized_disposition == Allow`.
- **Cognate/clients** consume the wire envelope — asserted in `crates/sea-forge-authority/tests/cep_authority.rs:136,155,214`, `tests/cep_approvals.rs:130`, `tests/cep_settlement.rs:169,388`, `tests/cep_transition.rs:150,191`, and end-to-end in `crates/sea-forge-server/tests/conformance_cep_authority.rs:199,348,412,620`.

---

## 5. Evidence contracts

### 5.1 What must accompany settlement (CEP path)

`evidence_packet` validation (`/home/sprime01/projects/sea-rs/crates/sea-forge-authority/src/cep.rs:1582-1688`), failing closed in this order:
1. size ≤ `MAX_PACKET_BYTES` = 256 KiB (`cep.rs:47`, `:1588-1593`);
2. `envelope_kind == "evidence_packet"` (`:1595`);
3. profile exactly `godspeed.evidence_packet@1.0.0` (`:1598-1613`);
4. `scope.world_ref` must be registered (`:1614-1618`);
5. required ids present and bounded — `operation_id`, `authority_decision_ref`, `execution_trace_ref` (`:1621-1631`; `id_text` at `:1433-1442` caps at 256 bytes and rejects control chars);
6. `lineage_refs` must contain `execution_trace_ref` (`:1637-1641`);
7. `questions` must be a non-empty array (`:1642-1650`);
8. `evidence` must be a non-empty array; each item's `direction ∈ {supports, contradicts, unknown}` (`:1454-1489`);
9. a committed **allow** decision must exist for the operation, and `authority_decision_ref` + `world_ref` must match it (`:1655-1667`);
10. policy gate `"evidence_mutation"` for the operation as the verified caller (`:1668`);
11. corroboration against the evidence source (`:1669`).

`EvidenceItem` (`cep.rs:1445-1452`): `raw`, `id`, `question`, `direction`, `reliability`. `SettlementCriteria = {min_supporting: u32, min_reliability}` (`cep.rs:52-97`) is bound into the allow decision at decision time (`bind_criteria`, `:1011`) so settlement can never be judged against criteria other than those in force when the decision was made.

### 5.2 Corroboration — the anti-self-assertion rule

`CORROBORATED_FIELDS = ["evidence_id","evidence_type","question_ref","representation_ref","relevance","reliability","direction"]` (`cep.rs:1356-1364`). For each item whose `source` equals the configured source's name, `EvidenceSource::recorded_item` re-reads the source's own record; `direction` and `reliability` must exist in the *original*, and every corroborated field must be identical (`cep.rs:1366-1418`). With `required: true`, an item from any other source is refused (`:1379-1385`). A packet with any item taken on trust is never marked corroborated (`:1373-1375`, `:1417`), and `settle_with(required_source)` counts only packets tagged `corroborated:<source>` (`:1702-1708`) — so a packet committed before corroboration was required can never decide a settlement.

Trait: `EvidenceSource { fn name(&self) -> &'static str; fn recorded_item(&self, item: &Value) -> Result<Value, String> }` (`cep.rs:1334-1340`), implemented for RealityTrace in `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/realitytrace_source.rs`.



### 5.3 There is no `Observation` type in the kernel

`rg -n 'Observation' crates -g '*.rs' -l` hits only tests plus three server modules: `governed_execution_boundary.rs`, `governed_settlement_return.rs`, and `crates/sea-forge-realitytrace`. The "observation" concept is wire-level (`ExecutionObservation` envelopes) plus `ObservationOutcome`, not a `sea-forge-core` type:
- `ObservationOutcome` (`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/governed_execution_boundary.rs:633-652`): `Settled { invocation_id, work_request_id, authority_decision_id, execution_status, observed_effects, world_ref }` or `DuplicateDelivery`; `settled()` at `:654-659`. `authority_decision_id` and `world_ref` are taken from the **ledger**, never from the observation (`:639-646`).
- `InvocationLedger` (`:667-676`) binds each observation to the exact authorized invocation — same `invocation_id`, the recorded `authority_decision_id`, `work_request_id`, and the *current generation* of slot `(work_request_id, operation, resource)`; superseded-generation observations are late and never settle (`:661-676`). Duplicates are tracked in `settled_event_ids` / `settled_idempotency_keys` (`:670-671`).
- File effects are independently observed *outside* the workload; an unavailable observer is a rejected witness, never a successful effect (`:678-700`).

### 5.4 Where evidence is validated for settlement

- `evaluate_operational_settlement(outcome, declared_criteria)` — `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/governed_settlement_return.rs:291`; returns `SettlementEvaluation { status: SettlementStatus, basis, satisfied_criteria, unsatisfied_criteria }` (`:248-254`). Rule (`:277-290`): any non-`completed` status rejects outright; `completed` contributes **nothing** on its own, so an exit-zero run that did not attest every declared criterion settles `Rejected`.
- `rejected_file_observation` refuses a `file_write` effect unless `/resource_observation/status == "verified"` (`:256-275`).
- Return envelope `OperationalSettlement` with seven frozen required fields `["work_request_id","authority_decision_id","execution_status","observed_effects","operational_settlement_status","evidence_refs","domain_model_ref"]` (`governed_settlement_return.rs:41-49`); validated by `validate_operational_settlement_wire` (`:431`) before `emit_operational_settlement` (`:646`).
- Upstream ingress gate: `accept_governed_work_request` / `accept_verified_governed_work_request` (`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/governed_work_ingress.rs:318,344`) enforce exclusive producer authority (`swe_seed`), a real SHA-256 model hash (never the `sha256("agentic_capability_loop")` placeholder, `governed_work_ingress.rs:32-36`), semantic work intent, and context-packet causality.



---

## 6. RealityTrace

### 6.1 Does `/home/sprime01/projects/RealityTrace` exist?

**No.** `ls -d /home/sprime01/projects/RealityTrace` → `No such file or directory`. There is no RealityTrace working tree on this machine. sea-rs reaches RealityTrace by two other routes:

1. **Git dependency (pinned, external).** `/home/sprime01/projects/sea-rs/crates/sea-forge-realitytrace/Cargo.toml:13` →
   `sxr-core = { version = "0.3.0", git = "https://github.com/GodSpeedAI/RealityTrace", rev = "42879a82115bcc3586a81a58c614ec5073656f6f" }`, present in `Cargo.lock:3033`. The *Rust* RealityTrace library is fetched from GitHub, not present locally.
2. **An out-of-process CLI.** The server shells out to a `realitytrace` binary named in `cep_authority.realitytrace.bin` (`CepRealityTraceConfig`); it never links the daemon.

### 6.2 `/home/sprime01/projects/cognate/packages/realitytrace` — present, TS, `@godspeedai/cognate-realitytrace@0.10.0`

`/home/sprime01/projects/cognate/packages/realitytrace/package.json:2-3`: "Observation port, committed-outbox delivery, receipts, strict gating, and the SXR CLI adapter (spec §20). RealityTrace stays unchanged." ESM/TypeScript with `exports: { ".": "./src/index.ts" }`.

**Public API** (barrel `/home/sprime01/projects/cognate/packages/realitytrace/src/index.ts:1-3` plus module exports):

| Export | File:line | What it is |
|---|---|---|
| `Observation` (interface) | `/home/sprime01/projects/cognate/packages/realitytrace/src/types.ts:5-37` | framework execution observation |
| `DeliveryReceipt` | `types.ts:39-41` | `accepted` \| `duplicate` only — explicitly *not* evidence or settlement |
| `ObservationPort` | `types.ts:44-47` | `{ id, record(observation) => Promise<DeliveryReceipt> }` |
| `ObservationStatus` | `types.ts:50-59` | `queued`/`accepted`/`duplicate`/`retrying`/`failed` + `attempts`, `targetRef`, `lastError` |
| `OBSERVATION_TOPIC = "observation.deliver"` | `index.ts:7` | outbox topic |
| `observationIntent(event, extra)` | `index.ts:13-46` | builds the `Observation` **and** the outbox intent in the same decision (spec §13.1) |
| `ObservationDelivery` (class) | `index.ts:55-59` | at-least-once delivery from the committed outbox with persisted receipts + reconciliation status |
| `executionTraceEnvelope(input)` | `godspeed.ts:41` | CEP `execution_trace` |
| `evidencePacketEnvelope(input)` | `godspeed.ts:93` | CEP `evidence_packet` |
| `TraceInput` / `SxrEvidenceRow` / `EvidencePacketInput` | `godspeed.ts:27,66,71` | input/row shapes |
| `governedTraceFor(event)` | `godspeed.ts:130` | governed-run trace |
| `interactionEventFor` / `interactionOutboxFor` / `uuidV5` | `interaction-event.ts:52,108,16` | `godspeed.interaction_event` v1 |
| `createSxrCliPort` / `SxrCliOptions` / `SxrWorkflow` | `sxr-cli.ts:77,8,20` | adapter that shells to the SXR CLI |

**Observation envelope emitted** (`types.ts:5-37`): `observationId` (`obs-<eventId>`, derived from the source event so every retry is identical — `index.ts:23`), `eventId`, `tenant`, `correlationId`, `causationId`, `runId`, `scopeId`, optional `componentId`/`fiberId`/`capabilityId`/`continuationId`/`semanticRef`, `classification ∈ {reversible_effect, durable_fact, external_commitment}`, `inputRef?`, `outputRef?`, `timestamp`, `trace?` (the CEP `execution_trace` envelope — recorded content-addressed *instead of* the bare observation when present), `interactionEvent?` (`godspeed.interaction_event` v1, unsigned in S1).

Role separation stated in code at `/home/sprime01/projects/cognate/packages/realitytrace/src/godspeed.ts:1-6`: "Cognate operates and reports completion (execution_trace); RealityTrace observes and derives its own evidence; SEA-Forge settles. … settlement is not built here at all." Constants `cep_version "1.0.0"`, `envelope_version "1.0"`, `PROFILE_VERSION "1.0.0"` (`godspeed.ts:10-11`); `executionTraceId(op) = "env-execution_trace-<op>"`, `transformationId(op) = "tr-<op>"` (`godspeed.ts:24-25`).



### 6.3 Does sea-rs consume it?

**Not this TS package.** `rg -n 'cognate-realitytrace|@godspeedai'` over sea-rs (including ignored files, excluding `target/` and `node_modules/`) returns **no hits**. sea-rs consumes RealityTrace through two decoupled surfaces:

1. **`sxr-core` (pinned git).** `/home/sprime01/projects/sea-rs/crates/sea-forge-realitytrace/src/lib.rs:21-23` imports `sxr_core::canonical_json::sha256_canonical`, `sxr_core::evidence_emit::{emit_evidence_recorded, EvidenceEmission}`, `sxr_core::proof_ingest::{ingest_from, Admission, IngestParams, IngestState, ProofProducer}`. Schema `sea-forge.security-trace/v1` (`lib.rs:18`); pre-committed ledger kind `security_frozen_expectation` (`lib.rs:20`); proof producer pinned to `"sea-forge-security-acceptance"`, and RealityTrace refuses any other stamp *including* `sea-forge`, the system under test (`lib.rs:22-27`). Public functions: `expectation_record` (`:66`), `project` (`:118`), `proof_completed` (`:200`), `record_evidence` (`:286`), `record_category` (`:337`), `project_attempts` (`:377`). Ownership split per ADR-012 (`lib.rs:1-16`): SEA-Forge owns authority, enforcement, trusted resource observation and settlement; RealityTrace owns expectation-vs-observation comparison; **the auditor owns the verdict**; absent records are reported `unresolved`, never inferred.
2. **CLI subprocess, read-only.** `/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/realitytrace_source.rs:1-3` — "SEA-Forge never writes to RealityTrace and never takes a packet's direction or reliability on the requester's word." Commands: `repo list`, `ledger export --repo <r> --json` (`:100`), `ledger verify --repo <r> --json` (`:114`). Hardening: `env_clear()` then only `XDG_DATA_HOME`/`XDG_STATE_HOME`/`PATH` (`:38-45`), stdin null, stdout piped, stderr null, 60 s timeout (`:14`); a missing binary is an error, never a pass (`:317-319`).
   - `impl EvidenceSource` (`:137-180`): parses `sxr:<ledger_id>:<append_ordinal>` out of `provenance_refs` (`position`, `:130-141`), finds the row in an enrolled repo's export, **verifies that repo's chain first**, requires `record_kind == "evidence.created"`, then returns `/payload/evidence[evidence_id == …]`.
   - `impl ObligationSource::ledger_rows` (`:203-217`): the rows of one ledger, after chain verification.
   - `rows_of_ledger` (`:181-201`) fails the whole read if any row lacks a numeric `append_ordinal` or a string `record_kind` — deliberate, so a dropped row cannot hide verification evidence from settlement.
   - Config: `CepRealityTraceConfig { bin, data_home, state_home, require_corroboration }`, with `require_corroboration` defaulting **true** (`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/config.rs:176-193`). Absent section ⇒ no RealityTrace ⇒ obligation settlement and closure refuse with `cep_evidence_refused` (`cep_authority.rs:287-296`, `:323-328`).



---

## 7. Recipes and tests that prove the authority → evidence → settlement chain

### 7.1 Justfile recipes

| Recipe | Location | What it proves |
|---|---|---|
| `just proof` (P1–P4b) | `/home/sprime01/projects/sea-rs/justfile:865-925` | The **local** chain end-to-end, asserted with `jq` over real run artifacts: run dir holds `plan.json`, `authority.json`, `trace.jsonl`, `evidence.jsonl`, `settlement.json`, `semantic-envelope.json` (`:885-887`); `settlement.status == "accepted"` with non-empty `basis` (`:888`); `semantic-envelope.settlement_ref == settlement.settlement_id` with one artifact ref (`:890`); every `authority_decisions[]` id resolves in `authority.json` (`:891`); every `evidence_refs[]` id resolves in `evidence.jsonl` (`:892`); every authority record carries `determinism.policy_bundle_hash` matching `^sha256:[a-f0-9]{64}$` plus `audit_record.engine/disposition/subject` (`:897`); each artifact `sha256` verifies (`:898-904`); a **deny** run exits 3, all verdicts deny, **no `command_started`** trace event, empty workspace (`:905-920`); a **boundary** run exits 3 with `reason_codes` containing `generated_zone_denied` and no generated-zone file (`:921-924`) |
| `just e2e-gate T0x` | `justfile:194-340` | Per-task convergence gates: T04 (`:230`), T05 (`:238`), T06 (`:248-249`), T07 (`:261-262`), T08 (`:277-278`), T11 (`:315-316`), T12 (`:331-332`). Several arms fan out to other repos via `$SWE_SEED_ROOT`, `$SXR_ROOT` (default `$HOME/projects/sxr`), `$GODSPEED_AGENT_ROOT`, `$CONTEXT_KERNEL_ROOT` |
| `just e2e-check` / `e2e-test` / `e2e-lint` | `justfile:170-186` | Aliases bound to the real `just check` / `just test` / fmt+lint — orchestration, never a weaker duplicate |
| `just e2e-delta-check` / `e2e-delta-report` | `justfile:188-193` | Mechanical requirement/evidence matrix (`scripts/e2e-delta-check.sh`) |
| `just ci` | `justfile:804-818` | lint, test, msrv, workbench, macos, `authority-opa` (`justfile:1044`). It does **not** include `just proof` |

### 7.2 Authority/settlement tests — `crates/sea-forge-authority/tests/cep_settlement.rs`

Fixtures/helpers `rule`/`policy`/`cell`/`item`/`packet`/`allow`/`submit`/`settle`/`status` at `:29-195`. Assertions:
- `supporting_evidence_settles_once_and_the_packet_names_its_basis` (`:197`) — settles once; basis names the criteria.
- `an_execution_with_only_unknown_evidence_is_not_settled_and_can_be_settled_later` (`:235`) — `unsettled` stays re-evaluable.
- `contradicting_evidence_rejects_and_later_support_cannot_undo_it` (`:272`) — rejection is monotone.
- `reliability_below_the_minimum_does_not_count_and_the_count_is_honored` (`:311`).
- `evidence_hangs_only_from_a_committed_allow` (`:371`).
- `a_broken_chain_is_refused` (`:425`).
- `evidence_submission_is_idempotent_by_envelope_id` (`:513`).
- `settlement_is_judged_only_against_the_criteria_bound_at_allow_time` (`:530`).
- `policy_must_grant_evidence_and_settlement_and_a_refusal_is_recorded` (`:554`).
- `settlement_without_any_evidence_is_refused_not_invented` (`:588`).
- `criteria_are_validated` (`:597`).
- `cep_validators_accept_the_evidence_packet_and_every_settlement_status` (`:642`) — the wire contract schemas.
- Corroboration block (`:689-919`): `corroborated_evidence_settles_and_says_what_it_was_checked_against` (`:723`), `a_packet_that_flips_the_recorded_direction_is_refused_and_settles_nothing` (`:744`), `an_upgraded_reliability_or_an_unrecorded_item_is_refused` (`:771`), `when_corroboration_is_required_unsourced_items_are_refused_and_old_packets_do_not_count` (`:810`), `a_recorded_item_without_a_direction_cannot_be_corroborated` (`:891`).

Sibling suites: `cep_authority.rs` (decision shape/profile/`request_ref`/`policy_basis`, `:136,155,214`), `cep_approvals.rs` (`:130`), `cep_transition.rs` (world transitions + human floor, `:150,191,202`), `cep_obligation.rs` (RealityTrace-derived obligation settlement: `settle()` `:293`, "known zero is supported and settled once" `:352`, "before the window closes closure and settlement are pending" `:486`, class `cep_settlement_pending` at `:500,519,550`), `approval_consumption.rs` (`:99,134`), `cep_compiler_upgrade.rs` (`:132`), `promote_changeset.rs` (`:176`).



### 7.3 Server-level chain tests — `crates/sea-forge-server/tests/`

- `conformance_cep_authority.rs` — the socket-facing loop: decision `:199`, approval extraction `:348`, `approval_used` `:412`, approval id `:620`.
- `convergence_t04_governed_ingress.rs` — E4 ingestion gate.
- `convergence_t05_authorized_execution.rs` + `convergence_t05_execution_observation.rs` — E5A emission from a real decision; E5B observation adjudication.
- `convergence_t06_operational_settlement.rs` — `e6_full_governed_chain_settles_operationally_and_emits_the_frozen_envelope` (`:261`), `t06_tooth1_exit_zero_with_violated_criteria_records_failure` (`:365`).
- `convergence_t11_whole_loop.rs` — `t11_whole_loop_traceability_via_real_gates` (`:205`), `t11_duplicate_observation_is_idempotent` (`:300`), `t11_duplicate_settlement_via_replay_is_idempotent` (`:353`), `t11_late_observation_cannot_settle_against_wrong_invocation` (`:428`), `t11_wrong_domain_identity_is_rejected_at_ingress` (`:540`), `t11_failed_settlement_remains_observable` (`:559`), plus stage-9/11 world cases `:658-921` — notably `stage9_settlement_world_comes_from_the_ledger_not_the_observation` (`:768`) and `stage11_verified_intake_recomputes_the_world_and_fails_closed_on_an_unknown_one` (`:921`).
- `convergence_t12_variation.rs` — variation classes (success, authority_denial_or_escalation, proof_failure, …; enumerated in `justfile:320-322`).
- Also `e2e_world_loop.rs`, `security_acceptance_v1.rs`, `file_effect_observation.rs`, `conformance_authority_nonbypass.rs`, `conformance_identity.rs`, `conformance_transport_hardening.rs`, `conformance_run_locator.rs:141` (`settlement.json` shape).
- `crates/sea-forge-server/src/realitytrace_source.rs` in-file tests (`:225+`) drive a **fake `realitytrace` shell script** dispatching `repo list` / `ledger export` / `ledger verify` from fixture files (`:251-262`), and assert a missing binary is an error (`:317`).

---

## 8. Recursive agent execution / child runs / context decomposition

**Essentially absent.** Searches and what they returned:

| Search | Result |
|---|---|
| `rg -n -i 'recursive' crates -g '*.rs'` | Only (a) stack-safety notes on *iterative* replacements — `crates/sea-forge-planner/src/case_engine.rs:405` ("F-25.m: iterative DFS. The recursive form could overflow the stack"), `crates/sea-forge-planner/tests/conformance_m2.rs:557`; (b) the pest recursive-descent parser's depth cap — `crates/sea-forge-domainforge/src/lib.rs:210,226,426,434`; (c) filesystem walkers (`crates/sea-forge-cli/src/commands/migrate.rs:199,226,240`, `crates/sea-forge-cli/tests/conformance_m0_migrate.rs:66`); (d) key-sorting (`crates/sea-forge-server/src/sfwp/precondition.rs:92`); (e) a test rejecting a recursive YAML alias (`crates/sea-forge-authority/tests/policy_yaml_strictness.rs:370,406`). **No recursive run tree.** |
| `rg -n -i 'child_run\|sub_run\|subrun\|child run' crates -g '*.rs'` | 3 incidental hits, none a child-run concept: `crates/sea-forge-os/src/imp.rs:1411` (child *process*), `crates/sea-forge-server/tests/conformance_case_views.rs:351` (`case.json` written by "a child run (or a crashed episode)" — prose only), `crates/sea-forge-sandbox/tests/windows_local_job.rs:375`. |
| `rg -n -i 'decompos' crates -g '*.rs'` | 4 hits, all Unicode NFC/NFD decomposition in `crates/sea-forge-core/src/canonical.rs:98,103,110,113`. **No context decomposition.** |
| `rg -n -i 'subagent\|sub-agent\|delegat' crates -g '*.rs' -l` | 20 files, but these implement **flat, capped, single-level** agent delegation, not recursion: `crates/sea-forge-agent/src/delegation.rs:1-6` ("Delegation loop for governed `agent_task` execution (spec §16.1) … Never judges success — that is the settlement layer's job"), config `{model, instruction, max_turns, token_budget, max_output_tokens}` (`:17-27`), loop `while turns < config.max_turns` (`:267`), budget termination reporting `DelegationTermination::TurnCapExceeded` + `"token_budget_exceeded"` (`:332-340`, `:379`). Termination basis constants `cancelled`/`turn_cap_exceeded`/`agent_endpoint_error`/`acp_disconnect` at `/home/sprime01/projects/sea-rs/crates/sea-forge-core/src/types.rs:928-934`. |

No depth limit, fan-out, spawn-child-run, or context-splitting construct exists because no such construct exists to limit. Delegation is capped by turns/tokens only, and by construction a delegated agent cannot settle its own work.



---

## 9. Cross-platform containment mechanisms

Selection is one exhaustive `match` on `SandboxClass { Local, Jail, Isolated, Microvm, OpenShell }` (`/home/sprime01/projects/sea-rs/crates/sea-forge-sandbox/src/lib.rs:42-49`, `select_sandbox` at `:188-210`). Every unavailable path returns an error; nothing degrades silently.

### 9.1 Per-platform mechanism

| Platform | Mechanism | Citation |
|---|---|---|
| **Linux — `jail`** | **Landlock** LSM: read-write on `workspace_root`/`artifacts_root`, read-only on the rest of the filesystem; TCP `BindTcp`/`ConnectTcp` rules per `NetworkPosture`. Network is a *hard* requirement at `landlock::ABI::V4`; filesystem degrades best-effort on older kernels. Documented gap: UDP/raw sockets have no Landlock coverage through ABI v6. | `crates/sea-forge-sandbox/src/jail.rs:10-19`, `:21-39`, `:41-60` |
| **Linux — seccomp** | Seccomp filter refuses `io_uring_setup/enter/register`, `unshare`, namespaced `clone`, `setns` (`EPERM`) and `clone3` (`ENOSYS` so libc falls back to `clone`, whose flags are inspectable); x32 numbers refused too. Needs `no_new_privs`, which the Landlock restriction sets first; on an architecture without a filter definition, installation **fails** rather than running unfiltered. | `crates/sea-forge-sandbox/src/syscall_filter.rs:1-20` |
| **Linux — cgroup v2** | One `sea-forge-<owner pid>-<seq>` cgroup-v2 **leaf per spawn**, closing the F08 gap where a process group cannot reach a `setsid(2)` descendant. A single `mkdir` is the delegation gate — `EACCES`/`EPERM` means the root is not delegated to this process. | `crates/sea-forge-sandbox/src/cgroup.rs:1-20` |
| **Linux — `isolated`** | One exclusive subordinate host identity per execution: `setpriv` parent-death signal, `unshare` single-ID user namespace with pid/mount/ipc/net namespaces, then the `sea-forge-workload-exec` Landlock posture; the identity grant is withdrawn before release. "Any missing prerequisite fails closed; there is no fallback to a shared-UID class." | `crates/sea-forge-sandbox/src/isolated.rs:1-7` |
| **macOS — `jail`** | **App Sandbox**: each execution gets its own ad-hoc-signed copy of `sea-forge-jail-launcher` carrying `com.apple.security.app-sandbox` plus temporary exceptions for exactly the workspace and artifacts trees; the copy execs the workload, so the kernel applies the sandbox at exec and every descendant inherits it. **No network entitlement is requested** ⇒ all TCP/UDP/DNS denied; per-port grants are inexpressible, so `AllowTcpPorts` is **refused**. Reads outside the default system set are denied (stricter than Linux's read-only `/`). `codesign` must exist or preflight fails. | `crates/sea-forge-sandbox/src/jail.rs:826-880`, `:905-932` |
| **Windows — Local** | **Job Object** with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` only; the breakaway flags are deliberately omitted so `CREATE_BREAKAWAY_FROM_JOB` fails with `ERROR_ACCESS_DENIED`. Spawn is `CREATE_SUSPENDED` → assign → `IsProcessInJob` read-back → resume, so no process ever runs unassigned; timeout teardown polls the job empty. | `crates/sea-forge-os/src/imp.rs:1335-1369`, `:1371-1400`; result class at `crates/sea-forge-core/src/types.rs:586-589` |
| **Windows — control plane** | Named pipe instead of a Unix socket, DACL-admitted SIDs, first-instance-only. | `crates/sea-forge-server/src/lib.rs:1747-1767` |

Honest limits are typed rather than claimed away: `LifecycleContainment` (`/home/sprime01/projects/sea-rs/crates/sea-forge-core/src/types.rs:577-602+`) distinguishes `CgroupLeaf { kill_verified }`, `JobObject`, `ProcessGroup`, macOS's `GroupTerminatedDescendantsUnverified`, and `SandboxCagePersists`, and states descendant-set completeness is *not* claimed on macOS (`jail.rs:851-877`).



### 9.2 Explicit refusal path when a mechanism is unavailable

| Condition | Refusal |
|---|---|
| `jail` requested on a platform that is neither Linux nor macOS | `SandboxError::new("unsupported_sandbox_class_error", "jail backend is unavailable on this platform")` — at construction (`jail.rs:439-445`) and at `execute` (`jail.rs:474-481`); interactive variant at `jail.rs:416-423` |
| Landlock setup fails, or the kernel lacks network Landlock | `jail_unavailable` (`jail.rs:52-54`, `:143`, `:156`, `:347`, `:392`; class doc `jail.rs:10-17`) — never a silent degrade to `local` |
| `SEA_FORGE_REQUIRE_CGROUP=1` and no cgroup containment | `cgroup_unavailable`, spawn refused (`jail.rs:330-347`, `:598-618`; asserted at `crates/sea-forge-sandbox/tests/setsid_lifecycle.rs:656-676`). Without the env var a typed degrade is surfaced on the result instead |
| `isolated` requested off Linux | `workload_isolation_unavailable`, "isolated workloads require Linux" (`crates/sea-forge-sandbox/src/lib.rs:196-200`) |
| `microvm` requested | `unsupported_sandbox_class_error`, "microvm backend is not available" (`lib.rs:201-204`) |
| `openshell` selected by class alone | `openshell_binding_missing` — it must be bound to a DomainForge projection via `select_openshell` (`lib.rs:205-215`) |
| OpenShell cannot meet a Landlock hard requirement | refused with an error naming Landlock; nothing is created (`crates/sea-forge-sandbox/tests/openshell_stub.rs:831-840`) |
| Postgres resources configured on Windows | `workload_isolation_unavailable` — "postgres resources require the Linux database broker; this host cannot supervise them" (`crates/sea-forge-server/src/lib.rs:1701-1707`) |
| Seatbelt network cases on non-macOS CI | reported **skipped**, never passed (`crates/sea-forge-sandbox/tests/conformance_m1.rs:724-725`) |

---

## VERIFIED / UNVERIFIED

**VERIFIED — read directly this session, cited above:**
- `authority_request` / `authority_decision` profiles, `parse_request`, `decide`, `decision_envelope`, ledger commits and idempotency keys, error classes — `crates/sea-forge-authority/src/cep.rs`.
- Server dispatch of all six authority verbs, identity gate, `0600` staging+rename socket creation, Windows named-pipe twin — `crates/sea-forge-server/src/{lib.rs,cep_authority.rs,identity.rs,config.rs}`.
- `SettlementEvent` / `SettlementClaim` / `settle()` and the `settlement.json` writers; **no** `SettlementReceipt` type exists; **no** idempotency key on the local settlement type — `crates/sea-forge-core/src/types.rs`, `crates/sea-forge-settlement`, `crates/sea-forge-cli`.
- `WorldRef` re-export plus `WorldRegistry` minting/verification semantics, including the explicit "digest proves agreement, not authenticity" comment — `crates/sea-forge-domainforge/src/world.rs`.
- Evidence-packet validation order, `CORROBORATED_FIELDS`, and the `unsettled`/`settled`/`rejected` threshold logic — `cep.rs`.
- `ObservationOutcome` / `InvocationLedger` / `GovernedWorkRequest` / `OperationalSettlement` boundaries — `crates/sea-forge-server/src/governed_*.rs`.
- `/home/sprime01/projects/RealityTrace` does not exist; `sxr-core` is a pinned git dep; the cognate TS package's public API and `Observation` shape; **no** `@godspeedai` import anywhere in sea-rs.
- Justfile recipes `proof`, `e2e-gate`, `e2e-check/test/lint`, `ci`; the named test functions and their line numbers.
- Absence of recursive / child-run / context-decomposition constructs; the flat, turn-and-token-capped delegation loop.
- Per-platform containment and every refusal path in §9.

**UNVERIFIED — not established by this recon:**
- `WorldRef`'s textual grammar and hashing: it lives in the external `domainforge-core = 0.22.0` crate, which was not read.
- Whether `sxr-core` at rev `42879a8…` matches the `packages/realitytrace` TS `evidence_packet` shape — the two were not cross-checked.
- Any runtime behaviour: **no tests, builds, or `just` recipes were executed**. Every claim here is code reading.
- The actual contents of the run artifacts in §7.1 beyond what the `jq` assertions state.
- Whether the T07/T08/T09/T10/T12 gate arms currently pass — they fan out to repos (`SWE_SEED`, `sxr`, `godspeed_agent`) that were not inspected.
- Variants of `LifecycleContainment` past `types.rs:602` — that enum read was truncated mid-variant.

**Environment note:** in this checkout, `rg` invoked with no explicit path argument returned zero hits for patterns that demonstrably exist (`settle`, `world_ref`), while the same pattern with an explicit path succeeded. Every search above passed an explicit path. Treat bare-`rg` results here as unreliable.

