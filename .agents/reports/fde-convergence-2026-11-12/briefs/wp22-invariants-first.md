# WP-22 promotion saga and world stamping: invariants-first brief (2026-11-17)

**Scope** (F-implementation-plan, WP-22; SEA-Forge, plus RealityTrace where it consumes the stamp):
- a saga keyed `(proposal_id, candidate_world_ref)`;
- a reconciler for partial freeze, orphan records and partial promotion;
- a key mapping derived from the semantic diff;
- in-flight cases keep their pin.

**Acceptance:**
- crash-injection tests after every saga step;
- NT-07 re-pin variant ("settled evidence stays with its world": read evidence after promotion and after re-pin, and it is still
  bound to the old `world_ref`);
- uncertain outcomes fail closed and surface in the reconciler.

**Lane:** SEA-Forge `fde/integration`, which must be free; WP-21B is finishing in it. The frozen `contracts/fde/` is read-only.

## Construction choices (fixing classes up front)

- **The saga is a state machine over an append-only step log.** Steps are an enum. Each step is `(precondition read from the log) ->
  idempotent effect keyed by the saga key -> step record`. Nothing else holds saga state.
- **The reconciler is the same state machine,** driven from the log after a crash. It is not a second implementation. Its only
  outputs are "resume to completion", "roll back to the pre-saga state", or "uncertain: fail closed and surface for the operator".
- **Every effect is idempotent:** re-running a step with the same key is a no-op, verified by content digest.
- **Evidence is immutable by type:** settled evidence carries its `world_ref` in its digest. Promotion and re-pin create new
  references and never rewrite old ones.
- **The crash-injection seam is generated from the step enum:** the test iterates `Step::ALL`, plus a "mid-write" point per step.
  A new step is covered automatically, and a step that has no fault point fails to compile or test.
- **Reuse WP-21B's case guard:** saga writes under a case dir go through `admit_write` / `WriteToken`.

## Invariants (proptest; random saga runs with a crash at a random step or mid-write, random repeats, concurrent duplicates)

- **W1, idempotence:** running the saga, or any step, n ≥ 1 times with the same key gives the same final state and the same bytes as
  running it once.
- **W2, crash consistency:** after a crash at any point, then the reconciler, the state is exactly one of: fully promoted, fully at
  the pre-saga state, or uncertain (fail-closed, surfaced, and refusing use as promoted). It is never a mixed, usable state.
- **W3, NT-07:** settled evidence keeps its original `world_ref` and digest after promotion and after re-pin.
- **W4, pins:** an in-flight case's pin is unchanged by any promotion.
- **W5, key mapping:** the mapping from the semantic diff is total over the changed keys and injective. An unmapped or ambiguous key
  refuses the promotion before any write.
- **W6, concurrency:** two sagas with the same key never both commit, and two with different keys on the same proposal serialize.

## Process

1. Write the invariants and the generator first.
2. Show them red where code exists.
3. Implement.
4. Run mutation checks per invariant.
5. Run one Codex round briefed to break W1–W6.

Run `scripts/fde-proof.sh --slice s2` before and after, and do not regress any row. Host rules: 8 GB, `flock -o
~/projects/_wt/cargo.lock cargo … -j2`, one heavy job at a time.
