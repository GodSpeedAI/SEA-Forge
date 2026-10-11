# Independent review: watcher/terminal fixture repair

Date: 2026-10-08

## Assignment and scope

I independently reviewed the complete fixture repair assignment
`run-observation-watcher-terminal-fixture-repair-assignment-oct08.md`
(SHA-256 `61d5bf6d435b73696b8b9adba7718b70b51a7f650a5a8ac40c013a301b65da6f`),
the preceding complete source-preparation assignment
`run-observation-watcher-terminal-tdd-source-assignment-oct08.md`
(SHA-256 `bfcfa5cd59f53bf5d81de7a349eac669e8899a8b842c863cb4cdb0f580a5ec82`),
the frozen repair result
`run-observation-watcher-terminal-fixture-repair-result-oct08.md`
(SHA-256 `12fe9a507c0effb02f3b2e33f92cf47322929fd45a990b7fe8a5b07afa73b8c7`),
the exact preimage wrapper, and current fixture source. I also read the prior
independent rejection (`run-observation-watcher-terminal-tdd-source-independent-review-oct08.md`,
SHA-256 `87aec11237499bbe7f2d4e178dc45c8898e4b9471f6387f478f353f5e28efd99`)
and the root clarifications for recurring cap proof, cleanup evidence, actual
worker authorization gating, and watcher membership during external checks.

This is source-only. No compiler, tests, formatter, scanner, build, Graft
build, or Git operation was run. This verdict does not authorize execution.

## Identity and scope verification

The decoded preimage wrapper records 29,548 bytes and SHA-256
`4934e9c5e8a5506f219f2c873856452e97f4c971f4c7a233dbc480fa79df7367`, matching
the assignment. The current fixture is 41,008 bytes with SHA-256
`51f73f94fed07eb97fd7b9b59d76cbf1ce65278d3acaca5b778c3a91782ad5c8`. A
read-only exact unified comparison of the decoded preimage to the current
source shows changes only in the authorized new fixture. The receipt's
ten-frozen-file identities were independently recomputed and match:

| Frozen file | SHA-256 |
|---|---|
| `run_observation_manager.go` | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` |
| `run_observation_manager_failure_test.go` | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

No other source, public interface, dependency, or policy change is present in
the reviewed scope.

## Resolved prior findings

1. **Recurring outer-cap cause is now independently established.** The helper
at fixture lines 356–400 proves the candidate state was accepted, key-matched,
and inner-encoded within the fixed cap (383–388); it marshals the existing
`runObservationPollerImageWithCurrent` wrapper with the canonical inner bytes
(390–395), and requires successful raw marshal with full length above the cap,
nil bounded bytes, and the unavailable sentinel (396–398). The actual recurring
case independently repeats these checks on the held real candidate (448–469).
This follows the test-2 clarification: the first initial read is fitting, and
the later held recurring read exercises the distinct terminal outer-refusal
path. The test keeps the existing cap and production encoder unchanged.

2. **The worker authorization barrier is tied to the real worker context.**
The test uses the existing injected `authorize` dependency (fixture lines
161–170). It gates only when that callback's context is pointer-identical to a
currently registered poller's `entry.ctx`, after checking identity under the
manager mutex and releasing that mutex before waiting (50–84). No context
method runs under the manager mutex. The pre-port subcase waits for this actual
worker boundary, attaches the survivor while it is held, revokes the initiator,
then releases it (681–752). It separately gates the existing present-context
guard before allowing the trace callback (753–776). This preserves the
independent-watcher requirement and avoids assuming an exact Prepare
authorization-call count.

3. **Failed-Prepare ownership is checked before teardown.** Groups 3–5 inspect
both cohort and poller registries immediately after the failed Prepare returns
(509–538, 540–578, 580–618); the actual-worker cases also require captured
`workerDone` to be closed before cleanup (564–568, 604–608). Both group-6
subcases check the invalid lease's cohort, reverse ref, and forward entry
membership are gone, while the survivor's cohort and lease/entry refs remain
(665–675, 787–797). The first group-6 subcase additionally checks the manager's
poller registry still maps the key to the exact entry (665–666). These
observations precede registered test cleanup and do not rely on global Stop.

The seven top-level test groups, the four final-handoff subcases, and the
existing held-read survivor case remain present. The repaired result correctly
makes no compile, test, or runtime claim. The historical result-file naming
deviation from the first source-preparation assignment remains nonmaterial:
that earlier assignment requested `run-observation-watcher-terminal-tdd-result-oct08.md`,
while the actual receipt is named `run-observation-watcher-terminal-tdd-source-result-oct08.md`.
The current repair assignment and its actual result use matching names.

## Remaining material gap

**Reject source readiness: the new pre-port survivor case does not prove the
exact poller entry remains registered after the invalid initiator is removed.**
In the new subcase, lines 787–797 assert that the survivor remains in
`manager.cohorts`, `entry.refs`, and `survivor.lease.pollers[key]`, is not
draining, and that the previously captured `entry.current` has
`retainedCurrent` availability. They do not assert
`manager.pollers[key] == entry` at that post-cleanup observation. A detached or
replaced registry entry could therefore coexist with the survivor's stale
forward/reverse lease references and retained pointer, while this assertion
passes. The held-read sibling correctly makes the missing exact registry check
at lines 665–675.

The cleanup clarification requires both group-6 scenarios to leave the
authorized survivor's “exact eligible entry” registered alongside its cohort
and valid retained value (cleanup clarification lines 12–15). The repair
assignment also requires the survivor's exact entry/membership to remain
(repair assignment line 17). A lookup made earlier by
`waitRunObservationAuthorityTerminalRefs` (fixture lines 229–245 and 738) proves
registration only at attachment time; it does not prove registration after
invalid-owner cleanup. Add the same read-only exact-map assertion used in the
held-read sibling to the new subcase, after the initiator's failed Prepare has
returned and before teardown. No behavior assertion or production state is to
change.

## Verdict

**REJECT source readiness pending that single bounded fixture assertion.**
The three previous proof gaps are otherwise resolved by source inspection.
This review grants no implementation or test execution. A fresh source-only
review is required after the new immutable fixture result records the exact
map-membership assertion and unchanged frozen identities.
