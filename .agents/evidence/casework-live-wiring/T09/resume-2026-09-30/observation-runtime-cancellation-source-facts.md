# Observation cancellation source facts

Independent read-only worker run_scope_live_fixture_repair; no code or tests changed,
no compiler, no architectural approval. These facts constrain unit5 cancel-and-drain.

Manual context cancellation does not reliably interrupt an in-flight run_get. Client
roundTrip adds RequestTimeout(default15s) and dials with context, but conn.call transfers
only the earlier socket/context deadline to SetWriteDeadline/SetReadDeadline; there is
no ctx.Done watcher/AfterFunc closing the checked-out connection. A blocked read may
continue until response/deadline, and a response may succeed after manual cancellation.
On I/O error it closes/discards rather than reuses a partially read connection.
Anchors internal/adapters/sfwp/client.go:188-228,400-439; defaults85-90; dial268-276.

Client.Close156-170 closes idle pooled connections only; it is not cancel-and-drain for
checked-out calls. Each connection has one in-flight call (172-194). Client pool(default4)
mutex protects selection/accounting(115-128,283-326,337-351), not I/O or dial. Waiting for
capacity honors ctx.Done318-324; active read may hold the slot until effective deadline.
Unrelated operations on other checked-out connections are not globally canceled.

run_get is an inspect constructor(frame.go308-313), not a mutation(101-118). Nonmutations
except Ask are transport-retry-safe(93-99). roundTrip permits one fresh-connection retry
only while ctx.Err()==nil(client.go400-439); Do may also retry one explicit pre-admission
transport refusal(385-397). sleepCtx544-553 honors cancellation. Socket read itself is
the missing manual-cancellation link. Global read-slot ownership must enclose all retries.

Unit5 must explicitly resolve this before claiming cancel-and-drain or immediate release
of run-read/poller capacity. Client.Close alone is insufficient; closing shared transport
globally would affect unrelated authorized calls. Any repair requires focused actual
blocked-read cancellation proof, connection-pool reuse/race checks and unchanged mutation/
Ask ambiguity/retry/correlation semantics. No implementation decision is made here.
