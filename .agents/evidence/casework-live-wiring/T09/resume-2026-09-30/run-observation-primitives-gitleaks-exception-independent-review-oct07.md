# Independent Gitleaks source-hash exception review — 2026-10-07

**Verdict: APPROVE the single exact fingerprint exception.** The classification
is supported by the committed source blob and the source line it records. This
approval is not a Gitleaks gate pass or push approval.

## Independent finding and blob checks

I read the full bounded repair assignment
`run-observation-primitives-gitleaks-fp-repair-assignment-oct07.md` (SHA-256
`bdbfe39a56d0d80f05967575ac0d23d9cb3232113dca1e0459923575fbb2d6a1`) and result
`run-observation-primitives-gitleaks-fp-repair-result-oct07.md` (SHA-256
`798f86a0ff8ceb07793c340f5914f4c65ea29d20ff40c796c0caed2d10fa122d`). I
independently queried only safe fields from the private diagnostic JSON. It
contains exactly one finding with rule `generic-api-key`, the specified
historical evidence path and line 83, and the expected fingerprint:

`617dac3ddf78b660ca95f1c7a53fdf59b87653d5:.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-policy-gofmt-repair-result-oct07.md:generic-api-key:83`

I did not inspect or emit Match, Secret, or candidate text. The report remains
private and is not copied into evidence.

Using the committed blob from commit `617dac3ddf78b660ca95f1c7a53fdf59b87653d5`,
I independently recomputed SHA-256 for
`apps/godspeed-casework-go/internal/server/run_observation_key.go`:

`a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`

I verified the committed evidence line 83 equals exactly
`- run_observation_key.go: ` followed by that digest and is 90 bytes. The
committed source defines only a private key struct with three string fields:
`caseID`, `runID`, and `planItemID`. It has no credential value or
authentication behavior. The classification as a source-integrity reference
is therefore supported by both the value's exact origin and its use.

## Scope of the exception

The `.gitleaksignore` diff adds exactly one line, using the existing full
fingerprint format for that historical commit, file, rule, and line. It does
not ignore the file, rule, commit, or a broader path. Other locations and
future findings have different fingerprints and remain subject to the normal
gate.

The documentation adds a specific source-hash-reference classification,
location, reason, committed-blob proof, and pending-review/gate status. The
repair result records that `.gitleaks.toml`, hooks, `justfile`, source,
immutable scan evidence, and Git history were not changed. It also discloses
the capture/archive deviations: raw-copy attempts that lost carriage returns
remain excluded, the lossless wrapper is separately compared, and the
four actual diagnostic capture pairs are reported as cmp0 by root. I did not
rewrite, promote, or independently claim those archives as native captures.

## Original direction differences and limits

The original diagnostic task stopped before implementation. The bounded repair
release authorized one exact historical fingerprint, a narrowly classified
documentation entry, and specified evidence/debt updates; the resulting diff
matches that scope. The original Secret placeholder was not treated as the
candidate, consistent with the assignment. No blanket ignore, rule change,
path allowlist, hook change, or source edit appears in the reviewed exception
delta.

No scanner, compiler, formatter, or Git mutation was run for this review. The
ordinary security gate remains pending after the original failing push hook;
root must run that unchanged gate and normal push retry separately. This
approval supports only the exception's benign classification and exact scope.
