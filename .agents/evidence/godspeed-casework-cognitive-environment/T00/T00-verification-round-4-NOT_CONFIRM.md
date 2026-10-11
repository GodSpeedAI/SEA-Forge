# T00 verification round 4 — verdict NOT_CONFIRM (preserved)

- **Round:** 4 (fourth independent adversarial confirmation)
- **Date:** 2026-09-19
- **Verifier:** a fresh independent subagent, read-only, specifically instructed to treat the
  builder's own self-audit as suspect and to falsify it.
- **Verdict:** **NOT_CONFIRM**.

## What held

The gitleaks re-measurement reproduced exactly (14 findings; spans 23×11 and 32×3; fields
`key`×11 and `idempotency_key`×3; value lengths 13×3 and 16×11; every value `[a-z0-9-]` with a
digit; three with a hyphen; entropy 3.5465937–3.75; all in commit `6ce518f`). All four refusal
guards behaved as claimed (empty report, whitespace-only span, unreadable file, unresolved value →
`exit 2`; missing report → `exit 1`). The regression set also held: spec sha256, validator exit 0,
86 requirement ids, T00..T14, both worktrees, operator checkouts untouched, the four `casework-*`
recipes absent, Go unavailable, `just context-check` exit 0, teeth exit 0, and the Gauntlet gates
still RESOURCE_DEFERRED with nothing claimed as passed.

## What failed

| # | Defect | Classification | Round-5 correction |
|---|---|---|---|
| R4-F1 | `CURRENT_STATUS.md` still said "CORRECTION ROUND 2 APPLIED" on one line while later lines and `current_status.yml` said round 4 — a recurrence of round 3's cross-artifact inconsistency, which the self-audit reported as `ok`. | **representation** (stale claim surviving its own correction; audit blindness) | The line now says round 4, and the audit's round check was rewritten to require a *single* current-round value across both files rather than merely the presence of a token. |
| R4-F2 | A unit error of the very class round 3 flagged survived: `3809564 kB` was labelled "≈ 3.69 GiB" (it is 3.63 GiB), and the swap low end "7.05 GiB" was wrong (the T00 gate series minimum is 7486428 kB = 7.14 GiB). Both were the *round-3 verifier's own arithmetic*, enshrined as corrections. | **representation** (inherited arithmetic error accepted without re-derivation) | Both recomputed from the raw logs (3.63 GiB; 7.14–7.47 GiB), and the audit now re-derives every `N GiB (M kB)` pair arithmetically. |
| R4-F3 | This settlement report's confirmation history ended at round 3, its evidence index omitted the round-3 record, and its "Next move" was stale. | **representation** (stale handoff section) | History and index completed through round 4; "Next move" updated; the audit now asserts that every handoff file it names exists. |
| R4-F4 | **The self-audit was partly theatre.** The verifier built a mirror and made it print `SELF-AUDIT: PASS` with a live defect of every one of its six classes: withdrawn wording in an unscanned file; line-wrapped withdrawn wording; a broken citation past its line cutoff; rewritten numbers it does not cover; a stale round line; an unlisted gate name (`GATE_GAUNTLET`, `just workbench-check`); and a deleted evidence file. | **oracle/test** (vacuity of the audit that gates re-confirmation) | The audit was rewritten: file set derived by walking the plan's own artifact directories; whole plan-owned region scanned; line-wrap-aware matching; arithmetic cross-check of `GiB`/`kB` pairs; single-round requirement; coverage of every `GATE_*`/`casework-*`/`workbench-check` name; existence assertion for every named handoff file. Non-vacuity is now demonstrated by `self-audit.sh --self-test`, which injects one defect per class into a temporary mirror and requires failure on each. |
| R4-F5 | Brief §3 truncated mid-sentence; `gate-baseline-seafoerge.md` had a 5-cell row in a 6-column table and an orphan duplicated line. | **representation** (edit damage) | Sentence restored; row given its raw-log cell; orphan removed. |
| R4-F6 | `target` was twice described as "gitignored"; `git check-ignore -v target` exits 1 and `git status` reports `?? target` because `.gitignore`'s `/target/` matches a directory, not a symlink. | **representation** (false claim about repository state) | Both statements corrected to "untracked, not ignored", with the reason. |
| R4-F7 | The re-measurement oracle can silently characterise the *wrong* field's value and still exit 0 (demonstrated by construction), and the refusal-guard tests are recorded as stdout only with no committed input fixtures, so they are not reproducible from the repository. | **oracle/test** | Disclosed as a residual limitation in the evidence: the script reports the enclosing field name and value class for every finding so a mismatch is visible to a reader, but it cannot prove the field is the one the rule intended; the guard tests remain stdout-only and are labelled as such. Recorded as debt rather than fixed, because a fixture-committed harness belongs to a later task's scope. |
| R4-F8 | Cosmetic: the evidence filename `gate-baseline-seafoerge.md` carries the same typo the prereg amendment says it corrected (`seafoerge`). | **representation** (naming) | Left as-is deliberately: every citation uses the typo'd name consistently, so renaming would break the audit trail for the sake of cosmetics. Recorded here so it is a known, harmless inconsistency rather than an accident. |

## What round 4 could not verify

Whether the round records and handoff files were rewritten rather than appended (all evidence is
untracked, so there is no diff history); the prereg's "semantically unchanged" claim (no snapshot or
self-hash exists); that the guard-test outputs were produced by the inputs they imply; that the
teeth "re-run" logs are genuine re-runs rather than byte-identical copies (the script is
deterministic, so both explanations fit); heavy gate re-execution (raw logs and time blocks were
inspected instead); Gauntlet's three gates (still RESOURCE_DEFERRED); and the secrecy of the matched
values (deliberately unread).
