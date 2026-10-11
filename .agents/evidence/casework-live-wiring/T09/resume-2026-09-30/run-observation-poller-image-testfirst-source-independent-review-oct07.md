# Independent review: poller image test-first source fixture

Date: 2026-10-07  
Verdict: **REJECT this fixture for focused RED readiness until the current branch has an exact nested-helper byte oracle.** The separate lifecycle lease-cardinality supplement is approved independently.  
Review scope: read-only source review; no test, compiler, formatter, scanner, runtime, or Git command was run.

## Reviewed evidence and exact identities

- Full accepted assignment run_observation_poller_image_testfirst_assignment-oct07.md (SHA-256 5c8b57bdf3b462fc366783bb304f1aff45af7c4f36a7e58dc3e7a55dada3a2c0) and archived original release instruction run_observation_poller_image_testfirst_release_assignment-oct07.md (SHA-256 4877d97deee28b572b6df553a4e7ac47e51aec26216c22e54e998ee9de49b5e8). The release authorizes only two new files, a throwing seam and seven prefixed tests; no algorithm or compile.
- Source-only result run-observation-poller-image-testfirst-source-result-oct07.md (SHA-256 4fdcde6655b358da2f6308697fc378e9738092a418c447bc50c816c7d0c01947).
- Full lifecycle supplement, its assignment and separate approval; the fixture’s manager integration claims remain deferred.
- Current new seam run_observation_poller_image.go SHA-256 32f57bd8aa6edb9c20b645e3138b4b9e9fc02076518a34587253a3af31af328e.
- Current new fixture run_observation_manager_retained_image_test.go SHA-256 5f4e1253fb3656df570a8f941cbca1686f03a6443b56e9821a7b08e9cae06e70.
- All five frozen identities rechecked and exact: manager fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d; manager fixture af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4; retained helper 2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd; base helper fixture 34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7; policy fixture 4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7.

## Accepted scope and source seam

The source file is exactly a compile-safe throwing seam: private phase/result enums use codes 0–3 and 0–5; explicit no-current/current wrapper structs have the required field order and JSON names; a fixed generic error is returned with nil bytes for every input. There is no partial encoder algorithm, manager access, port, state publication, lifecycle, cancellation, or join. This matches the narrow authorized RED source step.

The new fixture has seven top-level tests under the required TestRunObservationPollerImage prefix. Case 1 (lines 11–31) asserts the complete nil-current JSON string, exact controls/schema/key order, one key, no current, and no synthetic helper fields. Case 2 (lines 33–72) asserts the current branch schema/control prefix, deterministic output, one case key, no wrapper key, and that current parses as an object. Cases 3–7 cover exact and one-byte-over budgeting (lines 74–134), fixed marker width and prior immutability (lines 136–184), generic oversized nil-key refusal (lines 186–201), no state/output-byte aliasing (lines 203–245), and range/key validation (lines 247–299).

The timestamp growth in case 3 is a valid one-byte alternative to the assignment’s key-padding suggestion: it measures candidate-inner length equal to exact-inner length plus one, then calculates the predicted wrapper as exact-wrapper length plus the measured inner delta. The current branch’s RawMessage design makes the intended relationship plausible. This is adequately explained in the source result and is not itself a blocking deviation.

## Blocking fixture gap: exact current value is not asserted

Case 2 does not prove that the wrapper’s current bytes equal the complete output of marshalRunObservationRetainedImage(state). It only checks that the current value begins with an opening brace, that the string case_id occurs once, and that a wrapper key property is absent (lines 51–68). A partial or otherwise substituted JSON object containing one case_id can satisfy those assertions while omitting helper metadata, frames, the exact-ID ledger, availability, or other canonical fields. It also does not prove the current field is precisely the helper’s canonical JSON bytes.

Add an independent exact-byte assertion: call the unchanged pure helper marshaler on the same state, then compare the decoded wrapper current raw bytes with those helper bytes using bytes.Equal. This proves that the wrapper embeds the complete canonical helper image once as a raw object. Alternatively compare the full output to the exact ordered wrapper prefix plus exact helper bytes plus closing brace. The test should assert the inner helper contains the expected canonical ledger/metadata through the helper output, not merely one key occurrence.

This assertion also supports case 3’s one-for-one inner-to-outer growth claim. Once exact nested bytes are compared, keep the measured +1 inner length and predicted +1 wrapper assertions; a future serializer cannot satisfy the fixture with an arbitrary same-prefix object.

## Release recommendation and limits

Do not run the focused RED against this fixture yet. A fresh source-only patch should add the exact nested-helper byte oracle, update the immutable result with new full hashes/deviations, and return for a new independent review. All other reviewed fixture cases and the throwing seam are within the original release scope. The source result is accurate about no tests or compiler being run; this review does not claim a RED result. No manager admission/read suppression or lifecycle behavior is asserted by these encoder files.

This rejection does not revoke the separate bounded lifecycle design approval. It does not authorize edits, tests, compilation, or an implementation/GREEN phase.
