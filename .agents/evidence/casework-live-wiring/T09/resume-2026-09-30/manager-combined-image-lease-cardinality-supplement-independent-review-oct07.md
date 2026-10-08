# Independent review: lease-cardinality lifecycle supplement

Date: 2026-10-07  
Verdict: **APPROVE the bounded lifecycle test-matrix repair, with the stop-before-read test setup qualification below.**  
Scope: design/readiness only. This does not release lifecycle source, fixtures, compiler work, or runtime verification.

## Evidence reviewed

- Full `manager-combined-image-lease-cardinality-supplement-oct07.md` (SHA-256 `ce3dc62667c206f37cfa2a102d03dbd6d16d77587d0d2fe11a76c65408ddc4e9`).
- Its complete archived assignment `manager-combined-image-lease-cardinality-supplement-assignment-oct07.md` (SHA-256 `63301123330c5e0ba0112b81e4b6ff626e00c508490183dc77bb14bd9d367f1a`).
- The prior rejection, root signal/ref ownership decisions, initial-failure clarification, full original Unit 1 assignment, lifecycle preregistration, revision 6/addendum and retention corrections 2–3.
- Current source anchors in the unwired manager scaffold and frozen manager fixture; their identities remain manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` and fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.

## Finding and cited protocol coverage

The supplement repairs the per-Prepare cardinality defect. The source scaffold returns one lease per `prepare` call and stores each lease’s own membership map (`run_observation_manager.go:82–85,125–129`); distinct leases point to pollers in the poller’s refs set (`:73–80`). Its tests 3A and 3B now distinguish those cases rather than treating several Prepare calls as one lease.

Case 3A (§§34–65) has separate lease drain ownership for each pending Prepare and one manager-wide stop signal. It checks that each transition clears only its lease’s exact refs, no waiter clears absent membership, each Prepare completes its operation registration before awaiting its own drain, and the shared worker is canceled/joined once after all refs are gone. Case 3B (§§67–95) leaves one independent lease authorized while the other Prepare rolls back; the rollback removes only its own refs and cannot cancel the worker still needed by the surviving lease. This matches the original exact-ref rules and root’s decision that actual membership under the manager mutex chooses the one removal owner.

The untouched matrix retains the A-cleanup-versus-detach race (§§97–106), failed-Prepare self-join avoidance (3B.3), stop-before-read code 4 / phase 2 or 3 / nil current / no read / one ready close, and a genuine first read failure as successful A with a nonnil lease. The fixture list keeps the manager, helper, and policy sources frozen. This is a material improvement over the rejected “same lease / same drain” case and fits the full original scope without authorizing manager implementation.

## Stop-before-read setup qualification

For case 3A assertions 4–5 (§§55–61), the channel barrier must place the worker before its first `ReadRunTrace` start check, so manager Stop wins the actual mutex transition before any call begins. “No pending Prepare … starts/continues a read after Stop wins” must not be interpreted as requiring an already-started blocked port call to disappear. An in-flight adapter call may return only after cancellation and retirement; the manager must hold capacity and join it under the original lifecycle contract. The supplement’s separate actual-return/JOIN case remains responsible for that path. Under this explicit test setup, code 4/phase 2-or-3 no-read semantics are correctly tested.

## Material scope differences

This supplement only repairs future lifecycle test-matrix language; it creates no new API, state field, code, test, or lifecycle implementation authority. The pure encoder assignment remains separate. The later combined-image budget and wrapper design are still isolated from public APIs, `Next`/delta/SSE, auth policy, schema, and the unchanged retained helper. Root’s pointer-identity decision remains conditional on one publisher and actual JOIN-before-reuse. Initial failed selected reads remain successful A; auth, guard, cancellation, stop, and irreducible assembly failures remain failed Prepare with typed error/zero wrapper/nil lease and owned rollback.

Approval is limited to the bounded design/test-first plan, with the case 3A setup qualification stated above. Root must separately release any fixture or source phase, and a fresh implementation source review remains required before compiler ownership.
