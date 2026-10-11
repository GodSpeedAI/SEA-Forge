# T02 verification round 1 — verdict CONFIRM

- **Round:** 1 (first independent peer confirmation of T02)
- **Date:** 2026-09-19
- **Verifier:** a fresh independent agent, not the builder of any T02 artifact; worked against the
  plan worktree at `ff3c285`.
- **Verdict:** **CONFIRM.** The omit decisions are correctly grounded (optionality + unavailability,
  not absence alone), no donor code or obligation exists, and no contract depends on a donor.

## Claim-by-claim table (verifier's own commands and observations)

| # | Claim | My command / inspection | My observation |
|---|---|---|---|
| 1 | The searched roots were actually searched | read `teeth/run-teeth.sh`, then re-ran it, then re-ran the same find myself | tooth searches `$HOME/projects`, `$HOME`, `/opt` at maxdepth 3 for `*open*mct*` / `*openmontage*`; my independent re-run returned nothing under all three roots |
| 2 | No copied donor code, no donor licence/notice obligation | tooth's copied-code check + `find apps -type f` | 16 files under `apps/`, all authored for this plan (I read the core Go files and all six contract TS files in the T01 verification); zero third-party licence or `SPDX-License-Identifier` headers under `apps/` |
| 3 | No contract definition, import path, or namespaced reference depends on a donor | re-ran tooth 1 | it now inspects **real** surfaces (the implementation brief, `apps/godspeed-casework-go/go.mod`, and all Go/TS sources under `apps/`) and finds no donor marker; `go.mod` declares the module and `go 1.27` only — no requires at all |
| 4 | The vacuous provenance obligation is vacuous (asserted, not assumed) | tooth 2 output + decision record §2 | "no third-party licence or notice header under apps/ — no copied file to attribute"; the decision record states the obligation is vacuous *and* writes down what any future copied file would require |
| 5 | `GATE_SPEC_TRACE` | `python3 .agents/plans/validate-…py` (run during the T00 round-6 pass) | exit 0 |
| 6 | Decision distinguishes "names" from "depends on" | read `donor-and-license-decisions.md` and the tooth | tooth 1 explicitly counts decision prose naming a donor as expected, and inspects only definitions/imports/namespaced references — the corrected post-first-failure instrument |

## The NOT-APPLICABLE-YET status, stated plainly

At settlement time (commit `0c8117e`, before T01 existed) tooth 1's strongest form was
not-applicable: there were no Go ports or React contract sources to police, and the tooth said so
rather than passing silently. That is still the honest description for the surfaces T04 will create:
**the tooth has NOT yet inspected T04's Go projection/coordination ports (they do not exist), and
T04 and T07 MUST re-run it against the real interfaces before they settle.** On today's re-run,
however, the tooth's inspectable set is no longer empty — it covered T01's real module and contract
package and found no donor marker — so the confirmation rests on real, current evidence for
everything that exists, and on an explicit re-run obligation for what does not.

## Falsification attempts

1. **Independent donor search** across `$HOME/projects`, `$HOME`, `/opt` (the tooth's roots, run by
   me, not via the tooth): no donor checkout. The omit decision's factual basis holds.
2. **Donor-marker grep over the live app sources** (via tooth 1's re-run): none. If a donor
   namespace had been imported anywhere under `apps/`, tooth 1's declaration/import and
   namespaced-reference patterns would have matched it.

No findings. Nothing was corrected in response to this round.

## What I could not verify

- T04's Go ports and T07's temporal/narration mechanics do not exist yet; REQ-DONOR-003 compliance
  for them remains an open re-run obligation, recorded in the decision record §4 and above.
- Whether a donor exists beyond depth 3 in the searched roots or outside them (e.g. an unmounted
  archive). The spec makes donor use optional, so this cannot change the decision — only trigger the
  recorded revisit-as-new-decision rule if one ever appears.
