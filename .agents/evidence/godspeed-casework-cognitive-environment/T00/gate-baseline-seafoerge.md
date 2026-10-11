# T00 evidence — existing SEA Forge gate baseline

Revision `6ce518fcd9b01bc5a7037f80f5d8986a33cd2924`, dedicated worktree
`/home/sprime01/projects/sea-rs/.worktrees/godspeed-casework-cognitive-environment`,
`CARGO_BUILD_JOBS=1`, one heavy gate at a time, raw logs in `raw-logs/`.

Reproduction environment note (affects any later re-run): each gate ran with
`CARGO_TARGET_DIR=/home/sprime01/projects/sea-rs/target` — the **operator checkout's warm
cache** — so the baseline reflects the frozen revision's behaviour without a multi-hour cold
rebuild. This writes build artifacts only (no tracked or untracked source file in the active
checkout); the worktree additionally carries `target →` that cache as a symlink so
recipes that reference the relative path `target/...` still resolve.

| Gate (verbatim command) | Exit | Wall | Peak RSS | Verdict | Raw log |
|---|---|---|---|---|---|
| `just lint` | 0 | 52 s | — | PASS | `raw-logs/gate-lint.log` |
| `just typecheck` | 0 | 36 s | — | PASS | `raw-logs/gate-typecheck.log` |
| `just test` | 0 | 290 s | 0.99 GiB (1036928 kB) | PASS | `raw-logs/gate-test.log` |
| `just proof` (first attempt) | **127** | 25 s | 0.53 GiB (556952 kB) | harness defect — see below | `raw-logs/gate-proof.log` |
| `just proof` (spec-minimum §12.2 P1–P4b) | **0** | **3 s** (round-2 re-run under `/usr/bin/time -v`; the round-0 "2 s" observation had no time block and is retained only as a disclosed defect) | 72784 kB | PASS (`[proof] P1-P4b passed`) | `raw-logs/round2-proof.log` |
| `just security` | **1** | 52 s | — | **RED** — `cargo deny` green, `gitleaks` 14 findings | `raw-logs/gate-security.log`, `raw-logs/gitleaks.json` |
| `just build` | 0 | 70 s | — | PASS | `raw-logs/gate-build.log` |

Composite gates (`just check`, `just ci`) are recorded in `gate-baseline-composites.md`,
because they include `just context-check`, whose freshness rule only passes once this task's
handoff files are updated.

## The `just proof` exit-127 attempt, preserved

The first attempt failed with:

```
[proof] running spec-minimum §12.2 P1-P4b
/run/user/1000/just/just-DVfGgy/proof: line 917: target/debug/sea-forge: No such file or directory
error: recipe `proof` failed with exit code 127
```

Cause classified: **oracle/measurement harness defect, not a gate failure.** The recipe runs
`cargo build -q -p sea-forge-cli` and then executes the *relative* path
`target/debug/sea-forge`; with `CARGO_TARGET_DIR` overridden to the operator's cache the
binary was written there while the recipe looked inside the worktree. Fix applied: create the
worktree `target` symlink (no recipe edit — the gate is not weakened or modified). Re-run
result: exit 0, `[proof] P1-P4b passed`. Both attempts are retained, per the plan's
"preserve failed evidence and correction history".

Lesson recorded for later tasks: **a worktree that overrides `CARGO_TARGET_DIR` must
still expose `target/debug/sea-forge` relative to the worktree root, or `just proof` and
`just workbench-sidecar`-style recipes will fail for path reasons that look like gate
failures.**

## `just security` red baseline (GATE_SEAFORGE precondition), exact

`cargo deny check advisories bans licenses sources` → `advisories ok, bans ok, licenses ok,
sources ok` (green). `gitleaks detect --no-banner --redact` → 486 commits scanned,
214.61 MB, **14 leaks found**, exit 1.

Independent re-run with a JSON report (`raw-logs/gitleaks.json`) — 14/14 findings, all rule
`generic-api-key`, all in commit `6ce518fcd9`, in six files under
`.agents/evidence/godspeed-bounded-judgment/`: `T18/observations.jsonl` (6),
`T27/observations.jsonl` (4), `repeatability/observations.jsonl` (1),
`T00/gate-SEA_CHECK-fmt-drift.txt` (1), `T00/gate-SEA_CHECK.log` (1), `T00/gate-SEA_CI.log` (1).

### Corrected measurement (round 2) — replaces a withdrawn round-0 analysis

