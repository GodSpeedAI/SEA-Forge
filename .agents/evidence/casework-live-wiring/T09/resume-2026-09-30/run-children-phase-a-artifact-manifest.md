# Run-child Phase A assertion-RED artifact manifest

Date: 2026-10-01. Raw stdout, exit status and actual-host preflight files are archived separately from the review narrative.

## Sandbox-limited attempt

- Original preflight: /tmp/t09-run-children-phasea-preflight.txt
- Archive: run-children-phase-a-preflight-sandbox.txt
- SHA-256: 43c63e80ee015dec84d3918340fa3d601b7cedba9366ac143efcde040bb90edf
- Original stdout: /tmp/t09-run-children-phasea-scoped-red.log
- Archive: run-children-phase-a-sandbox-limited.log
- SHA-256: c2c39ed0ad1e3ed9e4ac24f10e37e5efc6ca6fccad6b50621800e3a6df685c8a
- Original exit file: /tmp/t09-run-children-phasea-scoped-red.exit
- Archive: run-children-phase-a-sandbox-limited.exit
- SHA-256: 4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865
- Result: exit 1; adapter fake Unix sockets were blocked by sandbox setsockopt restrictions. Not accepted as assertion behavior proof.

## Actual-host socket-enabled retry

- Original preflight: /tmp/t09-run-children-phasea-preflight-retry.txt
- Archive: run-children-phase-a-preflight-host.txt
- SHA-256: fbdc6c3417eabcdd9d42c04c166abd64d4ae2494c10dd4bb276e1beb550963f8
- Original stdout: /tmp/t09-run-children-phasea-scoped-red-host.log
- Archive: run-children-phase-a-scoped-red.log
- SHA-256: a6b05114166fbda6d5e4f02f8fbce8d16316ed1d7dd81f7e5c88db681b394261
- Original exit file: /tmp/t09-run-children-phasea-scoped-red-host.exit
- Archive: run-children-phase-a-scoped-red.exit
- SHA-256: 4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865
- Result: exit 1; the selected tests compiled and failed at the intended absent-scoped-method, legacy-path, invalid-request-contact and missing unreadable-slice-clone assertions.
- Host: RAM showed 2.6 GiB available. Compiler process scan found no go/compile/link/cargo/rustc/tsc process. Foreign Bun PID 2109808 was observed running server.ts, explicitly excluded as a non-compiler and left untouched.

Both log and exit pairs were compared byte-for-byte with their /tmp originals after archival. The preflight files were also archived from the exact captured /tmp command output.

## Test command

Run from apps/godspeed-casework-go:

GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/t09-run-children-gocache GOTMPDIR=/tmp/t09-run-children-gotmp TMPDIR=/tmp/t09-run-children-tmp go test -race -count=1 -parallel=1 ./internal/adapters/sfwp ./internal/projection -run 'TestScopedRunsListUsesCaseRequestAndMapsEveryField|TestLegacyRunListRequestRemainsUnscoped|TestScopedRunsListRejectsBlankCaseBeforeWireContact|TestScopedRunsListRejectsMalformedAuthorityRows|TestScopedRunsListRefusalRemainsTyped|TestLiveSourceUsesOnlyScopedRunListAndRetainsUnreadableIDs|TestLiveSourceRejectsInvalidRequestedCaseBeforeAuthorityCalls|TestLiveSourceRejectsMismatchedAndUnownedScopedRunFacts|TestStoreDeepCopiesUnreadableRunIDs'

The first invocation used the sandbox default. The retry used actual-host escalation for Unix socket test support. No broader Go gate or real-kernel proof was run in Phase A.
