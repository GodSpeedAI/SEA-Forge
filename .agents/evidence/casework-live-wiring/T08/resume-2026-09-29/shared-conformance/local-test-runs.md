# Focused local shared-suite runs

RAM was checked with `free -h` before each Bun command. Available memory was
2.3 GiB before the first run and 2.4 GiB before the rerun.

## First run

Command: `bun test src/adapters/local/localAdapter.test.ts`

Working directory: `apps/godspeed-cognitive-ui`

Exit code: 1. The initial shared suite failed because the local fixture's
simulated execution was still emitting progress and settlement events when the
future-only subscription was opened. The failure was preserved and used to
correct the assertion: future-only now means no replay of cursors at or before
the trajectory head at subscription time, while later simulated events remain
valid.

```text
bun test v1.4.0 (34cbb9a40)

src/adapters/local/localAdapter.test.ts:
27 |   captureConsequentialState: () => Promise<string>
28 |   stateBoundaryDescription: string
29 | }
30 | 
31 | function assertConforms(condition: unknown, behavior: string, detail = ''): asserts condition {
32 |   if (!condition) throw new Error(`CaseworkPort conformance failed: ${behavior}${detail ? ` — ${detail}` : ''}`)
                                 ^
error: CaseworkPort conformance failed: future-only fixture does not claim retained replay — 4 event(s) arrived before a new mutation
      at assertConforms (/home/sprime01/projects/sea-rs/apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:32:29)
      at runCaseworkPortConformance (/home/sprime01/projects/sea-rs/apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts:128:7)
      at async <anonymous> (/home/sprime01/projects/sea-rs/apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.test.ts:20:11)
(fail) LocalContractAdapter > shared CaseworkPort behavioral conformance [365.11ms]
(pass) LocalContractAdapter > duplicate intent_id returns DUPLICATE_IN_FLIGHT [364.55ms]
(pass) LocalContractAdapter > unauthorized role agent_operator returns AUTHORITY_DENIED [182.89ms]
(pass) LocalContractAdapter > unauthorized role developer returns UNAUTHORIZED_ROLE [183.32ms]
(pass) LocalContractAdapter > ESCALATE_OR_OVERRIDE without justification returns JUSTIFICATION_REQUIRED [182.50ms]
(pass) LocalContractAdapter > unknown target returns INVALID [192.44ms]
(pass) LocalContractAdapter > unavailable action on target returns UNAVAILABLE [182.56ms]
(pass) LocalContractAdapter > resolveArtifact returns independent clones [5.15ms]
(pass) LocalContractAdapter > failArtifacts rejects with error [3.87ms]
(pass) LocalContractAdapter > corruptArtifacts returns malformed content [3.28ms]
(pass) LocalContractAdapter > getSnapshot with agent_operator role has no consequential actions [2.84ms]
(pass) LocalContractAdapter > accepted intent completion precedes settlement [336.17ms]
(pass) LocalContractAdapter > accepted intent emits snapshot, execution progress, and settlement events [284.88ms]

 12 pass
 1 fail
 21 expect() calls
Ran 13 tests across 1 file. [2.33s]
```

## Corrected rerun

Command: `bun test src/adapters/local/localAdapter.test.ts`

Working directory: `apps/godspeed-cognitive-ui`

Exit code: 0.

```text
bun test v1.4.0 (34cbb9a40)

src/adapters/local/localAdapter.test.ts:
(pass) LocalContractAdapter > shared CaseworkPort behavioral conformance [546.38ms]
(pass) LocalContractAdapter > duplicate intent_id returns DUPLICATE_IN_FLIGHT [364.75ms]
(pass) LocalContractAdapter > unauthorized role agent_operator returns AUTHORITY_DENIED [185.02ms]
(pass) LocalContractAdapter > unauthorized role developer returns UNAUTHORIZED_ROLE [183.32ms]
(pass) LocalContractAdapter > ESCALATE_OR_OVERRIDE without justification returns JUSTIFICATION_REQUIRED [182.61ms]
(pass) LocalContractAdapter > unknown target returns INVALID [182.79ms]
(pass) LocalContractAdapter > unavailable action on target returns UNAVAILABLE [182.56ms]
(pass) LocalContractAdapter > resolveArtifact returns independent clones [3.64ms]
(pass) LocalContractAdapter > failArtifacts rejects with error [3.40ms]
(pass) LocalContractAdapter > corruptArtifacts returns malformed content [3.09ms]
(pass) LocalContractAdapter > getSnapshot with agent_operator role has no consequential actions [2.49ms]
(pass) LocalContractAdapter > accepted intent completion precedes settlement [334.95ms]
(pass) LocalContractAdapter > accepted intent emits snapshot, execution progress, and settlement events [283.60ms]

 13 pass
 0 fail
 21 expect() calls
Ran 13 tests across 1 file. [2.47s]
```
