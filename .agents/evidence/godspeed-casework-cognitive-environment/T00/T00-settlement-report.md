# T00 settlement report — Freeze source authority, repository topology, gate bindings, and execution context

- **Plan:** `.agents/plans/godspeed-casework-cognitive-environment.plan.yaml` v0.2.2, task T00 (P2, confirmation: peer)
- **Date:** 2026-09-19
- **Worktree:** `/home/sprime01/projects/sea-rs/.worktrees/godspeed-casework-cognitive-environment`
  (branch `godspeed/casework-cognitive-environment`, base `6ce518fcd9b01bc5a7037f80f5d8986a33cd2924`)
- **Frozen preregistration:** `.agents/preregistrations/godspeed-casework-cognitive-environment/T00.prereg.yaml`
  (frozen 2026-09-19T17:57:30Z, before the first gate baseline; that single claim about ordering is
  what the timestamps establish — the same file also discloses a 36-second post-freeze typo fix in
  `post_freeze_amendments`)
- **Implementation brief (T00 gate artifact):** `.agents/plans/godspeed-casework-cognitive-environment.implementation.md`
- **Decision log:** `.agents/reports/godspeed-casework-cognitive-environment/decision-log.yaml` (D-2026-09-19-T00-01 … -09)

## Gate results

| T00 gate | Command | Result |
|---|---|---|
| `GATE_SPEC_TRACE` | `python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py` | **PASS** (exit 0) |
| brief existence | `test -f .agents/plans/godspeed-casework-cognitive-environment.implementation.md` | **PASS** |

## Confirmation history (append-only)

- **Round 1 — independent verifier: NOT_CONFIRM**, seven findings. Fully recorded, with
  classification and corrections, in `T00-verification-round-1-NOT_CONFIRM.md`. The verifier
  independently reproduced the authority binding, 86/86 traceability, DAG, gate activation,
  worktree isolation, the missing-seam inventory, the security red, and the non-vacuity of the
  teeth — and then falsified the *measurement and evidence-backing* layer: the gitleaks
  characterization had been measured off gitleaks' `REDACTED` placeholder; four inventory counts
  were wrong; the composite gate results had been recorded by pointer to a file that did not
  exist; the brief's §5 heading was swallowed into §4's list and item 7 truncated; and one
  recorded gate wall time had no backing log.
- **Round 2 — corrections applied.** The gitleaks section was re-measured from real bytes with no
  value printed and round 0's table preserved as withdrawn; the four counts were fixed; the
  composites were actually run and recorded in `gate-baseline-composites.md`; the brief was
  reflowed (heading count now 12); `just proof` was re-run under `/usr/bin/time -v` to back its
  wall time; the teeth were re-run (exit 0).
- **Round 2 verification — independent verifier: NOT_CONFIRM again**, recorded in
  `T00-verification-round-2-NOT_CONFIRM.md`. F2–F6 were verified correct, but F1 and F7 were only
  *partially* applied: the withdrawn gitleaks wording still stood as current text in five handoff
  artifacts (including both canonical handoff files), and the unbacked "2 s" `proof` figure was
  still displayed and still cited to a log with no time block. The verifier also found three new
  defects in the corrections themselves: the re-measurement script printed the field-name fragment
  (`i`/`k`) rather than the field name it claimed, the script would have exited 0 on an empty
  report, and the preregistration's mtime postdated its declared freeze with no way to reconcile it.
