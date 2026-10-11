# Gitleaks push-gate reconnaissance — 2026-10-05

Read-only diagnosis of the reported normal-push secret-scan failure. No
allowlist/config, source, compiler, status/debt, or Git mutation was made.
Neither matched values, `Match`, full source lines, nor unredacted blob bytes
are recorded here.

## Reproduction and redaction

Ran once from the repository root, with the default Git scan and repository
configuration:

```text
gitleaks detect --no-banner --redact --report-format json --report-path /tmp/sea-gitleaks-push-oct05-report.json
```

Stdout/stderr were captured in `/tmp/sea-gitleaks-push-oct05-recon.raw`; the
joined process returned exit 1, saved as `/tmp/sea-gitleaks-push-oct05-recon.exit`.
The redacted JSON report contains 14 findings. This reproduces the normal-push
finding count. The joined scan took about 44 seconds. No `--no-git`, custom
config, ignore path, or bypass option was supplied. `GITLEAKS_CONFIG` and
`GITLEAKS_CONFIG_TOML` were both absent; the active worktree `.gitleaks.toml`
equals the tracked `HEAD` copy.

Native `gitleaks version` and `devbox run gitleaks version` both report
`8.30.1`. The local `detect --help` identifies `--no-git` as an explicit
directory-scan switch; it was not used. The report records commit provenance
for all findings. Gitleaks v8.30.1 documentation describes Git scanning as
history/patch scanning and documents config lookup precedence, the `AND`
allowlist condition, and global allowlists: [v8.30.1 README](https://github.com/gitleaks/gitleaks/blob/v8.30.1/README.md).
The [v8.30.1 allowlist source](https://github.com/gitleaks/gitleaks/blob/v8.30.1/config/allowlist.go)
compares configured commit IDs to finding commit IDs case-insensitively; it
does not treat sibling commits as equivalent.

Before writing this note, a local metadata-only check extracted the 14 source
field values in memory from their report commit/path/line and confirmed none
occur in either the redacted JSON report or captured CLI raw output. Those
values were never emitted. Capture SHA-256 values:

* raw: `847038f1eeda549613368636afe13520d5f8523e2c35e517d3b878fa15e179b3`
* exit: `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`
* redacted JSON report: `c476d67ce7b5b89896bd83728b06b79b94dafc1d5e1640e15a9ea3f489027e3f`

## Safe finding inventory

All 14 findings have rule `generic-api-key`, commit
`44b5a0b36399fffbd685de7d93bda1933ed75c96`, and the fingerprint is the exact
`commit:path:rule:line` value shown below. No finding value is reproduced.

| File | Line | Fingerprint |
| --- | ---: | --- |
| `.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK.log` | 590 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK.log:generic-api-key:590` |
| `.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK-fmt-drift.txt` | 587 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK-fmt-drift.txt:generic-api-key:587` |
| `.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CI.log` | 590 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CI.log:generic-api-key:590` |
| `.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl` | 23 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl:generic-api-key:23` |
| `.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl` | 26 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl:generic-api-key:26` |
| `.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl` | 49 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl:generic-api-key:49` |
| `.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl` | 81 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl:generic-api-key:81` |
| `.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl` | 99 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl:generic-api-key:99` |
| `.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl` | 150 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T18/observations.jsonl:generic-api-key:150` |
| `.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl` | 39 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl:generic-api-key:39` |
| `.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl` | 96 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl:generic-api-key:96` |
| `.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl` | 125 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl:generic-api-key:125` |
| `.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl` | 129 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl:generic-api-key:129` |
| `.agents/evidence/godspeed-bounded-judgment/repeatability/observations.jsonl` | 116 | `44b5a0b36399fffbd685de7d93bda1933ed75c96:.agents/evidence/godspeed-bounded-judgment/repeatability/observations.jsonl:generic-api-key:116` |

At the exact report columns, source-field metadata computed locally is:

| Findings | Source field | Length and character class | Entropy |
| --- | --- | --- | ---: |
| T00 gate logs, lines 587/590 (3) | `idempotency_key` | 13; lowercase ASCII letters, digits, hyphen; all contain digits | 3.546594 each |
| T18 lines 23, 26, 49, 81, 99, 150 (6) | `key` | 16; lowercase ASCII letters and digits; all contain digits | 3.750000, 3.625000, 3.625000, 3.577820, 3.625000, 3.750000, respectively |
| T27 lines 39, 96, 125, 129 (4) | `key` | 16; lowercase ASCII letters and digits; all contain digits | 3.577820, 3.577820, 3.625000, 3.577820, respectively |
| repeatability line 116 (1) | `key` | 16; lowercase ASCII letters and digits; contains digits | 3.625000 |

This independently reproduces the safe metadata described by the cited T00
baseline: the observation-record property is `key`, the gate-log property is
`idempotency_key`, and observed identifier lengths/entropy are in the recorded
range. The baseline also documents that an earlier round misread the redacted
`Secret` placeholder as the source value; its corrected round measured source
bytes at the report columns without printing values. The current metadata
check agrees. These are identifier-shaped occurrences in the sibling plan's
committed T00/T18/T27/repeatability evidence, not an occurrence in runtime
source. Character shape alone would not prove a value is nonsecret; this
classification is supported by the prior T00 adversarial review records and
the source-field/provenance check, not entropy alone.

## Exact cause of the gate failure

The active `.gitleaks.toml:50-77` contains the previously approved T00 global
allowlist. It specifies `condition = "AND"`, exact commit
`6ce518fcd9b01bc5a7037f80f5d8986a33cd2924`, rule `generic-api-key`, and the six
paths containing these findings. The observed paths and rule match those
criteria, but their finding commit is `44b5a0b36399fffbd685de7d93bda1933ed75c96`.
With `AND`, the configured commit criterion does not match, so this allowlist
does not suppress them.

History inspection establishes this is a sibling-commit identity mismatch,
not a changed value or expanded path/rule scope:

* `44b5a0b` and `6ce518f` have the same parent,
  `72348237412dc000533c05b6a36e01066ca3baed`, but are separate commit objects.
* `44b5a0b` is an ancestor of current `HEAD` (`8c6495d24c668d81587e42b5f690db90237c07e3`); `6ce518f` is not.
* The six affected paths are absent from their common parent. Every reported
  finding line is byte-identical at the same path and line in the allowlisted
  `6ce518f` tree. Thus the current history contains the same identifier-bearing
  evidence introduced under sibling commit `44b5a0b`, while the narrow
  commit-scoped allowlist names only `6ce518f`.

Conclusion: the push gate is correctly finding a historical occurrence on the
active branch. The immediate reason all 14 recur is the exact-commit allowlist
does not cover the sibling commit that is in current history. The known
identifier classification is corroborated; whether/how to adjust policy is
outside this read-only assignment and remains for root/operator decision. No
allowlist was changed.

## Evidence and limits

The cited local evidence is `.agents/evidence/godspeed-casework-cognitive-environment/T00/gate-baseline-seafoerge.md`
(especially its corrected source-column analysis) and
`T00-verification-round-1..6` records, plus the existing `.gitleaks.toml`
comment and exact allowlist. I reviewed the relevant evidence with token-like
sequences suppressed from output, then independently recomputed names, length,
character class, and entropy from the exact historical source columns. No
source value was printed or copied into an artifact.

Graft was attempted first, but its index did not cover the repository root
config/evidence; I used direct source/config/evidence inspection as documented
by the repository Graft skill's fallback. No compiler or tests were run.
