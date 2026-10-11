# T00 verification round 2 — verdict NOT_CONFIRM (preserved)

- **Round:** 2 (second independent adversarial confirmation)
- **Date:** 2026-09-19
- **Verifier:** a fresh independent subagent, prompted to falsify the *corrections* rather than
  accept them, working read-only.
- **Verdict:** **NOT_CONFIRM** — the seven round-1 findings were each substantively addressed,
  but two were only partially applied and the corrections introduced further defects. Materially:
  a false statement survived as *current* handoff text, and an unbacked measurement was still
  displayed and cited.
- **Disposition:** round-3 corrections were applied elsewhere (brief §12, the settlement report's
  confirmation history, and the canonical handoff files). This record is preserved, not rewritten.

## Round-2 verification of the round-1 corrections

| Round-1 finding | Round-2 verdict |
|---|---|
| F2 (`src-tauri/src` 7 files) | **Verified correct** — reproduced independently. |
| F3 (22 workspace members) | **Verified correct** — reproduced independently. |
| F4 (`contracts/schema` 57; generated 115; 172 to migrate) | **Verified correct** — reproduced independently. |
| F5 (composites now run and recorded) | **Verified correct** — artifact exists; claimed exits/wall times match the raw status file and the `/usr/bin/time -v` blocks; the failing step is really `security` in both; earlier steps evidenced as passing; `no-async-kernel` not claimed. |
| F6 (brief structure) | **Verified correct** — 12 top-level headings, §4 item 7 coherent, §5 dedented, the report's "12 sections" claim holds. |
| F1 (gitleaks) | **Partially fixed.** The numbers are right and were independently reproduced (23-char spans ×11, 32-char ×3; fields `key` ×11 and `idempotency_key` ×3; values 13/16 chars of `[a-z0-9-]`, all 14 containing digits; entropy 3.5465937–3.75; all 14 in commit `6ce518f`). But the **withdrawn characterization still stood as current text in five artifacts**, including both canonical handoff files. |
| F7 (`proof` wall time) | **Partially fixed.** The re-run is now backed, but the original unbacked "2 s" figure was **still displayed** and still cited to a log containing no time block. |

## New defects found by round 2

| # | Defect | Classification | Round-3 correction |
|---|---|---|---|
| R2-F1 | The withdrawn "false-positive-class / 8-character alphabetic / no digits or symbols" wording remained current in: brief §8 (B2), brief §10 item 2, `T00-settlement-report.md`'s B2 row, `.agents/CURRENT_STATUS.md` (Verification and Blockers), `.agents/current_status.yml` (`external_blockers.T05`), and the decision log's `D-…-T00-04` basis. The verifier measured all 14 values and confirmed all contain digits and are 13/16 chars — every clause of the surviving text was false. | **representation** (incomplete correction propagation) | All six locations rewritten to the corrected identifier-class statement, each pointing at the round-2 correction. A grep for the withdrawn wording across this plan's artifacts now returns nothing but explicit disclosures. |
| R2-F2 | The unbacked round-0 "2 s" `just proof` figure was still shown and cited to `raw-logs/gate-proof-rerun.log`, which has no time block. | **representation** (evidence backing) | Brief §9 and `gate-baseline-seafoerge.md` now show the **backed** round-2 value (exit 0, 3 s, max RSS 72784 kB, `raw-logs/round2-proof.log`) and label the "2 s" figure explicitly as an unbacked, disclosed round-0 defect. |
| R2-F3 | `teeth/remeasure-gitleaks.sh` printed `field='i'` / `field='k'` (the characters *before* the match start), so the raw log cited as the source did not show the claimed names `key` / `idempotency_key`. The claim was true (the verifier recovered the names from the report's own `Match` field) but the citation was wrong. | **representation** (oracle self-consistency) | `field_name()` rewritten to return the enclosing field's full name, then (round 4) the script was rewritten to measure the VALUE and report its class and digit/hyphen presence from the real bytes. |
| R2-F4 | The script would have exited 0 with `findings: 0` on a present-but-empty report — a vacuity hole. | **oracle/test** | Explicit non-vacuity guard, extended in round 4: an empty report, a whitespace-only span, or an unresolved value now refuses with `exit 2` (all three verified). |
| R2-F5 | `T00.prereg.yaml`'s mtime (17:58:06Z) postdates its declared `frozen_at` (17:57:30Z) and no self-hash or copy existed, so "unchanged since freeze" was not verifiable from the handoff. | **representation** (freeze evidence) | Cause disclosed rather than argued: a 36-second typo fix to one measurement id. Round 4 further narrowed the disclosure to what is actually establishable (see `post_freeze_amendments` in the prereg). |

## What round 2 independently re-confirmed after the edits

Spec sha256 in all three places; validator exit 0; 86 distinct requirement ids; tasks exactly
T00..T14; gate activation with zero premature uses; both worktrees at the claimed revisions; the
operator checkouts still untouched (Gauntlet status md5 `5abbc212…a557`) with no plan artifacts
written into them; the four `casework-*` recipes still absent; Go still unavailable;
`just context-check` exit 0; `.agents/CURRENT_STATUS.md` coherent with its `Updated:` line and all
eight headers; both status files parse and do not contradict each other; `.gitleaks.toml`'s
commit-scoped precedent present; Gauntlet TUI 162 files on `fe75108` and 0 on `main`; teeth
non-vacuous (exit 0 on re-run). The round-1 record was judged **not** to overstate what round 1
confirmed.

## What round 2 could not verify

Heavy gate *re-execution* (it verified the raw logs, status files, and time blocks instead);
Gauntlet's three gates (still RESOURCE_DEFERRED, not claimed); the secrecy of the matched values
(deliberately unread — the operator's decision); and the round-1 verifier's internal process.
