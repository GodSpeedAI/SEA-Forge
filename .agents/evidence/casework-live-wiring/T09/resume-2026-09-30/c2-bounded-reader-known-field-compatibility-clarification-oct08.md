# Bounded reader known-field compatibility clarification

Root rejects the revision2 proposal's new unknown-field rejection for existing
LedgerEntry and stored-event payloads. Preserve their existing unknown-field
tolerance in legacy and v2. Validate known required fields, field types,
ordinals, row/hash chain and required event kind/detail presence; explicit null
detail remains valid. Do not add a kind grammar/nonempty restriction absent
from the existing writer. Unknown fields remain part of the actual canonical
payload/entry hash input; do not discard bytes or unknown payload fields when
checking integrity. The new private signed continuation schema may reject
unknown fields because it has no legacy clients.

Also bind the exact stream identity without assuming arbitrary ledger IDs are
ASCII or letting canonical JSON normalize them. Use a domain-separated
SHA-256 exact-UTF8-byte binding with presence/checked-length encoding, as for
the already selected exact cursor filters; compare against the actual target
stream identity before seeking. Keep generated actual cursor identities exact.
Do not impose a new ledger ID grammar or normalize identifiers. A binding
digest is only token integrity input, never authority.

The corrected proposal should remove the inaccurate statement that the Rust
append constructor maintains the private Go poller's exact unseen-ID ledger.
Rust source anchors establish actual first append ordinal0 and serialization;
the private Go identity-ledger invariant is a separate component.

These corrections preserve existing compatibility and exact identity. They
do not approve the new signer/budgets/empty/filter wire details, a new
dependency, or implementation. Original proposals/reviews remain immutable.
