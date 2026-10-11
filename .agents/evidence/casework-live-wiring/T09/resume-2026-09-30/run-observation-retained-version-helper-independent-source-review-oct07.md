# Independent source review: retained-version helper scaffold

Date: 2026-10-07  
Disposition: **REJECT test-first fixture approval pending complete atomicity and boundary assertions**

## Scope and frozen identities

Reviewed the immutable helper assignment and source receipt/hash correction,
root's retained-publisher decomposition, the Unit 1 and revision 6 governing
records/corrections, the complete new helper and its tests, and relevant
`RunTraceSnapshot` / frame port declarations. No Go, tests, compiler,
typecheck, scanner, formatter, runtime, or Git command was run.

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `run_observation_retained_version.go` | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` | 5,836 |
| `run_observation_retained_version_test.go` | `5365c1ff47cdf0006868b102a1fdcdf9c77a12d51cee4ab06b04af536f494978` | 12,464 |
| helper assignment | `cc1fed59eb1f440e53756d3f2adee79369d0c57f17f800481b9c92a1093c446a` | 9,829 |

The final source/test hashes agree with the separate hash correction, which
explicitly supersedes the earlier pre-finalization hashes in the source
receipt. The helper is a newly added private pure unit. Its candidate builder
is an explicit always-rejected stub; no lifecycle, manager, `Next`, auth,
locking, I/O, or production integration was added. The scaffold boundary is
within the assigned release. The tests' first accepted-candidate assertion
fails on that stub as a semantic expectation; this review makes no claim that
the command ran or that compilation was established.

## Image representation review

`marshalRunObservationRetainedImage` serializes the ratified fields once:
exact case/run/plan key; generation and accepted timestamp; execution,
settlement, observation state and source total; retained/omitted frame counts
and derived truncation flag; high-water; availability; source-order frames
with event ID/kind/timestamp/optional pointer values/first ordinal; and the
full exact-ID ledger in deterministic `sort.Strings` order
(`run_observation_retained_version.go:90-172`). The canonical shape has a
schema-version tag and no duplicate next-ordinal field. Its pointer projection
copies `ExecutionStatus` and `ExitCode` values before JSON serialization.

Generation, high-water, and each first ordinal are encoded as 20-digit
decimal strings. The availability enum is range-checked and its five ratified
values encode as one decimal digit. Root ratified one availability state
covering current, read-unavailable, retention-unavailable, terminal-retention
failure, and stop-scheduling; one enum is consistent with that decision.
`Truncated` is a derived source-window metadata boolean, not a separately
changing control marker. Terminal/nonterminal marker-only transitions retain
the old total and frame window, so this derived boolean does not change during
those transitions; a newly accepted window recomputes the full candidate image
and its budget. I found no fixed-width defect in that distinction.

The stub intentionally does not implement candidate validation, cloning,
ordinal assignment, budget decisions, or marker transitions. These invariants
must be demonstrated by the fixtures before source release; the current tests
cover some but not all of the required outcomes.

## Blocking fixture findings

### 1. Nonterminal rejection does not assert full previous-state atomicity

`TestRunObservationRetainedCandidateRejectsBudgetAtomicallyAndRecovers`
captures canonical bytes at lines 113-120, but only later checks `len(before) >
0` (`:149-151`). It never proves those bytes or the complete `prior` state
remain unchanged. The rejected result is checked for ledger length,
high-water, and frame values (`:124-136`), but not exact `SeenByID` contents,
first ordinals, generation, accepted timestamp, execution/settlement/
observation state, source total, or all retained metadata. A same-size ledger
substitution or other prior-state corruption could evade those assertions.
The refusal image is checked only for `len(after) <= 1<<20` (`:132-136`), not
that changing the fixed-width availability marker leaves the old image's
encoded length and every old safe value unchanged. The later prior-ledger
comparison (`:142-144`) compares the input to a freshly constructed test
state, not the rejected result's entire ledger.

The recovery assertion (`:145-153`) checks acceptance, current availability,
high-water, and the new ID's ordinal. It does not compare the complete
recovered ledger (`opaque-A`=1, `opaque-B`=2, new ID=3) or prove that a
previously absent ID reappears with its original first ordinal. Root's model
requires the complete lifetime ledger to survive refusal/recovery exactly.

### 2. Terminal candidate-data nonretention is not fully asserted

`TestRunObservationRetainedCandidateTerminalRefusalKeepsOnlySafePriorState`
checks that the candidate ID is absent from `SeenByID`, execution does not
become `completed`, and the candidate timestamp is not accepted (`:156-172`).
It does not compare retained `Frames` to the prior safe frames or inspect the
canonical image for the candidate event ID/payload. A result can therefore
retain the over-budget candidate frame in `Frames` while still satisfying the
tested ledger/standing/timestamp checks. This fails the explicit no-candidate-
data retention requirement. Assert full prior safe state plus only the safe
terminal marker, and verify the candidate ID and other candidate-only values
are absent from every retained/image field.

### 3. Exact and one-byte-over image fixtures do not validate their byte sizes

`TestRunObservationRetainedImageFixedWidthControlsAndExactLimit` computes a
padding length intended to place the image exactly at `1<<20` and adds one
identity byte for the over case (`:207-229`), but it does not marshal the
accepted candidate state and assert its actual length equals the bound, nor
show that the rejected case's would-be canonical candidate is exactly one byte
over. The assertions only check the stub's accepted/rejected outcome. If the
fixture's assumed equivalence between `baseState` and the builder result is
wrong, the named exact-boundary test can still pass/fail for an unrelated
size. Assert the actual canonical image length is exactly the maximum for the
accepted case and exactly maximum+1 for the rejected case, while retaining the
no-truncation/irreducible-metadata expectations.

### 4. Declared identity and counter cases remain uncovered

The reorder test covers IDs repeated across successive windows and a payload
timestamp rewrite while present, then a shorter window (`:84-111`). It does
not exercise duplicate occurrences in one source window or disappearance
followed by reappearance after that shorter window. Those are material to the
exact first-ordinal ledger rule and should assert unchanged original ordinals
and exact source order.

The test covers ordinal overflow (`:232-260`) but does not exercise the
declared `retainedCandidateGenerationOverflow` outcome (`go:58-59`). The
ordinal-overflow assertion also checks only the outcome, nonnil state,
high-water, and a zero-value lookup for the new ID; it does not deep-compare
the full prior state/map to prove no partial mutation. Add the required
generation-overflow behavior or explicitly resolve its status with root before
the test-first matrix is approved.

## Correctly covered items and scope

The fresh-candidate pointer test checks both optional pointers are distinct
from the input, then mutates the retained pointer values and confirms the
input is unchanged (`:63-82`). The 1,024/1,025-frame assertions are explicit
and consistent with the 1,024 post-decode safe-frame window (`:232-250`).
Fixed-width tests encode `MaxUint64` as 20 decimal digits and verify all five
availability codes preserve marshaled length (`:174-205`). Tests are
synchronous and have no channels, sleeps, timers, or setup waits; I found no
fixture deadlock or timer-cost issue.

Source shape follows the root decomposition: the exact private retained image
and marshaller are local to the new helper; marker state is one availability
enum; and the builder remains a visibly unavailable stub. No forbidden manager
or public integration work was found. These correct aspects do not close the
atomicity, terminal leakage, exact-size, and identity-history coverage gaps
above.

## Disposition and limits

Reject the current test-first fixture bundle until the full-state atomicity,
terminal nonretention, actual exact/+1-byte image-size assertions, and missing
ledger/counter cases are repaired in a new immutable candidate. This is not an
algorithm rejection: the builder is intentionally a stub, and no algorithm
was reviewed or approved. No source/test execution or GREEN/RED observation is
claimed. The previous result receipt and its hash correction remain immutable.

Graft first-pass retrieval saved approximately 8,937 tokens (<$0.01) this
turn.
