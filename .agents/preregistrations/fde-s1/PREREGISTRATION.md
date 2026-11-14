# Pre-registration fde-s1: S1 gate of the FDE qualification plan

Registered by WP-09 on `fde/integration`. Source: `G-qualification-plan.md` draft 2 (sections 1, 2, 4, 5, 6, 9, 10), `F-implementation-plan.md` WP-09, `H-egm-review-record.md` RD-04, RD-20, RD-26, OD-6. Integrity: `MANIFEST.sha256` lists the sha256 of every other file here; the pre-registration digest is the sha256 of that manifest and is anchored as git tag `fde-anchor/prereg/fde-s1` (`scripts/fde-anchor.sh verify prereg/fde-s1`).

## Claim (exactly)

"the Mission §1–7 path exists and runs once on program execution."

It is not evidence about organisational behaviour, and the claim ledger says so.

## What is frozen

- `tests.yaml`: 21 entries. Acceptance AT-01, 04, 05, 06, 07, 08, 12, 13, 14 (read adapters), 18 (S1 form). Negative NT-01 to NT-09 (with NT-03b) and NT-16. Each has its oracle from G verbatim, a falsifier, the command `scripts/fde-proof.sh --slice s1 --only <ID>`, its owning work package and the status vocabulary PASS | FAIL | PARTIAL | NOT_TESTED | NOT_RUN.
- `episodes.yaml`: three `program` episodes in one world (`world:godspeed-fde-sop@sha256:78eddd4e…287b`): a conforming control (E2), a falsifiable inquiry (E3, hypothesis frozen; outcome unknown at freeze, predicted F), and a subsequent episode after one revision (E1 in the E4 slot). Inputs, expected change, invariants, predicted irrelevancies, success and failure regions, falsifiers, and the predicted comparison class per expectation.
- `predictions.yaml`: one predicted status per test with a reason. 12 PASS, 9 PARTIAL. No PASS is predicted for anything depending on postconditions, human-reserved decisions or independent anchoring.
- `READINGS.md`: where G is ambiguous, the quoted text and the reading used. G's criteria are not altered.

## Scope of the S1 gate

AT-07 requires `confirmed` plus one of `contradicted` / `observed_undeclared` to arise; other classes may be NOT TESTED without failing the gate. The three episodes are `program` executions; the harness's fresh-context verification step, not declared in model v1, is the revision candidate.

## Non-claims

No claim about `live` execution, the five behaviour kinds on `live` work (S5), transfer (AT-16), installation (AT-20), variation, or recovery beyond the S1 negative tests. `injected_fault` records never support an execution claim (NT-26).

## Limits

- Pre-S3 identity: principals are SEA-Forge actor identities at assurance `session`. Rows depending on A3 on a separate surface or any human-reserved decision (promotion approval, settlement, disposition) are PARTIAL until S3 and a real authenticator (OD-5).
- Anchoring rows are PARTIAL without OD-6 (a dedicated anchor credential and a protected `fde-anchor/*` namespace). On a single-operator host the pusher and the verifier are the same principal and OS user (RD-26): the anchor shows "not changed since publication", not independence. The tag is unsigned in this version and the server push time is not read from the GitHub API; both are recorded as limits, not hidden.
- DomainForge 0.21.2 accepts only `at precondition`; `evaluate` binds the outermost root quantifier only; `exists_unique` with an unknown item is downgraded to unknown. Expectations resting on a postcondition are classed `insufficient`.
- Emitter signatures arrive with WP-33 (S3); authenticity before then is source-record resolution plus the independent-emitter rule.
- Mutation patches (`mut/<NT-id>.patch`, `mut/SHA256SUMS`) are not in this version; they are created with their owning work packages. A version that adds them is a new version (`fde-s1-v2`).

## Versioning

A criterion change is a new version `fde-s1-v2` under a new tag, never an edit of these files or a moved tag.
