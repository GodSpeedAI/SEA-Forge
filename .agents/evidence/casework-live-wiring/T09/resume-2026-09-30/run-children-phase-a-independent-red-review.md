# Independent runtime Phase A source review and assertion-RED result

Review date: 2026-10-01

## Frozen source identity

Reviewed the original runtime assignment, test-phase supplement, root fixture clarification, prior immutable rejection, authorized repair scope, and actual frozen files:

- internal/ports/ports.go: 703bf0d94a5d235d4c69450838171a64bfddc999cc97cba6bfa30938b2cb9ec6
- internal/projection/builder.go: 69d9ba3cc7ef7788ed673059451ede73209f50673f6e184c0c283f189393203c
- internal/adapters/sfwp/scoped_runs_test.go: 0013b05f69def466e8434182642b4543e68c5b4ad7fd1a3f3f14608337fcc2b5
- internal/projection/scoped_live_test.go: 361fc0328217bf78d4ac3d7c92dad6f8641e931225e3c78cd58685a7bb4a7800
- internal/projection/unreadable_runs_test.go: de85a2edb9bd05dc5542c7c9850a9cf9caea69dbb491eb21f95fe93913bfcebb
- Preserved pure child fixture internal/projection/run_children_test.go: 4ddb2445a7ff6726c3b6fd93832ac4122b53da4d26e5b54180f9a1cafb802741

The two rejection repairs are present: the scoped fake records and asserts case_1; the blank-case adapter test installs a dial counter and asserts zero dials, accepted connections, and request lines. The only material scope deviation is the root-authorized whitespace-only gofmt correction on the same two new test files. The two production edits remain only the semantic RunListResult and factual CaseFacts.UnreadableRunIDs declarations. The pure child fixture and new Store unreadable-ID clone test are unchanged.

## Focused assertion-RED command and result

Run from apps/godspeed-casework-go:

GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/t09-run-children-gocache GOTMPDIR=/tmp/t09-run-children-gotmp TMPDIR=/tmp/t09-run-children-tmp go test -race -count=1 -parallel=1 ./internal/adapters/sfwp ./internal/projection -run 'TestScopedRunsListUsesCaseRequestAndMapsEveryField|TestLegacyRunListRequestRemainsUnscoped|TestScopedRunsListRejectsBlankCaseBeforeWireContact|TestScopedRunsListRejectsMalformedAuthorityRows|TestScopedRunsListRefusalRemainsTyped|TestLiveSourceUsesOnlyScopedRunListAndRetainsUnreadableIDs|TestLiveSourceRejectsInvalidRequestedCaseBeforeAuthorityCalls|TestLiveSourceRejectsMismatchedAndUnownedScopedRunFacts|TestStoreDeepCopiesUnreadableRunIDs'

The initial sandboxed invocation compiled and reached projection tests, but the SFWP fake-server Unix sockets failed with setsockopt: operation not permitted. It is preserved as run-children-phase-a-sandbox-limited.log with exit 1 and is not treated as assertion behavior evidence. After a new actual-host preflight, the same invocation was repeated with actual-host socket access.

The host retry compiled successfully and exited 1 on expected baseline assertions, not compile errors. Adapter calls fail the test-local interface assertion because Authority does not yet implement RunsListForCase; the exact legacy request assertion passes silently. Projection failures expose current unscoped source behavior, authority contact for invalid case input, and absent Store cloning for the new unreadable-ID field. The adapter malformed-row and blank-before-dial assertions are blocked at the absent-method assertion during this phase, so their wire behavior was not individually exercised. The earlier pure-child expected RED remains separately approved. No broad Go gates or real-kernel proof were run here.

The focused tests and all later negative guard suites are test design coverage, but this Phase A RED does not prove adapter parsing or refusal behavior because the scoped interface assertion stops those test bodies until Phase B supplies the method. This distinction is material to interpreting the expected RED.

The first archival patch content attempt did not create files; the final raw outputs were copied from /tmp with native apply_patch. A later byte comparison found two log copies had formatting mismatches; they were replaced from the exact /tmp stdout and byte comparisons then passed. These were archive preparation corrections only and did not affect source or test execution.

Raw stdout, exit and preflight artifacts are separate from this narrative. Exact original /tmp paths, archive paths, full hashes and byte-comparison results are in run-children-phase-a-artifact-manifest.md.

Disposition: Phase A declarations/test fixture and focused assertion-RED approved only. No runtime implementation, broad Go gates, or real-kernel proof are approved by this phase. Compiler token is released to root.
