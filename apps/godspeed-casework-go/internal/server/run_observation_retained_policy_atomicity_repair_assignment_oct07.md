# Retained policy fixture atomicity repair — immutable assignment

Date: 2026-10-07. Written before editing the test file. Source-only; no tests,
compiler, scanner, formatter, typecheck, runtime, or Git command.

## Full current root assignment

> FRESH builder after rejecta429d0cc. SOURCEONLY bounded test repair, no
> compile/scanner/formatter/Git. Read FULL original fresh-builder instructions
> BASE/run_observation_retained_policy_fixture_assignment_oct07.md ed290103,
> full result e49db80, independent REJECT
> BASE/run_observation_retained_policy_fixture_independent_review_oct07.md
> a429, complete current NEW run_observation_retained_policy_test.go
> d1b919e4(12288), root decisions and originalhelpercc. Root archived immediate
> full preimage BASE/retained-policy-d1b919-before-atomicity-repair-oct07.json;
> decoded compares exact before your edit. Edit ONLY new policy_test.go: (1)
> capture full caller prior deep snapshot AND canonical bytes BEFORE each
> candidate construction in all safe-copy marker/overflow cases; pass both
> pre-callbaseline to helper and require full original prior/object/image remain
> exact after build and after mutating returned clone. (2) ensure generation
> fixed-length/image assertions occur BEFORE mutating result (or helper performs
> checks then mutation LAST, no post-mutation budgetassertion). Fix no other
> semantics; all4criticgaps/extras assertions stay. Existing helperd8/base34df
> and managerfa/af frozen unchanged. Native new immutable exact original repair
> assignment BEFOREedit, disclose packagelocalfallbackifneeded; exacthash/result/
> actualdiffscope AFTER. No root codeimplementation. Critic mgr6 different fromyou
> will review; still no RED/algorithm release. Do not synthesize preimages or
> overwrite prior records. All Luna no nestedagents.

## Immutable prior instructions and findings

The full original fresh-builder assignment is
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run_observation_retained_policy_fixture_assignment_oct07.md`
(SHA-256 `ed290103a075fae97b438d3ed755ac4ef6e3553ed8712a973efbb5f607504a83`).
It authorizes changes only to the new retained policy test file; the helper,
original retained helper test, manager source, and manager test stay frozen.
It requires all five `TestRunObservationRetainedPolicy` fixtures and the
additional invalid-input cases to remain present.

The independent rejection is
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run_observation_retained_policy_fixture_independent_review_oct07.md`
(SHA-256 `a429d0cc` prefix, full hash recorded in that artifact). Its two
findings are narrow: the safe-copy helper snapshots caller state after the
builder call instead of before; and it mutates the generation-overflow result
before the generation image-length assertion. The repair must fix those exact
test flaws without changing behavior requirements.

The root-ratified state decisions are
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/retained-publisher-algorithm-root-decisions-oct07.md`.
In particular, ordinal overflow maps to retention-unavailable, except a
terminal candidate maps to terminal-retention-failure; generation overflow
maps to stop-scheduling. Each rejection returns a safe copy; candidate build
must not mutate caller prior. Read-unavailable/retention-unavailable can
recover, while terminal/stop states refuse further publication. Keep these
fixture expectations unchanged.

## Frozen identity and pre-edit comparison

Before editing, the live new test file was 12,288 bytes with SHA-256
`d1b919e4e432afc42940fb6b4da632c8d631a9ff016ed8e0e2eea0075523c8f0`. The
root archive
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/retained-policy-d1b919-before-atomicity-repair-oct07.json`
uses `encoding=utf8` and `exact_text`. Read-only UTF-8 decoding produced
12,288 bytes with the same SHA-256; decoded archived bytes compared exactly
equal to the live file. This was a pre-edit verification, not a raw-copy claim.

The helper remains `run_observation_retained_version.go`, SHA-256
`d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331`, and the
original focused test remains
`run_observation_retained_version_test.go`, SHA-256
`34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.
Neither may change. The manager files remain frozen at
`fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` and
`af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`.

## Exact allowed repair

Only `run_observation_retained_policy_test.go` may be edited. Before each
candidate construction whose safe-copy/overflow outcome is under test, the
fixture must preserve both `cloneRetainedTestState(prior)` and
`marshalRunObservationRetainedImage(prior)`. Compare the caller's full struct
and canonical bytes against those pre-call values after the candidate builder
returns, and after mutating the returned clone. Pass the pre-call snapshot and
bytes to the assertion helper; it must not derive its expected prior after the
operation. Keep nil/non-nil and optional pointer checks before dereferencing.

For generation overflow, run all safe copy/image/state/encoded-length
assertions before invoking the deliberate mutation-based alias probe, or move
the probe after the final length assertion. Preserve the canonical marker-only
length assertion. The overflow cases also need full prior/result equality,
unchanged accepted timestamp/generation and exact lifetime ledger.

No other test semantics, assertions, helpers, source files, or fixtures may
change. A different critic reviews this frozen repair. This task makes no RED,
algorithm, GREEN, or runtime claim and grants no algorithm or implementation
release. The separate focused RED and root release remain pending.
