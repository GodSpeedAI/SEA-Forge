# Private retained publisher: root decomposition

The manager lifecycle needs recurring reads to establish actual cancellation,
retirement and drain behavior. Any retained version must obey the reviewed
1 MiB combined canonical image and 1024-frame window. Implement a separate
pure private retained-version helper before lifecycle integration. This does
not require Next, SSE, public contracts or persisted schema changes.

Only test-first declarations, an explicitly unavailable scaffold and focused
fixtures are currently released. A different critic must review original
instructions and complete source, then an exclusive compiler owner records
expected RED before algorithm implementation. The existing lifecycle fixture
af2dfcb8 also needs a fresh current-candidate RED; prior b1b585 RED stays
historical. Lifecycle preregistration remains pending implementation release.

The canonical helper image includes the exact case/run/plan key, current safe
trace metadata and accepted timestamp, owned window with exact IDs and first
ordinals and cloned optional pointers, the entire lifetime ledger sorted by
opaque ID, and every actually retained publisher control. Control fields are
always present: fixed-width availability/failure/terminal/stop codes and
20-digit decimal uint64 generation/highest-assigned values as applicable.
Do not invent or duplicate unused counters. Integration must account for any
additional retained safe/control fields outside the helper image before
claiming a complete per-poller bound. This is not a heap/RSS bound.

Candidate construction is atomic. Reorder, shorter responses or changed
source values never reset or evict lifetime IDs/first ordinals. Reject new
ordinal overflow without wraparound or prior mutation. A rejected candidate
does not advance the accepted timestamp or highwater. Nonterminal overbudget
preserves all old safe values and ledger for later recovery; terminal
overbudget permits only the fixed-size safe marker and bars future read starts
before actual retirement/JOIN. Lifecycle owns cancellation and capacity.

Root ratifies this private representation under the operator's architecture
delegation. No dependency, public API, security model, persistence, writer
frontier or production integration change is authorized by this decision.
