# ADR-016: Project a minimal settlement receipt at the SEA boundary

Status: approved by the operator; hardened after two independent reviews.

## Context

Settlement packets can contain evidence and questions. Consumers need a
receipt for a specific operation without receiving that packet. Existing
`authority_settle` responses carry the complete packet, and older servers
ignore unknown request fields, so an optional flag cannot safely negotiate a
restricted response.

## Decision

Add a distinct, fail-closed `authority_settle_receipt` request, a CEP
extension verb alongside `authority_settle` (like every `authority_*` verb it
is outside the SFWP method catalog, so `system.describe` and the generated
SFWP schemas are unchanged; an older server refuses it as an unknown verb).

- **First call only.** The receipt is the response to the first FINAL
  settlement of an operation. A replay, including a same-request-id retry and
  a settlement already made through `authority_settle`, is refused with
  `receipt_replay_refused`. Clients that need a receipt must call the receipt
  verb for the first settlement; the legacy verb remains for compatible callers.
- **Final only.** A nonfinal evaluation is refused with
  `receipt_nonfinal_refused` after the settlement gate and before any
  settlement packet is committed. The gate's own decision record is the only
  row such a request can append.
- **Authorization.** The settlement declaration gate runs on every call, before
  any outcome is evaluated or released. The stored receipt is never returned
  later: `request.get_status` withholds the outcome of this verb
  (`outcome_withheld`), and same-id replay is refused, so no path releases a
  stored receipt without re-running authorization and integrity checks.
- **Integrity.** Before projecting, the authority ledger is read once, under
  its stream lock (required, no unlocked fallback; a symbolic link in the
  stream path is refused), with chain, MMR and entry-hash verification. The
  settlement row, the committed ALLOW decision and every evidence packet it
  relies on are then checked in that same snapshot. Failure is
  `receipt_integrity_failed`; a disagreement over operation, world or
  references is `receipt_reference_mismatch`. Operation lookups match the
  record's designated operation field exactly, never any subject (the fixed
  `cognate` marker or an approval id selects nothing it does not own), and the
  exported world and decision references are resolved from the committed ALLOW
  record, not from the request.
- **Export allowlist.** Exactly: `representation`, `operation_id`,
  `subject_refs`, `world_ref`, `authority_decision_ref`, `issuer`, `status`,
  `final`, `ledger_id`, `ledger_entry`, `payload_hash`, `envelope_id`. Raw
  evidence, questions, prompts and the full packet stay in governed SEA storage.
- **Errors.** Refusals carry a stable `error_class` / `reason_code` and a
  fixed public sentence, never stored payload or upstream diagnostic text.
- **Effect flag.** `no_side_effect` follows the ledger: false whenever a
  durable row was appended, including when the JSONL append succeeded and the
  MMR save then failed.
- **Bounds.** Operation ids are capped (`MAX_OPERATION_ID_BYTES`) so every
  generated reference of an accepted operation fits the projection limit.

The authenticated response attributes the projection to SEA; the opaque ledger
and payload hashes identify the source and are not an independent proof of the
fields. A receipt reference grants no read access. A caller must compare the
operation, world and independently retained authority decision reference, and
must preserve finality without interpreting it as success.
See the [migration specification](../../.agents/specs/cep-world-ref-migration.spec.md).

- **Obligation settlements.** An obligation settles from RealityTrace's ledger, so its lineage is the
  decision's envelope id rather than evidence packets; evidence settlements keep the evidence-row check.
  The decision envelope id is the deterministic `env-authority_decision-{operation}`; the committed
  `authority_decision` row does not store it, so what the receipt binds is the operation that ALLOW row
  names in its own `parameters.operation_id` (checked against the row's subject), not a stored string.
  An obligation whose run set judges inconsistent ("unsettled") is final but is neither settled nor
  rejected, so it is refused with `receipt_nonfinal_refused` before it commits; it remains obtainable
  through `authority_settle`.
- **One projectability predicate.** Whether a settlement packet can be projected (lineage bounds and
  shape, envelope id shape, status set, issuer, profile, agreement with the ALLOW record) is a single
  function. It runs on the packet about to commit, so an unprojectable settlement is refused instead of
  committed and then permanently unobtainable, and again on the committed packet.
- **Gate first.** The settlement declaration gate is evaluated before the existence and evidence
  errors, so a caller without settlement authority cannot probe whether an operation, its decision or
  its evidence exists. An allow is not recorded by that first evaluation (the existing gate records it
  where the work is done), so polling a not-yet-settleable operation appends nothing. A denial is
  recorded for any well-formed operation id (it is audit); an empty, over-long or control-character id is
  refused as malformed input before anything is written. The same gate-first order applies to
  `authority_settle`, `authority_settle_receipt` (including the RealityTrace-configured check) and
  `authority_closure`.

## Known limits

- **Repeated denied status polls are audited once.** After the first denied
  `request.get_status` audit decision per (cell, actor, request) in a process, further identical denials
  append nothing, so the ledger records that the actor was refused but not how often. The set is capped
  at 4096 entries (cleared at the cap) and empty after a restart, so the count is a floor, not a tally.
  This bounds ledger growth from any authenticated actor; granted polls are always recorded.
- **Admission is serialized per process.** `correlation_admission` serializes check-and-claim inside a
  server. Across processes it is the cell-root lock (`<root>/.server.lock`, taken exclusively by `run`
  before it binds and held for the server's lifetime), which refuses a second server on the same root
  (`a_second_server_cannot_own_the_same_cell_root`). Two servers sharing a root, and so both seeing
  "no claim", cannot exist.
- **Preflight and commit are not atomic.** The history is verified and the packet checked before the
  settlement commits, and the committed packet is verified again; a write to the ledger between the two
  can still produce a post-commit `receipt_integrity_failed` or `receipt_reference_mismatch`. The response
  then reports `no_side_effect: false` and the settlement stays obtainable through `authority_settle`.
- **`request.get_status` answers `unknown` when ownership cannot be proven.** That includes the true owner
  of a request whose claim ledger is unreadable or that predates claims, although the request executed;
  the owner learns the truth from the original response or an operator, not from status. A non-owner
  without the grant takes longer than for a never-seen id (policy load, possible audit append), so
  timing can still distinguish a claimed id. The log line for an unreadable claim carries no request id.
- **Request ids are cell-global.** See the open debt entry in `.agents/OBSERVED_DEBT.md`.

## Residual risk

The chain and MMR are unsigned local files: an attacker who can rewrite the
whole stream and its MMR consistently is not detected without checkpoint
signatures. The symbolic-link check is check-then-open, not an anchored
no-follow open, and hard links are not detected.
