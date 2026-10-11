# Independent watcher and terminal algorithm source review

Date: 2026-10-08

## Scope and verdict

I reviewed the complete Phase B algorithm assignment and result, the original
manager assignment and revision 6 correction chain, watcher/terminal proposal
and clarifications, the focused RED record, the three exact source preimages,
the final manager and worker, the authority/terminal fixture, and the frozen
manager/failure/primitive fixtures.

**APPROVE source readiness for the separately owned focused verification.**
The implementation satisfies the reviewed private algorithm requirements at
the source level. This does not claim compilation, GREEN, race cleanliness,
runtime correctness, integration, or T09 completion. No gate or formatter was
run during this review. The missing original `/tmp` formatter paths and a
worker formatter-input identity mismatch are material evidence-provenance
deviations; root must resolve formatter evidence separately before treating
that provenance as complete.

## Identity and scope checks

The full implementation grant is
`run-observation-watcher-terminal-algorithm-assignment-oct08.md`, SHA-256
`a886842c0d7d7ba4673916862a6bd56d1b2d2b336fd6342a983cda697e714ab2`. The
source result is `run-observation-watcher-terminal-algorithm-result-oct08.md`,
SHA-256 `b1c7e71370b1b523950be8fd9e2750d445d10eaf0e35030dcc521d539b5ba1da`.
I decoded and hashed each archived preimage and compared its declared path,
byte count, and SHA-256:

| File | Preimage SHA-256 / bytes | Final SHA-256 / bytes |
|---|---|---|
| `run_observation_manager.go` | `b7f0511b5a1690253d3adb0132beafc106d3b03f5218c6bde3f1c40f4217884a` / 26,548 | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` / 29,377 |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` / 8,873 | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` / 13,217 |
| `run_observation_manager_authority_terminal_test.go` | `ea181a2f5781a7b11fd683642cca89b89c2d902ea44d8ea276a5c9c5c1d4a829` / 41,237 | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` / 41,286 |

These identities were derived from the decoded preimages and actual files; no
hash was transcribed from the builder's abbreviated status messages.

The full frozen-file hash set matches the source result and the values verified
in this review:

