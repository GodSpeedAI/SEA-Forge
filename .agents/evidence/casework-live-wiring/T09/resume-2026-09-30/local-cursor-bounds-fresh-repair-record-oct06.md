# Local cursor bounds test fixture — fresh repair record

Date: 2026-10-06. TESTONLY source repair; no runtime or compiler claim.

## Assignment and review inputs

The bounded original assignment is `local-subscription-ordered-cursor-revision4-original-assignment-oct06.md` (SHA-256 `0af4db3bfee9d17621eb8f6a1e1f8ebb18c11b456b2f6140accb2a290b31d7b8`). It calls for a revision 4 source-only fixture against the accepted local authored cursor design, with validation, ordering, per-subscriber queue cap 64, cleanup, and deterministic timers; implementation remains held. The concrete proposal is `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md` (SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4a116998740339acd2086131873`).

The repair follows independent source review `local-subscription-ordered-cursor-revision4-tests-independent-source-review-oct06.md` (SHA-256 `c80c591af140180431e9ef505a041d64a359841de9a4556cf311cce08fdb5153`), its test-review assignment (SHA-256 `3eabff31dedc360898729de0b46469a0892934fbe86d06d7b47d4f9773c467bb`), the revision 4 architecture review (SHA-256 `893ddd2e2e75ed63b14689fa19ddbbb8f00d6e11e5725b523e57922eaba359ab`), and its citation erratum (SHA-256 `95ccd15e5fb00ace6a478204be1745d6ab693c6821f9f88f285ed81105018c6f`).

## Repair and assumptions

Repaired leading-zero future-floor setup so it has an actual timer and tests numeric equality at the future boundary. The FIFO test drains the newer healthy subscriber by selecting scheduled callbacks through their timer handles while leaving the older subscriber's callback pending; it asserts 64 ordered deliveries, then only the backlogged subscriber's overflow at event 65 and healthy continuity. Reentrant enqueue now checks the exact event order while active; self-unsubscribe and throwing `onEvent` / `onError` checks are separate. Added template-committed seed agreement and a subsequent cursor allocation check.

The timer helper uses monotonically assigned schedule handles and caps scheduled handles at 1,000 and `runAll` callbacks at 500. Selective drain relies on the fixture registering the intentionally slow subscriber before the healthy subscriber, so the latest schedule handle for each append belongs to healthy. It does not inspect production subscriber state or add production hooks.

For exhaustion, the helper consistently changes the final private snapshot and trajectory point cursor. It conditionally updates the root-selected future private `ordinaryFrontiers` map when present, with `{ epochText, epoch, sequence }` fields. This does not assume that the map exists on the current implementation baseline, and the source RED remains the observable acceptance/failure behavior. The future production implementation must preserve this private test contract or update the fixture and its review together. Progress is invoked through its private method at the ceiling. Settlement exhaustion selects the settlement timer callback (schedule handle 7) directly and asserts contained reporting without snapshot/side-event mutation. These are white-box fixture seams only; no runtime export, option, or public configuration was added.

Committed-case creation is invoked through the existing private `commitCase` method because its ordinary intent path requires asynchronous policy/preflight and would add setup unrelated to seed validation. The deterministic commit path sets the initial snapshot and trajectory point from the same cursor; the test checks that seed and the first subsequent allocation. No invalid committed-seed path is claimed: the current commit path provides no input seam for one, as the independent review documented.

## Scope, hashes, and verification boundary

The original source bytes were copied before editing to `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts.884987.original.raw`; its SHA-256 is `884987fb78dbe869c11addba3bc9a6e76101d41ce4af14f50156a47d5e31bc2f`, equal to the original fixture hash recorded by the independent review. Repaired fixture SHA-256: `3a801b86b5e39457f6b057777bbd29ea535634005eb8bb0211d2d888aaf14e11`. Frozen `localAdapter.cursorOrder.test.ts` SHA-256 remains `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`.

Only the bounds test fixture and this new evidence record were edited; the `.raw` file preserves original native bytes. No production source, frozen test, conformance fixture, config, schema, or public interface was changed. No Bun, typecheck, tests, compiler, build, Git, or network command was run, as assigned. The repaired fixture still requires a separate independent source critic and root authorization before any runtime gate.
