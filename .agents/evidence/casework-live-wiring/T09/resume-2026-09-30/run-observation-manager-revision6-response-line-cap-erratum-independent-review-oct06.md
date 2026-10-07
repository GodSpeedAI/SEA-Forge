# Independent review: manager revision 6 response-line cap erratum

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-revision6-response-line-cap-erratum-oct06.md`, SHA-256 `0f1c51ab64bc0646542764721ba09cb285d8195039409d4c967ef7640ae6fa98`.  
Disposition: **APPROVE as a DOCONLY factual correction to revision 6 and its addendum.** Its statements match the bounded original erratum instruction, the recon, the source/test/approval records, and the current inspected artifact hashes. It makes no new runtime claim and releases no implementation work.

## Scope and identity checks

The original erratum instruction was to correct only the mistaken claim that the approved 32 MiB source-line cap remained unimplemented; distinguish the Go inbound response-line limit, kernel whole-journal behavior, and post-decode DTO frame retention; identify that no approved 32 MiB kernel row cap was found; cite existing source/test/approval evidence; and run no tests or compiler. The erratum is limited to that correction, preserves prior evidence, and explicitly leaves the manager's proposed caps and overflow policies unapproved.

I independently hashed the current source and approval artifacts named in the erratum. They match its table: `client.go` `ad3d599224527beda603a320b1b86faa82b80c75480d914cd805fa1e4de3a857`; `subscribe.go` `117b7876c99a054c24828911c659d28c3888ac02d38f7ab104c20faef4a8eff5`; `response_limit_test.go` `dfb98f494880faf946ceaf3d0d10a139a20ed3260647112dd08dfc6372ccaa88`; kernel `mod.rs` `cb6780d752f931d9d25dbfb0438ce4f3a2c11e7ecc3ad6d4384f1755377e037d`; `run_views.rs` `52dd5abf1da9a2a6bc65c7d0a19ef3c16c28ab13b738240b9e63c932c505a1ca`; cap assignment `19481f4666463e4f7c3bdd7b0d9cf9d5c32d32c79ed957f8d42c92f88f41e591`; cap independent review `ffafc6b806daa138001b7f9836218c5eedaf1a781128bcf013ffb0d6247979dc`; and recon `a591c4b6f21ae504cce792548ace7743d2a0487d9e3199f4cad0d13c407a4de8`.

## Source and contract assessment

The Go source uses a 32 MiB default inbound response-line cap including LF, validates the configured ceiling, and applies its bounded line reader before decoding replies; subscriptions use the same reader. The prior independent cap review approves that bounded client unit and records exact-boundary, overflow/poison, EOF, retry/recovery, mutation non-resend, and subscription tests. The erratum does not imply that these tests were run again.

The kernel source has a 4 MiB per-record constant and a distinct 64 MiB cap on the whole journal file. `read_jsonl` checks and reads the whole file, yields an empty vector on cap/read failure, and stops parsing at the first malformed row while retaining the valid prefix. The erratum correctly identifies this as CW-23 incompleteness debt. No cited assignment or approval establishes a separate 32 MiB kernel JSONL-row ceiling.

The Go adapter's 1,024-frame limit applies to its safe DTO projection after the response has arrived and been decoded. The erratum correctly states that it does not bound transport input or kernel reads. It also correctly avoids strict socket-byte, decoded-heap, aggregate-work, RSS, or whole-process claims.

## Bundle effect and limits

This erratum must accompany the final revision 6 document bundle because the frozen proposal's “unimplemented prerequisite” wording is factually stale. With this erratum, the documentary statement is corrected without rewriting historical artifacts. The Go cap's prior approval covers only its existing inbound response-line contract; it does not approve the manager's 16-cohort/128-attachment limits, combined 1 MiB retained-state budget, or overflow behavior. Those remain pending exact operator approval before source work. No kernel cap or T09 settlement is approved.

No source, tests, compiler, scanner, Git, Graft build, or runtime command was run for this review.
