# Root acceptance: auth lifecycle compiling assertion RED

Root accepts the independent final source and actual focused RED reviews for
session.go f8f0df88 and fixture8de408be (full hashes in those records). Root
independently rehashed both source files, inspected the shared total timer and
buffered completion happens-before, and read all focused failures: they are the
expected Current-live-state assertions, not compilation, setup or timeout errors.
Both actual command sessions joined exit1 and compiler ownership returned.

Root byte-compared all six archived captures with their actual originals. The
archive `auth-phase1-package-{preflight,stdout,exit}-oct05.txt` maps respectively
to `/tmp/auth-phase1-red-{preflight,stdout,exit}-e8859aa2.txt`. The archive
`auth-phase1-focused-{preflight,stdout,exit}-oct05.txt` maps to `/tmp` basenames
`auth-phase1-focused-preflight-e8859aa2.txt`,
`auth-phase1-focused-red-stdout-e8859aa2.txt` and
`auth-phase1-focused-red-exit-e8859aa2.txt` respectively. All are exact copies.
The filename suffix is a cache/run label, not the frozen fixture identity.

The full-auth attempt's loopback socket restriction is preserved as a setup
failure and is not counted as intended RED or auth package coverage. No test
was weakened. Production is released ONLY under
observation-session-lifecycle-production-root-assignment-oct05.md to a bounded
builder. Runtime GREEN, pending-read drain, SSE lifecycle and T09 remain unapproved.
