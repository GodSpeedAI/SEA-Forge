# Source-only protocol supplement assignment

Date: 2026-10-07  
Builder: fresh bounded documentation repair following independent rejection.

## Full assignment received

Prepare one new immutable, bounded source-only supplement after the independent rejection of `manager-combined-image-testfirst-assignment-repair-oct07.md` (SHA-256 `477ee9aa26d3b68d6cc2a01ef72d0bed3a4cf358345950129c3068f40b937fdc`). Before the supplement is authored, preserve this complete assignment as a new immutable record under `BASE=.agents/evidence/casework-live-wiring/T09/resume-2026-09-30`.

Read and follow the full repair document, its independent review, the prior combined-image correction, the combined-image design proposal, the lifecycle preregistration, the original Unit 1 assignment, revision 6 and its addendum, retention corrections 2 and 3, and the applicable root decisions. The independent reviewer found exactly two blocking protocol defects in the repair: (1) ambiguous duplicate ownership of lease reference release, and (2) the stopped-before-first-read worker had no defined initializer result. Root resolved both in `manager-signal-ref-ownership-root-decisions-oct07.md` (SHA-256 `5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`). Apply those decisions in a new immutable supplement; do not rewrite, amend, or overwrite the rejected repair, its review, the earlier correction, or other evidence.

The supplement must define exact replacement clauses, distinguish the two reference-release paths, and add deterministic future lifecycle test cases for detach versus selected-A cleanup, awakened waiters, self-join avoidance, and stop-before-first-read. Use only existing initializer result code 4 and phase code 2 (stopping) or 3 (draining); add no field or code 7. A stopped-before-read initializer retains no current state, starts no trace read, and resolves ready exactly once outside the actual manager mutex. Prepare must recheck stop after wake and return typed stop/cancellation error, zero wrapper, nil lease, with internally owned rollback. The worker remains counted until actual worker completion and JOIN.

The supplement must retain initial failed/unretainable selected-read behavior from root's separate clarification: it is successful A (normal unavailable DTO with successful-list counts, nonnil cohort lease), no Run row, no retry; each Prepare releases only its own failed poller reference; shared waiters get the same fixed poller outcome while their actual-start counts may differ. Failed Prepare for auth/guard/cancellation/stop/irreducible assembly remains zero wrapper/nil lease and owned rollback. Do not conflate these cases.

Keep the pure image encoder unit separate from the future lifecycle implementation. Preserve the pure helper unchanged and keep the existing manager source, manager fixture, retained-helper fixture, and policy fixture exact at their known identities below. Do not add source/tests in this task. Do not run compiler, tests, builds, scanners, or Git. No implementation or fixture release is granted; the new supplement is for a different critic and subsequent root decision.

## Frozen identities and governing records read

- Rejected repair: `manager-combined-image-testfirst-assignment-repair-oct07.md`, SHA-256 `477ee9aa26d3b68d6cc2a01ef72d0bed3a4cf358345950129c3068f40b937fdc`.
- Independent repair review: `manager-combined-image-testfirst-assignment-repair-independent-review-oct07.md`, SHA-256 `c010c8ee2bc05bdafd07e25b286e786da10e483d84258085c902cbecfb572f88`.
- Previous combined-image correction: `manager-combined-image-testfirst-assignment-correction-oct07.md`, SHA-256 `5f5418f16193d41476f69554e366ff83d7490c68a917462f3a76252f050466ee`.
- Combined-image proposal: `run_observation_manager_combined_retained_image_design_proposal_oct07.md`, SHA-256 `e436298c9822a7e405e723ebba505ead8ad49006e246ab983a02b2ad88f9760c`.
- Lifecycle preregistration: `run_observation_manager_unit1_lifecycle_implementation_preregistration_oct07.md`, SHA-256 `f48df1c85ca67ce8dafa3d21b82f08a8c39aae1ec710d4ee78ce617e215869f5`.
- Frozen Unit 1 original assignment: `run-observation-manager-unit1-original-assignment-oct06.md`, SHA-256 `de8c9019ebdf98a43525264a32897868f05ab3346cacf7c4f013ccdae23d9916`.
- Revision 6: SHA-256 `60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`; addendum: `9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`.
- Retention corrections: correction 2 `3494e3de99b2f5d3fe725a85971dccd0d70cef226da257ce237cce7fba2d72e1`; correction 3 `b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`.
- Combined-image root decisions: `manager-combined-image-root-decisions-oct07.md`, SHA-256 `48ed5d0b01e25a080b8b2560462bfd6bdf4b196d9769ff917b6a9cb25b32fe96`.
- Initial-failure root clarification: `manager-initial-failure-root-clarification-oct07.md`, SHA-256 `34750ceb5e7dc99b0a7c3755c514f6a7fbdb6fd31ac60d99833fad181ed117c2`.
- New signal/reference ownership root decisions: `manager-signal-ref-ownership-root-decisions-oct07.md`, SHA-256 `5b9888e5a7212b8d0696f790822f08e718dd03a71f51b28c2bec2ea0098f8df4`.

Frozen code identities: manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`; manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; pure retained helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; base helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; policy fixture `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

## Completion boundary

Write the new supplement only after this assignment record. It must describe the repaired clauses and evidence-backed differences, name the future lifecycle tests precisely, preserve the pure encoder/lifecycle separation, and state that no source, tests, compiler, or Git operations were performed. Send the immutable supplement to the different critic; wait for critic/root approval before any later fixture or source phase.