| Frozen file | SHA-256 |
|---|---|
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_manager_failure_test.go` | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

Only the two authorized implementation files change algorithm behavior. The
authority/terminal fixture differs from its preimage in three formatting
hunks: field alignment, one composite-literal alignment, and indentation of
the existing `during-held-read` subtest body. Direct preimage diff shows no
test/helper/assertion changes. Decoded stdout in each archived formatter JSON
matches its final file byte-for-byte.

## Contract review

1. **Admission, batch launch, and counts.** The manager reserves a cohort and
   creator under `mu` before list work (`run_observation_manager.go:112-129,
   329-370`). It validates request identity/current context before admission,
   checks again after the list and before attachment/handoff (`:329-348,
   :386-414, :449-463, :562-570`). `launchPollerBatch` starts every owned entry
   before the first readiness wait (`run_observation_poller_worker.go:9-15`,
   manager `:456-489`). The selected-row accounting preserves `S`, `A`, `C`,
   `O`, unreadable count, no-backfill behavior, and the accepted exhaustion
   interpretation (`manager.go:416-458, 491-552`). Counts are derived from
   owned batch entries whose initializer completed after an actual call;
   pre-call stop/cancel paths fail Prepare rather than counting a reservation.

2. **Caller identity, context guard, and final handoff.** The lease stores only
   the required immutable caller value and captured cursor (`manager.go:81-90,
   112-129`); `runObservationCallerFromRequest` rejects malformed identity
   without retaining a bearer token (`:304-319`). `authorizeLeasePresent`
   invokes authorization and Store/Relay guard outside the manager mutex and
   checks case, exact captured cursor, and each requested parent (`:588-612`).
   All successful initial DTO branches, including empty and list-unavailable,
   recheck authorization/guard and membership. `leaseCanDisclose` checks the
   nonblocking `prepareDone` channel, current exact lease membership, retained
   pointer, and `retainedCurrent` state while holding `mu` (`:615-646`). The
   local finalizer cancels the Prepare context only after that handoff
   linearization, joins its cancellation bridge, then completes creator
   accounting (`:350-381, :566-570`). This matches revision 6's boundary that
   the request context owns this Prepare's list and initial wait while the
   shared worker has its manager-owned context (`revision6-oct06.md:294-327`);
   a caller leaving does not directly cancel a shared worker still required by
   another eligible lease.

3. **Read eligibility and new-member races.** The worker checks real map entry,
   exact prior pointer, phase, and eligible forward membership. It snapshots
   each eligible watcher under `mu`, invokes authorization/current-session,
   perspective, and exact cursor/parent guard checks outside `mu`, then
   rechecks state and membership. A member admitted during external checks is
   independently validated before the worker proceeds (`worker.go:151-241,
   :71-107`). The actual worker context is checked immediately after the final
   membership check and before `ReadRunTrace` (`:101-107`). Invalid watchers
   enter existing lease drain; an independently authorized survivor continues
   to authorize the shared call (`:185-223, :227-241`). These paths satisfy
   the parent’s newcomer-during-validation requirement without an extra
   generation or persisted control field.

4. **Candidate publication and immutable limits.** The worker builds a
   complete immutable candidate, checks the unchanged retained and outer image
   caps, derives terminal fallback from the final post-fallback state, then
   repeats watcher authorization/guard validation before publication. It
   rechecks entry, prior pointer, phase, and validated membership under `mu`
   before swapping the state pointer (`worker.go:270-429`). No callback,
   context method, I/O, cancellation, channel close/send, wait, or join is
   performed under `mu`. The existing pure encoder and ledger helpers are
   unchanged, and the eight frozen helper/fixture files retain their hashes.

5. **Terminal and retention behavior.** A fitting terminal snapshot is
   published as current, `continuePolling` becomes false, the worker returns,
   and the read claim explicitly refuses another terminal read
   (`worker.go:19-47, :291-296, :401-429`). It does not drain valid leases just
   because terminal data fit; those leases can reuse the current version and
   release capacity only through actual detach/Stop join (`manager.go:219-223,
   :785-815`). An unretainable terminal candidate derives stop/drain after the
   outer-cap fallback, keeps the prior immutable state/ledger with the safe
   marker, signals only exact eligible leases outside the lock, and exits only
   after the actual port call returns (`worker.go:333-365, :401-429`).
   Nonterminal read failure and retention failure preserve their distinct
   marker/recovery rules in the existing retained-version helper and worker
   handling (`retained_version.go:94-167`, `worker.go:282-329`).

6. **Reference, creator, and Stop ownership.** Lease eligibility uses the
   non-draining forward map relation, not raw reverse-ref count
   (`manager.go:264-284, :719-757`). The first drain transition claims the
   lease, removes forward eligibility, retains watcherless reverse references
   until worker completion, and runs cancels/waits outside `mu` (`:759-814`).
   A shared survivor prevents cancellation; detach never waits on unrelated
   creators. Stop closes admission and snapshots/drains leases under `mu`,
   signals/cancels outside it, waits for creator registrations and workers,
   and closes completion once (`:828-908`). Timeout returns without freeing
   capacity (`:816-825`). The creator cancellation bridge and local cancel/join
   ordering follow the approved creator-drain clause; the request's `Done`
   channel is not conflated with the shared worker context.

7. **Fixture reachability.** The seven focused groups exercise actual read and
   list callbacks, fitting terminal shared initialization, recurring outer-cap
   refusal, pre-read and during-read authorization failure, cursor change,
   invalid watcher with a valid survivor both during-read and before-port, and
   four final-handoff cells (`authority_terminal_test.go:273-955`). They use
   explicit channels, bounded waits, and read-only ownership checks. The
   recurring-cap test independently proves a valid inner image with raw
   complete wrapper length above cap and the canonical refusal sentinel
   (`:337-458`). The failure fixture remains frozen and retains its existing
   Stop/Detach/cancellation race cases. The previous focused RED remains only
   a prerequisite; no GREEN is inferred.

## Material deviations and evidence limits

- The fixture edit is formatting-only, as required. No source, tests, public
  surface, dependency, port, or policy outside the authorized scope changed.
- The requested original formatter `/tmp` paths were not preserved. The
  clarification `run-observation-watcher-terminal-formatter-provenance-clarification-oct08.md`
  records that limitation; final-source equality with the archived stdout is
  verified, but the missing original raw paths remain a provenance gap.
- There is a worker formatter-input mismatch: the archived worker pre-format
  source JSON is SHA-256
  `5deebf4b629e24356b2a79dfd95388d9b2d69864d608fc3efdf8c6e2fb239bbe` /
  13,243 bytes, while the gofmt witness declares preformatted SHA-256
  `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` /
  13,217 bytes. Their only hunk removes the redundant
  `|| entry.ctx.Err() != nil` from the `!validWatcher` branch immediately
  before the separate outside-lock context check (`worker.go:97-107`). That
  source hunk is inside the authorized algorithm file and leaves the required
  following check intact, but it is not a gofmt-only change and the witness
  does not prove its input was the archived pre-format source. Root retains
  the separate formatter-evidence decision.
- The builder result says it preserves all operator changes and eleven
  foreign staged blobs. This source-only review does not claim a Git/index
  audit, compile, race test, or runtime result.

The differences above are disclosed rather than treated as semantic test
evidence. I found no source-level contract defect blocking the separately
authorized focused verification. All runtime and integration claims remain
open pending root-owned gates and review.
