# Private Next TDD Builder Result

Date: 2026-10-09. Immutable builder record for the released TDD-fixture-only
phase. This is source-preparation evidence only; no RED, compiler, test, gate,
runtime, or implementation result is claimed.

## Source identities after final edit

| File | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `b45dc39bc7466160cec9bc441d2635d577ceae69f64f165f19dbca7f7605737d` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `2e5ef4ee58b1c821b6b5a9fdee83a2bce38f504dae3001f08e2b73306f350766` |
| `apps/godspeed-casework-go/internal/server/run_observation_next.go` | `f362b461744b7b474f71a6d91c01f01eba76752c4c82761af5fa06ddb3db4015` |
| `apps/godspeed-casework-go/internal/server/run_observation_next_test.go` | `c8e8253539ed7d72f61b417f99e2849bcc711fb8401cd395c6c35094ba216261` |

Formatting was inspected nonmutatively with `gofmt -d`; it emitted no diff.
No Go compiler, test, gate, Git, status, or debt command was run by this builder.

## Material changes and limits

- Added only the private Next aggregate, exact pure-projector function type,
  private already-in-progress error type, and compiling unavailable stub.
- Added only future lease ownership field declarations required by the TDD
  fixtures: wake, operation wait group, notifier wait group, and in-flight
  token.
- Added private notifier-target acquisition/send helper signatures as inert
  compiling stubs, plus tests that call the actual registration path and hold
  its acquired target across drain. This follows the root-approved proof seam
  recorded below; no manager-global callback or alternate lifecycle path was
  added.
- Added focused real-Prepare/worker/read fixtures for complete-empty versus
  unavailable list, fresh caller context, accepted terminal/no replay after
  hydration pruning, multi-run all-or-nothing failure, transient read failure
  recovery, nonterminal retention recovery, terminal retention stop, capture
  while the worker publishes multiple later generations, wake coalescing,
  independent shared-lease watermarks, shared survivor, overlapping Next,
  authorization/session and cursor refusal, caller cancellation, Stop, detach,
  and counted operation/notifier/worker joins.
- No lifecycle behavior, notifier behavior, capture/commit algorithm, old
  assertion, or pure delta helper was implemented or changed. No watermarks or
  retained state were manually changed by the fixtures.
- Root owns the actual behavioral RED and compilation. Fresh implementation
  builder and independent critic review remain required.

## Root-approved notifier test seam

Root decision, 2026-10-09: “do NOT add notifier callback/hook. Factor normal
production notifier path into private acquire-targets-under-mu helper (checks
exact eligible attached leases, does real per-target notifyWG.Add) and private
send-and-Done-outside-mu helper. Worker production uses these exact helpers.
Deterministic ownership unit test may call actual registration helper under mu
on an actual Prepare lease/entry, hold returned registered targets across
drain, then invoke actual send/Done helper; assert capacity retained until real
notifier release and drain joins. No hand-adding WaitGroup counts, invented
lease state or alternate lifecycle. For TDD compile stub may declare these
private helper signatures returning no targets/noeffect, expectation should
fail behavioralRED. This is a natural production decomposition, not a mutable
hook.”

Root also clarified that a lease whose initial Prepare accepted list/attachment
but had no `retainedCurrent` capture has no scalar baseline. Fixtures must not
invent one; a later fitting value uses zero prior and delivers its genuine
first-observed frames. Initial validated captures use their seeded high-water
mark and do not replay hydrated-away frames.

## Full original assignment: `run-observation-private-next-assignment-oct09.md`

