# Independent primitive-checkpoint audit — 2026-10-07

**Verdict: APPROVE the exact primitive checkpoint commit only.** This is not a
manager-lifecycle, live-SSE, public-C2, or T09 completion approval. I reviewed
the checkpoint as an independent critic; I did not author its implementation.

## Commit and exact scope

Read-only inspection found HEAD at
`617dac3ddf78b660ca95f1c7a53fdf59b87653d5`, with subject
`feat(casework): add bounded private observation images`. Its committed path
list has exactly 40 entries:

1. `.agents/CURRENT_STATUS.md`
2. `.agents/current_status.yml`
3. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-key-extraction-assignment-oct07.md`
4. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-key-extraction-independent-source-review-oct07.md`
5. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-key-extraction-result-oct07.md`
6. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-poller-image-exact-oracle-source-independent-review-oct07.md`
7. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-poller-image-green-capture-root-correction-oct07.md`
8. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-poller-image-green-duplicate-write-root-erratum-oct07.md`
9. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-poller-image-green-independent-acceptance-oct07.md`
10. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-poller-image-testfirst-source-independent-review-oct07.md`
11. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-canonical-fullrace-final-acceptance-oct07.md`
12. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-canonical-retry-escalation-assignment-oct07.md`
13. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-canonical-retry-escalation-result-oct07.md`
14. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-canonical-retry-exit-oct07.raw`
15. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-canonical-retry-output-oct07.raw`
16. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-canonical-retry-preflight-exit-oct07.raw`
17. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-canonical-retry-preflight-oct07.raw`
18. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-fullrace-exit-oct07.raw`
19. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-fullrace-output-oct07.raw`
20. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-fullrace-preflight-exit-oct07.raw`
21. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-fullrace-preflight-oct07.raw`
22. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-policy-gofmt-independent-review-oct07.md`
23. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-policy-gofmt-repair-result-oct07.md`
24. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-root-graft-exit-oct07.raw`
25. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-root-graft-output-oct07.raw`
26. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-root-graft-preflight-oct07.raw`
27. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-snapshot-copy-assignment-oct07.md`
28. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-snapshot-copy-result-oct07.md`
29. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-snapshot-independent-review-oct07.md`
30. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-retained-format-independent-review-oct07.md`
31. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-retained-helper-algorithm-independent-assignment-oct07.md`
32. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-retained-helper-algorithm-independent-review-oct07.md`
33. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-retained-helper-review-format-erratum-oct07.md`
34. `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-retained-version-terminal-fixture-independent-source-review-oct07.md`
35. `apps/godspeed-casework-go/internal/server/run_observation_key.go`
36. `apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go`
37. `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go`
38. `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go`
39. `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go`
40. `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go`

The test file named `run_observation_manager_retained_image_test.go` is the
private pure encoder fixture: its cases assert exact wire shape, bounds,
immutability, and refusal behavior. Its name does not add manager behavior to
the checkpoint. The manager implementation and lifecycle scaffold are absent.

## Source identity and gate evidence

The six committed source/fixture blobs match the final acceptance record and
the isolated primitive set:

| File | SHA-256 |
| --- | --- |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

The final acceptance report records the current normal-permission escalation
PASS for canonical format/vet/tests and the full-module race gate, with actual
preflight/output/exit captures. It does not misstate the earlier sandbox
listener `EPERM` attempt as a pass. It also records the policy fixture's final
formatter repair and the root Graft build and captures. These results align
with the original execution assignment's gate and evidence requirements; they
support only the isolated six-file primitive checkpoint.

## Manifest reconciliation and exclusions

Compared with historical manifest proposal `primitive-checkpoint-manifest-proposal-oct07.md`,
the committed evidence corrects the proposal's pending-gate status by including
the passing escalation assignment/result and actual captures, final helper and
policy format evidence, relevant errata, and root Graft captures. The proposal
itself is not treated as a PASS record.

The commit contains none of the eleven unrelated paths retained in the index:

- `.agents/AGENTS.md`
- `.agents/DEBT.md`
- `observation-safe-trace-port-proposal-independent-review-anchor-clarification.md`
- `observation-safe-trace-port-proposal-independent-review.md`
- `observation-safe-trace-port-root-proposal.md`
- `.agents/plans/sea-forge-trusted-daemon-target-plan_v0.1.0.yml`
- `.agents/specs/sea-forge-trusted-daemon-target-spec_v0.1.0.yml`
- `.jolli/jollimemory/debug.log`
- `.jolli/jollimemory/discovery-cursors.json`
- `.jolli/jollimemory/plans.json`
- `.jolli/jollimemory/sessions.json`

Read-only status inspection after the commit confirmed eleven staged paths
remain; those names match this exclusion set. Operator hook and Jolli changes,
DEBT worktree changes, public drafts, and manager/lifecycle drafts remain outside
the commit.

`git show --check HEAD` reports only the known two-space Markdown line breaks
in immutable evidence and the extra EOF blank in the policy-format result; no
source-file whitespace finding was observed. No archives were changed for this
review. This audit did not run tests, compilers, formatters, scanners, or Git
mutations.
