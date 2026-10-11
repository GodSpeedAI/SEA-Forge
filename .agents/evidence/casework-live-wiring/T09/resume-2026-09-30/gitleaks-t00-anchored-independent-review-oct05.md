# Independent review: anchored T00 Gitleaks scope and history scan

Date: 2026-10-05. Verdict: **APPROVE the frozen anchored exception's tested scope and the completed actual-config history scan.** Gitleaks version: 8.30.1.

## Source and scope review

Reviewed the original sibling-commit assignment/provenance, the independent selector rejection, the targetRules correction record, and the six-path anchor builder record. The frozen repository config SHA-256 is `608a687e53ba365fc743ae6c8d2ded7a1c006e8c6b0f5f209109d055f6e5ad05`. The exact local diff is independently recorded in `gitleaks-t00-path-anchor-builder-oct05.md`: six path regexes add only `^` and `$`; the interiors are unchanged. The T00 entry retains `condition = "AND"`, the two reviewed commit IDs, `targetRules = ["generic-api-key"]`, all six paths, and all other config remains unchanged. The earlier `rules` selector was invalid for pinned 8.30.1; current source uses its supported `targetRules` selector.

## Synthetic controls

Two disposable `/tmp` Git repositories/configs were used. First attempt `20261005T231801Z` joined exit 0 but its no-T00 baseline detected no candidate; it is archived as **INCONCLUSIVE** and does not support approval. The former marker could not be recovered because prior captured reports intentionally redacted Secret/Match values. The corrected distinct matrix uses one fresh synthetic generic marker line, byte-identical at each scenario; its SHA-256 (without the marker value) is archived in `gitleaks-t00-anchored-v2-marker-line.sha256`. The no-T00 scan established detection before any T00 scope conclusions.

The no-T00 baseline (exit 1, 21 findings: 19 `generic-api-key`, 2 `github-pat`) detected the same generic marker at all six exact paths for both synthetic allowlisted commits, plus prefix lookalike, suffix lookalike, unlisted path, and the exact path under an unlisted commit. The separate `github-pat` control was detected at the same exact path under both allowlisted commits. All 21 expected positive scenarios passed.

The anchored T00 synthetic scan (exit 1, exactly 9 findings) produced precisely the expected residual findings:
* All 12 generic findings across two allowed commits × six exact paths were suppressed. This proves each commit-array and path-array alternative works while the AND condition combines criterion classes.
* Prefix and suffix lookalike paths and the unlisted path were reported under both allowed commits.
* The exact T18 path was reported under the unlisted commit.
* `github-pat` remained reported at the exact allowed path under both allowlisted commits.
* No unexpected findings occurred.

The no-T00 and T00 reports include raw synthetic match fields, so those reports remain in `/tmp` and were not copied here. Their archived redacted reports retain only rule, synthetic commit, path, and line metadata. Gitleaks CLI output was captured with `--redact`; preflights and actual exit files were copied by native patch and byte-compared.

## Commands and exact evidence

The two synthetic scans used:

```text
gitleaks git --config <no-t00-synthetic.toml> --report-format json --report-path <private-report.json> --redact --no-color --log-level info <synthetic-repo>
gitleaks git --config <production-synthetic.toml> --report-format json --report-path <private-report.json> --redact --no-color --log-level info <synthetic-repo>
```

The actual history scan used the repository's exact frozen config:

```text
gitleaks git --config "$PWD/.gitleaks.toml" --report-format json --report-path <private-report.json> --redact --no-color --log-level info "$PWD"
```

It joined exit 0: 504 commits, about 80.37 MB, 2m00s, no leaks; its report JSON is empty. The immediately preceding preflight records config SHA and Gitleaks version. All history captures are archived as `gitleaks-history-608a-20261005T232352Z-{preflight.raw,cli.raw,report.json,exit}` and cmp-verified against the `/tmp` originals.

Synthetic evidence, redacted reports, configs, setup identities, matrix summary, and exact CLI/preflight/exit captures are archived under the same directory with `gitleaks-t00-anchored-v2-` prefixes. The initial inconclusive attempt is separately archived as `gitleaks-t00-anchored-first-20261005T231801Z-*`.

Each preflight captured `/proc/meminfo` and unfiltered visible `ps -eo pid,comm,rss`. The tool process namespace exposed only codex/bash/ps; that is not proof of host-wide process absence. No cgroup memory max/current file was readable. Heavy commands were serialized by the transferred exclusive verifier token. No scanner, compiler, source/config implementation, Git history, hook, status, or debt change was made by this reviewer. No broader test or publish gate is claimed.
