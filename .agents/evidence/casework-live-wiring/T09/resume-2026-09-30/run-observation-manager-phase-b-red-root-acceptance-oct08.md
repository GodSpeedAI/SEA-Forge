# Root acceptance: twelve-case private lifecycle RED

Date: 2026-10-08. Bounded acceptance of test-first execution only.

Root read the independent source-readiness review `ea9228f5…`, both
decoded repair preimages and the exact four repair hunks. Root then read
the actual command, preflight, output and both exit captures and compared
each archived JSON's decoded `raw` bytes with its recorded original file.
All five comparisons were byte-exact. Captures are the five immutable
`run-observation-manager-phase-b-focused-red-*.raw.json` files.

Preflight at 03:49:32 UTC recorded 3,343,648 KiB available RAM, 3,565,852
KiB free swap, no competing compiler, HEAD7c65be7, all ten expected source
identities and twelve cases. The command used the approved memory limits,
single compilation job and focused `go test -race` filter. It joined with
exit1: twelve cases compiled and ran, zero passed and twelve failed at the
expected unwired assertions or bounded setup waits. There was no compiler,
environment, panic or race failure. The three new tests failed at actual
list-entry setup, rather than accidentally accepting the stub's zero result.

Actual output: 5,758 bytes; SHA-256
`95051f1b8bcc7e12fe235a73bef2387303868241185da86094cc8aff87e143bc`.
Managerb7f0 and failurefixtureef58 are the tested source identities; workerf90,
originalfixtureaf and six published primitive files remained frozen.

This establishes the twelve-case RED baseline only. Later lifecycle
assertions were not reached. It proves no implementation, GREEN, whole-module,
live wiring or T09 settlement. A different implementation builder and an
independent source/runtime critic must follow before those claims.