**Disclosure: the round-0 characterization measured gitleaks' redaction placeholder, and is
withdrawn.** Round 0 read the `Secret` field of the `--redact` report and reported "8 characters,
mixed-case alphabetic, entropy 3.55–3.75" for all 14 findings, concluding they were "word-like,
not credential-shaped". `gitleaks --redact` replaces every `Secret` with the literal string
`REDACTED` — exactly 8 mixed-case alphabetic characters — so the columns measured the placeholder,
not the findings. The same report's column offsets disagreed (23 and 32 characters) and that
internal contradiction was missed. An independent verifier (round-1 confirmation, recorded in
`T00-verification-round-1-NOT_CONFIRM.md`) falsified the characterization.

Round 0's withdrawn table, preserved verbatim for the record:

| rule | count | file | commit | "secret length" (wrong) | entropy (real) | charset (wrong) |
|---|---|---|---|---|---|---|
| `generic-api-key` | 6 | `…/T18/observations.jsonl` | `6ce518fcd9` | 8 | 3.58–3.75 | mixed-case alphabetic |
| `generic-api-key` | 4 | `…/T27/observations.jsonl` | `6ce518fcd9` | 8 | 3.58–3.62 | mixed-case alphabetic |
| `generic-api-key` | 1 | `…/repeatability/observations.jsonl` | `6ce518fcd9` | 8 | 3.62 | mixed-case alphabetic |
| `generic-api-key` | 1 | `…/T00/gate-SEA_CHECK-fmt-drift.txt` | `6ce518fcd9` | 8 | 3.55 | mixed-case alphabetic |
| `generic-api-key` | 1 | `…/T00/gate-SEA_CHECK.log` | `6ce518fcd9` | 8 | 3.55 | mixed-case alphabetic |
| `generic-api-key` | 1 | `…/T00/gate-SEA_CI.log` | `6ce518fcd9` | 8 | 3.55 | mixed-case alphabetic |

Corrected non-exposing re-measurement — `teeth/remeasure-gitleaks.sh` reads the real bytes at the
recorded column offsets and never prints a matched value (raw output:
`teeth/gitleaks-remeasure.log`):

| property | observed |
|---|---|
| matched span length | **23 characters × 11 findings**, **32 characters × 3 findings** |
| matched field name | `key` (the 11 observation-record findings), `idempotency_key` (the 3 gate-log findings) |
| matched structure | a JSON **field name plus its value** (`key": "<value>"`), not a bare secret |
| value shape | 13–16 characters, lower-case alnum with hyphens (identifier-shaped) |
| entropy | 3.55–3.75 — that column was real, and is low for key material |
| commit / location | all 14 in `6ce518fcd9`, all inside the sibling plan's committed evidence |

**What this establishes:** the flagged text is a JSON field named `key` / `idempotency_key` plus a
short identifier-shaped value inside committed observation records and gate logs — the shape of a
record identifier or idempotency key, not prose and not obviously credential material.

**What it does not establish, and why B2 stays an operator decision:** an identifier-shaped value
cannot be proven non-secret by shape alone from a redacted report, and this plan will not read the
unredacted values (AGENTS.md forbids putting secret material into logs or evidence). The honest
statement is narrower than round 0's: *the findings are identifiers in a named field; confirm the
field semantics before allowlisting anything.* `.agents/AGENTS.md` makes historical evidence
immutable, so the findings cannot be removed by rewriting the records.

The repository already carries a reviewed precedent for this exact class in `.gitleaks.toml`:
a *commit-scoped + path-scoped* allowlist introduced during "T17 gate-remediation
(2026-09-18)" for "test fixtures and redaction sentinels with simulated credentials",
explicitly commit-scoped "so any NEW occurrence in these paths still fails". The round-1
independent verifier read `.gitleaks.toml` itself and confirmed this precedent exists.

**Classification:** pre-existing, inherited, **identifier-class** red gate. **Repair is
withheld pending approval**, because editing `.gitleaks.toml` is a change to the security gate
configuration that AGENTS.md requires asking about first, and because a careless allowlist would
weaken the gate. Proposed narrow repair (not applied, and explicitly conditional on the operator
confirming the field semantics): a new `[[allowlists]]` entry with
`commits = ["6ce518fcd9b01bc5a7037f80f5d8986a33cd2924"]`, paths limited to the six files named
above, `rules = ["generic-api-key"]`, and a `description` recording the round-2 verification in
this file. This is a concrete prerequisite on `GATE_SEAFORGE` (T05), not a resource deferral, and
it is recorded rather than silently greenwashed.