# Gitleaks sibling allowlist builder supplement — 2026-10-05

This immutable supplement completes the builder record after interruption. The
prior `gitleaks-sibling-builder-oct05.md` recorded the commit addition and
redacted local identity checks, but omitted the requested adjacent config
comment and exact config hashes. It is retained unchanged.

## Exact config hashes

The pre-change baseline is the original `.gitleaks.toml` with only the
operator-approved commit in the T00 allowlist. Its SHA-256 is
`9dc614892f889e228d1ade247f66ed30a93080631411cf7d8e9ed976c1bdb62e`.
The resulting config, including the sibling commit and its explanatory
comment, has SHA-256
`52aac03b5fc0874af78e901f9aed029037f2e5835fdd3a7cfc496c3862783591`.
The baseline hash was reconstructed from the current file by reversing exactly
the prior builder's recorded single commit-list addition and this supplement's
comment; the intermediate sibling-only config hash recorded before this
supplement was
`e3fcb8aff51f03f257fd252bf6753ed063ab1b3a9779a3ff7044e8b43722db3c`.

## Full baseline-to-current config diff

```diff
 # Deliberately narrow, following the commit-scoped precedent above: condition = AND so the
 # allowlist requires the commit AND the path AND the rule to all match. Any NEW occurrence in
 # these paths - or any occurrence elsewhere - still fails the gate.
+# The second commit is the same-parent sibling carrying the same 14 byte-identical historical
+# evidence lines at these paths and line numbers; add its identity without broadening any scope.
 [[allowlists]]
 description = "casework T00: identifier-class generic-api-key matches in the bounded-judgment plan's evidence at 6ce518f (operator-approved 2026-09-19; analysis in .agents/evidence/godspeed-casework-cognitive-environment/T00/gate-baseline-seafoerge.md)"
 condition = "AND"
-commits = ["6ce518fcd9b01bc5a7037f80f5d8986a33cd2924"]
+commits = ["6ce518fcd9b01bc5a7037f80f5d8986a33cd2924", "44b5a0b36399fffbd685de7d93bda1933ed75c96"]
 rules = ["generic-api-key"]
 paths = [
   '''\.agents/evidence/godspeed-bounded-judgment/T18/observations\.jsonl''',
   '''\.agents/evidence/godspeed-bounded-judgment/T27/observations\.jsonl''',
   '''\.agents/evidence/godspeed-bounded-judgment/repeatability/observations\.jsonl''',
   '''\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK-fmt-drift\.txt''',
   '''\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK\.log''',
   '''\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CI\.log''',
 ]
```

The existing `AND`, rule, six paths, and original commit remain intact. The
comment documents why the one sibling identity was added; it does not change
matching behavior. No scanner, compiler, or test was run.