```text
# Private Next transaction and lifecycle assignment

Root orchestration decision, 2026-10-09. Source release is HELD until root
confirms checkpoint 9efd039 was published and its actual captures compared.
This document is an implementation specification, not runtime evidence.

## Authority and bounded scope

Read root and nearest instructions, the current canonical status snapshot,
the governing casework spec, manager concrete proposal revision6/addendum,
retention initializer correction revision3 and its independent review, private
policy operator approval, and next-integration-root-decisions-oct08.md in this
directory. Earlier proposal-only labels are superseded only by the recorded
specific policy approval. Preserve all existing authority and admission paths.

The source scope is exactly manager.go, poller_worker.go, and new private
run_observation_next.go / run_observation_next_test.go under the Go server
package. Preserve every existing assertion and the accepted pure per-run
delta helper. No public SSE/UI/wire/schema, dependency, persisted storage,
authority, retention budget, polling/admission, or unrelated formatting change.
Read each file before editing and use native apply_patch for persistent writes.

## Required transaction

Provide private lease next(ctx) returning a private aggregate with exact
case/as-of cursor, observation time, nonnil runs and window gaps. Use existing
private deltas and scalar watermarks; sort runs/gaps by exact run-ID bytes and
retain captured frame source order. Do not invent source continuity.

Each call uses its own context. Under manager.mu admit at most one Next for
an exact active, non-draining lease and register its operation before
releasing the mutex. No operation or per-target notifier Add after drain
begins. Wait outside locks on capacity-one wake (never closed), caller
cancellation and leaseDone. A successful Prepare seeds one coalesced wake,
including complete empty and successful terminal cohorts; unavailable list
fails typed without waiting. This permits the initial call without replaying
hydrated-away frames; subsequent calls require publication/wake or
cancellation.

Capture all exact attached entry pointers, immutable current pointers, copied
SeenByID ledgers and prior watermarks together under the mutex. Assemble only
those captured values outside it. Reuse current authorization/present-context
checks outside the mutex before and immediately before disclosure. Commit all
candidate watermarks together only under final exact active-cohort/token,
manager-entry, forward/reverse attachment, non-stop/non-drain and
caller-context checks. Do not require current-pointer equality: publication
during assembly belongs to a subsequent call and must retain a coalesced wake.
Never recapture or retry the aggregate. Any run failure returns zero
aggregate/no advancement.

Read-unavailable and nonterminal retention-unavailable retain refs/capacity and
may recover only after a later complete fitting candidate. Successful terminal
retainedCurrent remains deliverable after workerDone. Terminal retention stop,
auth/session/cursor/context failure, cancellation, detach and Stop schedule the
single drain owner. Release the call's own operation before any drain wait to
avoid self-deadlock. Drain joins creator, operations, per-target notifiers and
actual owned workers outside mu before removing counted capacity; timeouts do
not free it. Preserve work still required by a surviving authorized lease.

Publishers acquire exact eligible target notifier references under mu, then
send nonblocking and balance Done outside it even when sends coalesce. No
callbacks, auth/guard calls, sends, cancel, waits, reads or joins under mu.

## Deterministic proof and review loop

Production next delegates to one private transaction helper with a typed pure
projector function argument fixed to the existing assembler. Tests may wrap
that same assembler with a barrier after capture and before projection. No
manager-global mutable hook, alternate lifecycle implementation or bypass.

First submit tightly scoped compiling TDD fixtures and a minimal typed-error
stub if required for the missing private method. Root owns actual behavioral
RED and compiler token. Freeze after source review; do not implement before
root accepts expected RED. A fresh builder then implements the full unit.

Prove complete-empty versus unavailable and fresh context after Prepare;
multi-run all-or-nothing/no candidate commit and fitting recovery; exact
captured generation despite publication during projection and next-call wake;
detach between capture and commit with counted actual JOIN; shared survivor;
single-call rejection; successful terminal and terminal-retention disposition;
auth/session/cursor/cancellation/Stop races; notifier coalescing/ownership;
initial pre-hydration no replay and distinct lease watermarks. Prefer barriers
and exact counters over sleeps; never manipulate watermarks to simulate Next.

Independent critic receives this FULL assignment plus resulting diff, verifies
all material deviations and source/runtime evidence, and cannot approve on
insufficient evidence. Rejection requires a fresh builder and re-review.
Required actual gates: focused manager/Next race, just casework-go-check, and
full-module Go race. Only root's granted compiler owner runs gates; actual
RAM/swap/no competing heavy process preflight applies. Immediately after each
JOIN archive all six actual captures and root losslessly compares before any
next gate. No T09 settlement or public readiness follows from private approval.
```

## Full mandatory addendum: `run-observation-private-next-assignment-addendum-oct09.md`

```text
# Private Next assignment sequencing clarification

Root decision, 2026-10-09. This supplements the immutable private-next
assignment in this directory; source release remains held until publication.

Admit the exact single-call token and register its operation under manager.mu
before invoking authorization or context callbacks. This makes any blocked
callback owned by the drain. Outside the mutex, authorize and check present
context before blocking on wake. After wake, revalidate authorization/context
before capturing and projecting; immediately before final disclosure revalidate
again. All callbacks remain outside mu. A failed terminal check schedules the
single drain owner, returns zero aggregate/no commit, and releases the call's
own operation before waiting for teardown. These checks are as-of observations,
not continuous revocation or atomic Store/Relay promises.

A concurrently attempted second Next is rejected with the private typed
already-in-progress distinction. It owns no operation/token and must not drain
or invalidate the already admitted call or consume its wake. This is distinct
from an admitted call's terminal auth/context/cancellation failure.

The deterministic projector seam wraps the existing pure per-run
buildRunObservationRunDelta function. Production supplies that exact function;
tests may pause its first invocation after the entire captured set has been
frozen. Aggregation and lifecycle use the same transaction helper in both
cases. No manager-global callback, fake commit path, public interface or
weaker test claim is introduced.
```

## Full source-phase release from root

```text
RELEASE bounded Next TDD source phase now: checkpoint9efd039 published,
push02 actual0 CI1110pass0fail4ignored/noLeaks526commits, ALL6rootcmp;
remote01 actual0 ALL6rootcmp exactremoteSHA9efd039. FULL original instructions
are BASE/run-observation-private-next-assignment-oct09.md PLUS mandatory
addendum-oct09.md, reviewed ccc150c1. Read them fully; this phase ONLY
compiling focused TDD fixtures, minimal private aggregate/projector/next
typed-error stub and field declarations necessary to compile missing API; do
NOT implement lifecycle/notifications/capture/commit or overwrite approved
pure delta/oldassertions. Scope exact4Go files. Cover ALL specified vectors
using nested tests if economical, actual Prepare/worker/read path and
deterministic barriers; no manualwatermarkmutation. No
compiler/tests/gates/Git/status/DEBT writes; root owns compiler and actualRED.
Nativeapply_patch only. Before source writes send brief intendedstub/fixture
grouping if needed; after write freeze, report completepath+sourcehash
identities and everydeviation in NEW immutable
private-next-tdd-builder-result-oct09.md inBASE via nativepatch AFTER
finalsourceedit (never mutate artifacts). Prior graphs/specs source authority
asassignment. Escalate any ambiguity promptly; root will help with
semanticreasoning. Critic gets FULL original assignment+addendum+thisrelease
and result. No compiler.
```

Root approved an additional deterministic notifier ownership proof: use the
actual locked target-registration helper and actual outside-lock send/Done
helper; hold a registered target while drain runs; verify capacity remains
counted until that target is released. No test callback, manually-added wait
group count, or alternate lifecycle is allowed.

