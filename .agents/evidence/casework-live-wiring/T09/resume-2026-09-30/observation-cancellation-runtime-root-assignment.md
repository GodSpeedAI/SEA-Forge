# Unit5A production cancellation repair: bounded root assignment

HOLD production writes until independent source approval and compiling intended
assertion RED of the cancellation fixture. This supplements the original
observation-cancellation-test-first-assignment.md and critique supplement;
the original tests and transport semantics remain binding.

Fresh implementation builder owns ONLY internal/adapters/sfwp/client.go. Keep
the independently frozen cancellation fixture and every existing test unchanged.
No dependency, public API, correlation, kernel, identity or retry-policy changes.

## Required lifetime structure

Install cancellation interruption for the call's checked-out network connection,
covering blocking request writes and response reads. Affect only its net.Conn,
never global Client.Close or another pool member. The cancellation callback must
not access conn.dead or pool bookkeeping; those remain with the call owner.
On every call exit, retire the callback or wait until an already started callback
finishes, BEFORE unlocking/returning the connection to the pool. A cancellation
callback that ran makes that connection retired even if a full valid response
was received concurrently. No callback may act after pool release/reassignment.
Context AfterFunc plus a completion channel is a candidate using stdlib only;
builder must inspect ownership and justify any equivalent structure.

Canceled I/O must return typed unavailable preserving context.Canceled in the
error chain. Existing deadline classification and ambiguity remain authoritative.
Never reset a canceled operation to a future deadline. Preserve full valid
received responses; simultaneous cancellation and receipt may retire that
connection while still returning the real response. Do-return-before-cancel
remains stricter: later cancellation cannot retire its already returned idle
connection, and the next call must reuse it normally.

Existing inspect retry, Ask no-resend/status, mutation correlation and explicit
pre-admission busy retry must not change. Under a canceled caller context no
retry/recovery request may be admitted. Preserve existing response cap and
positional-pairing discard rules. No speculative protocol behavior or fallback.

## Independent proof

After valid expected RED, builder may run the narrow frozen cancellation matrix
only while explicitly owning root's sole compiler token and checking actual
HOST RAM/processes before every command. Immutable raw stdout/exit/preflight
captures, including failures, must be retained exactly. No fixture edits to
obtain GREEN. Freeze full source identity and diff, explain every material
deviation, and return token. Independent critic receives ALL original and
supplemental assignments plus frozen implementation, checks lifetime/races and
re-runs focused cancellation, full SFWP race, canonical casework-go-check and
full-module race/count1/parallel1. Approval is only this client prerequisite;
it cannot settle pollers/SSE/unit5/T09. Root owns integration/final decisions.
