# T09 Ask runtime independent source and gate rejection

Date: 2026-10-01

## Verdict and scope

REJECT. This review covers the original `ask-runtime-implementation-assignment.md`,
`ask-runtime-root-integration-supplement.md`, `ask-live-ledger-projection-clarification.md`,
the approved frozen fixture review, and the actual frozen runtime source below. The bounded
runtime implementation is not approved. The first focused Go command failed during compilation;
no Ask tests ran. No other compiler/test command was started.

The source otherwise follows the assigned route, adapter, identity, transport, and live-proof
scope. The new server handler decodes/validates the request before `verifySessionPerspective`
and Ask dispatch (`internal/server/ask.go:25-55`); the Ask route is session -> CSRF -> distinct
limiter -> handler (`internal/server/server.go:145`). The adapter supplies configured gateway
and effective-user `actor`/`on_behalf_of` plus effective `actor_id`, preserves optional case
absence, and creates no request ID (`internal/adapters/sfwp/ask.go:53-75`). The answer adapter
rejects unsupported/missing/null required disclosure fields and passes complete modeled fields
through (`ask.go:81-114,117-179`). The transport code excludes Ask from ambiguous resend while
leaving explicit `server_busy` retry separate (`frame.go:93-99`; `client.go:381-389,405-431`).
The live test uses a fresh temporary cell, real bundled-model concept, real denied and granted
answers, effective writer/delegation attribution, linked four-record chains, exact canonical
answer projection comparison, and ledger verification (`server/ask_live_test.go:75-165,286-389`).
The kernel-only persisted claim fields are kept separate from the public projection per the
root clarification.

## Bounded source findings

1. **Compile blocker: production type shadows frozen fixture helper.**
   `internal/server/ask.go:18` declares `type askBody struct`, while the frozen
   `internal/server/ask_test.go:115` declares `func askBody(kind, subject string) string` in the
   same Go package. The test helper call sites then fail as conversions with arguments. Resolve
   the production decoder type name or otherwise avoid changing frozen fixtures. The full exact
   compiler output is preserved in `ask-runtime-focused-server-reject.log`; actual exit is
   `ask-runtime-focused-server-reject.exit`.

2. **Invalid raw UTF-8 is not rejected before JSON decoding.**
   `internal/server/ask.go:58-84` reads the bounded bytes and passes them directly to
   `encoding/json`. Go's JSON decoder replaces invalid UTF-8 bytes inside JSON strings with
   U+FFFD rather than returning an error. The later `utf8.ValidString(purpose)` check therefore
   cannot detect invalid bytes already replaced, and subject is not checked for byte validity.
   This violates the assignment's strict malformed-body/no-kernel-call boundary. Check
   `utf8.Valid(raw)` before decoding and add invalid raw-byte cases in a new test file, preserving
   the independently frozen Ask fixture files. Ensure rejected bytes reach neither perspective
   verification nor Ask dispatch.

3. **Transport comments still conflate protected effects and correlated mutations.**
   `internal/adapters/sfwp/frame.go:86-88` says every protected side effect is correlated,
   deduplicated and recoverable by `request.get_status`, although kernel Ask is protected but has
   no request ID or status record (`crates/sea-forge-server/src/identity.rs:803-820` and
   `lib.rs:1429-1476`). `client.go:260-261` says the correlation store is the only way to settle
   any crossed-wire outcome. `client.go:362-368` documents all non-mutation/inspect failures as
   safe to retry and correlated mutations as the only non-retry class; Ask is neither. Correct
   these comments to describe the separate transport retry-safety classifier and uncorrelated Ask
   unavailable outcome. Runtime behavior itself has the distinct classifier and is in the
   intended scope.

These are the material deviations/blockers found against the assignments. Frozen
`server/ask_test.go` and `sfwp/ask_transport_test.go` were not edited. No dependency, Rust,
contract, UI, golden, or unrelated production change was made by this critic.

## First focused gate: actual result

