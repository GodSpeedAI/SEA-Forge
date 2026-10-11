# Bounded reader response boundary addendum

Root chooses the following concrete delivery/frontier rule for the corrected
proposal. When the next validated matching event would exceed frame count or
the complete serialized response budget, do not acknowledge that row: return
the previously fully validated row as scanned-through frontier and a token at
the byte boundary immediately before the undelivered row. The next page must
reread/validate that row. Non-event and filtered rows may advance the frontier.
All bytes read, including lookahead, count toward raw input budgets; no seeking
past an unvalidated or undelivered matching row is allowed.

If a single matching frame plus the smallest required response metadata cannot
fit1MiB, fail the whole request typed unavailable with no partial success. If
there is no acknowledged preceding row, continuation must preserve a signed
pinned-head origin position rather than manufacture a prior row or ordinal.
For origin offset0, verify actual origin on resume; nullable predecessor is
permitted only at that exact boundary. Ordinal0 is the first actual row.

This is a proposal clarification, not operator approval or a code grant. It
adds no response size increase and prevents a response-budget stop from
silently skipping an undelivered event. Read it with the root clarification.
