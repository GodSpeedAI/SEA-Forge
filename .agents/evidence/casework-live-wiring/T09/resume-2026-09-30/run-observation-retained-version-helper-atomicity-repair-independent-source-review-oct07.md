# Independent source review: retained-version helper atomicity fixture repair

Date: 2026-10-07  
Disposition: **REJECT focused test-first fixture pending a valid terminal candidate fixture**

## Scope and identities

Reviewed the full original helper assignment and its source receipt/hash correction, the immutable atomicity-repair assignment/result, the prior independent rejection, the retained-publisher root decomposition, the full current helper and test, and the relevant trace adapter contracts. No compiler, tests, scanner, formatter, typecheck, runtime, or Git command was run.

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `run_observation_retained_version.go` | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` | 5,836 |
| `run_observation_retained_version_test.go` | `6ba0ab0ddbd80614d25c48d6485f1af70a75e8f670a72fac535c8ec128652d00` | 26,472 |
| atomicity-repair assignment | `abf191672db3e08e2c827068dfab97f4564cb0ae2a1f19a865370ef9f7494b20` | 6,541 |
| atomicity-repair result | `0b4f2d7965d7711c032a70b53b46d996e90e5829fcd857e18050644a7da71de7` | 7,626 |

The recorded exact JSON preimage decodes to 12,464 bytes with SHA-256 `5365c1ff47cdf0006868b102a1fdcdf9c77a12d51cee4ab06b04af536f494978`, matching the assignment's preimage identity. The helper source hash is unchanged and remains an explicitly unavailable candidate-builder stub. Only the existing helper test file changed. This review makes no execution or RED/GREEN claim.

## Repaired coverage

The new tests directly address all four groups from the prior rejection:

* Nonterminal budget refusal snapshots and compares the full prior struct and canonical bytes, checks the returned state against a deep clone changed only to the retention-unavailable marker, checks byte-for-byte expected marker image and unchanged image length, and checks recovery's full A/B/new ledger, exact frame order and ordinals, generation, and accepted timestamp (`run_observation_retained_version_test.go:242-306`).
* Terminal refusal checks caller prior immutability, exact prior state plus terminal marker, exact image and image length, and candidate-value nonretention in image and frames (`:308-365`), subject to the invalid-input issue below.
* Reorder, shorter window, disappearance/reappearance, exact original ordinals and source order, and prior-state/image immutability are asserted (`:143-218`). A same-window duplicate is marked invalid without winner selection, consistent with `internal/adapters/sfwp/run_trace.go:143-146` (`:220-240`).
* Candidate actual image is checked at exactly 1 MiB; computed irreducible metadata image is checked at 1 MiB + 1 (`:367-458`). Optional source/prior/result pointers are checked nonnil and nonaliasing before mutation (`:91-141`). Ordinal and generation overflow each compare full prior and result states and images, and max-counter marker widths include frame and ledger ordinals (`:460-540`).

These checks are materially stronger than the original tests and satisfy the previous missing assertions. The fixtures remain synchronous and contain no sleeps, timers, or setup waits. The encoder and test support helper are unchanged beyond the authorized scope; no lifecycle or integration claim follows.

## Blocking issue: terminal fixture uses impossible trace standings

`TestRunObservationRetainedCandidateTerminalRefusalKeepsOnlySafePriorState` constructs its over-budget snapshot with `Execution = "candidate-only-execution"` and `Settlement = "candidate-only-settlement"` (`:323-326`). The existing SFWP trace adapter accepts only `contract.AllRunExecutionStandings` and `contract.AllRunSettlementStandings`; it rejects any other value as unavailable before producing a snapshot (`internal/adapters/sfwp/run_trace.go:88-94`). The actual vocabularies are execution `pending`, `enabled`, `active`, `completed`, `failed`, `terminated` and settlement `unsettled`, `accepted`, `rejected`, `escalated` (`internal/contract/contract.go:434-436`).

The test then requires this malformed snapshot to produce `retainedTerminalRetentionFailure` (`run_observation_retained_version_test.go:327-329`). That cannot demonstrate terminal refusal for a valid source candidate; a correct implementation may reject the invalid input as invalid/read-unavailable before measuring its candidate image. It also uses `candidate-only-status` for a `command_finished` frame (`:317-322`), while the adapter accepts only `completed`, `spawn_failed`, `timed_out`, `sandbox_violation`, and `suspected_sandbox_violation` (`internal/contract/contract.go:426-431`, validated at `internal/adapters/sfwp/run_trace.go:254-270`). Thus multiple invalid fields undermine the asserted terminal path.

Minimal repair: keep terminal-candidate values distinct from the prior image but within the existing vocabularies, for example execution `failed`, settlement `accepted`, and frame status `timed_out`. The prior fixture contains `active`, `unsettled`, and frame status `completed`, so those candidate strings remain absent from the prior terminal image. Keep its exact-state/image comparison, unique candidate ID/timestamp, and exit-code checks. Avoid searching for a shared valid value such as `completed` because it already appears in the prior frame.

## Disposition and limits

Reject this fixture version pending that focused correction. The other prior atomicity, recovery, image-boundary, identity-history, pointer, and overflow findings are repaired at source level. This does not reject or approve the helper algorithm: the builder is still the intentional always-rejected stub. No test, compile, typecheck, or runtime result is claimed. No production behavior or integration is approved.

Graft saved ~42,980 tokens (~$0.03) this turn.
