# T07 session-bound retained reads — bounded repair specification

## Independent finding

The recovered T07 critic found that authenticated SSE and `GET /api/world?cursor=...`
return the relay's default-perspective snapshot, including role-derived action descriptors.
The historical branch also precedes the actor/role override refusal and kernel perspective
verification. Mutation actor overwrite does not correct inaccurate displayed authority.
This specification awaits a fresh builder and independent verification; it is not approval.

## Required behavior

- Current, retained and streamed snapshots identify the authenticated session's actor/role.
- Refuse actor/role query overrides before any historical lookup. Verify the session's kernel
  delegation before accessing retained history or starting SSE; revocation must fail closed.
- If `case_id` is supplied with a cursor, reject a cursor belonging to a different case.
- Preserve the state recorded at the requested cursor. Never fetch current state and stamp
  it with a historical cursor. Missing or evicted truth remains a typed refusal.
- Preserve separate roles' action descriptors: an R-SO stream must not advertise operator
  execution offers. Do not grant, synthesize or replay consequential operations.
- Preserve kernel cursor ordering, bounded retention, resume and reconnect behavior.

## Architecture direction

The projection already has a pure `Build(CaseFacts)` boundary. Retain the source facts used
to build each revision, then render a deep-cloned fact set for the verified session actor.
Factor live fact collection so the snapshot and retained facts come from the same captured
view. Keep retained data bounded by the existing revision store and immutable to callers.
Do not issue a second current-state fetch to reconstruct an older revision. Where an injected
source cannot provide captured facts, an exact same-perspective snapshot may be served after
verification; a different perspective must fail closed instead of guessing.

No new dependency, persisted schema, ID grammar, kernel policy, identity model or external
interface contract is authorized by this repair. Internal seams may be factored narrowly.

## Required evidence

Focused tests and fresh-cell probes must cover distinct operator/R-SO sessions, streamed and
historical actor/action identity, rejected query overrides with a cursor, cross-case cursor
rejection, invalid/revoked delegation on cached reads, immutable retained facts, and genuine
old state after a subsequent mutation. Preserve existing task/global gate strength and log
the initial rejection and all repair attempts separately. An independent critic must receive
this original specification and the implementation and explain all material differences.