Command (from `apps/godspeed-casework-go`):

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 go test -race -count=1 -parallel=1 ./internal/server -run '^TestAsk'
```

Before the command, the host preflight reported 2,288,816,128 bytes available and both existing
Rust binaries (`target/debug/sea-forge`, `target/debug/sea-forge-server`) present. The
compiler-specific process scan printed the `bash` command wrapper containing the planned command;
it found no competing Go/Cargo/Rust/Bun/TypeScript build or test process. The complete captured
preflight is preserved byte-for-byte in `ask-runtime-focused-server-reject-preflight.txt`.
The test command ran with the required memory/runtime/cache limits and actual host socket access.

Exit status was 1 at package compile, with `askBody redeclared` plus helper conversion errors.
There is no test result and no behavioral failure inference from this command. Original full raw
output, exit file, and preflight were each copied byte-for-byte from `/tmp` and their copies were
verified with `cmp` before this record was written. No additional commands were run after this
compile failure.

## Frozen source identities at rejection

```text
4eb5440cd23e8e8a2cf0d929147ce2b71d756bf2fa9e870f3f5b259e983362b3  apps/godspeed-casework-go/internal/server/ask.go
31d5d38e8f4f080d472179ce0da7f63ac853a2ed57021d99eea2708c8cd7a023  apps/godspeed-casework-go/internal/server/server.go
53e91b67580f79fa002073cb8eef2b43e0d63a1d21548d4f4cbb4b32686edd7b  apps/godspeed-casework-go/internal/server/ratelimit.go
ebe23204a2390e4611c77d594e82cd6dfcb0a32343c1113e4900d11cfc5cf25c  apps/godspeed-casework-go/internal/server/http.go
75702c8577327b84d7cafeb23171c400e914d1a60f575e9e47588f701e8749eb  apps/godspeed-casework-go/internal/adapters/sfwp/ask.go
b21c1f018607eb28d41fc2dd7b27f2da43f2818c0f8091d11d7905566b8b277c  apps/godspeed-casework-go/internal/adapters/sfwp/client.go
17a9156ec9c21d2a50a816c97fa10b3916cc756fc42eaaeffbbc690f7da94c2b  apps/godspeed-casework-go/internal/adapters/sfwp/frame.go
7564a7ef04c2b0af242b6a7f270b4a9aef701221ec8ce39ec3f6c51cb4678200  apps/godspeed-casework-go/internal/adapters/sfwp/ask_adapter_test.go
8f3613930cea8052671611549646419c6e6f6e02007a7d28631417f1d3d7eb0f  apps/godspeed-casework-go/internal/server/ask_live_test.go
6c1021b2204cd1f4327a527b3feebd480e1e8007b0bc6bd31f2d7564b8629af5  apps/godspeed-casework-go/internal/livestack/stack.go
2a9b5c6620aef9d8a7b8cd9f597f245a8a69eae17c84ff172bf55f9c773771a6  apps/godspeed-casework-go/cmd/godspeed-casework/main.go
90612d084a10fce6ec4871faae563790c7de5267d2ef60369bc1fc23bd538cbe  apps/godspeed-casework-go/internal/server/ask_test.go (frozen fixture, unchanged)
94764ad34ca808bfa92ed96bfd91acd23fb8767c5fa9645515e57f91395426cc  apps/godspeed-casework-go/internal/adapters/sfwp/ask_transport_test.go (frozen fixture, unchanged)
```

Raw command log SHA-256: `1e7a64f33d95714de330a360da94f03628ada0d0b17b1095a1547d6e2b73700c`.
Exit file SHA-256: `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`.
Host preflight SHA-256: `121ecd958172d633755754c03e6f83d0b94e12479f820b1e21bf3dc3b345d8ea`.

## Next move

Fresh builder repairs only the compile-name collision, raw UTF-8 validation plus a new raw-byte
no-kernel-contact test, and the stale transport comments. Frozen fixture files stay unchanged.
The independent critic re-reviews the repaired source and reruns the complete required gates
from a fresh actual-host preflight. This rejection does not settle T09.
