# RealityTrace (sxr) — Contract Recon

Target: `/home/sprime01/projects/sxr` @ `42f38ca8e0cedb7807309469fa2984d60f4248fd`
("ci(workflows): rust-cache, docs-only short-circuit, SHA-pinned actions (#20)").
Consumer: `/home/sprime01/projects/sea-rs` (SEA-Forge).

Workspace = 7 crates: `sxr-core`, `sxr-ledger`, `sxr-git`, `sxr-df`, `sxr-verify`,
`sxr-observed`, `sxr-cli` (`/home/sprime01/projects/sxr/Cargo.toml:2-10`).

Note on tooling: `rg` traversal in `sea-rs` silently returned zero hits for
strings that provably exist; all `sea-rs` results below were re-confirmed with
`grep -rn`. Do not trust a single negative `rg` pass in that tree.

## 1. Core observation / evidence data model

Crate: **`sxr-core`**. There is no struct literally named `Observation`; the
observation/evidence vocabulary is split between the frozen ledger row shape and
the CEP-shaped payload types in `sxr-core/src/types.rs`.

Version constants (`/home/sprime01/projects/sxr/sxr-core/src/types.rs:3-8`):

```
pub const SXROUTPUT_VERSION: &str = "1";
pub const SXR_RECORD_VERSION: &str = "sxr-record/v1";
pub const SXR_CANONICALIZATION: &str = "sxr-canonical-json/v1";
pub const SXR_HASH_ALGORITHM: &str = "sha256";
pub const CEP_VERSION: &str = "0.1.0";
pub const ENVELOPE_VERSION: &str = "1.0.0";
```

### 1a. The ledger row (the durable record)

`LedgerRow` — `/home/sprime01/projects/sxr/sxr-core/src/types.rs:73`. Doc comment
says field names deliberately mirror `sea-forge-ledger::LedgerEntry` so export is
a projection. Fields: `ledger_id`, `append_ordinal`, `entry_ulid`, `record_kind`,
`record_version`, `idempotency_key: Option<String>`, `subject_refs: Vec<String>`,
`canonicalization`, `hash_algorithm`, `payload: String`, `payload_hash`,
`previous_entry_hash: Option<String>`, `entry_hash`, `committed_at`,
`writer_identity_ref`.

Positions: `LedgerPosition { ledger_id, append_ordinal: u64 }`
(`types.rs:21`). `LedgerHead { position, entry_hash }` (`types.rs:298`).

### 1b. The observation/evidence payload types

`Evidence` — `/home/sprime01/projects/sxr/sxr-core/src/types.rs:207`. Doc:
"CEP-0006 §6.1 Evidence fields; every field must be present, never defaulted."
Fields: `evidence_id`, `evidence_type`, `question_ref`, `target_claim_ref`,
`target_entity_ref`, `representation_ref`, `source`, `provenance_refs`,
`relevance`, `reliability`, `direction`, `confidence_impact`, `timestamp`,
`scope`, `limitations`.

`EvidenceRow` (`types.rs:280`) is the projected SQL slice: `evidence_id`,
`run_id`, `question_id`, `evidence_type`, `target_kind`, `target_ref`,
`relevance`, `reliability`, `direction`, `confidence_impact`,
`representation_ref`, `ledger_id`, `append_ordinal`.

Neighbouring state rows: `SnapshotRow` (`types.rs:187`), `SettlementRow`
(`types.rs:265`), `SettlementStatus` closed list (`types.rs:227`),
`SemanticContext` resolved/failed sum (`types.rs:127`), `DeclKey` (`types.rs:38`),
`OmissionRecord` (`types.rs:112`).

The record-kind vocabulary is CSV-generated, not hand-written:
`/home/sprime01/projects/sxr/sxr-core/src/record_kinds.rs:10` is
`include!(concat!(env!("OUT_DIR"), "/record_kinds_gen.rs"))` over
`/home/sprime01/projects/sxr/sxr-core/src/record_kinds.csv:16` —
`observation.recorded|execution_trace|Observation + referenced Representations|…|yes only after qualification|runtime-authoritative observation`
and `:17` — `evidence.created|evidence_packet|CEP Evidence|…`.

### 1c. Append / idempotency semantics (sxr-ledger)

Crate **`sxr-ledger`**, `/home/sprime01/projects/sxr/sxr-ledger/src/ledger.rs`.

- Header: "append-only rows, dense ordinals, idempotent append, hash-chain
  verification, WAL pragmas, and the `user_version` schema guard"
  (`ledger.rs:1-3`).
- `PendingEntry` (`ledger.rs:42`): `record_kind`, `idempotency_key:
  Option<String>`, `subject_refs`, `payload`, `entry_ulid`, `committed_at`.
- `AppendOutcome` (`ledger.rs:60`) is a two-way sum — `Committed { row, position }`
  or `AlreadyCommitted { position }`. So a re-append is a *reported duplicate*,
  not an error and not a second row.
- `append_batch` (`ledger.rs:287`) runs one `TransactionBehavior::Immediate`
  transaction; idempotency is checked **first**: "`idempotency_key` that already
  exists returns the existing position without inserting (AC-INGEST-002)"
  (`ledger.rs:285-287`, check at `:298-301`).
- `position_by_idempotency_key` (`ledger.rs:272`) exists so spool drain/recovery
  distinguishes duplicate replay from fresh commit **without parsing error text**.
- Dense ordinals are enforced by the writer holding the lock; a duplicate insert
  on the same key is diagnosed as "writer lock must be held" (`ledger.rs:385`).
- `VerifyReport { ledger_id, verified_count, first_divergence: Option<LedgerPosition>, complete, limitations }` (`ledger.rs:75`) is the §10.6 verification report shape.
- `writer_identity_default()` = `sxr@<host>/<pid>`; "Hostname is observed
  metadata, never identity" (`ledger.rs:36-40`).

## 2. CLI surface (sxr-cli)

Binary name: **`realitytrace`** (`/home/sprime01/projects/sxr/sxr-cli/Cargo.toml:13-15`:
`[[bin]] name = "realitytrace"`); package name is `realitytrace`, crate short-name
`sxr-cli` (`Cargo.toml:2`). Version 0.5.0.

Global flag: `--json` — "Emit machine-readable JSON on stdout (always on for
slice 1)" (`/home/sprime01/projects/sxr/sxr-cli/src/main.rs:30-32`). **JSON is the
output format for effectively every command.** Errors go to stderr as
`prog: {err:#}` (`main.rs:746-749`) and use §15 exit codes.

