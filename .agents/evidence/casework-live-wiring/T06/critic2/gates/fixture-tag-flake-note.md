# Fixture-tag coordinator flake — attribution note (critic2)

`go test -count=1 -tags casework_fixture ./internal/...` flaked ONCE during this
verification (internal/coordinator, TestAcceptedImplementRunsStagedFixtureRevisions:
"acceptance must append revision 1151 immediately, live is 1153"). Attribution runs:

- The failing test passes 4/4 in isolation at HEAD (b5fd2c3).
- The same full sweep at the PRE-FIX commit a1129f1 (throwaway worktree /tmp, since
  removed) also flaked once in five rounds, with a DIFFERENT test
  (TestIdempotencyReplayAndConflict) in the same package.
- At HEAD: 1 flake in ~10 full rounds; 9 subsequent rounds clean.

Conclusion: a PRE-EXISTING order-dependent flake inside internal/coordinator's
fixture-tag suite (shared mutable fixture state across the package's tests). b5fd2c3
touches neither internal/coordinator nor any fixture file, and the flake reproduces at
a1129f1 — not introduced or worsened by the fixes. Non-blocking for T06; recommended
for OBSERVED_DEBT (coordinator-owned; this critic did not edit the ledger).

Evidence: go-test-fixture-tag-critic2.log (the one failing sweep; the 9 clean rounds
were not individually logged — the final 6-round loop printed "clean" per round and is
summarized here verbatim: total flakes in 6 HEAD rounds: 0).
