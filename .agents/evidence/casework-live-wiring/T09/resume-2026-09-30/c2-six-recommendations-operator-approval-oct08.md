# Operator approval of six C2 recommendations

Date: 2026-10-08

The operator asked exactly what required approval, including recommendations.
Root presented six changes and recommended approving each:

1. Shared journal for CLI/server case mutations, with crash recovery and every
   writer participating so coherent captures can be established.
2. Versioned bounded event reader with real ledger ordering and continuations,
   preserving the existing response format for older callers.
3. Separate global ledger integrity from per-case delivery. Unknown global
   failures make the cell unavailable; proven case-local failures remain local.
4. Cursor plus snapshot digest for history, SSE reconnection and mutation
   preconditions, distinguishing captures at the same cursor after restart.
5. Correct case/world ID schemas and explicit complete-empty bootstrap/create
   response; actual kernel IDs and errors distinct from empty success.
6. Initial retention limits, stream budgets, deadlines and explicit resync
   errors, with load verification before claiming operational readiness.

The operator replied: **“i approve all, following your recommendations”.**
This is explicit approval of these recommended architecture, persisted-storage
and public-contract changes. Earlier generic continue/push approvals are not
the basis for this authorization. Do not ask again for the same six changes.

Revision6 candidate SHA-256
`7e9009c5d733c8517531e705b9e5b2d6478bda5c9e4a551ce03e65a46e8b258d`
was completed concurrently with the reply. Root must verify, with a fresh
independent critic, that the concrete candidate implements these recommendations
and disclose any material decision outside that approved scope. This record
does not claim the operator read or approved that exact candidate hash.

Independent design review, authored spec/schema/ADR updates, complete writer
participation audit, TDD and required runtime gates remain required before
implementation readiness or settlement. New dependencies, authority weakening
or unrelated architectural changes are not authorized by this approval.
