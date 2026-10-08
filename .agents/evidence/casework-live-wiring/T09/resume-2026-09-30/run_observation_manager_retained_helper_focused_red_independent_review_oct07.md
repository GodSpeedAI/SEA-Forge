# Independent review: Unit 1 / retained-helper focused RED evidence

Date: 2026-10-07  
Disposition: **REJECT evidence package for independent approval pending capture-comparison provenance**

## Scope and exact identities

Source-only review of the full focused-RED assignment/result, the original manager Unit 1 and retained-helper instructions, the complete six archived raw captures, and the current candidate source/test hashes. No tests, compiler, scanner, Git, source edits, or runtime actions were performed.

| Artifact | SHA-256 |
|---|---|
| Verification assignment | `45682c73c2db251a860aa616a3783b0388b2f074a1b62c3371644681e172e53e` |
| Verification result | `305912c6a22aed541eeea8265b0fdfa438ffacb43af354af883553af9f1bd541` |
| Manager source, current | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| Manager tests, current | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| Retained helper, current | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` |
| Retained-helper tests, current | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |

All four current file hashes match both recorded preflight captures and the assignment's frozen candidates. The helper remains an explicit rejected stub (`run_observation_retained_version.go:67-80`); manager lifecycle functions remain unwired as described by the scoped assignment. These are test-first negative attempts only.

## What the archived captures support

The archive hashes exactly match the values in the immutable result record:

| Capture | SHA-256 | Observed content |
|---|---|---|
| `run-observation-focused-red-manager-preflight-oct07.raw` | `c213482c8e0e18c24401253de9ff909eb75c68a4cd55dc82cf7f9761b8e3ed7d` | UTC `12:44:06.891550`, 4,396 MiB available, 12,080 MiB swap free, four expected hashes, and the exact manager selector/command. |
| `run-observation-focused-red-manager-output-oct07.raw` | `ff3700ad56b29591174dbda13eac97325430e155087fb20cc4d70b8610c6e9aa` | Three selected tests ran and failed at lines 971, 1016, and 1099 on the deliberate `unit 1 is not wired` response. |
| `run-observation-focused-red-manager-exit-oct07.raw` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | Numeric exit `1`. |
| `run-observation-focused-red-retained-preflight-oct07.raw` | `d445294b4460f0e28295fe0e87c3565bcb9e948d5b67d12fb7cc5048896afb1c` | UTC `12:46:06.942834`, 4,212 MiB available, 12,080 MiB swap free, the same four hashes, and the exact retained-test selector/command. |
| `run-observation-focused-red-retained-output-oct07.raw` | `51f72fa297984754c42f3e2e1e4e5148cbe7490e4e374b189f6489016567af40` | All eight selected tests ran and failed at their first semantic assertions on the deliberate generic rejected/nil stub result. |
| `run-observation-focused-red-retained-exit-oct07.raw` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` | Numeric exit `1`. |

The Go outputs show the package reached test execution, with individual named tests and `FAIL` summaries; this rules out a package setup failure or compiler error for these invocations. The outputs contain no race-detector report, timeout, panic, or deadlock. They support classifying the observed failures as semantic stub failures, not as successful algorithm/lifecycle assertions.

For the manager command, the authorization test's stale-claim path reached its refusal checks, then the corrected-claim retry failed at line 971 on the unwired stub. The refused-list fixture failed at line 1016 before it demonstrated the specified unavailable DTO/empty lease. The canceled-partial-Prepare fixture intentionally fail-fast failed at line 1099 before the controlled read; it does not prove a read began, was canceled, joined, or retried. For the helper command, each of the eight selected tests reached the expected first assertion listed in the result (`run_observation_retained_version_test.go:95,117,148,231,254,329,435,468`). Later assertions in each test were not reached and prove nothing from this run.

## Blocking provenance gap

The assignment requires the actual command's output and exit captures to be archived and compared byte-for-byte against their original capture files before the compiler token is returned (assignment lines 55-72). The result gives the six archive filenames and SHA-256 values but does not name any original `/tmp` capture paths or record any `cmp`/byte-comparison result. I searched the current `/tmp` tree for focused-RED captures; no original capture files are present. Consequently I could recalculate the six archive hashes and inspect their contents, but cannot independently establish that each archived byte stream is identical to the command's original `/tmp` output/preflight/exit file. The captures are internally consistent with the result, but hash agreement with the result is not an original-to-archive comparison.

This is a verification provenance gap, not evidence that the observed test failures were fabricated or that the commands failed to run. It means the independent approval standard requested here is unmet. If root has a separate immutable record with the six original capture paths and exact comparison results, that record must be cited and reviewed; otherwise the original capture copies/comparisons need to be supplied in a new immutable addendum. Do not reconstruct command output from these archives and describe the reconstruction as original evidence.

## Bounded conclusion

The archived contents, hashes, preflight values and selected failure lines are consistent with the claimed focused semantic RED for the three manager tests and eight helper tests against the exact four frozen current candidates. Subject to original-to-archive provenance, the only claim these runs support is that the selected fixtures compiled and reached their intended failures on the deliberate stubs. They do not prove any subsequent assertion, any manager lifecycle, retained algorithm behavior, lifecycle integration, `Next`, production behavior, or settlement. Because original `/tmp` capture comparisons are not documented or currently reproducible, I **do not approve the evidence package as independently verified**.

No run or typecheck was performed in this review. Graft first-pass retrieval saved ~64,973 tokens (~$0.05) this turn.
