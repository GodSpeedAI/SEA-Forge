# Independent review: live cursor contract proposal (2026-10-05)

**Verdict: REJECT for operator review in its current form.** The proposal correctly
identifies a real producer/contract mismatch and appropriately holds all implementation
until operator approval. Its bounded regex union is plausible as a syntax allowance, but
the proposed edit inventory is incomplete and the claim that this is an additive,
backwards-compatible validation correction does not address existing consumers' cursor
ordering semantics. This is a source-only review; no code, schema, tests, status, or Git
state was changed and no compiler/test command was run.

## Evidence reviewed

- Root instructions, `.agents/AGENTS.md`, current status files, and the `graft` repository
  skill were reviewed. Root AGENTS §4 requires prior review/approval for public contract
  and persisted schema changes. `.agents/AGENTS.md` makes specs normative and requires
  spec/ADR updates with public contract changes.
- Governing spec: `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`,
  especially `metadata.version: 0.2.5`, `metadata.change_rule`, frozen interface inputs,
  and `t09_additive_contract_amendment`.
- T01 confirmation: `.agents/evidence/casework-live-wiring/T01/confirmation.md:54-76`;
  T09 amendment source: `.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:31-35`;
  immutable canonical schemas and interface prose; the root proposal and independent
  source recon `c7fabaaa` at
  `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/live-cursor-contract-independent-recon-oct05.md`;
  UI validator recon at `observation-ui-validator-recon-oct05.md`.
- Graft query located the raw ledger cursor and server event-frame path at
  `crates/sea-forge-server/src/sfwp/events.rs:16-21,59-100` and
  `crates/sea-forge-ledger/src/types.rs:81-93,127-147`. The Go path preserves it through
  `internal/server/feed.go`, relay, projection, SSE and intent freshness as detailed in
  the c7fabaaa recon. `apps/godspeed-cognitive-ui/src/adapters/http/httpCaseworkAdapter.ts:344-347,419-426`
  compares ordinary event cursors lexically against `lastCursor`; comments call them
  monotonic. The recon separately confirms legacy cognitive history helpers and fixtures
  assume epoch/sequence cursors.

## Material findings

1. **The authored constraint inventory omits a cursor-bearing schema arm.** In
   `.agents/reports/interface-contracts/schemas/event-stream.schema.json`, the shared
   envelope cursor pattern at lines 24-27 and the named `ExecutionObservationEvent.cursor`
   at 176-183 are covered by the proposal, but `definitions.ResyncRequired.properties.last_cursor`
   at 215 independently repeats `^[0-9]+\\.[0-9]{10}$`. A resync response reports the
   oldest retained real cursor and must accept the actual ledger cursor too. Update this
   authored constraint and its description/fixtures in the same contract change, or the
   recovery path still rejects valid values. The adjacent requested/oldest cursor fields
   are unconstrained strings today; they should be checked for semantic agreement with
   the selected cursor grammar rather than silently treated as already constrained.

2. **The proposal defers a behavior-changing ordering question while saying runtime
   ordering stays unchanged.** The approved T01 contract and schema call these cursors a
   monotonic logical sequence. The proposal admits its two forms do not establish a
   mixed-history ordering relation, yet ordinary native and fetch event ingestion compare
   raw cursor strings (`<=` / `>`). The live ULID is not an append-order guarantee; the
   kernel spec says append ordinal, not ULID, is authoritative (`.agents/specs/spec-full.md:544-557`).
   A regex union alone therefore does not define the existing dedupe, replay, gap, or
   ordering behavior when values are ULIDs, nor establish that lexicographic UI ordering
   tracks server revision order. The observation path explicitly bypasses this comparison,
   but ordinary snapshots still use it. Before approval, specify the contract semantics
   for ordinary cursors and their ordering/resume comparisons across Go and UI. This review
   does not recommend converting kernel IDs or creating IDs; it flags that accepting a
   syntax does not settle ordering semantics.

3. **The compatibility characterization is too strong.** Current canonical consumers
   deliberately reject ULIDs: JSON schemas constrain event, snapshot and intent cursors;
   T01 froze epoch/sequence semantics; interface prose and TypeScript examples repeat
   those semantics; the UI cursor comparator and history tests are built around that
   model. Existing runtime producers already emit ULIDs, so this proposal repairs an
   observed mismatch, but changing a public validator's accepted language is a public
   contract revision, not automatically backwards-compatible for older strict readers.
   State the supported producer/consumer compatibility window and whether dual-format
   acceptance is temporary or durable. Keep the existing logical examples/fixtures, but
   add actual source-format cases to each affected authored conformance surface.

4. **The specification/version decision is left unresolved.** The governing spec
   explicitly requires prior approval plus ADR/spec update. It is versioned `0.2.5`, and
   each canonical schema has an unversioned stable `$id` (for example,
   `https://godspeed.ai/schemas/event-stream.schema.json`). The proposal says the critic
   should decide whether version changes are required but does not state the intended
   versioning consequence or identify the approval record/ADR location. Approval should
   explicitly record whether to increment the governing spec version and whether stable
   schema IDs remain intentionally unversioned; do not infer that a schema's unchanged
   `$id` makes a public grammar change non-versioned.

5. **Authored semantic descriptions need a complete pass, not only regex replacement.**
   `cognitive-world.schema.json:29-33` describes a “Monotonic logical sequence cursor
   (<epoch>.<seq>)”; `interaction-intents.schema.json:53-56` calls `client_cursor` a
   “monotonic cursor”; `typescript/types.ts:101-105`, the T01 confirmation, and
   `04-REACT-COGNITIVE-ENVIRONMENT-CONTRACT-SPECIFICATION.md:109,197-222` give the old
   epoch/sequence examples and replay/order semantics. The event-stream top-level
   description currently says observations do not advance “logical history”. The proposal
   only says “directly related canonical comments/docs”; approval needs an explicit list
   of semantic statements to revise or explicitly preserve as logical-cursor-only
   statements, so the authored contract does not describe ULIDs as ordered logical
   sequences. Distinguish syntactic cursor validity from ordering guarantees.

6. **Scope boundaries need to be explicit for adjacent cursor fields.** The independent
   recon correctly limits explicit schema findings to event/snapshot/intent and notes no
   dedicated trajectory schema. The proposal's phrase “authored canonical cursor
   validators” should not imply that every cursor-looking field is governed by this
   union. It should enumerate in-scope schema locations (including resync `last_cursor`)
   and state whether unconstrained response fields such as `new_cursor`/`current_cursor`,
   trajectory cursors, and untyped recovery request fields are intentionally opaque or
   need their own contract constraints. Do not expand this into unrelated identifier
   changes.

## Approval boundary and disposition

The T09 trigger and root AGENTS require operator review before public-contract edits. The
recon establishes a mismatch; it is not approval to change canonical schemas, version,
runtime behavior, generated mirrors, or IDs. The regex itself is a reasonable candidate
for review, but this proposal is not ready for that review until findings 1-5 are answered
and the exact authored/generated/fixture scope is enumerated. In particular, do not treat
the proposal's requested positive/negative regex fixtures as proof that cursor ordering
and compatibility are settled. No compiler, tests, implementation, or status/debt/Git
changes were performed here.

🌱 graft saved ~60,756 tokens (~$0.05) this turn (1 call).
