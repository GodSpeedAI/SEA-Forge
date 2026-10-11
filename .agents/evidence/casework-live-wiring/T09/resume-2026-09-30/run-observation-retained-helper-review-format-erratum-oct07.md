# Retained helper review — formatting erratum and scope note

Date: 2026-10-07. Append-only correction to
`run-observation-retained-helper-algorithm-independent-review-oct07.md`.

## Formatting correction

The earlier review's statement “No material algorithm mismatch or formatting
defect was found” is corrected. Root reports that a read-only `gofmt -d` of the
pinned helper produced four formatting hunks covering literal-column
alignment and the `retainedImage` struct alignment. I did not run another
formatter and make no independent formatter-output claim here. These are
format-only findings; they do not change the semantic approval of the bounded
pure-helper algorithm. The earlier review should not be read as asserting that
the file is gofmt-clean.

If formatting is repaired, keep the builder's scope to that single helper file
and the exact formatter diff; do not combine it with algorithm, tests, or
lifecycle edits. Recheck the source hash and inspect the resulting diff before
any new verification claim.

## Combined image design note — root reconstruction still pending

The existing `marshalRunObservationRetainedImage` accounts for the retained
helper state only (`run_observation_retained_version.go:222-310`). Its 1 MiB
check does not prove that the eventual complete per-poller manager state fits
the same limit. Root decision 7 explicitly reserves additional lifecycle state
for full integration and prohibits a full-bound claim until that state is
accounted for.

For root's source reconstruction, inventory every manager-owned per-poller
field and identify its single owner before choosing the accounting shape. The
eventual budget proof should use one deterministic canonical wrapper that
encodes the retained helper image and every actual per-poller lifecycle field
exactly once, then checks the combined serialized byte length before
publication. Avoid assigning a second independent 1 MiB allowance or
speculating fields before the manager model is reconstructed. This is design
advice only; it does not authorize lifecycle work or alter the pure-helper
approval boundary.

No source/tests were edited and no test, compiler, formatter, or runtime command
was run for this erratum.
