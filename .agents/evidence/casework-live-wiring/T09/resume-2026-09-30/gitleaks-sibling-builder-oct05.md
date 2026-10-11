# Gitleaks sibling-commit allowlist builder record — 2026-10-05

## Scope and provenance

This source-only change implements the root-directed repair for the failed
normal push gate. It retains the operator-approved T00 allowlist's `AND`
condition, `generic-api-key` rule, and all six exact paths, and adds only the
reported sibling commit identity `44b5a0b36399fffbd685de7d93bda1933ed75c96`
beside the existing `6ce518fcd9b01bc5a7037f80f5d8986a33cd2924` identity.

The original T00 provenance is documented in
`.agents/evidence/godspeed-casework-cognitive-environment/T00/gate-baseline-seafoerge.md`
and its six-round adversarial review; round 6 is
`T00-verification-round-6-CONFIRM.md`. That provenance treats the historical
matches as identifier-class values in committed evidence, documents corrected
source-column measurement, preserves the original AND-scoped allowlist, and
does not authorize broad path or rule exclusions. The newly inspected T09
recon is `gitleaks-push-recon-oct05.md`; it records 14 redacted findings, all
under the same rule and exact six paths, attributed to the sibling commit, and
explains that each reported line matches the allowlisted sibling tree at the
same path and line.

## Independent local checks

Without emitting matched values, `Match`, full source lines, or blob content,
the builder compared the 14 corresponding historical source lines in both
commit trees: all 14 were byte-identical. The two commits have the same parent.
The check emitted only the match count, byte-length set, and parent-equality
result. No credential-bearing field values or line hashes were emitted.

This is a commit-identity extension for the already reviewed occurrences in
the same immutable evidence and same-parent sibling tree. It does not create a
path-wide or rule-wide exclusion: the original `AND` conjunction still requires
commit, rule, and one of the six existing paths to match. New paths, rules, or
commit identities remain subject to scanning.

## Exact change

```diff
-commits = ["6ce518fcd9b01bc5a7037f80f5d8986a33cd2924"]
+commits = ["6ce518fcd9b01bc5a7037f80f5d8986a33cd2924", "44b5a0b36399fffbd685de7d93bda1933ed75c96"]
```

Only `.gitleaks.toml` and this new evidence record were changed. No source,
hook, scanner version, dependency, history, status, or debt file changed. No
compiler, test, or scanner was run by this builder; the designated independent
reviewer owns verification.
