# Root acceptance: bounded historical Gitleaks exception

Accept the independent anchored T00 review at config SHA
`608a687e53ba365fc743ae6c8d2ded7a1c006e8c6b0f5f209109d055f6e5ad05`.
Root verified the sibling occurrence provenance earlier, exact source deltas,
and supported selector/AND/whole-path semantics from executable controls.

Root parsed the actual and synthetic TOML: the synthetic config differs only
in the two test commit IDs; the positive config differs only by removing T00.
Root independently derived the expected residual from the redacted baseline:
21 detected controls minus exactly 12 allowed commit/path/rule pairs equals
the actual 9 residuals, including other rule/path/commit and prefix/suffix cases.
No matched values were printed or added to source evidence.

Root byte-compared all six safe matrix preflight/CLI/exit captures and all four
history captures with their original `/tmp` files. The real 608a history scan
joined exit 0 with an empty report. Normal checkpoint/push gates remain required;
this does not approve trace/auth prototypes, full current Go or T09.

## Capture naming clarification

The independent review's brace notation suggesting all history suffixes use a
hyphen is imprecise. Actual archive and original basenames are identical:

- `gitleaks-history-608a-20261005T232352Z-preflight.raw`
- `gitleaks-history-608a-20261005T232352Z.cli.raw`
- `gitleaks-history-608a-20261005T232352Z.report.json`
- `gitleaks-history-608a-20261005T232352Z.exit`

These four actual mappings are root byte-verified. Original records remain
immutable; the naming correction changes no scan result.
