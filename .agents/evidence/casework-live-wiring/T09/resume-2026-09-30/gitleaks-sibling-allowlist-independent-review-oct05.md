# Independent review: Gitleaks sibling allowlist repair (2026-10-05)

**Verdict: REJECT pending repair and executable negative controls.** The builder's
claim that this preserves a rule-scoped exception is not supported by the pinned
Gitleaks version. The TOML uses a non-existent selector key, so the configured
allowlist is global. No scan, compilation, or source-policy mutation was performed by
this reviewer. This source review does not establish whether the redacted matches are
benign; that classification rests on the cited T00/T09 evidence and is outside this
configuration-semantics finding.

## Pinned-version evidence

Local `gitleaks version` reports 8.30.1. I checked the exact upstream v8.30.1 source and
documentation:

- In [`config/config.go` at v8.30.1](https://github.com/gitleaks/gitleaks/blob/v8.30.1/config/config.go),
  `viperGlobalAllowlist` declares `TargetRules []string` and embeds the shared allowlist
  fields; that shared struct has `Condition`, `Commits`, `Paths`, `RegexTarget`, `Regexes`,
  and `StopWords`, but no `Rules` field. Translation applies a global allowlist unless
  `TargetRules` is nonempty; when set, it attaches the allowlist to those rule IDs.
- The [`v8.30.1 README`](https://github.com/gitleaks/gitleaks/blob/v8.30.1/README.md)
  documents `targetRules` (introduced in 8.25.0) as the field for restricting an allowlist
  to specified rules, rather than applying it globally.
- The exact local config has `rules = ["generic-api-key"]` at the T00 allowlist. The
  v8.30.1 schema/source has no such selector. The prior recon reports the scanner loaded
  this config and proceeded to findings, so this typo did not fail closed at config load.

Consequently, the active criteria in this entry are `condition = "AND"`, the listed
commit identities, and the six listed paths. The rule criterion claimed in the comment
and both builder reports is absent. The two effective criteria are conjunctive, but any
finding from any rule on one of those commit/path combinations may be suppressed. This
is narrower than a path-only or commit-only allowlist but broader than the reviewed
commit + path + `generic-api-key` scope.

## Required repair and proof before acceptance

Use the field supported by the pinned version, `targetRules = ["generic-api-key"]`, if
the intended scope remains the previously reviewed rule. Keep the `AND` condition, exact
commit identities, and exact paths. Then demonstrate the semantics with a minimal
isolated fixture under Gitleaks 8.30.1; do not use a whole-history scan as the only proof.
The controls should produce these outcomes without exposing any matched value:

1. The exact allowed commit + exact path + `generic-api-key` finding is suppressed.
2. A different rule at the same allowed commit/path remains reportable.
3. `generic-api-key` at an unlisted path remains reportable.
4. `generic-api-key` at an unlisted commit remains reportable.
5. A valid in-scope finding with the listed criteria is still suppressed when the TOML
   arrays contain multiple values, confirming each array is treated as alternatives and
   the `AND` combines the criterion classes.

Use fully synthetic secrets in a disposable `/tmp` repository and a temporary custom
config if required. Capture redacted output and exit status only; do not print `Secret`,
`Match`, or source lines. After those isolated controls pass, the root-assigned verifier
can run the separately authorized normal push gate. Preserve raw logs safely and retain
the existing scan no-output/secret-value restrictions.

The sibling builder's source-line equality and hash supplement are useful provenance
checks but cannot prove selector behavior. Neither the scan report nor config inspection
alone demonstrates that the exception remains rule-specific. No full scan was run here;
the root explicitly held scans until the compiler-token owner releases that gate.
