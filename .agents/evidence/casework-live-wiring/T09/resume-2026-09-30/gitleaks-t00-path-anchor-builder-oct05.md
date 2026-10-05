# T00 Gitleaks exact-path anchor repair — builder record

Date: 2026-10-05

## Original bounded authorization

The assignment authorizes only anchoring all six existing T00 entry path
regexes with `^` and `$`, so they match the whole intended Git paths. Preserve
each regex interior, the `targetRules = ["generic-api-key"]` condition, the
exact two commit identities, `condition = "AND"`, and every other allowlist,
comment, and config line. This narrows the existing six-path exception; it
does not change scanner behavior, tests, hooks, versions, dependencies, or
history. Do not run scanners, compilers, tests, Git, hooks, status, or debt
operations. Record exact hashes/diff/deviations and freeze for a different
critic's synthetic matrix and history review.

## Exact config identities

Before SHA-256: `c0185761adad0db725590f0a96db07663ad9e179b1c5edad9c35cc060b7a7346`

After SHA-256: `608a687e53ba365fc743ae6c8d2ded7a1c006e8c6b0f5f209109d055f6e5ad05`

## Exact diff

```diff
-  '''\.agents/evidence/godspeed-bounded-judgment/T18/observations\.jsonl''',
+  '''^\.agents/evidence/godspeed-bounded-judgment/T18/observations\.jsonl$''',
-  '''\.agents/evidence/godspeed-bounded-judgment/T27/observations\.jsonl''',
+  '''^\.agents/evidence/godspeed-bounded-judgment/T27/observations\.jsonl$''',
-  '''\.agents/evidence/godspeed-bounded-judgment/repeatability/observations\.jsonl''',
+  '''^\.agents/evidence/godspeed-bounded-judgment/repeatability/observations\.jsonl$''',
-  '''\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK-fmt-drift\.txt''',
+  '''^\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK-fmt-drift\.txt$''',
-  '''\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK\.log''',
+  '''^\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CHECK\.log$''',
-  '''\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CI\.log''',
+  '''^\.agents/evidence/godspeed-bounded-judgment/T00/gate-SEA_CI\.log$''',
```

The six interiors are unchanged. Post-edit metadata inspection found exactly six
anchored T00 paths, with AND, target rule, and both commit identities preserved.
No scanner or other verification command was run. No material deviation.