Exit-code contract (`main.rs:1-3`): `0 ok, 2 usage, 3 precondition,
4 CEP validation, 5 integrity, 6 verifier, 75 spooled`. Mapping implemented in
`exit_for_error` (`main.rs:783-806`): `CHAIN_VERIFY_FAILED`/`TAIL_TRUNCATED` → 5;
`AC-SETTLEMENT`/`AC-EVIDENCE`/`AC-DIAG-001`/`precondition`/`predecessor rule`/
`not found`/`not enrolled`/`no semantic configuration`/`queued_for_commit`/
`spooled`/`not committed`/`corrupt segments` → 3; else 2. Note this is
**substring matching on error text**, not typed dispatch.

JSON envelope: every output document begins with `sxr_output_version: "1"` and
always carries `unknowns` and `omissions` arrays, even when empty
(`/home/sprime01/projects/sxr/sxr-cli/src/output.rs:1-22`).

### 2a. Top-level subcommands (verbatim variant/CLI names)

From `/home/sprime01/projects/sxr/sxr-cli/src/main.rs:39-115` (clap
`#[derive(Subcommand)]`, so `Observed` → `observed`):

`repo`, `semantic`, `snapshot`, `question`, `threshold`, `run`, `plan`,
`contract`, `verify`, `evidence`, `claim`, `settlement`, `ingest`, `ledger`,
`query`, `projections`, `spool`, `await`, `doctor`, `queue`, `daemon`, `delta`,
`diagnostic`, `residual`, `explain`, `context`, `observed`.

### 2b. Second-level subcommands

- `repo` → `enroll <path>`, `list`, `identity <repo_id>` (`main.rs:306-311`)
- `semantic` → `configure`, `status` (`main.rs:313-325`)
- `snapshot` → `create` (`main.rs:327`)
- `question` → `declare`; `threshold` → `declare` (`main.rs:335,345`)
- `run` → `begin`, `finish` (`main.rs:357`)
- `plan` → `create` (`main.rs:379`)
- `contract` → `import` (`main.rs:409`)
- `verify` → `run` — "Execute the plan's verifier (cargo-test or command); other
  verifier ids are refused" (`main.rs:419-421`)
- `evidence` → `create`, `for-claim`, `for-hypothesis` (`main.rs:427`)
- `residual` → `compute`; `diagnostic` → `enumerate`, `settle`, `propose`, `status`
- `claim` → `create`; `settlement` → `register`
- `ledger` → `verify`, `export`, `history` (`main.rs:584-598`)
- `query` → `evidence` (`main.rs:600-608`)
- `projections` → `status`, `rebuild`, `export`, `export-anti-collapse-corpus`
  (`main.rs:610-637`)
- `spool` → `status`, `drain`; `queue` → `recover` (`main.rs:639,678`)
- `observed` → `ingest`, `derive`, `compare`, `propose`, `regression`,
  `capability-delta` (`main.rs:118-300`)

### 2c. How an external process records an observation

Two ingress paths, both NDJSON, both idempotent:

1. **`observed ingest --repo <id> --file <ndjson|->`** — "Classify CEP
   `execution_trace` envelopes carrying
   `extensions[\"godspeed.interaction_event\"]` (NDJSON, `-` for stdin) and record
   each as `observation.recorded` or `ingest.rejected`. **Exits 4 if any line was
   rejected** (the rejections are recorded)" (`main.rs:119-125`). Dispatch reads
   `-` from stdin (`main.rs:1048-1055`).
