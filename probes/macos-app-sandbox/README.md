# macOS App Sandbox evidence probe

Measures what the App Sandbox actually enforces on an arbitrary command-line
tool. A launcher binary carries `com.apple.security.app-sandbox` (plus a
per-variant network toggle and a workspace-only temporary exception), is
ad-hoc signed, and runs each attack as a child process. Ground truth is checked
from the host (files, hashes, listener logs, process table), not from the child.

Files: `launcher.c` (execs argv), `entitlements.plist.in` (template),
`run.sh` (driver, scenarios, verdict). CI: `.github/workflows/macos-app-sandbox-probe.yml`.

## What it measures

Scenarios 1-12: workspace write (1), write to canary outside workspace (2),
symlink escape (3), hard link escape (4), read of `~/.sf-probe-secret` (5),
credential env var `SF_PROBE_SECRET` (6), TCP loopback (7), UDP loopback (8),
DNS (9), outbound HTTPS (10), grandchild `/usr/bin/python3` inheritance (11),
process-group kill and orphaned `sleep 300` (12).

Variants: A (network.client=false), B (network.client=true, scenarios 7-10 only),
C (control: no sandbox entitlement), D (three more runs of A, fresh workspaces).
The control proves the probe can see escapes: C must show writes succeeding.

## How to run

Runs only on a macOS runner (CI) because it needs clang, codesign and the
App Sandbox:

    EVIDENCE_DIR=$PWD/evidence bash probes/macos-app-sandbox/run.sh

Results go to `evidence/results.jsonl` (one JSON row per scenario) and
`evidence/summary.md`; the markdown table also goes to `$GITHUB_STEP_SUMMARY`.
The environment is cleared with `env -i` except `PATH` and the planted
`SF_PROBE_SECRET`. `$HOME/.sf-probe-secret` is created per run and removed.

## Interpretation rules

- `expected` is the ideal policy. `denied` means the action should not
  succeed. `allowed` means it should.
- PASS: denied and not done, or allowed and done.
- FAIL: a denied action succeeded (escape or exposure). This is a finding.
- ERROR: the probe broke. Examples: the attempt marker never appeared, an
  expected-allowed action was blocked, or a denial had an unrecognised error
  text. An ERROR is never counted as PASS, and any ERROR fails the job.
- Scenario 6 (env) and 12 (orphans) are lifecycle or exposure checks, not
  sandbox guarantees. They are reported the same way.
- The job succeeds when the probe runs fully, even with FAIL rows. It fails on
  any ERROR row, or when control C does not show a write (1), canary write (2)
  and secret read (5) succeeding, which would mean the probe cannot detect
  escapes.
- Host-side checks are authoritative. The child's `RESULT_*` line only explains
  why an attempt failed. The secret value is never printed; the secret read
  reports byte counts only, and the value is redacted from evidence.
- Network rows in variant A and D count any child failure as blocked. The
  observed text carries the child's error so the mechanism is visible.
