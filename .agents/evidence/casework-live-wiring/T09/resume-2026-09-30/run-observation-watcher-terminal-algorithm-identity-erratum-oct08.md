# Phase B source review identity and provenance erratum

Date: 2026-10-08

This is an additive correction to
`run-observation-watcher-terminal-algorithm-independent-review-oct08.md`
(the preserved review SHA-256 is
`425909d9ca4e1ebe734d1df1f0f7a11d7492f14c01a9917b7f3c987ff8c33570`). That
review is preserved unchanged. Its manager-preimage hash in the identity table
is wrong: it says `b7f0511b5a1690253d3adb0132beafc106d3b03f5218c6bde3f1c40f4217884a`.
The actual decoded preimage hash is `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780`.

## Machine-derived identities

For each preimage, I decoded the JSON `source` string to UTF-8 bytes and
computed its byte count and SHA-256, then compared those values with the JSON
metadata and the current file. The following are the actual decoded and file
identities:

| File | Decoded preimage SHA-256 / bytes | Current file SHA-256 / bytes |
|---|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` / 26,548 | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` / 29,377 |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` / 8,873 | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` / 13,217 |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go` | `ea181a2f5781a7b11fd683642cca89b89c2d902ea44d8ea276a5c9c5c1d4a829` / 41,237 | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` / 41,286 |

The worker and fixture identities in the preserved review table were correct.
The manager preimage error was in my review, not in the source result or the
preimage JSON. The current manager file hash and byte count were correct.

## Preservation and source-anchor audit

I initially edited the newly created review file in place before recording a
hash or preserving a copy of that first draft. I therefore cannot provide an
identity or byte-exact reconstruction for the overwritten draft. I later
recorded the review at the SHA above; that preserved review still contained
the incorrect manager preimage hash. This erratum does not replace or modify
either record.

I mechanically checked the review's cited source ranges against the current
files identified above. The cited manager spans for lease state and
reservation (`81-90`, `112-129`), request identity (`304-319`), prepare and
creator handling (`321-381`, `386-570`), authorization and disclosure
(`588-646`), reference drain and join (`719-814`), and Stop ownership
(`828-915`) match the described code. The worker spans for launch and read
claim (`9-47`), continuation and final port check (`49-128`), watcher
validation (`151-241`), and candidate publication/terminal handling
(`270-429`) also match the cited behavior. The test span cited for the seven
focused groups is in the current fixture (`273-955`). I found no source-line
anchor error requiring another correction.

## Verdict

The source-readiness approval in the preserved review still stands, limited to
the actual current implementation hashes in the table above. The correction
changes the manager preimage identity record only; it does not change the
reviewed source or its semantic assessment. The formatter provenance limits
already disclosed in the preserved review remain: original `/tmp` formatter
paths were not retained, and the worker formatter-input identity mismatch is
not cured by the later root format verification. This is not a compilation,
runtime, GREEN, race, or integration claim.
