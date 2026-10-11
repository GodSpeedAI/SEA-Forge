# Initial lease bookkeeping implementation source review

**NeatCode · review** · manager implementation · 1 source file · depth: deep

**Verdict** · SOURCE READY for root's serialized runtime gates. The implementation
matches the bounded lease-bookkeeping grant by inspection; this is not runtime,
canonical-gate, full-module, or T09 approval.

## Contract and identity

I read the full original assignment
`run-observation-initial-lease-bookkeeping-assignment-oct08.md` (SHA-256
`2be0bcd6c5f7f043e482716095df32409b0147dac3f920162cd8dd76de659e03`), the
fresh implementation result
`run-observation-initial-lease-bookkeeping-implementation-result-oct08.md`
(`38a1ee193299cf8be841234423ddb655ed51154f7eff4d9dff2d46a4852f18fc`), and
the prior TDD source review (`run-observation-initial-lease-bookkeeping-independent-source-review-oct08.md`,
`ceea832caafc792bb26f4d7f6930dd6a549a3151d85290385dde8440a2340f6f`). The
result includes the full fresh builder grant and full original assignment.

Current hashes were recomputed: manager
`2d01953469e3d7eedc5688ceb68c9e2d64aca86590251dcd11f3a59afbb3a6cd`; frozen
manager test `08852d598e91264a81a745ac9ba08b31e7eeb5cdb60dd01324e578967f260f50`;
delta source `5241e656d65f86a315e37df44b48d5eaae865c92cf80cc306c9bf4fc0fd7e128`;
delta test `48c6d28e658b8bd4e8fd3c6650ff26c73b0c791f35feff83d056ff8f501c89ed`;
worker `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403`.
The implementation is confined to the manager file, as the fresh grant permits;
the test file and all four other listed inputs remain at their frozen hashes.

## Findings

**No blocking finding.** The implementation initializes each lease's own
watermark map while `beginPrepareOperation` holds `m.mu`
(`run_observation_manager.go:118-131`). It records `unavailable` for list
failure and `complete` for successful list results under the same mutex after
verifying the cohort remains active and the manager/lease are not stopping or
draining (`:397-436`). Those values also feed the existing initial DTO path
(`:411,578-581`), preserving the distinction between an empty successful list
and a refused list.

For each valid selected row, the capture and seed are one critical section:
the code checks active cohort, nonstopping/nondraining lease, exact manager
entry, exact lease forward entry, and reverse `entry.refs` membership, then
checks running phase, exact retained-state key, and `retainedCurrent` before
writing the watermark (`:537-553`). The state pointer is read from the current
entry under that lock. The conversion copies only highest ordinal, execution,
settlement, observation state, generation, total/retained/omitted counts, and
truncation (`:678-694`); it stores no frame, pointer, event ID, or second
identity ledger. This happens before `runObservationInitialRun` and before
`boundRunObservationHydration` (`:561-562,577-589`), so later aggregate DTO
pruning cannot erase the captured baseline. The accepted initial state remains
the source for the DTO.

The post-hydration authorization and `leaseCanDisclose` gate remain present at
`:592-600`; the unchanged helper still checks cohort, forward/reverse refs,
current-state identity, phase, key, availability, and prepare cancellation
(`:645-675`). Authorization and guard calls remain outside the mutex. The
implementation adds no public contract, dependency, budget, notifier, wake,
wait-group, or drain change. This satisfies the original requirement that the
new bookkeeping not become an authority token.

## Material differences and evidence limits

The original TDD assignment allowed fields only as uninitialized scaffolding;
the fresh implementation grant explicitly releases initialization, list-state
recording, and scalar seeding. That is the only behavior expansion, and it is
authorized by that new grant. The implementation result says root separately
accepted RED03 and compared all six captures; I did not rerun or independently
recompare those runtime artifacts here. The builder ran no tests, compiler,
formatter, or gates. Root retains those gates. This review establishes only
source-level readiness for that verification.
