# Independent runtime verification: private ordinary cursor unit

Date: 2026-10-07.  
Disposition: **APPROVE the current private local ordinary-cursor unit for the bounded checkpoint.** Its focused suite, typecheck, full UI suite, production build, canonical casework UI gate, and complete local UI journey ladder passed. This does not approve all T09 work, a real-server integration claim, or a public/kernel cursor change.

## Assignment and scope

The full implementation assignment is recorded in
`local-ordinary-cursor-implementation-original-record-oct07.md` (SHA-256
`21f835418a4d3380469309abc3d8f6d4bee421c6d9723e5870bed8f183acbd?`—the
correct recorded value is `21f835418a4d3380469309abc3d8f6d4bee421c6d9723e5870bed8f183acbd?`; see the immutable citation correction below).
Root authorized this verifier to run tests/builds serially, with no source,
test, dependency, or Git changes. The saved assignment record for this
verification is
`ui-private-cursor-runtime-verification-original-assignment-oct07.md`
(SHA-256 `4ae3c38fc83dbbd3aee0a6a10d3cf60e7e48f1806115a8efea41c0153a331bf7`).
The accepted design is revision 4 plus the root two-ordinal settlement
decision; the source-only fixture correction was independently approved in
`ui-private-focused-attempt01-fixture-repair-independent-review-oct07.md`
(SHA-256 `779a269acefa6d2e033607a16b84f97711ad6658ac15830a533df2ebba447bdb`).

## Candidate identities

| File | Verified SHA-256 |
|---|---|
| `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.ts` | `19c767cee3da406f5729a29177f360638d2b6ac3830294126f564c5d857b4700` |
| `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorOrder.test.ts` | `b2a8a22b123382fe6b0fd788c4f385de77ee58bccfee58cdf32af272af0ec552` |
| `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts` | `a5ca9298467460a32350e85dee1c412a0b4a636efbe8ca3786ce4ab0802390f5` |
| `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.settlementAtomicity.test.ts` | `ef938f96e04a04198793fa65d1a1331b874873314a3cd07fddd7db30babe674c` |
| `apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.hostileThrownValues.test.ts` | `31f8a71c2fa550ef0ea6f22259e25c48742a836fea85e52616d6da3356913920` |
| `apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts` | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |

These identities matched the assignment's candidate hashes at each gate. No
source or test file was edited by this verifier.

## Conformance source change review

The conformance source contains an authored resume-cursor correction at
`caseworkPortConformance.ts:109-116`: it captures the trajectory head before
the second subscription and chooses that head for `resume: 'future-only'`,
while retaining the original `snapshot.cursor` for `resume: 'replay-retained'`.
This is not part of the later three-hunk fixture repair. The original Phase 1
assignment explicitly authorized exactly this shared-conformance change:
`local-cursor-order-phase1-original-assignment-oct06.md` specifies the captured
`headCursorBeforeResume` for future-only subscriptions and the unchanged
snapshot cursor for replay.

The distinction matches the current scenario declarations and adapter
behavior. The local scenario is `future-only`
(`localAdapter.test.ts:19-34`), while the live HTTP scenario is
`replay-retained` (`localAdapter.test.ts:239-253`). The common assertions now
check that future-only delivery is strictly after the captured current head
(`caseworkPortConformance.ts:135-140`); the retained replay branch still
subscribes from the earlier snapshot cursor and checks the accepted mutation
event and chronological order (`:119-134`). The local adapter's listener
floor is `max(current allocator sequence, parsed since)` (`localAdapter.ts:249-271`),
so this correction describes its actual future-only contract rather than
pretending it can replay history.

The present conformance SHA is the frozen `edf8...` identity carried in the
original unit record and every verification preflight. Because the assignment
prohibited Git commands, I did not independently compute a `git diff` against
HEAD; the conclusion is limited to the authorized source hunk and the source
semantics above. I make no broader HEAD-diff claim.

## Verification results

| Command and scope | Result |
|---|---|
| Focused four-file Bun run (`cursorOrder`, `cursorBounds`, `settlementAtomicity`, `hostileThrownValues`) | **PASS** — 23 tests, 0 failures, 212 expectations, 170 ms. This reached the released unsubscribe/no-pending-timer assertion, both required `new_cursor` presence assertions, both settlement atomicity cases, and the hostile thrown-value case. |
| `bun run typecheck` | **PASS**, exit 0. |
| Initial full `bun test` under the default sandbox | **ENVIRONMENT FAILURE** — 298 pass, 1 fail, 1668 expectations, exit 1. The sole failing journey test tried `Bun.serve({ hostname: '127.0.0.1', port: 0 })` at `src/ui/journeys.test.tsx:352`; Bun reported `EPERM: operation not permitted, listen`. This is a sandbox listener refusal, not a cursor assertion failure. |
| Root-authorized full-suite retry with local listener access | **PASS** — 299 tests, 0 failures, 1674 expectations, 26 files, 9.56 s, exit 0. No test/source change occurred between attempts. |
| `bun run build` | **PASS**, exit 0; 109 modules transformed; renderer chunk contract verified nine distinct renderer chunks. Vite emitted its existing >500 kB chunk-size advisory. |
| Root-authorized `just casework-ui-check` | **PASS**, exit 0. The canonical recipe ran frozen install, typecheck, build, and full suite; 299 tests passed with 1674 expectations. This intentionally repeats the individual gates. |
| Local `bun e2e/run.ts --out /tmp/sea-ui-private-cursor-e2e-attempt02` with the managed local Vite server | **PASS**, exit 0. All 11 ladder entries passed: J0–J9 plus RECOVERY; each result entry reports all its steps passing (6/6, 6/6, 7/7, 6/6, 8/8, 6/6, 6/6, 8/8, 8/8, 9/9, 8/8). This is the local contract-adapter UI ladder, not real-server/Tauri evidence. |

