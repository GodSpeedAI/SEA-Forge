# ADR-016: Project a minimal settlement receipt at the SEA boundary

Status: approved by the operator; implementation and integration verification pending.

## Context

Settlement packets can contain evidence and questions. Consumers need a
receipt for a specific operation without receiving that packet. Existing
`authority_settle` responses carry the complete packet, and older servers
ignore unknown request fields, so an optional flag cannot safely negotiate a
restricted response.

## Decision

Add a distinct, fail-closed `authority_settle_receipt` request to the existing
settlement authority. Apply the same authenticated standing and settlement
gate on initial settlement and replay. Project only the approved references,
issuer, disposition, and finality from the committed winning ledger record.
Keep the full packet in governed SEA storage. The existing `authority_settle`
response remains available to authorized compatible callers.

The authenticated response attributes the projection to SEA; the opaque
ledger and payload hashes identify the source but do not independently prove
the projection's fields. A receipt reference grants no read access. A caller
must compare the operation, world and independently retained authority decision
reference, and must preserve finality without interpreting it as success.
See the [migration specification](../../.agents/specs/cep-world-ref-migration.spec.md).