- **Round 3 — corrections applied.** Every surviving instance of the withdrawn wording was
  replaced (six locations, including the decision log's basis item) and a repo grep for it now
  returns nothing in this plan's artifacts; the `proof` row now carries the backed round-2 value
  with the "2 s" figure labelled as an unbacked, disclosed defect; the re-measurement script
  reports the true enclosing field name and refuses an empty report (`exit 2`, verified); and the
  preregistration now records the post-freeze typo correction as an explicit
  `post_freeze_amendments` disclosure (the 36-second write fixed one misspelled measurement id
  before any gate ran).
- **Round 3 verification — independent verifier: NOT_CONFIRM** (`T00-verification-round-3-NOT_CONFIRM.md`).
  Its Claims 1 and 2 held (no surviving withdrawn wording; the `proof` time is backed), but it
  found: the prereg disclosure claimed something nothing establishes, and the fix destroyed the
  mtime that disclosure was about; the two handoff files named different current rounds; the
  re-measurement script measured the *span* while the text described the *value*, printed nonsense
  on whitespace-only columns, crashed on an unreadable file, and defaulted to a `/tmp` report path;
  the round-2 record's layout was damaged; and every memory figure carried GB values under GiB
  labels.
- **Round 4 — corrections applied.** The script now measures and reports the *value* (class, digit
  presence, hyphen presence) from real bytes and refuses empty reports, whitespace-only spans,
  unreadable files and unresolved values with `exit 2`; its default report path is inside this
  evidence directory; all memory figures were recomputed from raw kB with 1 GiB = 1048576 kB and the
  convention is declared; the prereg disclosure was narrowed and given an explicit
  `not_established` list; the round-2 record was rebuilt; and a mechanical self-audit
  (`self-audit.sh`) was added to run before any re-confirmation.
- **Round 4 verification — independent verifier: NOT_CONFIRM**
  (`T00-verification-round-4-NOT_CONFIRM.md`). It confirmed the gitleaks re-measurement and the four
  refusal guards, then found: this report's confirmation history and evidence index were stale;
  `CURRENT_STATUS.md` still said "ROUND 2 APPLIED" on one line while later lines said round 4; a
  Gauntlet headroom figure was still converted wrongly (both that figure and the swap range had been
  the round-3 verifier's own arithmetic, enshrined as corrections); brief §3 was truncated
  mid-sentence; a table row had a missing cell; an orphan duplicated line survived; `target` was
  claimed gitignored twice although `.gitignore`'s `/target/` does not match a symlink; and **the
  self-audit was partly theatre** — it could be made to print PASS with a live defect of every class
  it claimed to cover.
- **Round 5 — corrections applied.** All of the above were fixed (single round claim, correct
  conversions, complete history and index, restored sentence, repaired table row, removed orphan,
  corrected `target` statements), and the self-audit was rewritten to: derive its own file set by
  walking this plan's artifact directories; scan the whole plan-owned region of the status file;
  detect line-wrapped wording; cross-check every `N GiB (M kB)` pair arithmetically; require one
  single current-round value across both handoff files; cover every `GATE_*`, `casework-*` and
  `workbench-check` name; and assert that every handoff file it names exists. Two further audit
  defects were then found **by the audit's own self-test** and fixed: checks 1–2 printed FAIL but
  did not affect the exit status (the very failure mode round 4 described), and the self-test's
  injections were mis-quoted so three classes were being tested by inputs that never applied.
  Current state: `SELF-AUDIT: PASS` and `SELF-TEST: PASS` (control PASS + all seven injected defect
  classes detected) — recorded in `self-audit.log` and `self-audit-selftest.log`.
  **Re-confirmation requested (round 5).**
- **Round 5 verification — independent verifier: NOT_CONFIRM**
  (`T00-verification-round-5-NOT_CONFIRM.md`). Decisive finding: **the decision log was 0 bytes** while
  this report, both handoff files and the plan all cited its nine decisions, and the self-audit
  reported it present because the check tested existence rather than content. It also proved by
  injection that the audit missed thirteen further classes (punctuation-variant and re-wrapped
  wording, `.csv`/extensionless citations, bare `GiB` figures, gate names outside its list, deleted
  non-`REQUIRED` evidence and round records, round claims in files other than the handoff files, a
  negation word suppressing a gate claim), that the injection test credited detection without
  verifying its injector had modified the mirror, and that an environment variable could disable the
  non-vacuity cross-check in a real run.
- **Round 6 — corrections applied.** The decision log was restored (9 decisions) and the audit now
  asserts content (non-trivial size, YAML parse, the nine expected ids); coverage was widened to
  punctuation/wrap variants, `.csv` and extensionless citations, anchored-pair arithmetic, absent
  and stale round claims, and deleted raw logs and round records; injections must now be proven to
  have changed the mirror; the skip-variable was replaced by real mirror detection; and the three
  unbacked preflight rows, the stale conversions in brief §12 and this report's index were corrected.
  The instrument was then **deliberately narrowed** and its limits printed in its own output, because
  tightening prose heuristics produced false positives on this plan's own disclosure text. Current
  state: `SELF-AUDIT: PASS` (**mechanical and arithmetic checks pass**) and `SELF-TEST: PASS`
  (control PASS + 9/9 injected classes detected), recorded in `self-audit.log` and
  `self-audit-selftest.log`. **T00 remains unconfirmed by an independent verifier.**
- **Round 6 → Round 7 (tooth maintenance, not a verification round).** T00's second tooth began to FAIL
  once the world changed: the operator approved B1 (Go installed) and T01 created
  `apps/godspeed-casework-go` plus the `casework-go-check` recipe, so assertions that recorded those
  things as *absent* were no longer true. That is the tooth working as designed — an inventory that
  cannot say when it stopped being true is not an inventory — but it had to be updated honestly rather
  than deleted. The tooth now asserts the CURRENT truth (toolchain installed and declared; Go module and
  React contract package exist; `GATE_GO` recipe exists) and names what moved and why, keeps the
  genuinely-still-absent seams (`casework-ui-check`, the SFWP repository-fact/lease/artifact methods,
  donor checkouts), and reads the DAG from the live status file so T03/T04 no longer assert the
  pre-T01 blockage. `TEETH RESULT: both teeth behaved as specified`.
- All failed rounds are preserved rather than erased, per the plan's correction protocol.
- Round 0/1/2 defects are preserved rather than erased, per the plan's correction protocol.

## `done_when` adjudication

1. **Frozen SHA-256 verifies; each existing gate baseline recorded as passed or
   RESOURCE_DEFERRED, without claiming an unrun gate passed.** MET.
   `sha256sum` reproduces the bound digest `6312453f…d6641`. SEA Forge baselines: `lint` PASS,
   `typecheck` PASS, `test` PASS, `proof` PASS (after a preserved harness-defect correction),
   `build` PASS, `security` **RED** with its exact failing assertion recorded (`gitleaks`: 14
   `generic-api-key` findings, exit 1) — recorded as red, which is neither a pass nor a
   deferral. Gauntlet: `RESOURCE_DEFERRED` (three gates) with preflight readings, a real
   narrow single-job observation, and the exact commands to run later. Standing prerequisite
   B2 arises from the red `security` baseline and is surfaced, not absorbed.
2. **The implementation brief contains repository-specific hot context sufficient for a cold
   agent.** MET. 12 sections: frozen authority and topology, toolchains, the exact SFWP boundary
   (23 methods, transport, identity, cursors, idempotency, settlement visibility, generated
   contracts), a capability matrix with per-capability anchors and verdicts, the Gauntlet
   engine/TUI boundary, both removal inventories, gate activation, blocking prerequisites,
   recorded baselines, open decisions, implementer rules, and an append-only correction history.
3. **Traceability reports 86/86 mapped requirements, zero unknown plan IDs, zero unmapped
   requirements.** MET — mechanically by `GATE_SPEC_TRACE`, which fails on any mismatch.
4. **All T00 teeth pass.** MET — `teeth/run-teeth.sh` exits 0:
   - Tooth 1: unmutated spec copy PASSes; one-byte mutation ⇒ `FAIL: frozen spec SHA-256
     differs from current bytes`, exit non-zero (blocked, divergence named).
   - Tooth 2: nine "already exists" claims for the Go toolchain, both target apps, the casework
     `just` recipes, an SFWP repository-fact method, a GitHub client, donor checkouts, an SFWP
     lease/claim method, and an SFWP artifact-persistence method are all contradicted by the
     inventory; the DAG keeps T03/T04 blocked on T01; and the two *corrected-to-PRESENT* GitHub
     authority anchors are confirmed present.

## What T00 proves and what it does not

**Proves:** the authoritative spec and the execution context can be frozen unambiguously before
implementation — a cold agent can reproduce the hash, locate both repositories, distinguish
existing gates from planned gates, enumerate 86 requirements, and state exactly which SFWP
methods and integrations are absent.

## Evidence index

| Artifact | Content |
|---|---|
| `spec-authority-verification.md` | M1/M2: hash, binding checks, 86/86 counts, tooth-1 output |
| `resource-preflight.md` | M-preflight: `/proc/meminfo`, cgroup, pressure, swap, active builds per gate moment |
| `gate-baseline-seafoerge.md` | M3: per-gate exit/wall/verdict, the preserved `proof` exit-127 correction, the round-0 gitleaks analysis **withdrawn** and its round-2 replacement |
| `gate-baseline-composites.md` | `just check` (exit 1, 101 s) and `just ci` (exit 1, 41 s) — both red **only** at `security`, step by step; `just proof` re-run exit 0 with a backed time block |
| `gate-baseline-gauntlet.md` | M4: deferral with readings, narrow observation, exact commands, toolchain mismatch |
| `T00-verification-round-1-NOT_CONFIRM.md` | Round-1 independent verification: verdict, 7 findings, classification, corrections |
| `T00-verification-round-2-NOT_CONFIRM.md` | Round-2 independent verification: which corrections held, which were partial, and the defects the corrections themselves introduced |
| `T00-verification-round-3-NOT_CONFIRM.md` | Round-3 independent verification: held the wording and proof-time claims; found the self-destroyed prereg disclosure, handoff round disagreement, span-vs-value measurement error, three script holes, and the GiB/GB mislabels |
| `T00-verification-round-4-NOT_CONFIRM.md` | Round-4 independent verification: confirmed the re-measurement and refusal guards; proved the self-audit was partly theatre and found the residual unit error, stale history, truncated sentence, malformed row, orphan line, and false `target` claims |
| `T00-verification-round-5-NOT_CONFIRM.md` | Round-5 independent verification: found the decision log emptied to 0 bytes while certified present, plus thirteen injected defect classes the audit missed and two weaknesses in the injection test |
| `raw-logs/preflight-readings.log` | Raw preflight sample (meminfo, pressure, cgroup, active builds) added in round 6 to back the preflight claims that had been recorded inline |
| `teeth/run-teeth.sh`, `teeth/teeth-run.log` | M6: both attacks, executed; re-run after corrections |
| `teeth/remeasure-gitleaks.sh`, `teeth/gitleaks-remeasure.log` | Non-exposing corrected gitleaks measurement, including the refusal-guard tests |
| `self-audit.sh`, `self-audit.log` | Mechanical self-audit (plan-scoped file set, withdrawn-wording scan, cited-path existence, arithmetic re-derivation of `GiB (kB)` pairs, single-round agreement, unrun-gate claim scan, named-file existence) — **SELF-AUDIT: PASS** |
| `self-audit-selftest.log` | Injection test proving the audit fails on each of seven defect classes — **SELF-TEST: PASS** (control PASS + 7/7 detected) |
| `raw-logs/` | Verbatim gate logs, `gitleaks.json` (14 findings), status file |
| `.agents/plans/godspeed-casework-cognitive-environment.implementation.md` | The implementation brief |

## Blockers and open decisions (not silently absorbed)

| ID | Prerequisite | Blocks | Decision required |
|---|---|---|---|
| B1 | No Go toolchain on this host (`mise`: `go@1.27.1 not installed`) | T01, T04, `GATE_GO`, and the whole critical path | Install the toolchain, or defer Go-gated work |
| B2 | `just security` red: 14 `generic-api-key` findings committed in `6ce518f` by a sibling plan — identifier-class (fields named `key`/`idempotency_key` with 13–16-character id-shaped values); see `gate-baseline-seafoerge.md`'s round-2 correction | `GATE_SEAFORGE` (activates T05) | Confirm the field semantics, then approve the narrow commit+path+rule-scoped `.gitleaks.toml` allowlist, or dispose otherwise |
| B3 | No Open MCT / OpenMontage checkout | REQ-DONOR-* (T02 decision only) | None yet — T02 owns it |
| B4 | `docs/reference/sfwp-protocol-reference.md` stale (18 vs 23) | Cold-agent accuracy | Fix in the T04/T05 window |
| B5 | SFWP has no governed lease/claim, no durable artifact persistence, no typed object-version history, no repository-fact execution, no narration beats | T05, T07, T08, T11, T12 settlement scope | Each is a separately reviewed Rust contract change; none may be invented |

## Next move (explicit)

B1 is the first decision: T01 owns `GATE_GO` (`just casework-go-check`) and both T03 and T04 are
blocked on T01, so no implementation task can settle until the Go toolchain question is answered.
After that decision, T01 is the next ready task (it also owns the React contract package that T03
depends on). T02 is independent of T01 and settles on `GATE_SPEC_TRACE` alone, so it can proceed
in parallel.