# Next projection fixture repair result

Date: 2026-10-08

## Scope and identity

This repair followed the complete bounded release recorded in run-observation-next-projection-fixture-repair-assignment-oct08.md. The target test preimage was SHA-256 6e6e3c0e8388104dca64ea66a3487e0b7174fa75235894bfd5ab669ac04629ce. Its final SHA-256 is af862f3bd18e866db8ccb8c84ef784a80d8a3d0d49e5dc8f47f3969d78a77cef. The frozen seam file run_observation_delta.go remains exactly SHA-256 aff93e3a1b01aedd8075be1f59ff52cebb4dc36de59eaf4140cb939cdf148973.

## Exact change

Only six settlement literals changed in run_observation_delta_test.go, all from invalid "settled" to allowed "accepted":

- TestRunObservationDeltaUsesCapturedOrdinalRangeAndSourceOrder: state fixture and expected delta settlement.
- TestRunObservationDeltaAdvancesWindowAndStandingsWithoutNewFrames: grown fixture and expected delta settlement.
- TestRunObservationDeltaDeepCopiesOptionalFramePointersAndInputs: state fixture.
- TestRunObservationDeltaAcceptsEmptySuccessfulTerminalCapture: state fixture.

The before/after diff contains exactly those six literal substitutions. No test names, test structure, assertions other than the expected enum value, algorithm/stub, type, or other source was changed. Contract authority is apps/godspeed-casework-go/internal/contract/contract.go:436, where accepted is a valid RunSettlementStanding.

## Verification boundary

The two source identities above were checked after the edit, and the focused diff was inspected. No test, compilation, formatter, scanner, build, Git mutation, or runtime gate was run. This does not establish expected RED, algorithm correctness, Next integration, or T09 completion. The repaired test file is ready for a different independent source critic against the full original assignment and repair assignment; root must separately run the authorized expected-RED step after SOURCE-READY.
