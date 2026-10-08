# Ask fixture capture erratum v2

Date: 2026-10-01

Root byte-checked the persisted logs and exit files against the original `/tmp` captures. Both
escalated expected-RED logs and both corresponding exit files match the original captures.
Expected-RED evidence and its scope are unchanged.

The earlier non-escalated sandbox socket failure copy
`ask-fixture-server-sandbox-socket-failure.log` is not a byte-identical raw capture: it contains
2,417 bytes and hashes to `68c81f...`, while the original
`/tmp/t09-ask-critic-server-expected-red.log` contains 1,974 bytes and hashes to
`f56f172b378253b930af59ee47e4428e8138070d7b1422bab5afe210d795ee6d`. The evidence copy had a
narrative appended after the original `FAIL` output. Its separate `.exit` artifact is retained;
the contaminated log must not be represented as the original raw output. The original capture
remains available at the `/tmp` path above. A separate exact raw copy is assigned elsewhere; the
existing evidence artifact is left untouched.

Host RAM/process preflight outputs were reported in the review but were not persisted as files.
Per root direction, the expected-RED commands are not rerun solely to recreate those preflight
captures.
