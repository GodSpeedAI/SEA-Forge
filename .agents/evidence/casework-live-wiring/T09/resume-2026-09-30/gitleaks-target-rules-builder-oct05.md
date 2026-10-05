# Gitleaks pinned-version selector correction — builder record (2026-10-05)

## Scope and authorization

Applied the root-directed, one-key correction to `.gitleaks.toml` only. The prior
independent review found that Gitleaks 8.30.1 does not define `rules` as a global
allowlist selector; its supported selector is `targetRules`. The v8.30.1 upstream
`config/config.go` defines `viperGlobalAllowlist.TargetRules`, and translation routes
such allowlists to the named rule IDs. The v8.30.1 README documents `targetRules`.
This edit restores the previously intended `generic-api-key` restriction instead of
leaving the exception global.

## Exact edit and hashes

Before this correction, the complete config hash recorded by the preceding builder
supplement was `52aac03b5fc0874af78e901f9aed029037f2e5835fdd3a7cfc496c3862783591`.
After this correction, `sha256sum .gitleaks.toml` returned
`c0185761adad0db725590f0a96db07663ad9e179b1c5edad9c35cc060b7a7346`.

The full before/after diff for this bounded correction is:

```diff
-rules = ["generic-api-key"]
+targetRules = ["generic-api-key"]
```

No other line was changed by this correction. The resulting T00 entry retains
`condition = "AND"`, both exact commit IDs
(`6ce518fcd9b01bc5a7037f80f5d8986a33cd2924` and
`44b5a0b36399fffbd685de7d93bda1933ed75c96`), `targetRules = ["generic-api-key"]`,
all six original exact path expressions, and the adjacent explanation of the sibling
commit.

## Material clarification to the original builder claim

The earlier sibling builder record described the `rules` entry as a functioning criterion
and claimed the rule scope remained intact. That claim was incorrect: the key is not in
the pinned 8.30.1 config struct. The scanner did not fail closed on that unknown key, so
the earlier entry's effective `AND` checks were commit plus path, with no rule selector.
This correction narrows the exception back to the intended named rule. It does not broaden
scope or add another exception; the field becomes effective using the pinned supported
selector. The initial authorized sibling-commit addition remains separately documented in
`gitleaks-sibling-builder-oct05.md` and its supplement.

## Verification boundary

This builder made no scanner, compiler, test, hook, status, debt, or Git operation. The
config is frozen for a different independent critic to verify. Required synthetic positive
and negative controls, followed by any authorized normal push scan, remain pending that
review and explicit gate release.
