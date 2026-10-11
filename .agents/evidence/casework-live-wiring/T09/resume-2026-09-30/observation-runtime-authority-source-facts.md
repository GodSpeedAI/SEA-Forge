# Observation authority and trace source facts

Independent read-only clarification by the unit4 critic; facts only, no implementation
or sharing design approval. Original unit5 bounds and effective-session authority apply.

Rust run.get RunRecord has optional string started_at/finished_at and optional termination
(crates/sea-forge-server/src/sfwp/run_views.rs:283-305). RunTermination carries real at:string,
optional execution_status:string and optional exit_code:i64 (:97-114). Raw TraceRow carries
timestamp:string and dynamic payload (:116-130). Fold reads status as string and exit_code
via JSON as_i64 only on command_finished (:552-567); other terminal kinds carry neither
(:576-584). Go observation DTO carries timestamp:string, optional typed command status
and optional int64 (internal/contract/contract.go:178-200); DTO itself does not parse time.
This does not yet prove the exact safe per-frame projection of the complete raw trace.

Kernel run.list/run.get are Inspect verbs (sfwp/mod.rs:208-220), exempt from request actor
requirements (identity.rs:822-850). Case scoping uses persisted case run_ids and exact owner
filtering (run_views.rs:484-505,776-791). Kernel read invariance alone is insufficient proof
that gateway projection/disclosure permits sharing across effective sessions.

Go session middleware resolves the cookie once and attaches requestIdentity
(internal/server/session.go:85-95,126-141). SessionStore.Resolve checks absolute/idle TTL,
removes expired sessions, otherwise slides LastSeen (internal/auth/session.go:111-130).
Destroy deletes an entry (:133-138); Sweep exists (:147-160), but no production call site
was found. Logout destroys session and clears cookie (server/session.go:374-384), guarded
by session/CSRF route (server.go:134). Open SSE verifies perspective once then subscribes
(server.go:293-318,345-375); it has no current session re-resolution or revocation signal.

Existing seams: PerspectiveVerifier.VerifyPerspective(server.go:38-43), request context
cancellation (:348-352), Store.Subscribe channel/cancel (projection/store.go:193-226).
There is no connection from Destroy/expiry to cancellation of an already open stream.
Unit5 must specify current effective authorization before each read/fanout and drain on
revocation/logout/expiry. These source facts authorize no security boundary change.
