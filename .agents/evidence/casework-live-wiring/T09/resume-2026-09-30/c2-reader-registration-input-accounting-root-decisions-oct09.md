# Reader registration and raw-input accounting decisions

Root architecture clarification, 2026-10-09. No Rust source release or runtime
claim. Read alongside the approved reader proposal, policy receipt, spec/ADR,
and their normative reviews. This preserves the existing 4 MiB bound rather
than introducing an additional uncounted allowance.

## Registration and placement

The existing ServerState-owned events_ledger, constructed by the fixed
open_events_ledger helper, is the registered stream for events.get_range.
V2 consumes that specific handle; no arbitrary client path, new registry or
authority mechanism is introduced. Ledger owns bounded raw access, cooperative
locking and the established typed hash protocol; server owns the ephemeral
signer, token/filter semantics and separate V2 DTOs. Use existing dependencies
and nonblocking lock acquisition with a monotonic 50 ms wait budget. No async
kernel primitive or unbounded compatibility fallback.

Only successful absent/zero-byte proof for that registered stream under the
cooperative lock permits complete-empty. Metadata/open failures, directories,
special nonregular files, whitespace or torn/corrupt history remain unavailable.
Do not invent a new symlink prohibition: verify the actual opened backing file
is a regular file and follows the approved registered-path semantics.

## One raw-input budget

Every actual raw ledger byte read for one page request consumes the approved
4 MiB page-input budget: head pin/tail verification, continuation predecessor
revalidation, forward scan, non-events and lookahead. Repeated reads count
again. Metadata and seeks consume no raw bytes. Limit read requests by the
remaining budget before allocation/read; reaching the budget cannot cause a
partial row, undelivered lookahead or incomplete validation to be acknowledged.

The proposal describes auxiliary reads separately and does not explicitly
charge or exempt them. The approved normative phrase is raw input per page,
including non-events/lookahead, not a separate 4 MiB forward-scan allowance.
Counting auxiliary reads makes that approved total enforceable; excluding
them would silently permit more raw input. Valid histories may consequently
return unavailable when the remaining budget cannot establish any further
acknowledged progress. Do not increase the budget, skip integrity checks,
fabricate progress or fall back to a full-ledger read to avoid that result.

The 2 MiB row cap including LF and complete 1 MiB serialized response cap
remain independent and unchanged. Raw-byte accounting is not an RSS/heap or
OS-read-latency guarantee. Acceptance tests must cover the combined auxiliary
and forward-read total at the cap and cap-plus-one, with no failed-page ACK.
Independent review of this clarification and normative traceability precede
the future bounded reader implementation grant.