2. **`ingest --run <id> --verifier <id> --stdin-format <fmt>`** — verifier stream
   from stdin (`main.rs:88`, struct at `:560-582`), with `--spool` (exit 75,
   "never a silent fallback on lock contention"), `--async-mode` (daemon queue,
   "exit 0 with `queued_for_commit` only after a fsynced receipt; transparently
   falls back to a direct commit when the daemon is unavailable (F-15)"), and
   `--extensions` (CEP §39 caller extensions, "Not part of the idempotency key;
   the `domainforge` key is reserved").
   `--stdin-format agent-event-v1` selects the agent-event parser
   (`/home/sprime01/projects/sxr/sxr-cli/src/agent_event.rs:18`), whose
   `schema` per line must be `agent-event/v1` (`:20`). Batch is
   **all-or-nothing**: "Every line is validated before anything is appended"
   (`agent_event.rs:4-7`); the envelope "carries `payload_ref` only: unknown
   fields are refused, so raw transcript text cannot ride along under a stray
   key" (`agent_event.rs:7-9`).

### 2d. How an external process verifies one

Read-only, JSON-array output, no writes:
`realitytrace ledger verify --repo <id> --json` → `VerifyReport`
(`ledger.rs:75`) with `verified_count`, `first_divergence`, `complete`,
`limitations`; `realitytrace ledger export --repo <id> --json` → array of
`LedgerRow`-shaped JSON; `realitytrace ledger history --repo <id> --json`.
This triple (`repo list`, `ledger export`, `ledger verify`) is exactly the
contract SEA-Forge consumes (§7).

## 3. sxr-verify — what "verification" means here

**Verification in `sxr-verify` does NOT mean signatures or attestations. It means
"run an external process and record what it observably did."**

`/home/sprime01/projects/sxr/sxr-verify/src/lib.rs:1-3`:
"`VerifierPort` (§7.1, §14.1–14.3): process runners that return **observations
only — no evidence, relevance, reliability, or settlement vocabulary**."

Registered verifiers (`/home/sprime01/projects/sxr/sxr-verify/src/verifier.rs`):

- `VERIFIER_ID = "cargo-test"`, version `"1"`, requires `Cargo.toml` present
  (`verifier.rs:16-18`), parses `cargo-json`.
- `COMMAND_VERIFIER_ID = "command"` (`verifier.rs:63`): "runs exactly the argv a
  plan declares (optionally under a launcher), in a cleared environment… No
  output parser: **the exit code is the result**" (`verifier.rs:61-65`).

Refusal paths (`verifier.rs:47-56`): `VerifierError::UnsupportedVerifier(id)`
when `step.verifier_id != VERIFIER_ID`, and `VerifierError::NotARepo(cwd)` when
`cwd.join("Cargo.toml")` is not a file. Error enum:
`/home/sprime01/projects/sxr/sxr-verify/src/error.rs`.

Two digest mechanisms define "what exactly was verified":

- `tree_digest(root) -> "sha256:<hex>"` over sorted lines
  `<relative path>\0<kind>\0<sha256 hex>\n`, one per regular file (`f`) and
  symlink (`l`, hash of target path, **never followed**), skipping every `.git`
  directory (`/home/sprime01/projects/sxr/sxr-verify/src/digest.rs:1-27`).
- `port.rs` supplies `direction_for`, `DIRECTION_RULE_REF`, `filtered_env`
  (strips cargo recursion vars and internal telemetry tokens —
  `/home/sprime01/projects/sxr/docs/subsystems/sxr-verify.md:63`).

The **cryptographic** verification lives in `sxr-core::chain`, not `sxr-verify`
(`/home/sprime01/projects/sxr/sxr-core/src/chain.rs:13-24`):

```
entry_hash(n) = sha256( canonical_json({
    ledger_id, append_ordinal, entry_ulid, record_kind, record_version,
    idempotency_key, subject_refs, canonicalization, hash_algorithm,
    payload_hash, previous_entry_hash, committed_at, writer_identity_ref
}) )
```

`payload` is excluded and bound via `payload_hash`; `previous_entry_hash` is
null only at ordinal 1 of each generation. `verify_entry_hashes` (`chain.rs:46`)
recomputes it; `verify` "names ledger_id and ordinal on divergence"
(`chain.rs:179`); `verify_from` accepts a tail window anchored on its predecessor
(`chain.rs:191`).

So: **integrity = SHA-256 hash chain over canonical JSON. No signatures in the
ledger path.** Signature-shaped concepts exist only as an *ingest hook*:
`RejectCode::Signature` → `INGEST_REJECTED_SIGNATURE`
(`/home/sprime01/projects/sxr/sxr-observed/src/ingest.rs:30-47`), and an unsigned
event is recorded as `unsigned`, not rejected
(`/home/sprime01/projects/sxr/sxr-observed/tests/wp16.rs:210`).

## 4. sxr-df — binding to DomainForge semantic artifacts

**`sxr-df` is not a DomainForge implementation. It is a subprocess adapter that
shells out to a `domainforge` binary and hashes/verifies what comes back.**

Implemented trait: `SemanticSourcePort` (§7.1)
(`/home/sprime01/projects/sxr/sxr-df/src/lib.rs:1-2`). Concrete type
`CliDomainForgeAdapter` (`/home/sprime01/projects/sxr/sxr-df/src/adapter.rs`).

### 4a. What it reads

`SemanticContextRequest` (`adapter.rs:38-60`): `semantic_root: PathBuf` (the
process `current_dir`, keeping the logical entry path stable), `entry_logical_path`
("Never absolute: the producer embeds it in the envelope scope verbatim"),
`registry_path`, `default_namespace`, `semantic_packs: Vec<(String, String)>`
passed as `--pack id=hash`, `cep_scope: Value` passed as `--scope`, and an
optional CAS store root.

Invocation (`adapter.rs:140-155`):

```
domainforge envelope --emit both [--pack id=hash]... [--default-namespace ns]
           [--registry path] --scope <canonical-json scope> <entry_logical_path>
```

Note `--emit both`, not `--emit cep`: the adapter wants D *and* the envelope.

`capabilities()` (`adapter.rs:119-126`): `producer: "domainforge (Slice 0B)"`,
`cep_envelope_mode: "semantic_snapshot"`, `supports_semantic_packs: true`,
`supports_default_namespace: true`, `verifies_d_hashes: true`.

Pinned producer version: `PINNED_DOMAINFORGE_VERSION = "0.22.0"`
(`/home/sprime01/projects/sxr/sxr-df/src/lib.rs:11`), with a test asserting the pin
so "an accidental edit cannot silently re-point the enforced dependency"
(`lib.rs:19-31`). Enforced only when a caller opts in via
`with_required_version` (`adapter.rs:95-116`).

### 4b. What it hashes

`verify_d_hashes` (`adapter.rs:569-598`) — two independent SHA-256 checks over
canonical JSON:

- `content_hash` = `sha256(canonical_json(D without self_hash))`
  (`adapter.rs:580-590`). "`self_hash` is excluded from its own digest, exactly
  as… AC-REPLAY-001" (`adapter.rs:407-411`).
- `semantic_closure_hash` = `sha256(canonical_json(envelope))`
  (`adapter.rs:591-596`).

CAS carriage (`adapter.rs:360-430`): `content_ref` must be `cas://sha256:<hex>`,
and the address hash **including the `sha256:` prefix** must equal `content_hash`
exactly; an expired `content_availability` is a hard error. "the adapter never
silently falls back to a different snapshot or re-resolves different semantics"
(`adapter.rs:371-374`).

### 4c. What it rejects

`SemanticError` (`/home/sprime01/projects/sxr/sxr-df/src/error.rs:4-22`):
`BinaryUnavailable`, `UnsupportedVersion { expected, found }`,
`UnexpectedExit { code, stderr }`, `MalformedOutput`,
`EnvelopeSchemaViolation` ("envelope fails the pinned CEP schema (§52/§53)"),
`HashMismatch`, `Io`.

Classification is **by envelope shape, never by exit code** (`adapter.rs:1-19`):
`domainforge envelope --emit cep` exits 0 for a fully valid model and 1 for *any*
validation failure, but those split into two shapes —

- D was constructed (`validation_status: "validated"` + `representations`) →
  `SemanticContext::Resolved`, whatever the diagnostics say;
- D could not be constructed (`conformance_status: "non_conformant"`, no
  representations, missing-D omission) → `SemanticContext::ValidationFailed`
  carrying `invalid_declared_checkpoint_hash` and a `representation_omission`.

"Any other shape is an error (§10.4a: malformed output is never misclassified
into the closed sum)." The `ValidationFailed` arm "NEVER fabricate[s] semantic
hashes" (`/home/sprime01/projects/sxr/sxr-core/src/types.rs:122-126`).

## 5. sxr-observed — what "observed" means

Crate doc: "Observed representation: ingest classification and deterministic
derive. No I/O except `df_cli` (spawns the DomainForge binary); the CLI owns the
ledger." (`/home/sprime01/projects/sxr/sxr-observed/src/lib.rs:1-3`).

### 5a. observed ≠ asserted ≠ recorded

- **Recorded** = a row exists in the hash-chained ledger (durable, verifiable).
- **Asserted / declared** = the *expectation* side, held **outside** the ledger
  as a frozen document: the declared model (`.sea`), the frozen reference
  `{episode_ref, body}` passed as `--reference`, the pinned comparison profile,
  the frozen held-out rule, the frozen capability definition. Each is bound by
  a digest supplied "from outside (the pre-registration)"
  (`/home/sprime01/projects/sxr/contracts/heldout-rule.v1.json` description;
  `contracts/capability-frozen.v1.json`).
- **Observed** = computed *only* from verified ledger records. Contract
  description (`/home/sprime01/projects/sxr/contracts/observed-representation.v1.json`):
  "What `observed derive` computes from verified records only: a directly-follows
  graph over declared and undeclared elements with, for every node and edge, the
  occurrence ids behind it. Deterministic canonical JSON (RFC 8785)".

`ingest.rs` is explicit that classification is not observation:
"Nothing here counts a rejected event as an observation" (`ingest.rs:6-7`); a
rejected event is returned with a code and the caller records it as
`ingest.rejected`.

### 5b. Types

`Derived` (`/home/sprime01/projects/sxr/sxr-observed/src/derive.rs:92`):
`representation: Value` (validates against the contract schema), `document: Value`
(representation plus `representation_id`, `input_set_sha256`, window),
`canonical: String` (what the CLI prints), `representation_id: String`.

`Options` (`derive.rs:62-89`) — the knobs that make observation auditable:
`world_ref`, `scope` (`case:<id>`), `episode_ref`, `kind` (`evaluated` |
`exploratory`), `profile_id`, `until: Until`, `declared_keys:
Option<BTreeSet<String>>` ("An observed key outside the set becomes an
`undeclared:` node. `None`: every observed key is declared"),
`coverage_manifest: Option<Vec<(String, String)>>`, `independent_systems:
BTreeSet<String>` ("Source systems known independent of the executor… Empty: no
occurrence counts as independent"), `expected_head: Option<ExpectedHead>`,
`events_world_ref` (re-classification, AT-13).

`Occ` (private, `derive.rs:104-120`) carries `authority: &'static str` with values
`"decided"` / `"not_required"` / other, plus `independent: bool` and
`entry_hash: String` — i.e. **authority status and independence are derived
attributes of each occurrence, not claims the emitter makes**.

`ExpectedHead` (`/home/sprime01/projects/sxr/sxr-observed/src/tail.rs:16-21`)
and its doc is the sharpest statement of the epistemic limit in the repo:
"A hash chain proves that no row was changed or removed from the middle. It
cannot prove that rows were not removed from the END: a chain that ends one row
earlier is itself a valid chain (CEP-0008 section 55). The only defence is a head
recorded elsewhere… Without it the result says `unverified`; it is never reported
as verified." (`tail.rs:1-7`).

### 5c. The comparison refuses false positives

`compare.rs` (`/home/sprime01/projects/sxr/sxr-observed/src/compare.rs:1-21`):
"Missing evidence is never success: a frozen evidence source that emitted nothing
makes its expectations `insufficient` / `evidence_source_silent`, and a
`confirmed` without a single occurrence from an independent emitter is
`insufficient` / `executor_only_evidence`." Classes: `confirmed`,
`contradicted`, `observed_undeclared`, `declared_unobserved`, `insufficient` (with
bases `not_evaluable`, `below_minimum_support`). A vacuous truth "cannot be
written as `confirmed`; it stays insufficient" (`compare.rs:325-327`).

## 6. contracts/ and schemas/ — wire contracts and versions

### 6a. `/home/sprime01/projects/sxr/contracts/` (FDE comparison contracts, WP-11)

"JSON Schema draft 2020-12, every object closed (`additionalProperties: false`).
Hand-authored. A change to a schema file needs a new line in `DIGESTS` in the
same commit… consumers vendor by digest"
(`/home/sprime01/projects/sxr/contracts/README.md:3-5`).

| File | `$id` |
|---|---|
| `capability-delta.v1.json` | `https://sxr.dev/contracts/capability-delta.v1.json` |
| `capability-frozen.v1.json` | `https://sxr.dev/contracts/capability-frozen.v1.json` |
| `comparison-profile.v1.json` | `https://sxr.dev/contracts/comparison-profile.v1.json` |
| `heldout-rule.v1.json` | `https://sxr.dev/contracts/heldout-rule.v1.json` |
| `observed-representation.v1.json` | `https://sxr.dev/contracts/observed-representation.v1.json` |
| `regression-matrix.v1.json` | `https://sxr.dev/contracts/regression-matrix.v1.json` |
| `residual.v1.json` | `https://sxr.dev/contracts/residual.v1.json` |
| `revision-proposal.v1.json` | `https://sxr.dev/contracts/revision-proposal.v1.json` |

Plus `contracts/DIGESTS` — 8 lines of `<sha256>  <filename>`
(`contracts/DIGESTS:1-8`), e.g. `d662cc73…88a7c7  comparison-profile.v1.json`.
And `contracts/fixtures/<contract>/` for valid/invalid cases
(`contracts/README.md:7-14`: 3/15+1 for comparison-profile, 3/17+8 for
observed-representation, 7/24+0 for residual, …).

Conventions (`contracts/README.md:21`): "Hashes are `sha256:<64 lowercase hex>`.
Ids follow E §2: `world_ref`, `episode_ref`, `concept:<uuidv5>` declaration keys,
`undeclared:<sha256>` keys, `question:`, `evidence:`, `residual:`, `profile:`. An
occurrence id is the source record's key `(system, record_id)` written
`<system>:<record_id>`. Documents are stored as RFC 8785 canonical JSON."

### 6b. `/home/sprime01/projects/sxr/schemas/` (top level)

| File | `$id` / title |
|---|---|
| `cep-semantic-envelope.schema.json` | `https://cep.dev/schemas/cep-semantic-envelope.schema.json` — "CEP-0008 Semantic Envelope (canonical full profile)" |
| `sxr-output-v1.json` | `https://sxr.dev/schemas/sxr-output-v1.json` — "sxr CLI output document (v1)" |
| `sxr-plan-v1.json` | `https://sxr.dev/schemas/sxr-plan-v1.json` — "sxr VerificationPlan (v1)" |
| `sxr-record-v1.json` | `https://sxr.dev/schemas/sxr-record-v1.json` — "sxr ledger row contract (v1)" |
| `cep-semantic-envelope.mapping.md` | field-by-field CEP-0008 → v1 mapping + "the v1 field-set gap" |

Also `sxr-core/schemas/cep-semantic-envelope.schema.json` (compiled into the
binary via `sxr_core::embedded_schema`), and `sxr-observed/schemas/`, which
**vendors duplicates** of all 8 contracts plus two it does not own —
`evaluate-facts.v1.json`, `evaluate-output.v1.json` (WP-12: "this repository only
names them and carries no schema for them" — `contracts/README.md:19`) — and
`godspeed.interaction_event.v1.json`, loaded by
`include_str!("../schemas/godspeed.interaction_event.v1.json")`
(`/home/sprime01/projects/sxr/sxr-observed/src/ingest.rs:24`).

### 6c. CEP profile references — yes, extensive

- `CEP_VERSION = "0.1.0"`, `ENVELOPE_VERSION = "1.0.0"`
  (`/home/sprime01/projects/sxr/sxr-core/src/types.rs:7-8`), injected into every
  built envelope as `cep_version` / `envelope_kind`
  (`/home/sprime01/projects/sxr/sxr-core/src/envelope.rs:156-160`).
- Record kinds "MUST first be assigned an existing CEP-0008 §9 `envelope_kind`"
  (`/home/sprime01/projects/sxr/sxr-core/src/record_kinds.rs:1-8`), asserted by
  `every_kind_maps_to_a_known_cep_envelope_kind` (`record_kinds.rs:44-50`).
- `Evidence` is "CEP-0006 §6.1"; `SettlementStatus` is "CEP-0007 §11 — closed
  list" (`types.rs:206-227`); `OmissionRecord` "mirroring CEP-0008's omission
  item shape" (`types.rs:110`).
- Comparison kinds/class mapping are keyed to E §6 / RD-08 / RD-12
  (`contracts/README.md:32-40`).

The CEP profile the CLI defaults to is **`profile:s1-default`**
(`/home/sprime01/projects/sxr/sxr-cli/src/main.rs:144-146`), which must match the
digest pinned in the frozen reference or `observed compare` refuses
(`main.rs:196-199`).

## 7. How SEA-Forge consumes RealityTrace

Two genuinely distinct coupling surfaces.

### 7a. `crates/sea-forge-server/src/realitytrace_source.rs` (348 lines) — CLI read-back

This file **does exist** (confirmed by `find`; content `rg` missed it). It is a
subprocess client, not a linked dependency. Header
(`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/realitytrace_source.rs:1-3`):

> RealityTrace as an evidence source: SEA-Forge reads RealityTrace's own ledger,
> read-only, through its CLI, and checks an evidence packet item against the
> `evidence.created` record it names. SEA-Forge never writes to RealityTrace and
> never takes a packet's direction or reliability on the requester's word.

Three CLI calls, exactly the §2d surface:

- `realitytrace repo list --json` → `repos[].repo_id` (`:82-94`)
- `realitytrace ledger export --repo <repo> --json` → row array, cached per
  request (`:100-110`)
- `realitytrace ledger verify --repo <repo> --json` → must be `"ok": true`, else
  `"RealityTrace's ledger for {repo} does not verify (first divergence: …)"`
  (`:114-119`)

Process hygiene (`:36-57`): `env_clear()` then only `XDG_DATA_HOME`,
`XDG_STATE_HOME`, `PATH`; `stdin(Stdio::null())`, `stderr(Stdio::null())`;
`TIMEOUT = 60s` (`:14`) with kill-on-deadline (`:55`).

The addressing contract (`:130-140`): an evidence item names its RealityTrace
position via `provenance_refs` entries shaped **`sxr:<ledger_id>:<append_ordinal>`**
— `strip_prefix("sxr:")` then `rsplit_once(':')`.

`recorded_item` (`:147-185`) then: find the row with matching `ledger_id` +
`append_ordinal`; `verify_chain(repo)`; require
`record_kind == "evidence.created"` (else `"{ledger}:{ordinal} is not an
evidence.created record"`); then return the `payload.evidence[]` entry whose
`evidence_id` matches. Every miss is an explicit string; never silent success.

`rows_of_ledger` (`:188-`) is stricter: a row of that ledger without a numeric
`append_ordinal` or a string `record_kind` fails the **whole read**, because
"dropping it could hide verification evidence from settlement".

Beyond `EvidenceSource` (`:142`), the same type also implements `ObligationSource`
(`:214`).

### 7b. The CEP `evidence_packet` corroboration gate

`/home/sprime01/projects/sea-rs/crates/sea-forge-authority/src/cep.rs`.

The trait (`cep.rs:1330-1339`):

```rust
pub trait EvidenceSource {
    fn name(&self) -> &'static str;                     // recorded as `realitytrace`
    fn recorded_item(&self, item: &Value) -> Result<Value, String>;
}
```

`Corroboration` (`cep.rs:1341-1352`): `None` = "Items are taken as stated (no
evidence source configured)"; `Source { source, required }` = "Every item whose
`source` is the configured source's name must match its **recorded original**;
with `required`, an item from any other source is refused."

The field list — the actual cross-check (`cep.rs:1356-1364`):

```rust
const CORROBORATED_FIELDS: [&str; 7] = [
    "evidence_id", "evidence_type", "question_ref", "representation_ref",
    "relevance", "reliability", "direction",
];
```

with the reason stated: "Direction and reliability decide settlement, so they
must be present in the original" (`cep.rs:1354-1355`).

`corroborate` (`cep.rs:1366-1421`) enforces, in order:

1. An item whose `source` ≠ `source.name()`: **refused** when `required` —
   `"evidence {id} is not from {name}; this cell settles only on corroborated
   evidence"` (`cep.rs:1378-1385`) — otherwise the packet is marked *not*
   corroborated (`every_item_checked = false`).
2. `source.recorded_item(&item.raw)`; `Err` → `"evidence {id} cannot be
   corroborated against {name}: {why}"` (`cep.rs:1386-1393`).
3. `direction` and `reliability` must exist as strings in the recorded original,
   else `"{name}'s record of evidence {id} has no {field}"` (`cep.rs:1394-1401`).
4. For each of the 7 `CORROBORATED_FIELDS`, if the recorded original carries it,
   `item.raw.get(field)` must equal it exactly — else `"evidence {id}: {field}
   differs from what {name} recorded"` (`cep.rs:1405-1413`).

Return: `every_item_checked.then(|| source.name())` — so "a packet with any item
taken as stated must never count later as corroborated evidence"
(`cep.rs:1371-1373`). That `Option` lands in `EvidenceAck.corroborated_by`
(`cep.rs:1326`) and is surfaced to the client as
`"corroboration": ack.corroborated_by.unwrap_or("none")`.

### 7c. Wiring and configuration

`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/cep_authority.rs:246-265`
(`submit_evidence`): builds `RealityTraceSource::new(rt)` when
`config.cep_authority.realitytrace` is set, wraps it in
`Corroboration::Source { source, required: rt.require_corroboration }`, else
`Corroboration::None`. `settle` (`:267-300`) uses the same source for obligation
settlement and errors with `"obligation settlement needs RealityTrace configured
(cep_authority.realitytrace)"` when absent.

Config (`/home/sprime01/projects/sea-rs/crates/sea-forge-server/src/config.rs:176-193`):

```rust
pub struct CepRealityTraceConfig {
    pub bin: String,           // absolute path of the `realitytrace` binary
    pub data_home: String,     // RealityTrace's XDG_DATA_HOME
    pub state_home: String,    // RealityTrace's XDG_STATE_HOME
    #[serde(default = "default_true")]
    pub require_corroboration: bool,  // Default true
}
```

`deny_unknown_fields` + `require_corroboration` defaulting to **true** means the
fail-closed posture is the default.

### 7d. `crates/sea-forge-realitytrace` — the reverse direction (library, not CLI)

`/home/sprime01/projects/sea-rs/crates/sea-forge-realitytrace/src/lib.rs` is a
*Rust library* dependency on sxr — the opposite mechanism from 7a. It imports sxr
directly (`lib.rs:17-20`):

```rust
use sxr_core::canonical_json::sha256_canonical;
use sxr_core::evidence_emit::{emit_evidence_recorded, EvidenceEmission};
use sxr_core::proof_ingest::{ingest_from, Admission, IngestParams, IngestState, ProofProducer};
```

Header (`lib.rs:12-16`): "SEA Forge's `OperationalSettlement` (E6) plus a
`ProofCompleted` (E7) carrying the pre-committed frozen expectation are admitted
by `sxr_core::proof_ingest`, and `sxr_core::evidence_emit` produces the
`EvidenceRecorded` (E8) expected-versus-observed difference. **No daemon, no
second authority store.**"

Contract constants (`lib.rs:22-31`): `TRACE_SCHEMA = "sea-forge.security-trace/v1"`,
`EXPECTATION_KIND = "security_frozen_expectation"`, `PROOF_PRODUCER =
"sea-forge-security-acceptance"` — the last documented as the only producer sxr
binds to: "RealityTrace binds each cycle to this producer
(`ProofProducer::SecurityAcceptance`) and refuses any other stamp, including
`sea-forge`, the system under test" (`lib.rs:26-31`).

So SEA-Forge both **writes into sxr's proof/evidence model as a library** and
**reads sxr's ledger back over its CLI**, while sxr never links SEA-Forge.
The crate's own README states the boundary: "The crate is an evidence consumer
only: no execution crate depends on it"
(`/home/sprime01/projects/sea-rs/crates/sea-forge-realitytrace/src/README.md`,
Invariants).

## 8. Library, daemon, or CLI? Non-Rust consumption?

**All three, in descending order of importance: CLI (primary), libraries, and an
optional Unix-socket daemon. There is NO HTTP surface.**

- **CLI**: binary `realitytrace`, the documented primary entry point —
  "The binary (`main.rs`) is the primary entry point; this lib mirrors the same
  module set" (`/home/sprime01/projects/sxr/sxr-cli/src/lib.rs:1-4`).
- **Libraries**: 6 library crates (`sxr-core`, `sxr-ledger`, `sxr-git`, `sxr-df`,
  `sxr-verify`, `sxr-observed`) — SEA-Forge's `sea-forge-realitytrace` depends on
  `sxr_core` directly.
- **Daemon**: optional, Unix domain socket only. "Optional daemon (§16): a
  Unix-socket service that holds the writer lock, accepts validated envelope
  requests, appends them to the durable commit queue (fsynced segment + receipt),
  and commits them from the queue in batches… **The daemon adds convenience,
  never semantics (§16.3 parity)**" (`/home/sprime01/projects/sxr/sxr-cli/src/daemon.rs:1-6`).
  `UnixListener::bind(socket)` only (`daemon.rs:145-158`); no `TcpListener`
  anywhere in the daemon. Socket path: `$XDG_RUNTIME_DIR/sxr.sock` or
  `$SXR_DAEMON_SOCKET`, overridable by `--socket` (`main.rs:689-696`).
  Wire format is two lines — "one metadata line then the envelope line"
  (`daemon.rs:19-21`), the metadata being a `request_kind: "sxr.record/v1"`
  JSON object with `repo_id`, `record_kind`, `idempotency_key`, `subject_refs`,
  `envelope` (`daemon.rs:22-31`).

**Can a non-Rust (Bun/TS) process invoke it? Yes — over the CLI, and that is the
sanctioned path.** Every operation the daemon or library performs is reachable
from `sxr`/`realitytrace` argv with `--json` output and a stable
`sxr_output_version: "1"` envelope (`output.rs:1-14`). SEA-Forge is proof of
concept: it is a Rust process, but it consumes sxr purely as
`Command::new(bin).args([...])` + parse stdout JSON
(`sea-forge-server/src/realitytrace_source.rs:36-78`). A Bun process does exactly
the same with `Bun.spawn` / `child_process`.

Constraints a non-Rust caller must respect:

- The `XDG_DATA_HOME` / `XDG_STATE_HOME` environment determines which store is
  read; SEA-Forge sets both explicitly and clears everything else
  (`realitytrace_source.rs:39-45`).
- Exit codes are the error channel (§2), so the caller must read the code as
  well as stdout.
- The daemon socket, if used, is Unix-domain — reachable from Bun on Linux/macOS
  via `net.Socket({ path })`, but it is **not** an HTTP/JSON-RPC API and requires
  the `sxr.record/v1` two-line framing.

## 9. Recursion / agent-execution concepts in sxr

Reporting honestly: **there is no agent execution, no recursion, and no RLM
anywhere in sxr.** Searches run over all crate sources, `docs/`, `contracts/`
and `schemas/`:

- `rlm` — **zero hits** anywhere outside `target/`.
- `recursi` — 5 hits, all unrelated: JSON key-sorting at
  `/home/sprime01/projects/sxr/sxr-core/src/canonical_json.rs:5,55`,
  `set_read_only_recursive` in `sxr-ledger/src/quarantine.rs:55,59,72`, and
  "strips cargo recursion vars" in `docs/subsystems/sxr-verify.md:63`.
- `orchestrat` / `autonomous` / `self-improv` / `meta-agent` — **zero hits** in
  Rust source; "autonomous agents" appears only as prose motivation in
  `docs/index.md:9` and `docs/subsystems/ablation.md:8`.

What *does* exist is **agent telemetry ingest** — recording what an agent did, not
running one. `/home/sprime01/projects/sxr/sxr-cli/src/agent_event.rs` defines the
`agent-event/v1` format: `SOURCES = ["hook", "te_file", "continuity",
"owner_output"]`, `RUNTIMES = ["claude", "codex", "opencode", "prime", "pi",
"cline", "agy"]`, and `KINDS` = `session_start`, `tool_post`, `tool_fail`,
`journey_check`, `stop`, `session_end`, `usage`, `te_injection`, `te_overhead`,
`backfill`, `conversion` (`agent_event.rs:123-135`), plus `ROLES`, `ARMS`,
`QUALITIES` (`:136-138`). It is an *observation* channel: `payload_ref` only,
unknown fields refused so "raw transcript text cannot ride along under a stray
key" (`agent_event.rs:7-9`, enforced at `:265` and `:315`).

`ablation/` (harness/corpus/eval/analysis/runs + `run_*.sh`) is an empirical
evaluation harness for SXR's own guarantees "across multiple revisions of
language models and autonomous agents" (`docs/subsystems/ablation.md:8`) — a
research scaffold over sxr's records, **not** an agent runtime, and not part of
the Cargo workspace (`/home/sprime01/projects/sxr/Cargo.toml:2-10`).

Adjacent but distinct: `docs/index.md:25` claims "A 9-locus **Diagnostic
Attribution Loop** prevents agents from modifying code or domain models until
infrastructure, verifier, and environmental failure loci have been systematically
excluded with evidence" — a *diagnostic decision procedure* over recorded evidence
(`diagnostic enumerate|settle|propose|status`, §2b), not an agent loop.

**Verdict for §9: no recursion. No agent execution. sxr is an observation and
evidence substrate that agents can report into and be audited through — not a
harness that executes or recurses over agents.**

## 10. Tests that prove the evidence contract

### sxr side

- **`sxr-ledger/src/ledger.rs:643` `idempotent_append_returns_existing_position`**
  — asserts the second `append` of the same key yields
  `AppendOutcome::AlreadyCommitted` with `p1 == p2` and `ledger.count() == 1`.
  This is the proof of §1c. `append_chain_verifies_and_tamper_is_named` (`:660`)
  flips a payload byte mid-chain and asserts `!report.complete`.
- **`sxr-core/tests/convergence_t07_proof_ingestion.rs`** (895 lines) — the E7
  admission battery: `t07_tooth1_a_passing_proof_cannot_ride_a_forged_or_missing_settlement_reference`
  (`:280`), `t07_tooth2_altered_claims_are_detected_and_the_first_fact_stands`
  (`:402`), `t07_identical_redelivery_of_a_settled_cycle_is_consequence_free`
  (`:480`), `t07_duplicate_recognition_survives_a_restart_of_the_gate_state`
  (`:510`), `t07_wrong_producers_are_refused_in_both_directions` (`:529`),
  `t07_namespace_and_schema_tampering_is_refused` (`:692`),
  `t07_bound_security_acceptance_producer_is_exclusive_for_its_cycles` (`:850`).
- **`sxr-core/tests/convergence_t08_evidence_emission.rs`** (457 lines) —
  `t08_emission_from_real_ingested_comparison_carries_all_nine_frozen_fields`
  (`:207`), `t08_emission_is_deterministic_under_fixed_inputs` (`:290`),
  `t08_tooth1_no_emission_exists_when_the_observed_consequence_never_resolved`
  (`:303`), `t08_tooth1_missing_or_null_expected_binding_refuses_emission`
  (`:328`), `t08_inconsistent_resolution_inputs_refuse_even_when_individually_well_formed`
  (`:403`).
- **`sxr-core/tests/evidence_record_tests.rs`** — `emitted_e8_persists_as_a_conformant_evidence_packet`
  (`:45`), `direction_follows_the_verdict_mechanically` (`:64`),
  `compared_values_never_enter_the_envelope` (`:73`),
  `malformed_e8_is_refused_before_any_envelope` (`:97`),
  `mapping_is_deterministic_for_a_fixed_context` (`:149`).
- **`sxr-core/tests/fde_contracts.rs`** (326 lines) — the §6a suite:
  `comparison_profile_fixtures` (`:220`), `observed_representation_fixtures`
  (`:225`), `residual_fixtures` (`:230`), `heldout_rule_fixtures` (`:235`),
  `revision_proposal_fixtures` (`:240`), `regression_matrix_fixtures` (`:245`),
  `capability_frozen_fixtures` (`:250`), driven by
  `run_contract(contract, min_valid, min_invalid)` (`:143`) which asserts both
  schema-validity **and** rules the schema cannot state (`check_profile` `:64`,
  `check_representation` `:81`). Run:
  `cargo test --locked -p sxr-core --test fde_contracts` (`contracts/README.md:17`).
  `contracts/README.md:16` notes "The test does no hashing" — digest checking is
  `scripts/check-contract-digests.sh`, a separate gate.
- **`sxr-observed/tests/wp16.rs`** — `rejected_and_unresolved_events_are_counted_as_uncertainty_not_observations`
  (`:454`), `valid_event_is_accepted_and_unsigned_is_recorded_as_unsigned`
  (`:210`), `present_signature_without_a_verifier_is_unverified_and_a_failing_hook_rejects`
  (`:225`).
- **`sxr-observed/tests/wp17.rs`** — the strongest anti-false-positive set:
  `nt06_a_frozen_evidence_source_that_emitted_nothing_is_insufficient_and_the_result_is_u…`
  (`:625`), `nt20_executor_only_evidence_cannot_be_confirmed` (`:644`),
  `tail_deletion_is_detected_with_an_expected_head_and_reported_unverified_without`
  (`:669`), `at08_nt03_one_changed_input_changes_class_and_hash_without_editing_a_file`
  (`:519`), `nt04_undeclared_behaviour_is_observed_undeclared_and_the_model_is_unchanged`
  (`:556`), `inputs_are_refused_when_they_do_not_match_each_other` (`:760`).
- **`sxr-cli/tests/observed.rs:89`
  `ingest_records_rejections_and_derive_is_deterministic_bounded_and_tamper_evident`**
  — end-to-end CLI proof for `observed ingest` + `observed derive`.
- **`sxr-ledger/tests/`** — `ac_ledger_001.rs`
  (`flipped_payload_byte_is_named_with_ledger_id_and_ordinal` `:76`),
  `ac_ledger_002.rs` (quarantine + anchored successor, destructive recovery),
  `tail_verify_window.rs` (`tail_window_still_detects_corruption_inside_it` `:42`).
- Shell gates: `/home/sprime01/projects/sxr/proof/{gate1,red-green,slice0b-gate1,slice3,slice4,slice5,slice6}.sh`.

### SEA-Forge side

- **`crates/sea-forge-authority/tests/cep_settlement.rs:671-880`** — the §7b
  corroboration proof, with a `Recorded` struct standing in for RealityTrace's
  ledger (`:676-687`, `name() == "realitytrace"`):
  - `corroborated_evidence_settles_and_says_what_it_was_checked_against` (`:716`)
    asserts `ack.corroborated_by == Some("realitytrace")` and status `settled`.
  - `a_packet_that_flips_the_recorded_direction_is_refused_and_settles_nothing`
    (`:740`) asserts `"direction differs"`, then `"no corroborated evidence"`.
  - `an_upgraded_reliability_or_an_unrecorded_item_is_refused` (`:772`) asserts
    `"reliability differs"` and `"cannot be corroborated"`.
  - `when_corroboration_is_required_unsourced_items_are_refused_and_old_packets_do_not_count`
    (`:814`) asserts `"is not from realitytrace"`, and that a packet accepted
    before corroboration was required can never later decide a corroborated
    settlement — the `every_item_checked` invariant.
- **`crates/sea-forge-server/src/realitytrace_source.rs:239-348`** (inline
  `#[cfg(test)]`) — `returns_the_recorded_item_named_by_the_ledger_position`
  (`:282`) asserts `recorded["direction"] == "contradicts"`;
  `refuses_a_broken_chain_a_wrong_record_and_an_unknown_position` (`:291`);
  `a_missing_binary_is_an_error_not_a_pass` (`:317`);
  `a_malformed_row_of_the_ledger_fails_the_read_instead_of_being_dropped` (`:331`)
  asserts the error mentions `append_ordinal` and `record_kind`.
- **`crates/sea-forge-realitytrace/tests/sec12_trace.rs`** —
  `auditor_reconstructs_denied_write_and_authorized_control_from_traces` (`:416`),
  `fabricated_observation_is_refused_or_detected` (`:468`),
  `missing_or_late_expectation_provenance_is_refused` (`:514`),
  `broken_correlation_and_mutable_criteria_are_refused` (`:554`),
  `proofs_are_bound_to_the_frozen_acceptance_producer` (`:654`).
- **`crates/sea-forge-realitytrace/tests/evidence_recorded_contract.rs`** —
  `registered_row_matches_the_accepted_contract` (`:79`),
  `proposed_row_matches_the_pinned_registry_format` (`:103`),
  `emitted_evidence_conforms` (`:127`), `schema_and_invariants_reject_drift`
  (`:137`).
- Docs of record: `/home/sprime01/projects/sea-rs/docs/decisions/ADR-012-realitytrace-security-evidence.md`,
  `/home/sprime01/projects/sea-rs/docs/how-to/corroborate-evidence-with-realitytrace.md`,
  `/home/sprime01/projects/sea-rs/docs/security/realitytrace-e8-attribution-proposal.md`,
  `/home/sprime01/projects/sea-rs/docs/security/realitytrace-evidence-recorded-proposal.md`.


---

## VERIFIED / UNVERIFIED

**VERIFIED** (read directly from source at `42f38ca`, quoted above with path:line):

1. `sxr-core` data model — `LedgerRow` (`types.rs:73`), `Evidence` (`:207`),
   `EvidenceRow` (`:280`), version constants (`:3-8`). No type is named
   `Observation`; the observation record kind is `observation.recorded` in
   `record_kinds.csv:16`.
2. Ledger append/idempotency — `AppendOutcome::{Committed, AlreadyCommitted}`,
   idempotency checked first in `append_batch` (`ledger.rs:285-301`), proven by
   `ledger.rs:643`.
3. CLI surface — binary `realitytrace`, `--json` global, 27 top-level
   subcommands (verbatim list in §2a), §15 exit codes 0/2/3/4/5/6/75 with
   substring-based mapping (`main.rs:783-806`).
4. `sxr-verify` = process runners returning observations only; two verifier ids
   (`cargo-test`, `command`); `tree_digest`; cryptographic integrity is the
   SHA-256 chain in `sxr-core::chain`, **not** signatures.
5. `sxr-df` = subprocess adapter to `domainforge` pinned at `0.22.0`, invoking
   `envelope --emit both`, verifying `content_hash` and `semantic_closure_hash`,
   classifying by envelope shape not exit code.
6. `sxr-observed` distinction between recorded / asserted (frozen, digest-pinned,
   outside the ledger) / observed (derived from verified rows only), including
   the explicit `tail.rs:1-7` admission that end-truncation is undetectable
   without an externally anchored head.
7. `contracts/` 8 JSON-Schema contracts + `DIGESTS`; `schemas/` 4 schemas +
   mapping doc; `sxr-observed/schemas/` 11 vendored files; extensive CEP-0006/
   0007/0008 references; CLI default profile `profile:s1-default`.
8. SEA-Forge consumption in **both** directions:
   `sea-forge-server/src/realitytrace_source.rs` (CLI read-back, three
   subcommands, `sxr:<ledger>:<ordinal>` addressing) and
   `sea-forge-realitytrace/src/lib.rs` (direct `sxr_core` library use).
9. `CORROBORATED_FIELDS: [&str; 7]` at
   `sea-forge-authority/src/cep.rs:1356-1364` and the four-stage refusal order in
   `corroborate` (`:1366-1421`).
10. CLI + libraries + optional **Unix-socket** daemon; **no HTTP**, no
    `TcpListener`; non-Rust callers use the CLI with `--json`.
11. No `rlm`, no recursion, no agent execution in sxr.

**UNVERIFIED / not checked:**

- **No build or test was executed.** Every test claim above is the *existence and
  asserted content of the test source*, not an observed green run. Neither
  `cargo test` nor `just test` was run in either repo.
- `sea-forge-realitytrace/tests/broker_attempt_trace.rs` was not read.
- `sxr-cli/src/commands.rs` (3797 lines) and `projections.rs` (2822 lines) were
  not read; the subcommand list comes from the clap enums in `main.rs`, and I did
  not verify that every dispatch arm's behaviour matches its doc comment.
- `sxr-observed/src/compare.rs` (1174 lines) and `revision.rs` (835 lines) were
  read only at headers and class/verdict sites.
- `contracts/README.md` beyond line 45, and 7 of the 8 per-contract tables, were
  not read.
- The `docs/architecture/` tree was not read; the "§" numbers in this report are
  sxr's own citations to its spec, which I did not open.
- `sxr-core/src/proof_ingest.rs` (877 lines) was not read; the T07 test names are
  the evidence for its behaviour.
- ADR-012 and the two `docs/security/realitytrace-*` proposals were located by
  filename, not read.
- **Tooling anomaly worth re-checking:** `rg` returned zero hits in `sea-rs` for
  `realitytrace`, `RealityTrace`, and `CORROBORATED_FIELDS`, while `grep -rn` and
  a direct single-file `rg` both found them. Any other recon in this series that
  relied on a bare recursive `rg` over `sea-rs` has suspect negatives.

