# Private run-observation manager revision 6 — response-line cap erratum

Date: 2026-10-06  
Status: DOCONLY normative correction. Preserve revision 6, its addendum, and
all recon/review records unchanged. This erratum supersedes only revision 6
and its addendum's statements that a separate approved 32 MiB source-line cap
is unimplemented or remains an unimplemented prerequisite. No source, test,
runtime, or policy change is made or authorized.

## Corrected distinction

The approved 32 MiB limit is implemented in the Go SFWP client's **inbound
serialized response-line** reader, including the terminating LF, before JSON
decoding. It also covers subscription lines. This is not a 32 MiB limit on a
kernel JSONL source row, kernel file read, response construction, decoded heap,
or aggregate work. The existing focused boundary/recovery tests and prior
independent approval establish this already implemented client contract; this
erratum runs no new test or compile command and makes no new runtime claim.

The existing kernel `MAX_JOURNAL_BYTES` limit is 64 MiB for the whole journal
file before JSONL parsing. `read_jsonl` returns an empty vector when that
whole-file cap/read fails and decodes only the valid prefix when a malformed
row occurs (`crates/sea-forge-server/src/sfwp/mod.rs:50-60`;
`crates/sea-forge-server/src/sfwp/run_views.rs:417-436`). This remains the
separate false-empty/prefix-incompleteness debt recorded as CW-23
(`.agents/DEBT.md:843-850`). No separate approved 32 MiB per-row kernel cap
was found or is implied by the Go response-line contract. Do not invent that
source prerequisite or treat either limit as satisfying the other.

The Go run-trace adapter's 1,024-frame retention is a post-response,
post-decode safe-DTO projection limit (`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go:20,67-177`).
It does not bound raw transport input, kernel file reads, decoding expansion,
or serialization. An accepted kernel read/response can exceed the Go line cap
and then be rejected as unavailable by the client. No strict socket-byte,
heap/RSS, or whole-process bound follows from these three distinct limits.

## Evidence anchors and immutable identities

The directly inspected source/test identities are recorded for review:

| Artifact | SHA-256 | Relevant spans |
|---|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` | `ad3d599224527beda603a320b1b86faa82b80c75480d914cd805fa1e4de3a857` | `32,35-40,71-80,252-299`: 32 MiB default including LF, pre-dial config validation, bounded line read before decode, typed overflow and connection close |
| `apps/godspeed-casework-go/internal/adapters/sfwp/subscribe.go` | `117b7876c99a054c24828911c659d28c3888ac02d38f7ab104c20faef4a8eff5` | `121-140`: same bounded reader for subscription lines |
| `apps/godspeed-casework-go/internal/adapters/sfwp/response_limit_test.go` | `dfb98f494880faf946ceaf3d0d10a139a20ed3260647112dd08dfc6372ccaa88` | `20-21,45-155,185-227,249-338`: configured cap tests, exact 32 MiB fragmented boundary, one-byte-over poison, EOF, retry/recovery, mutation non-resend, subscription rejection |
| `crates/sea-forge-server/src/sfwp/mod.rs` | `cb6780d752f931d9d25dbfb0438ce4f3a2c11e7ecc3ad6d4384f1755377e037d` | `50-60`: distinct 4 MiB record and 64 MiB journal constants |
| `crates/sea-forge-server/src/sfwp/run_views.rs` | `52dd5abf1da9a2a6bc65c7d0a19ef3c16c28ab13b738240b9e63c932c505a1ca` | `417-436,793,842-857,947-963`: whole-journal check/read/prefix parsing and trace projection |
| `cap-implementation-assignment.md` | `19481f4666463e4f7c3bdd7b0d9cf9d5c32d32c79ed957f8d42c92f88f41e591` | `14-26`: approved default/client-line boundary and explicit aggregate-work exclusions |
| `cap-implementation-independent-review.md` | `ffafc6b806daa138001b7f9836218c5eedaf1a781128bcf013ffb0d6247979dc` | `1-19` and Verification: bounded client unit approval and recorded focused results |
| `observation-cohort-runlist-response-cap-correction-oct05.md` | `a76e8003361f43d7d31d8eb88164f6923cd2f95e27d60e51dcf9e6fca9fa930f` | `1-34`: prior append-only correction retracting the stale unimplemented-cap claim |
| `response-line-cap-manager-revision6-recon-oct06.md` | `a591c4b6f21ae504cce792548ace7743d2a0487d9e3199f4cad0d13c407a4de8` | `22-38,42-80,88-130`: distinction, source evidence, and bounded correction |

These are hashes recorded/read at this review point, not newly produced test
captures. The cap assignment and its independent review are the approved cap
artifacts. Their approval applies only to the existing Go inbound response
line contract. The old kernel/source-line prerequisite wording is stale and
superseded by this erratum.

## Unchanged decisions and hold

The per-poller combined 1 MiB retained-value budget, 16-cohort/128-attachment
limits, terminal-overflow behavior, and every proposed manager policy remain
unapproved pending exact operator approval. This erratum adds no public policy,
DTO, error, API, schema, or kernel cap. The existing 32 MiB client cap and
64 MiB kernel whole-journal cap are described only to distinguish their
already recorded behavior. Keep CW-23 open; do not claim complete trace
inventory, absence of a false-empty prefix, or bounded kernel source work from
this response-line limit.

Have the independent `guard_ui_evidence_critic` review the original erratum
assignment and this frozen path/hash before any later source assignment. No
tests, compiler, runtime, scanner, Git, or external action was performed for
this erratum.
