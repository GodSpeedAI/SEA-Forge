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

## Residual risk

The chain and MMR are unsigned local files: an attacker who can rewrite the
whole stream and its MMR consistently is not detected without checkpoint
signatures. The symbolic-link check is check-then-open, not an anchored
no-follow open, and hard links are not detected.
