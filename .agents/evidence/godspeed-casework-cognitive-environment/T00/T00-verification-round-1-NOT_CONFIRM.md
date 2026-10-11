# T00 verification round 1 — verdict NOT_CONFIRM (preserved)

- **Round:** 1 (first peer/independent confirmation attempt for T00, P2, confirmation mode `peer`)
- **Date:** 2026-09-19
- **Verifier:** a fresh independent subagent, prompted to falsify rather than confirm, working
  read-only and recomputing every number itself from the frozen revision.
- **Verdict:** **NOT_CONFIRM** — 7 findings.
- **Disposition:** correction round 2 applied (below), then re-confirmation requested. This
  record is preserved verbatim in substance and is **not** rewritten; the corrections are
  appended, not substituted.

## What the verifier confirmed (independently reproduced, no defect)

Spec sha256 reproduced (all three places: file, plan `source.spec.sha256`, status
`metadata.spec_sha256`); validator exit 0; 86 distinct `REQ-*` ids; 15 tasks T00..T14; gate
activation order with **zero** premature uses; the four `casework-*` recipes genuinely absent; both
worktrees at the claimed paths/branches/revisions; operator checkouts untouched (Gauntlet status
md5 `5abbc212…a557` reproduced) with **no** plan artifacts written into either; `main` in Gauntlet
genuinely has no `workbench/` TUI while the `fe75108` branch has 162 files; SFWP 23 methods in code
vs 18 in the doc with the five methods named correctly; the three load-bearing ABSENT claims (no
lease/claim/reserve method, no artifact-persistence method, no repository-fact/webhook method) and
the corrected-to-PRESENT GitHub authority anchor; Gauntlet TUI file count 162; Go genuinely
unavailable; the security red reproduced exactly (14/14, same six paths, same commit);
`.gitleaks.toml`'s commit-scoped precedent; teeth non-vacuous and working-directory independent;
`just context-check` exit 0; the handoff contract's `Updated:` line and eight section headers; and
that the preregistration's `frozen_at 17:57:30Z` precedes the first gate `started_at 17:59:48Z`.

## Findings, classification, and correction

| # | Finding | Classification (plan vocabulary) | Correction |
|---|---|---|---|
| F1 | The gitleaks finding characterization was measured off the `--redact` placeholder: every `Secret` is the literal `REDACTED` (8 mixed-case alphabetic characters), so "8 characters, mixed-case alphabetic" was a property of the placeholder, not of the findings. The report's own column offsets (23 / 32 characters) contradicted it. | **representation** (measurement design; also a missing self-consistency check) | Re-measured from the real bytes at the recorded column offsets with no value printed (`teeth/remeasure-gitleaks.sh`): 23-char spans ×11, 32-char ×3; matched fields named `key` / `idempotency_key`; values 13–16 chars, lower-alnum-hyphen. Round 0's table preserved verbatim as withdrawn. The B2 rationale is narrowed accordingly. |
| F2 | `src-tauri/src` claimed "9 files"; actual 7 (the 9 conflated 7 source files + 2 test files). | **representation** | Corrected in brief §6 to 7 files (plus 2 test files on their own row). |
| F3 | Workspace members claimed "23 entries"; `Cargo.toml` has 22 (the 23 came from counting `crates/AGENTS.md`). | **representation** | Corrected in brief §1 to **22 members**. |
| F4 | `contracts/schema` claimed as 174 files; actual 57 `*.schema.json` (174 is the whole `contracts` package; `generated/` holds 115). | **representation** | Corrected in brief §6 to 57 schema + 115 generated = 172 files to migrate. |
| F5 | `just check` / `just ci` results were recorded by pointer to `gate-baseline-composites.md`, which did not exist — an unsupported claim about two existing gates. | **missing prerequisite** (evidence produced as a promise) | The composites were actually run and recorded in `gate-baseline-composites.md`: `just check` exit 1 (101 s) and `just ci` exit 1 (41 s), both failing **only** at `security`, with the passing steps evidenced line by line; `just proof` re-run exit 0 with a real time block. |
| F6 | Brief §5's heading was indented inside §4's numbered list, §4 item 7 was truncated mid-sentence, and the report's "12 sections" claim was false (`grep -c '^## '` = 11). | **representation** | §5 dedented, §4 item 7's tail restored, heading count now 12 as verified. |
| F7 | The recorded 2 s `just proof` wall time was not backed by any raw log (the re-run log had no time block). | **representation** (evidence backing) | `just proof` re-run under `/usr/bin/time -v`; raw log `raw-logs/round2-proof.log`, wall 3 s, exit 0, max RSS 72784 kB. |

**Governing invariant remains valid.** None of the seven findings falsifies T00's claim or the
plan's structure: the authority binding, the 86/86 traceability, the DAG, the worktree isolation,
the missing-seam inventory, and the teeth all reproduced. The defects are in *measurement,
bookkeeping, and evidence backing* — the layer T00 exists to make honest — so the smallest
falsified layer was corrected rather than the claim abandoned.

## Re-run of the original teeth/falsification check (protocol step)

`teeth/run-teeth.sh` re-run after the corrections: exit 0, both teeth behave as specified
(control PASS → one-byte mutation detected as `frozen spec SHA-256 differs from current bytes`
with a non-zero exit; nine absent-capability claims contradicted; two corrected-to-PRESENT GitHub
authority anchors confirmed; T03/T04 still blocked on T01). Output: `teeth/teeth-run.log`.

## Consequence for the operator decision

B2's proposed repair is **not** the same recommendation it was in round 0. Round 0 implied "clearly
word-like false positives". Round 2 says: the flagged text is a JSON field named
`key`/`idempotency_key` with a 13–16-character identifier-shaped value inside committed evidence;
that is consistent with record identifiers, but shape alone cannot prove non-secrecy from a
redacted report, so the field semantics should be confirmed before any allowlist is written. The
repair remains withheld pending that decision.