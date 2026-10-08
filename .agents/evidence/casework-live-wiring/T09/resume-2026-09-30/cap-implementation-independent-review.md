# T09 response-line-cap implementation independent review

## Verdict

APPROVE the frozen SFWP response-line-cap implementation for its bounded client unit. All eight focused cap tests pass, and the complete SFWP adapter package also passes in the serial module-wide race run. The wider Go gate remains red only in `internal/contract` because the canonical T09 Ask and observation mirrors are still in progress; this is not a cap-unit approval of those mirrors or of T09 as a whole.

Reviewed against the original `cap-implementation-assignment.md`, `test-first-builder-assignments.md`, approved T09 proposal, and `contract-source-preparation.md`. Implementation scope is the assigned `client.go` and `subscribe.go`; the frozen `response_limit_test.go` was unchanged.

## Source review

- `Config.fill` rejects negative and above-32-MiB values before a client can dial; zero becomes the 32-MiB default, and positive values through the ceiling remain configurable. Each production request connection receives the validated per-client limit. The direct `conn` test helper initializes no limit, and `readBoundedLine` intentionally maps that zero to the default, so the direct default-limit behavior is exercised.
- `readBoundedLine` uses `bufio.Reader.ReadSlice('\n')`, checks each returned chunk against remaining logical capacity before appending it, and treats the LF as part of the cap. Exact-limit lines succeed; a cap-crossing chunk returns typed `unavailable`; partial EOF returns the transport-unavailable path. The caller closes the request connection on every read error, and the pool discards it, so an incomplete positional response cannot be reused. The subscription caller defers closing its owned connection and propagates the typed overflow from `subscriberPass`.
- The same helper is used for request replies, inspect retries, `request_get_status` recovery replies, subscription events, and subscribe acknowledgments. Repository search found no production `ReadBytes`/`ReadString` response reader outside the two updated call sites; remaining occurrences are tests. Recovery remains correlation-based: the original mutation is not resent, while status reads retain the inspect retry. Focused tests assert one mutation send, two status reads, and the fresh inspect retry.
- The existing `bufio.Reader` is retained across reads. `ReadSlice` consumes one line and leaves coalesced following bytes buffered, including replay events preceding a subscribe acknowledgment. Existing SFWP replay/ack tests plus the complete adapter race suite passed.
- The helper keeps at most the logical line limit in its accumulated byte slice. The underlying `bufio.Reader` can read ahead by one bounded chunk before overflow detection; in production it is created with Go's default 4-KiB buffer. The code documents that behavior and does not claim strict socket-byte consumption, aggregate transport limits, or a total allocation ceiling. The response slice's backing capacity may exceed its current length through normal Go growth; no exact-allocation claim is made.

Error and cancellation boundary: cap overflow, EOF, and read-deadline errors close/poison the request connection; the frozen tests explicitly verify overflow and truncated-EOF poison. There is no new cancellation-specific test. A context cancellation without an earlier deadline does not interrupt the existing blocking socket read immediately; the configured request or subscription deadline still bounds that wait, and the eventual read error closes the connection. This behavior predates the cap and remains a documented limitation rather than a cap regression.

Ask has not yet been added to this Go client. The current `mutationVerbs` in `internal/adapters/sfwp/frame.go:95-110` does not include `ask`; when the later Ask mirror is added, it must be classified as a durable mutation so an oversized Ask response follows correlation recovery and is never resent. This is explicitly outside the cap implementation assignment and remains a downstream integration requirement.

## Verification

Every command was preceded by host RAM/process preflight; runs were serial and used the task-scoped Go cache, `GOMEMLIMIT=256MiB`, `GOGC=50`, and `GOMAXPROCS=2`. The fake-server tests require local Unix sockets, so commands ran with the previously authorized escalation. Exact original output filenames are retained below for byte comparison.

1. Focused cap tests, from `apps/godspeed-casework-go`: **exit 0**. Exact command:

   ```text
   GOCACHE=/tmp/sea-rs-t09-critic-gocache GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 go test -p=1 -parallel=1 -race -count=1 ./internal/adapters/sfwp -run '^(TestResponseLineLimitConfigValidationBeforeDial|TestLowerResponseLineLimitAppliesToRPC|TestLowerResponseLineLimitAppliesToSubscription|TestResponseLineLimitExactBoundaryAndOverflowPoison|TestResponseLineLimitTruncatedEOFIsUnavailable|TestInspectOverLimitResponseRetriesOnceOnFreshConnection|TestOverLimitMutationAndRecoveryResponsesNeverResendMutation|TestSubscribeOverLimitEventIsRejectedBeforeDecode)$'
   ```

   Preflight showed 2.3 GiB available and no competing Bun/Node/Go/Cargo/TypeScript process. Original output: `cap-focused-green.log` (captured from `/tmp/t09-cap-focused-green.log`), SHA-256 `601469306b5992c143bbc769e409b2ace037be5afe7b0e401fac8cfc64721d8c`.
2. Repository recipe **exit 1**. Exact command:

   ```text
   GOFLAGS=-p=1 GOCACHE=/tmp/sea-rs-t09-critic-gocache GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 just casework-go-check
   ```

   Its gofmt check and `go vet ./...` passed; all SFWP tests passed. `internal/contract` failed on the active canonical change: four new Ask goldens have no Go contract structs, the observation SSE golden is not byte-stable under the current Go mirror, and `execution_observation` is absent from `AllStreamEventKinds`. Preflight showed 2.2 GiB available and no competing compiler. Original output: `cap-just-casework-go-check.log` (captured from `/tmp/t09-cap-just-casework-go-check.log`), SHA-256 `e3925e22a53c5bdd27f67c98c6b34e6ba9d6c367126d50bee40448ccbe991182`.
3. Module-wide race suite **exit 1** for the same `internal/contract` canonical mismatch. Exact command:

   ```text
   GOFLAGS=-p=1 GOCACHE=/tmp/sea-rs-t09-critic-gocache GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 go test -p=1 -parallel=1 -race -count=1 ./...
   ```

   `internal/adapters/sfwp` passed under race in 17.691 seconds; every other listed module package passed. Preflight showed 1.6 GiB available and no competing compiler. Original output: `cap-module-race.log` (captured from `/tmp/t09-cap-module-race.log`), SHA-256 `4abee7c6675b3c74d5fb6efb9025560092a5b9dfc6d1b0bb56e7b195aa713ded`.

Copied output files compare byte-for-byte with their original `/tmp` captures. Frozen source hashes at review: `client.go` `e7068fb0c618922a4b3f94b1a994670741ea2bec5e5ee28b0ea127343f0c1c97`; `subscribe.go` `117b7876c99a054c24828911c659d28c3888ac02d38f7ab104c20faef4a8eff5`; unchanged focused fixture `response_limit_test.go` `7ec880bc2a7cdf15eb8c4dfa26daca7d4ea021b7c50940a20e16839cec7b6299`.

The earlier sandbox-denied attempt and expected-RED run remain preserved as separate records; neither was replaced. No dependencies, tests, generated files, or unrelated paths were changed during implementation review.
