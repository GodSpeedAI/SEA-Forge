# Independent review: existing manager fixture correction

Date: 2026-10-08

## Verdict

APPROVE source readiness for the separately authorized focused rerun only. The correction is limited to the three assigned tests, fixes the observed false fixture/expected count/read-start ordering, and preserves the contract assertions. It makes no production change and does not establish that the corrected command passes. I ran no compiler, formatter, test, scanner, build, or Git command for this review.

## Machine-derived source identities

Full correction assignment run-observation-existing-fixture-correction-assignment-oct08.md, SHA-256 75cd9bfd5f49ab11dce02f759543ea1e2c20d6f490ee3b10e983463ccbacd982. Builder result run-observation-existing-fixture-correction-result-oct08.md, SHA-256 588375ea008f61ed1380513838c4004f880e13f8dfd177aebe029e5ff48d4f86. Prior focused run receipt run-observation-watcher-terminal-focused-green-result-oct08.md, SHA-256 dab62acbba5d783b4051dd45dd691c8e5a2fdff95f7bb32f7b320bbbb3e24404.

| Source | Bytes | SHA-256 |
|---|---:|---|
| apps/godspeed-casework-go/internal/server/run_observation_manager_test.go decoded original preimage | 55815 | af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4 |
| apps/godspeed-casework-go/internal/server/run_observation_manager_test.go current final | 56118 | cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86 |

Preimage wrapper run-observation-existing-fixture-correction-preimage-oct08.json, SHA-256 6ade27afc2b7719955d72b9c6909d6b46f14369bb337e6079be17060e3eb7153, declares this same source path, size, and hash; decoded content matches. The exact preimage-to-final diff has three function-local regions: @@ -377,6 +377,9 @@, @@ -988,8 +991,8 @@, @@ -1056,6 +1059,7 @@, @@ -1066,7 +1070,9 @@, @@ -1096,7 +1102,8 @@. Target-file changes are limited to the three assigned tests; the ten other frozen source identities below remain unchanged.

## Review of the three corrections

1. Real Store/Relay mismatch, lines 362–405. runObservationManagerContext initializes both history revision cursor and Relay cursor to its argument (run_observation_manager_test.go:81–88), and the fake guard passes that object as both history and relay (:97–99). The test now changes only the fake Relay cursor under its mutex (:379–382), leaving the Store revision at cursor-old. checkRunObservationPresentContext compares Relay cursor to the newest revision cursor (run_observation_present_context.go:42–49,67–69), so the fixture exercises a genuine mismatch. The zero DTO/nil lease/unavailable and zero list/read checks remain (run_observation_manager_test.go:384–388); the valid-current-cursor positive path and exact one-list/one-read checks remain (:391–403). This corrects the fixture rather than adding fixture-name or arbitrary-cursor behavior to production.

2. Exact authorization count, lines 944–1000. The initial refusal still asserts zero list and perspective calls (:962–967), and successful retry still deep-compares the complete no-runs DTO and requires one list, zero trace reads, and successful drain (:971–999). Only the successful empty-retry perspective expectation changes from 1 to 4 (:994–995). These are the four existing Prepare authorization boundaries: before list (run_observation_manager.go:339), after list (:390), before attachment (:453), and before final disclosure (:566). The exact count strengthens that path assertion; it does not relax the refusal or DTO checks.

3. Actual first-read start before cancellation, lines 1052–1140. The capacity-one firstReadStarted signal is sent only when the first run-first callback increments the counter from zero (:1062–1075). The test waits with the existing bounded read-or-semantic-failure helper for both first and second actual callbacks before cancellation (:1101–1108; helper timeout/result behavior at :136–155). The second callback remains held on its context and reports actual return. The canceled Prepare still must produce an unavailable error, zero DTO, and nil lease; the distinct retry still waits for its own callback, releases it, drains, and preserves exact final list=2, first=2, second=2 assertions (:1108–1139). This addresses the prior observed 1/2 count caused by waiting only for the second callback. The change adds no sleep, production hook, or lifecycle-field mutation.

## Ten frozen non-target identities

| File | Bytes | SHA-256 |
|---|---:|---|
| run_observation_manager.go | 29377 | 22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57 |
| run_observation_poller_worker.go | 13217 | b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403 |
| run_observation_manager_authority_terminal_test.go | 41286 | 889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b |
| run_observation_manager_failure_test.go | 47657 | ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4 |
| run_observation_key.go | 193 | a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b |
| run_observation_retained_version.go | 10456 | 2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd |
| run_observation_retained_version_test.go | 26425 | 34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7 |
| run_observation_retained_policy_test.go | 13240 | e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28 |
| run_observation_poller_image.go | 3208 | 3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9 |
| run_observation_manager_retained_image_test.go | 11463 | cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256 |

## Prior run and limits

The prior focused race command exited 1: 50/53 top-level outcomes passed, all 22 nested outcomes passed, and three manager assertions failed. Those failures were the false Store/Relay fixture, the successful empty retry expecting one perspective call although the path makes four checks, and cancellation before the first callback had been observed. This correction addresses those observations. It is not a new runtime result; root must run the separately authorized focused rerun. Formatter provenance limitations and broader lifecycle/T09 claims remain outside this fixture-only review.

