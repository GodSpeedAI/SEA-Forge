# Phase A focused RED retry 02 execution assignment — 2026-10-07

## Full original execution protocol

This run is governed by the full prepared assignment
`run-observation-manager-phase-a-focused-red-execution-assignment-oct07.md`
(SHA-256 `b7fc844d44f76083b81028a0038f54f1e92804d8225a672cfe9a0068db52082e`)
and the complete Phase A release/repair/source-review basis archived there.
The exact original gate protocol is:

> Run only from primary `apps/godspeed-casework-go`, with
> `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1'
> GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1
> CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp`, command
> `go test -race -count=1 -parallel=1 -timeout=60s -run
> '^TestRunObservationManagerFailure' -v ./internal/server`.
>
> A fresh preflight is required: UTC timestamp; available RAM >=1200 MiB;
> free swap >=512 MiB; no competing Go/Rust compiler; current HEAD; all ten
> source hashes and the nine-case inventory. Capture actual preflight/output/
> exit (and preflight exit if captured) to unique `/tmp` originals. After JOIN,
> immediately archive every capture immutably using native `apply_patch` only
> and byte-compare each archive with its original before any other gate or
> response. For CR content use an escaped-JSON lossless wrapper and decoded
> byte comparison; for raw LF content read the original JSON strings without
> losing tabs or CR, ensure destinations are absent, then write natively. Do
> not use shell `cp`, escalated writers, manual transcription, overwrite, or a
> second attempt. Do not edit source/tests, format, scan, or mutate Git. Expected
> RED means all nine cases compile and fail only by ordinary assertions from
> intentionally unwired lifecycle stubs—not compile/name, environment, race,
> or unintended-pass failures. Record exact observed failure points and later
> assertions not reached. The existing manager fixture compiles but is not
> selected by the filter. Return the heavy token after all captures and the
> bounded receipt.

## Current explicit grant and source update

Root explicitly grants the sole heavy/compiler token for exactly one focused
RED retry 02. Updated source identities for this run are the final fixture
`run_observation_manager_failure_test.go` SHA-256
`e63e050a7999b5c239257e57c6642baa346c59a40dfa1871e58d7b64ef80a5cf` and
current HEAD `7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. The source-only
readiness review is
`run-observation-manager-phase-a-compile-repair-correction-independent-review-oct07.md`
(SHA-256 `0ec6cfa45054d78a5b7618d65b832d9258df9868cab7fa99f4965024aa1d1a60`);
the full corrective assignment/result are the records named in that review.
All other source hashes remain as listed in the prepared execution assignment.

Before the one command, freshly verify the current HEAD, all ten identities,
all nine test declarations, UTC time, resources, and absence of competing
compilers. Run precisely the command and environment above from the primary Go
application. Preserve actual command preflight, stdout/stderr, and exit status
to unique originals. After the process JOIN, archive and byte-compare every
original immediately, before any additional command. No retry or additional
gate is authorized. Expected source-level assertion RED is only a bounded
test-first prerequisite; this execution cannot approve lifecycle behavior.
