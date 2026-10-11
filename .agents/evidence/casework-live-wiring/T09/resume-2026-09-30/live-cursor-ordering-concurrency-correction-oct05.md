# Live cursor ordering concurrency correction

Date: 2026-10-05. This immutable source note corrects an ordering implication
in `live-cursor-ordering-source-facts-oct05.md`. It is a factual correction,
not an architecture decision or runtime change.

`crates/sea-forge-server/src/lib.rs:208-233` implements `publish_event` by
spawning each durable append on `tokio::task::spawn_blocking`, awaiting that
task, then broadcasting the resulting frame. Independent calls can therefore
have concurrent append tasks and complete/broadcast in an order not proven by
these source lines to match invocation or client arrival order. The event
broadcast is not evidence that its frame sequence equals ledger append order.
Any prior inference that a Go Store's broadcast arrival order preserves
durable append order is unsupported.

The durable source of order is explicit in
`crates/sea-forge-server/src/sfwp/events.rs:123-192`: replay and `get_range`
resolve opaque cursors to ledger `append_ordinal`, filter strictly after the
from ordinal and through the optional to ordinal, and emit entries in ledger
iteration order. `get_range` caps each read at `EVENTS_REPLAY_CAP` (500).
`spec-full.md:544-557` identifies append ordinal as authoritative. Thus any
future client feed proposal that needs durable order must ground recovery in
that existing range operation; the live broadcast can serve as a wake-up/hint
only if the consumer also reconciles omissions and reordered arrivals through
bounded range reads. This note does not establish a polling interval, page
loop termination policy, or client algorithm.

Other extant bounds relevant to a future design: the Rust live broadcast
channel has capacity 256 (`crates/sea-forge-server/src/lib.rs:166`); Rust
initial replay and Go SFWP replay are capped at 500; Go's projection Store is
in-memory with a default 512-entry retention window
(`apps/godspeed-casework-go/internal/projection/store.go:15-29`). A reconnect
cursor that no longer resolves produces an explicit unknown-cursor error in
`events.rs:123-151`; it does not silently mean “from the beginning.”

No source in this review established cursor-format negotiation or a durable
mixed-format migration history. This source correction does not authorize a
public contract, schema, or implementation change.
