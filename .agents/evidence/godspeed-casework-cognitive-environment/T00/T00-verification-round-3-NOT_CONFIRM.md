# T00 verification round 3 — verdict NOT_CONFIRM (preserved)

- **Round:** 3 (third independent adversarial confirmation)
- **Date:** 2026-09-19
- **Verifier:** a fresh independent subagent, read-only, instructed to falsify the round-3 claims
  and to hunt for defects the earlier rounds never examined.
- **Verdict:** **NOT_CONFIRM**. Claim 1 (no surviving assertion of the withdrawn gitleaks wording)
  and Claim 2 (the `proof` wall time is now backed) **held**. Claim 4, Claim 5 and part of Claim 3
  failed, and the verifier also found unit-mislabel defects in the measurement layer that rounds 1
  and 2 had never examined.
- **Disposition:** round-4 corrections applied (below). This round is preserved, not rewritten.

## What held

- **Claim 1 held.** A broad search over this plan's artifacts found every occurrence of the
  withdrawn wording only as an explicit disclosure, with the plan's own handoff region containing
  exactly one hit — the withdrawal sentence. The verifier also checked the reverse direction (is a
  "benign/false positive" conclusion still asserted anywhere as current?) and found only *inherited*
  bounded-judgment history about unrelated fixtures, correctly fenced by the scope note.
- **Claim 2 held.** `raw-logs/round2-proof.log` really contains a `/usr/bin/time -v` block with
  `Elapsed … 0:03.17`, max RSS `72784`, `Exit status: 0`, and the recipe output `[proof] P1-P4b
  passed`; the `justfile` recipe is unmodified; and the round-0 "2 s" figure is only labelled as an
  unbacked defect.

## What failed

| # | Defect | Classification | Round-4 correction |
|---|---|---|---|
| R3-F1 | **Claim 4 failed, and the round-3 disclosure made it worse.** The preregistration amendment asserted that no command ran before the freeze, which the evidence cannot establish; and the round-3 *edit itself overwrote the mtime* the disclosure was about, destroying the only trace of the 36-second gap. | **representation** (unfalsifiable claim; evidence destroyed by the fix) | The amendment was narrowed to exactly what is establishable (the six measurements/claim/falsifier/confounds/deferral rule/confirmation condition are semantically unchanged, and the first gate started after the write) and now carries an explicit `not_established` list: that no non-gate command ran before the freeze, and that the 17:58:06Z write had exactly the recorded content. A lesson was recorded: freeze-time hash pinning, not just a declared timestamp. |
| R3-F2 | **Claim 5 failed.** The two canonical handoff files disagreed about which round was current: `CURRENT_STATUS.md` said re-confirmation round 2 was outstanding while `current_status.yml` said round 3. | **representation** (cross-artifact inconsistency) | Both handoff files are aligned on the same round and the same next action, and this class of drift is now checked mechanically (a `grep` for the round token across all handoff files is part of the round-4 self-audit). |
| R3-F3 | **Claim 3 was narrower than presented.** The cited raw log could not show the textual claims "values are lower alnum with hyphens" and "all 14 contain digits", because the script masked alphanumeric runs and reported the *span* class, not the *value* class. The substance was true and the verifier reproduced it, but the attribution was wrong. | **representation** (oracle measures the wrong object) | The script was rewritten to measure the **value** (the quoted string after the matched field name) and to print its length, charset class, digit presence and hyphen presence from the real bytes. Its log now directly backs every textual claim: `value classes: {'lower-alnum-hyphen': 14}`, `value lengths: {13: 3, 16: 11}`, `digit: True` ×14, `hyphen: True` ×3. |
| R3-F4 | Adversarial holes in the script: columns pointing at whitespace printed a nonsense characterisation and exited 0; a span shorter than the value was silently truncated; an unreadable file crashed with `IndexError` on a mis-sized tuple. | **oracle/test** | The script was rewritten: unresolvable values, whitespace-only spans, unreadable files and empty reports all refuse with `exit 2` (all four verified). The old crash path is gone. |
| R3-F5 | The script's default report path pointed outside the evidence directory (`/tmp`), making the cited log non-reproducible later. | **representation** (non-durable citation) | The default now resolves to the in-repo `raw-logs/gitleaks.json`, which the log records verbatim. |
| R3-F6 | Unit mislabels throughout the memory figures: `MemTotal` "8.13 GiB" (actually GB; 7.76 GiB), `just test` RSS "1.04 GiB" (actually 0.99 GiB), `just proof` "0.56 GiB" (0.53 GiB), `MemAvailable` range "3.39–3.83 GiB" (3.24–3.65 GiB), swap "7.3–7.8 GiB" (7.05–7.47 GiB), Gauntlet headroom "≈3.8 GiB" (3.69 GiB). | **representation** (unit convention) | Every memory figure in the brief, `gate-baseline-seafoerge.md`, `gate-baseline-gauntlet.md` and `resource-preflight.md` was recomputed from the raw kB values with 1 GiB = 1048576 kB and re-stated alongside its kB source; the convention is now declared explicitly in both the brief and the evidence. |
| R3-F7 | `T00-settlement-report.md` claimed the prereg was frozen before any "gate baseline or inventory command"; only the gate ordering is evidenced. | **representation** | Narrowed to the gate-baseline ordering claim that the timestamps actually establish. |
| R3-F8 | The round-2 record's structure was damaged by an earlier edit (a stray table row below the "could not verify" heading; a disposition line promising content "below" that was not there). | **representation** (edit damage) | The round-2 record was rewritten with the same substance and correct structure. |

## What round 3 could not verify

Heavy gate re-execution (raw logs and time blocks were inspected instead); Gauntlet's three gates
(still RESOURCE_DEFERRED); the secrecy of the matched values (deliberately unread); whether
`teeth-rerun-round3.log` is a genuine re-run rather than a byte-identical copy of the round-2 log
(the script is deterministic, so both explanations fit); and the earlier verifiers' internal
processes.
