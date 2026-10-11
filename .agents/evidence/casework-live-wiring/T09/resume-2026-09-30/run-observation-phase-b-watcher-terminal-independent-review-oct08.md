# Independent DOCONLY watcher and terminal correction review

Date: 2026-10-08

## Review assignment

Review the complete DOCONLY builder assignment and resulting watcher/terminal
proposal against the original Phase B source assignment, its independent source
rejection, and revision 6's caller, present-context, read-start, retention,
capacity, and terminal clauses. Decide whether the proposal is sufficiently
complete to release a bounded TDD source-preparation assignment. This review
does not authorize source edits, tests, compilation, formatting, runtime
verification, Git operations, or release of the manager. Preserve all earlier
assignments and reviews. Verify that current implementation and fixture
identities remain frozen. Record material deviations and the exact scope of any
verdict.

## Reviewed inputs

- Full DOCONLY builder assignment:
  `run-observation-phase-b-watcher-terminal-repair-assignment-oct08.md`,
  SHA-256 `7ffdd48ccbfd69d4ff0e8bf5d7812405b7385651c5b5396e8b4c30b4e8deb852`.
- Actual proposal:
  `run-observation-phase-b-watcher-terminal-repair-proposal-oct08.md`,
  SHA-256 `de8c1647153e06a5fc492671d14c36d3e562f1cd1f079086b62546304e90f0ad`.
- Original two-file algorithm assignment:
  `run-observation-phase-b-algorithm-source-assignment-oct08.md`,
  SHA-256 `e5fc9e0aa9a8549aa75bcf78040b7012e79e8a882a176965dc80ce6eace8ef42`.
- Full prior independent source rejection:
  `run-observation-phase-b-algorithm-independent-review-oct08.md`,
  SHA-256 `ecf2e06f34a57257cc99480dc17a41b40cdaff0a34248ceeb7c940c91473b644`.
- Revision 6 source of truth:
  `run-observation-manager-concrete-proposal-revision6-oct06.md`, especially
  clauses 246–258, 329–348, 361–390, 392–409, 427–447 and 463–468.
- Additive root clarification:
  `run-observation-phase-b-watcher-terminal-root-clarification-oct08.md`,
  SHA-256 `92c0a5bcbdc309c1f0e09c3ac4bf61dc350aee685985a5a824ee2dff72cabba5`.

The correction to the original source grant is limited to the two required
private lease metadata values and required admission inputs. The old grant's
claim that existing fields sufficed is superseded because revision 6 already
requires watcher identity and `asOfCursor`. The remaining original capacity,
ownership, immutable image, cancellation, and no-public-surface constraints
remain binding.

## Frozen source identity check

The actual current hashes match the source review's previously recorded
identities. All ten requested paths are listed here; the two algorithm files
and eight frozen fixtures/helpers were independently hashed for this review.

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `d7be1d7a724ed65575ccaaf35cdf1aa5af133dbdd4952d592699b7eb443b6350` |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` |
| `run_observation_manager_failure_test.go` | `ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

No source or fixture was changed. This is a read-only identity check, not a
compiler or runtime result.

## Verdict

**REJECT TDD source-preparation readiness pending a narrow fixture-matrix
addition.** The private architecture proposal itself corrects the identity
omission, preserves the existing auth model, and describes coherent per-watcher
read and publication checks. Its six proposed cases do not prove all of its
required handoff branches. In particular, there is no deterministic case for
reauthorization and cursor validation before returning an empty-list or
list-unavailable DTO. Those are separate early-return paths, and current source
returns from the list-error path before its later guard check.

## Architecture findings

1. **The private identity correction is necessary and within the existing
   contract.** Revision 6 includes `identity watcherIdentity` and `asOfCursor`
   on each cohort lease (clauses 246–258). It requires a fresh watcher auth and
   present-context check before each selected read/attachment, publication,
   initial handoff, and later disclosure (329–348). The proposal adds exactly
   `runObservationCaller` and a cursor string, passes both as required
   `beginPrepareOperation` inputs after real request identity resolution and
   the existing guard, and excludes bearer tokens, session pointers, optional
   modes, public fields, and new result codes. This is the minimal private
   state needed to re-check the existing authorization model; it does not
   expand that model.

2. **The per-watcher shared-read protocol is coherent as proposed.** The
   proposal captures exact forward membership and entry/version under the
   mutex, checks each relevant caller and its case/cursor/parent context
   outside the mutex, drains invalid watchers independently, and revalidates
   the entry/pointer/eligible membership before read or publication. This
   preserves an authorized shared survivor when a different lease becomes
   invalid. It also puts the worker context check after the final mutex check
   and immediately before the port call. That is consistent with revision 6's
   non-atomic Store/Relay as-of limit and exact lease membership rules; it does
   not promise continuous authorization truth.

