# Private Next static debt follow-up

Date: 2026-10-09. Read-only provenance follow-up and one bounded debt-ledger
append. This is not source review, compiler, test, or behavioral RED evidence.

## Grant and scope

This follow-up follows root's CW41 instruction to read the full current
`.agents/OBSERVED_DEBT.md`, append the specified factual evidence, and create a
new builder record for critic review with the repair-2 packet. It is disjoint
from the repair-2 builder and does not edit Next source.

Only these paths were written:

- `.agents/OBSERVED_DEBT.md`: appended one `Open` entry about private Next
  fixture provenance and repair-1 review evidence. Existing entries and
  historical claims were left intact.
- This evidence record.

The final debt-ledger SHA-256 is
`90aeb3c78f58ad298b911a766408ef3538f659107f51cc72fee6cf33c4701e57`.

## Recorded evidence

- Independent source review `b5693180` rejected repair 1, test-file SHA-256
  `72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620`, for
  ungated recovery overwrite/same-reference checks, seeded-wake oracle timing,
  terminal `workerDone` proof, and alignment.
- Root's read-only `gofmt -d` probe 1 exited 1 with alignment differences only
  and no stderr. All six files were archived and byte-compared by root.
- No Go compiler, tests, or behavioral RED run has occurred.
- The original untracked `c8e8253539ed7d72f61b417f99e2849bcc711fb8401cd395c6c35094ba216261`
  preimage was not retained. Exact old-file diff and proof that every original
  assertion was preserved are therefore unavailable.
- The repair-1 snapshot with SHA-256
  `72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620` was
  later saved and byte-verified by root; it is explicitly a current repair-1
  snapshot, not the original file.
- An earlier snapshot attempt labeled `preimage` was rejected by automatic
  review for provenance ambiguity and was not written. The later
  current-snapshot capture used an explicit label.
- Next move: critic reviews repair 2 against the accepted findings; root owns
  compiler and focused behavioral RED evidence. Validate current requirements
  and unchanged accepted manager/delta tests independently of unavailable
  predecessor-byte comparison.

## Limits

No source, status, Git, gate, compiler, test, or formatter command was run by
this follow-up. The `gofmt -d` evidence above is reported from root's probe,
not performed here. The provenance limitation must remain explicit; neither
this record nor the repair-1 snapshot proves the original untracked file's
exact contents.
