# Retained policy fixture — source result receipt

Date: 2026-10-07. Source-only. No compiler, scanner, formatter, typecheck,
test, runtime, or Git command was run. The independent source review and
focused semantic RED remain pending; no implementation release is claimed.

## Immutable assignment and file identities

The full original assignment is preserved in
`run_observation_retained_policy_fixture_assignment_oct07.md` (SHA-256
`ed290103a075fae97b438d3ed755ac4ef6e3553ed8712a973efbb5f607504a83`, 5,202
bytes). Because the requested BASE evidence directory was not used for this
fresh package-local handoff, both assignment and result records are stored
beside the package source as the allowed fallback.

| File | SHA-256 | Bytes | Disposition |
|---|---|---:|---|
| `run_observation_retained_policy_test.go` | `d1b919e4e432afc42940fb6b4da632c8d631a9ff016ed8e0e2eea0075523c8f0` | 12,288 | New focused fixture candidate |
| `run_observation_retained_version.go` | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` | 5,836 | Preserved unchanged |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` | 26,425 | Preserved unchanged |

No helper algorithm, frozen test, manager, or AF fixture source was edited.

## Exact new assertions

All six new test functions use the `TestRunObservationRetainedPolicy` prefix:

* `OverflowMarkersAreExactAndAtomic`: nonterminal ordinal overflow is
  `retainedRetentionUnavailable`; terminal ordinal overflow is
  `retainedTerminalRetentionFailure`; generation overflow is
  `retainedStopScheduling`. Each checks the full prior safe image/state,
  same encoded length, caller prior immutability, and a deep result copy.
* `TerminalityUsesExecutionOnly`: `completed`, `failed`, and `terminated` with
  `unsettled` settlement get the terminal-retention marker; `active` with
  `accepted` or `rejected` settlement gets the nonterminal retention marker.
  Each oversized candidate uses a valid frame with an opaque oversized event
  ID and checks that candidate data does not enter the safe image.
* `FinalMarkersRefuseLaterCandidates`: prior terminal-retention-failure and
  stop-scheduling states reject a later fitting candidate, preserving the
  exact marker, safe state, generation, accepted timestamp, and ledger, with
  nonaliased optional pointers.
* `PreservesResponseCounts`: encoded total/retained/omitted/truncated counts
  follow the returned source total and frame window; a later lower total is
  accepted and reflected in the encoded image.
* `RecoveryAndInputRejection`: read-unavailable recovers to current; requested
  versus snapshot key mismatch, prior versus requested key mismatch, negative
  total, and total below window length return invalid/nil.

This receipt records source identities and intended assertions only. No
behavioral RED, compile, or runtime outcome is claimed. The new file is held
for independent source review before any focused semantic RED attempt; the
algorithm implementation remains held pending root release.