3. **The terminal clarification resolves worker/DTO ordering without a new
   wait or lifecycle field.** The root clarification says an accepted fitting
   terminal snapshot publishes immutable current state and readiness, then the
   worker returns without another read; Prepare may construct and return the
   final DTO afterward from that retained value. The worker must not wait for
   DTO or creator completion, valid leases are not drained merely for accepting
   a fitting terminal state, and capacity remains until real Detach/Stop
   completion. I interpret the proposal's “finish any DTO handoff” as
   preserving and resolving readiness for that final value, not making the
   worker wait for Prepare's DTO return. This matches the additive
   clarification and avoids self-join or a new synchronization field. The
   valid retained accepted phase/current remains usable for the initial DTO and
   authorized cache reuse, while terminal current must bar any new read claim.

4. **Unretainable terminal disposition uses the final marker state.** The
   proposal recomputes stop/drain from the post-outer-image-fallback state,
   publishes only the existing safe marker, prevents another read start, and
   retains capacity through actual read return and worker join. This addresses
   the stale `terminalStop` defect in the rejected implementation without
   inventing settlement or a new marker.

5. **The existing read/count/capacity rules remain stated.** The proposal
   retains the 16 cohort, 16 poller, 128 attachment and 1 MiB complete-image
   limits; actual-read `A` counts, cache/shared zero counts, Stop-before-first-
   read behavior, launch-all-before-wait, and exact cleanup/JOIN ownership. It
   does not change public APIs, DTOs, authorization policy, ports, schemas, or
   dependencies. The unreadable-ID count remains backed by the production
   adapter's duplicate/overlap rejection; this proposal does not claim that an
   arbitrary replacement list reader is self-validating.

## Required test-matrix correction

The proposal mandates final Prepare reauthorization and cursor/guard validation
“including list-unavailable and empty list branches,” but the six tests cover
pre-read invalidation, invalidation during a held actual read, cursor changes
during a held read, and shared-survivor cleanup. None changes auth or cursor
after list entry and before handoff when the list returns an error or zero rows.
The existing source has a distinct early return for `listErr` before the later
present-context guard (`run_observation_manager.go:382-403`); the empty-list
path also has no actual read to exercise the held-read tests. A fix could pass
all six listed tests while still returning an unavailable or `no_runs` DTO to
a watcher whose current authorization/cursor changed during the list call.

Extend the focused matrix before source preparation, preferably as one
deterministic table-driven Prepare-handoff case with list-error and empty-list
subcases. Use the real list boundary to signal entry and return only after the
test changes the injected session/guard state; assert typed failure, zero DTO,
nil lease, and owned cleanup. Preserve the existing six cases and all frozen
assertions. Also make terminal case 1 assert owner `readsAttempted=1`, a shared
waiter/cache result `readsAttempted=0`, and no subsequent call, so it directly
proves the root clarification's cache/no-new-claim rule rather than only the
single owner's first publication.

These are narrow test-proof additions, not a request for a new production
field, mode, hook, endpoint, or authorization policy. With these additions,
the described private architecture is otherwise suitable for a separately
released TDD-first source-preparation review. This verdict does not approve
implementation, GREEN, race/runtime behavior, or lifecycle completion.

## Root clarification preserved verbatim

The following is the complete additive clarification reviewed above:

> For an accepted fitting terminal snapshot, publish the immutable valid current
> state and resolve initializer readiness. Then the worker returns without a
> later read; actual worker completion may precede Prepare's final DTO return.
> The valid retained state remains available for that DTO and authorized cache
> reuse. Do not make the worker wait for DTO/creator completion, and do not drain
> valid leases just because a fitting terminal state was accepted. Keep the
> existing accepted phase/current representation; bar new claims for terminal
> current state. Detach/Stop releases capacity only after actual completion.
> Thus the proposal's handoff wording concerns preservation of the final value,
> not an additional worker wait or a new completion field.
>
> An invalid or canceled creator does not authorize a shared read. A different
> eligible watcher that passes the existing current authorization and exact
> cursor/parent checks can still authorize that manager-owned shared worker.
> Drain the invalid watcher independently; preserve the valid survivor. Do not
> equate loss of the initiating caller with loss of every watcher's authority.
>
> This clarifies the existing independent-watcher and immutable-publication
> contract. All other proposal limits and independent review requirements stand.

No compiler, formatter, scanner, runtime, or Git operation was run. The Graft
context calls used for this review are source orientation only and do not
constitute implementation verification.
