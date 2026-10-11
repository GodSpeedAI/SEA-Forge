# Independent review: retained helper format and focused race gate

Date: 2026-10-07. Verdict: **APPROVE the authorized format-only source result
and the bounded focused race verification.** This does not approve or verify
the retained algorithm beyond the selected tests, lifecycle integration,
manager behavior, `Next`, SSE, or T09 settlement.

## Source review

I read the full format-only assignment, result, exact UTF-8 preimage JSON, the
full original retained helper assignment, and its prior immutable source
review. The JSON preimage decoded to 10,389 bytes and SHA-256
`38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a`, matching
the assignment's authorized pre-edit identity. Current source SHA-256 is
`2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`, matching
the recorded format result.

The preimage/current unified diff contains only four zero-context hunks:

1. candidate `Execution` field alignment (`run_observation_retained_version.go:148`);
2. `retainedImage` field-column alignment (`:239-255`);
3. image key identity alignment (`:301`);
4. availability and frames alignment (`:307-308`).

All 21 removed and 21 added changed lines pair after whitespace removal; the
full preimage/current byte streams with whitespace removed are identical.
Review of the complete diff confirms only indentation/alignment spaces changed;
identifiers, operators, literal contents, comments, field order, expressions,
and punctuation are unchanged. This is a format-only comparison, not an
algorithm approval.

Read-only `gofmt` output is byte-identical to the current source: both hash to
`2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`, and
`cmp` returned 0. `gofmt -d` returned exit 0 with empty output (SHA-256 of the
empty output: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`).

All frozen inputs still match their assigned identities:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` |

## Focused race evidence

After checking current and frozen identities, a fresh preflight reported
3,656 MiB available RAM and 12,284 MiB free swap, above the assigned 1,200 MiB
and 512 MiB floors. It recorded Go `go1.27.1`, the full command/environment,
and all five current/frozen source hashes.

Executed from `apps/godspeed-casework-go`:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^TestRunObservationRetained'
```

Joined exit was 0; Go reported `ok .../internal/server 2.202s`. The selector
matches 13 top-level retained tests in the two frozen retained test files
(8 base and 5 policy). Their table/loop cases comprise 13 nested cases (2
ordinal-overflow, 5 terminality, 2 final-marker, 2 key-mismatch, and 2 invalid
total-count cases). This command did not use verbose output, so names and case
counts are derived from the frozen source selector/test structure; the actual
runtime output is the preserved non-verbose `ok` result.

The actual preflight, combined stdout/stderr, and joined exit were each
immediately archived and byte-compared with their unique `/tmp` originals;
all three `cmp` exits were 0. Their SHA-256 values are respectively:

* preflight: `46bf0e19cdc46504fd3b7ca65b7785041816741913ba935de7e1dd6086a347f1`
* output: `59a68ee04e369f59fa4c8bf54c5128d914257eacd7c6f3813797a06668365167`
* exit: `5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9`

Archived captures are `run-observation-retained-format-race-preflight-oct07.raw`,
`run-observation-retained-format-race-output-oct07.raw`, and
`run-observation-retained-format-race-exit-oct07.raw`. The selected focused
race tests passed. Other package tests and all manager lifecycle behavior
remain outside this verification.
