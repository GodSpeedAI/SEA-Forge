# T08 independent confirmation supplement: post-Store Go gates

This is an append-only correction to the Go gate references in
`independent-final-confirmation.md`. It does not alter or replace that report. The earlier
report's T07 authorization Go logs were not the right evidence for the production
`Store.Append` change in scoped checkpoint `48783dc`.

## Fresh post-Store verification

Because the prior T08 gate logs did not bind the shared stack helper's source hash, I reran
both Go gates against the current worktree after the Store repair. No Go source was edited
between the gates and this record.

Actual-host preflight before each command found no Go/compile/link/Bun/Cargo/Rust/Vite compiler
process and a writable `/home/sprime01/.cache/go-build`. Available RAM was 2,653,972 kB before
the canonical gate and 2,778,280 kB before full race. Both commands were serialized with
`GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1`.

1. From repository root, exit 0:

   ```sh
   GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOCACHE=/home/sprime01/.cache/go-build just casework-go-check
   ```

   This canonical gate checked gofmt, ran `go vet ./...`, and ran `go test ./...`; all
   packages passed. Captured stdout is
   `store-retention-green-evidence/independent-final-casework-go-check.log` (SHA-256
   `7e1fc8ba5409da6f99155709a5de7e3b2fa5e0f12f775eca398e1a6f271d07e5`).

2. From the Go module, exit 0:

   ```sh
   GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOCACHE=/home/sprime01/.cache/go-build go test -p=1 -parallel=1 -race -count=1 ./...
   ```

   All Go packages passed. Captured stdout is
   `store-retention-green-evidence/independent-final-go-module-race.log` (SHA-256
   `bce7574863a4f5bd7d9f56536e0afd6b6644d75c5b7cf3b2751a0f53638312be`).

## Source identity

Current source hashes recorded immediately after the post-Store gates:

```text
044a44b4b42efb6e79f7bf0a61d003d174d17500f147d2eb80e1a61779e43e64  apps/godspeed-casework-go/internal/projection/store.go
de60d75f3cb91aab30726545355b93cd612f1d94b2cc5f771c52c101525c769a  apps/godspeed-casework-go/internal/livestack/stack.go
```

The Store hash matches the independently approved `Store.Append` repair manifest in
`store-retention-green-evidence/independent-green-review.md`. The stack helper is unchanged
from the retention-helper change that was in place before these fresh gates; its only T08
change delegates the original test stack constructor to `AssembleStackWithRetention`, which
passes the requested retention limit to `projection.NewStoreWithRetention`. The live-tagged
native test is excluded from these untagged global gates and has its separate fresh `-tags=live`
race proof recorded in the main independent confirmation.