### Full-suite environmental retry

The first full-suite exit was retained as an observation. Its failing source
line opens an ephemeral loopback server; the captured OS error is `EPERM` at
`listen`. Root authorized a normal sandbox-escalation retry specifically for
that local listener. The retry and canonical gate both passed with that local
access. No test, fixture, source, security, or assertion was bypassed or
changed.

### E2E interruption and recovery

An initial local E2E attempt was interrupted by a usage-limit/environment
reset after its J0 result. Its session, `/tmp` capture, and temporary result
directory were gone; no completed result or exit was claimed from that partial
observation. Before starting attempt 02, I checked that the prior session was
unavailable, no Bun/browser process or owned Vite server remained, and the
temporary output directory was absent. Attempt 02 then ran from a fresh
preflight. Its unique `/tmp` output preserved the pre-existing
`ui-journeys/latest` artifacts (dated Sep 23).

Attempt 02's raw preflight, command output, and exit archives compare byte for
byte with their runtime files. The runner's `results.json` and `results.md`
were first written as readable copies with a trailing newline; those copies
are explicitly not described as byte-exact. Separate UTF-8 wrappers preserve
the exact original text and decode byte-for-byte to the original files:

| Evidence | Original SHA-256 | Exact wrapper |
|---|---|---|
| E2E `results.json` | `104569cd23ad338a556f2d450d3c67cde4fa714df5123e282728758c5dba86f8` | `ui-private-cursor-e2e02-results-exact-wrapper-oct07.json` |
| E2E `results.md` | `ea725ee749fb59a781d0a86f2c11f5071c0fe49fe743c06c153107094f238f2c` | `ui-private-cursor-e2e02-results-md-exact-wrapper-oct07.json` |

The copy caveat and repair are recorded in
`ui-private-cursor-e2e02-result-copy-provenance-oct07.md`. The first readable
copies remain immutable; the wrappers, rather than those copies, establish
exact source bytes.

## Evidence manifest and provenance

All paths below are under
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/`. Output and
exit archives are the exact raw captures; preflights record command, cwd,
timestamp, memory, and candidate hashes. Unless stated otherwise, each
complete triplet was immediately compared byte-for-byte with its `/tmp`
original before advancing. Original temporary files were later removed by
the environment reset, so these are the contemporaneous comparisons, not
claims of a current comparison against vanished files.

| Run | Preflight SHA-256 | Output SHA-256 | Exit SHA-256 | Pair status |
|---|---|---|---|---|
| Focused four-file attempt 01 | `e95361e730b62073e333e88cc592cd2ef0c0e2158416ed4d1ba21d40016658c1` | `5410f0dfed1fd9c2516c17ffb7b8a9430b0475c1b22a9d4df497b9e998eadc3d` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | stdout/exit cmp 0; preflight was captured in tool output, not `/tmp`, then archived from that verbatim output, so no preflight-file cmp is claimed |
| Typecheck | `92235450528cc76aa0f2bc447d0ab2b25fe68311b5b6bdca45fe728b23f599c5` | `8366207267355d3e3d5bf3bf6e8c94c5f93f6078c34f08973fa2b38cdda6cc92` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | all cmp 0 |
| Full suite, default sandbox | `33557a91af60aa70c56cda4f72a7bc886f9bd67de33468f8730c00ef46b99e17` | `448915e4a0b66ba7ec17b63e2e299206f4a540918f9c4136493736b7f99811e5` (`ui-private-independent-fulltest-attempt01-output-oct07.raw`) | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | all cmp 0 |
| Full-suite retry, root-authorized local network access | `5549f7624791117cf2d083cc594d03c87afebd36e48b0352e0d11f7609b0e7ea` | `3734a9e516b09a340fa93fa4a3b23056803671c58b34bf397900e3bd01c0d1e0` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | all cmp 0 |
| Standalone build | `cbf14445b5990758bb0fc6db6e5ac255ed034698c7672e8a404aa2a48d96ec87` | `751b87068539d83c322f9c61645c2a9efe045be5571a03e2f58d058e69e5d063` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | all cmp 0 |
| Canonical `just casework-ui-check` | `634cb976eee87e2941bfdfb3f29de4f06cbfc5e6def4094a354f1fffda42bdc8` | `c4832c38395b30beea70b8e520528329181275ec4a662a3e1365e1ec26414a2b` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | all cmp 0 |
| Local E2E attempt 02 | `a935b746b06735a21f32cb6e79e7aa670662748e6fdeb5a1f4649feb7a96d2be` | `f4ad3b00a72c686587a9da5f106f33de3d766e705b29f061d6af44bd4cb792d2` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | all cmp 0; currently rechecked against `/tmp` originals |

The first attempt's E2E partial captures are not final gate evidence. No
full-suite, build, canonical, or E2E result is inferred from those partials.
The existing failed attempt 01 cursor captures and all independent fixture
reviews remain immutable.

## Cleanup and remaining limits

The E2E output used `/tmp`, leaving the prior `ui-journeys/latest` directory
untouched. After completion, the managed server had already exited and its
PID file was stale. A first `just casework-ui-down` could not start because
`just` could not create its temp directory under read-only `/run/user/1000/just`.
The guarded cleanup then ran as `JUST_TEMPDIR=/tmp just casework-ui-down` and
reported `not running (stale pidfile removed)`; no other process was targeted.

No Go manager tests, Workbench/Tauri gates, real-server E2E, other platform
gates, or all-T09 settlement were run or approved here. The UI E2E uses the
local contract adapter. No Git command was run under the explicit restriction.
No source/test edits, dependency changes, scanners, or publication occurred.
