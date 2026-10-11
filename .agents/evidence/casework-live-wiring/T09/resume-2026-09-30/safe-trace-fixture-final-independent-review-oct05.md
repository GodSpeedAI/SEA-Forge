# Safe-trace Phase1 final independent fixture review

Date: 2026-10-05. Verdict: REJECT for fixture approval and handoff to production. The source-only matrix met the previous review after the exact Close-call repair, and the escalated test reached the expected stub, but the run exposed a remaining negative-case assertion defect.

## Binding inputs and source identities

Reviewed the original test-first assignment, original proposal and all review/rejection history, accepted V3 proposal SHA e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6, accepted shape-matrix review SHA 91a5249278f2dfdab02e9687223389f6e1903390c6839dee5519059428137fd5, the prior compile-blocker review and correction builder record. Current hashes are port 88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5; temporary stub 944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51; fixture fc9bbb748766dd2590b73c2a0ad1b0309e9dc62c2bd7d163e862d310a7b391cd. The Close repair is exactly one line; reversing it reconstructs the frozen e8859aa2 fixture hash, confirming no other source delta.

The default-sandbox run failed before fixture setup because Unix socket creation was denied. It is preserved in the preceding socket-sandbox review and exact captures. After a new full host preflight, the separately authorized escalated run compiled and executed the focused suite. It joined exit 1; no compiler process remained afterward.

## Observed intended RED and cleanup

The temporary adapter stub returns typed unavailable immediately. Positive-result cases fail with that typed unavailable result, including exact ownership and the 24 valid standing combinations. This is the intended behavioral RED signal.

Fixtures did create Unix peers in this escalated run and reached assertions. The raw output reports 66 cases failing because a valid projection was unavailable from the stub. It reports no race warning and no run trace fixture worker cleanup timeout. Bounded cleanup therefore showed no timeout for the exercised peers.

Exact preflight, complete test raw output, and exit are adjacent in safe-trace-phase1-red-critic-fc9bbb74-escalated1-preflight.raw, safe-trace-phase1-red-critic-fc9bbb74-escalated1.raw, and safe-trace-phase1-red-critic-fc9bbb74-escalated1.exit; all were byte-matched against their /tmp originals. Preflight at 2026-10-05 22:46:27 UTC captured full /proc/meminfo and unfiltered PID/comm/RSS; MemAvailable was 2,589,800 kB, SwapFree 4,711,108 kB, and only codex, bash and ps were present. Post-run scan showed no compiler and no lingering worker.

## Blocking assertion defect

In TestReadRunTraceV3CommandMetadataProjection (around line 551), the table builds a populated expected snapshot for every case, regardless of tc.wantKind. For the 12 malformed nonnull optional-metadata cases that expect unavailable, correct behavior must return the exact zero snapshot. Instead, the test passes the populated successful-frame snapshot to assertRunTraceCase. The stub returns unavailable plus an empty snapshot; the common helper correctly compares actual against the supplied want and fails with a failed-run-trace-snapshot versus want-empty-snapshot message in 35 captured assertion instances. Thus these rows do not express the required exact-empty error postcondition. This is a test-fixture defect independent of production behavior and must be corrected without weakening assertions, then independently reviewed.

The 35 observed messages include repeated/per-run rows for malformed metadata; the key source defect is unconditional successful want construction when tc.wantKind is nonempty. Other failures are expected while the stub is in place: unavailable on valid projections, no request line because the stub performs no transport, blank-input expected-invalid, and authority refusal expected-denied. Overall the selected suite reports 153 failing tests; this is a focused test-first run, not a passing verification gate.

The earlier socket-sandbox supplement incorrectly claimed failed listener setup might leak temporary directories. Its separate erratum is authoritative: source lines 56-60 remove the directory before t.Fatal. No leak is claimed.

## Disposition

The suite did reach the intended typed-unavailable behavior RED and exercised bounded worker cleanup without a reported timeout, but its malformed command-metadata cases have incorrect expected snapshots. Therefore this does not approve the fixture. No source/production edit, broader Go gate, status/debt update, or Git operation was performed by this critic. Compiler token is returned after the joined run. A fresh fixture repair and independent review are required before accepting Phase1 RED.
