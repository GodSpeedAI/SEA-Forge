# T00 verification round 5 — verdict NOT_CONFIRM (preserved)

- **Round:** 5 (fifth independent adversarial confirmation)
- **Date:** 2026-09-19
- **Verifier:** a fresh independent subagent, read-only, instructed to attack the rebuilt self-audit
  and to treat it as a suspect instrument.
- **Verdict:** **NOT_CONFIRM**.

## The decisive finding

**`.agents/reports/godspeed-casework-cognitive-environment/decision-log.yaml` was empty (0 bytes)**, and
`CURRENT_STATUS.md`, `T00-settlement-report.md`, `current_status.yml` and the plan all cited it as
holding decisions `D-2026-09-19-T00-01 … -09`. The self-audit listed that exact path as required and
reported `ok: 13 of 13 required files present` → `SELF-AUDIT: PASS`. **Existence is not content**, and
the instrument had certified the emptiness as fine. The log is restored (9 decisions) and the audit
now asserts non-trivial size plus parseability plus the nine expected decision ids.

## Coverage gaps the verifier proved by injection

Every row below was injected into a private mirror with a verified diff against pristine:

| Injected defect | Result |
|---|---|
| withdrawn wording split across a paragraph break | caught |
| withdrawn wording re-asserted with different punctuation ("8 characters in mixed case, alphabetic only, containing neither digits nor symbols") | **missed** |
| broken citation `…/missing-table.csv` (extension outside its grammar) | **missed** |
| broken citation with no extension | **missed** |
| broken citation without backticks | **missed** |
| bare `GiB` figure with no `(M kB)` parenthetical | **missed** |
| `GATE_SPEC_TRACE … PASS` claim | **missed** |
| `just ci passed … just proof passed` claim | **missed** |
| deleted non-REQUIRED evidence (`raw-logs/gate-lint.log`, the inherited status copy) | **missed** |
| deleted `T00-verification-round-4-NOT_CONFIRM.md` | **missed** |
| withdrawn wording placed inside `self-audit.sh` itself | **missed** (the instrument excluded itself from content checks) |
| `CORRECTION ROUND 2 APPLIED` injected into the brief | **missed** (round check read only the two handoff files) |
| applied-round token deleted from `current_status.yml` | **missed** (the union still read `[4]`) |
| in-list gate name with a pass claim | caught |
| the same claim with a negation word anywhere on the line | **missed** (negation suppressible) |
| handoff files disagreeing on the round (`[2,4]`) | caught |

It also showed that the injection test verified *exit status* rather than *cause* (an injector whose
`sed` matched nothing was still reported `ok: detected`, and with an unrelated persistent defect the
control merely printed `NOTE:`), and that `AUDIT_SKIP_SELFTEST_LOG=1` disabled the non-vacuity
cross-check in a **real** run, not just in mirror mode.

## Round-6 response: fix what was fixable, and narrow the instrument honestly

Fixed: the emptied decision log was restored and now has a content assertion; the audit gained
content checks (non-trivial size, YAML parse, expected decision ids, verification-round records
1–5 present and stating their verdict); it now detects punctuation-variant and wrapped withdrawn
wording, `.csv`/extensionless citations, arithmetic mismatches, stale/absent round claims (including
"no applied round named at all"), and deleted evidence including round records; its injections are
verified to have modified the mirror before a detection is credited; `AUDIT_SKIP_SELFTEST_LOG` was
replaced by real mirror detection (comparing `AUDIT_ROOT` with the script-derived root); the `target`
claims, the three unbacked preflight rows, the stale conversions in brief §12, and the settlement
report's incomplete index were corrected.

**Narrowed, deliberately and in writing:** keyword rules over natural language produced false
negatives in rounds 4–5 and then false positives on this plan's own disclosure text in round 6
(four `mixed-case` hits on correction prose). The instrument no longer attempts to police bare prose
figures or generic wording variants, and its header now states that a PASS means *the mechanical
and arithmetic checks pass*, never *every prose claim is true*. Gate-claim scanning is best-effort
over unrun gates only, and the adversarial round records are excluded from it by design because
they quote attacks verbatim.

## What round 5 could not verify

Whether the artifacts were rewritten rather than appended (all T00 artifacts are untracked, so no
diff history exists); that the recorded guard-test outputs came from the inputs they imply (no
committed fixtures — disclosed debt); whether the teeth "re-run" logs are genuine re-runs rather
than byte-identical copies (the scripts are deterministic); heavy gate re-execution (raw logs and
time blocks were inspected); Gauntlet's three gates (still RESOURCE_DEFERRED, not claimed); and the
secrecy of the matched values (deliberately unread).
