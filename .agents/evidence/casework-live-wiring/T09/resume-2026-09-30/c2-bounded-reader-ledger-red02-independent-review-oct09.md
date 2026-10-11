# Bounded ledger reader regression RED02: independent evidence review

Date: 2026-10-09

## Verdict and limits

**APPROVE the RED02 evidence as expected behavioral RED for the two Phase A
regressions only.** Both regressions failed at their expected assertions
against the Phase A source. This establishes neither production correctness
nor a fix for any finding. In particular, the allocation-order finding was
not tested by RED02. Phase B remains separately authorized by root; this
review does not approve its implementation.

## Captures verified

The six immutable capture archives under
`.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/` were decoded and
compared byte-for-byte with `/tmp/ledger-reader-regression-red02-z0az6rna/`.
All lengths and hashes agree:

| Capture | Bytes | SHA-256 |
|---|---:|---|
| command | 72 | `ad420cbb202467090662fe20b0535cf66e03ab30c0e480da93758d92bb99f26b` |
| preflight | 33,617 | `7b0ea1b25177785068350d0ebddb69ecfa7474f6dd9d3fb7fc5432561f5d81fe` |
| preflight exit | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| stdout | 1,315 | `503baf711f9a4d3d0e8dc7de2ff3776f20ed4ddd576dfa382d07ec86e41b891e` |
| stderr | 6,326 | `43c9ba9de2eda06417c45e4e17cc82418d367a194eb13166bce84926c4e4ea0f` |
| exit | 4 | `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` |

The command capture is `timeout 180s just crate-test sea-forge-ledger
bounded_reader_regression_`. Preflight exited 0 and recorded source
`types.rs` SHA-256
`84ee1cba001defd573f93a71f6465b77e7e41d5770395a3d6b580782b7e16ed3`, matching
the Phase A frozen source. Its resource sample reported `MemAvailable` 3,880,568
kB and `SwapFree` 6,296,824 kB. The process scan showed no active `cargo` or
`rustc` compiler process (the visible Rust analysis services are not compiler
builds).

Stderr records successful compilation in 10.64 seconds and no compiler error.
The child exited 101; stdout reports `0 passed; 2 failed; 32 filtered out`.
Each test fails at the expected assertion: the malformed-row test receives a
non-error on the call after its first parse error, and the pretty-JSON resume
test receives `Ok` for its multi-line claimed row. There is no timeout.

## Scope

The result is the intended expected RED for the two new tests. It does not
test allocation-before-budget ordering and proves no reader acceptance
vector, implementation behavior, ACK eligibility, or runtime correctness.
No test was rerun and no source, debt, status, or Git file was changed during
this review.
