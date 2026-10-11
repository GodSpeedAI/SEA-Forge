# Independent review — CW-37 impact correction

**Disposition: APPROVE the narrow CW-37 repair.** The stale active-design reference to a 1 MiB raw-row limit is corrected to the approved 2 MiB row and 4 MiB page-input caps, distinct from the 1 MiB serialized-response cap. CW-37 remains open for implementation and runtime/migration proof. CW-38 and the preserved provenance/compiler limitations remain consistent with their evidence.

## Reviewed evidence

- Current debt entries CW-37 and CW-38: `.agents/DEBT.md:1433–1480`.
- Prior independent rejection and requested correction: `debt-initial-lease-normative-independent-review-oct09.md`, CW-37 finding.
- Policy approval: `c2-bounded-reader-additional-policy-operator-approval-oct08.md`, item 2.
- Normative limits: `.agents/specs/casework-live-cursor-v4-spec.yaml:107–120` and `REQ-C2-RANGE-004` at lines 266–300.
- Lease scope and evidence: `run-observation-initial-lease-bookkeeping-final-independent-review-oct09.md:3–5,15–30,50–54`; its wording erratum; and `initial-lease-red-root-disposition-oct08.md`.

## Findings

CW-37’s revised impact now says the approved 2 MiB raw-row limit includes LF and the 4 MiB page-input limit includes non-events and lookahead; it distinguishes both from the 1 MiB complete serialized-response limit. These values and boundaries match the operator receipt and `limits.event_page`. The correction retains the underlying risks around untrusted offsets, repeated-prefix work, and allocation before checking row size. Its status and next step continue to require a separate reader implementation/TDD grant and runtime/migration proofs; normative approval is not presented as implementation completion.

CW-38 remains accurately limited to the bounded Prepare lease bookkeeping unit. The cited final review approves that scope only, reports focused race, canonical Go, and full-module race as passed (729 cases, zero failures), and reports root’s and critic’s lossless comparisons of all 18 captured files. It preserves RED01/02 as setup/preflight holds and accepts RED03 as the behavioral RED. The formatter stdout/exit provenance gap remains disclosed alongside independently reproduced `gofmt` byte equivalence and canonical formatting success; the additive erratum correctly clarifies that the authorized gates were run. No Next/wake/drain, reader runtime, public wiring, or T09 closure is implied.

CW-26’s formatter/original-artifact provenance limits and CW-39’s active foreign-compiler constraint remain intact in `.agents/DEBT.md:1016–1052,1482–1494`. The lease review’s no-competing-heavy-process preflights do not erase the earlier cross-project constraint or claim that another project’s compiler was interrupted.

No further discrepancy was found in the requested bounded repair. No debt or source file was edited by this critic; no tests, compiler, gates, or Git mutation were performed.
