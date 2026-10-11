# Gitleaks source-hash finding repair result — 2026-10-07

Status: bounded source-only repair complete; independent critic and normal
security/push gate remain pending. No scanner, compiler, or Git mutation was
performed for this repair.

## Finding and source proof

- Detector: `generic-api-key`
- Commit: `617dac3ddf78b660ca95f1c7a53fdf59b87653d5`
- Location: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-policy-gofmt-repair-result-oct07.md:83`
- Fingerprint: `617dac3ddf78b660ca95f1c7a53fdf59b87653d5:.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-primitives-policy-gofmt-repair-result-oct07.md:generic-api-key:83`
- Entropy: `3.8483343`
- Root independently ran `git show
  617dac3ddf78b660ca95f1c7a53fdf59b87653d5:apps/godspeed-casework-go/internal/server/run_observation_key.go`
  and recomputed SHA-256 over the committed blob. The equality check against
  the line's recorded digest succeeded (digest prefix `a6f0114d`). The
  90-character Markdown line is exactly the source filename label followed by
  that committed source hash. This is an integrity reference; it has no
  credential or authentication behavior.
- The JSON diagnostic report is not archived: its sensitive Match field was
  not confirmed fully redacted. No Match, Secret, or matched candidate text is
  included here or elsewhere in this evidence.

## Narrow repair

- Added exactly one full finding fingerprint to `.gitleaksignore` using its
  existing exact-fingerprint format.
- Added the classification, source proof, exact location, and review status to
  `docs/security/gitleaks-exceptions.md`.
- Updated CW-30 to record the identified source-hash reference and pending
  independent review/normal gate.
- Added the archival-method recurrence to CW-18. The four direct copies made
  after an escalated read-only error remain byte-exact to their private
  originals but preserve a disclosed method deviation. Root's later plain raw
  preflight copies lost 14 CR bytes and remain excluded; its lossless escaped
  preflight wrapper decodes byte-exactly to the original.

No `.gitleaks.toml`, scanner rule, path allowlist, commit-wide allowlist, hook,
`justfile`, source, immutable scan evidence, or Git history was changed. The
ignore delta is one fingerprint line only. Other pre-existing exceptions and
all foreign staged entries remain outside this repair.

## Captures and provenance

The scanner command was `devbox run -- gitleaks detect --no-banner --redact
--report-format json --report-path
/tmp/sea-gitleaks-diag-20261007T223332Z/findings.json`; it exited `1` with one
finding. The report remained private in `/tmp` (1,159 bytes; SHA-256
`a645805b3aae92f395a1071729c41e9eb73452dcfe25293f2bff0ffa7dc5346c`). The
original report is not copied into repository evidence.

The actual preflight/output/exit captures were copied before this repair and
compared with their `/tmp` originals. Their BASE paths and hashes are:

- `run-observation-primitives-gitleaks-diag-preflight-oct07.raw` — 857 bytes,
  SHA-256 `824c110962c6b62f62e3f9f4088555c9249f3da8570c82a69be508398c9cf992`.
- `run-observation-primitives-gitleaks-diag-preflight-exit-oct07.raw` — 2
  bytes, SHA-256
  `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`.
- `run-observation-primitives-gitleaks-diag-output-oct07.raw` — 327 bytes,
  SHA-256 `75c153a2770055a3541d56c2bc02be954603ed7c684653ffbe560a6cd85b2a7f`.
- `run-observation-primitives-gitleaks-diag-exit-oct07.raw` — 2 bytes, SHA-256
  `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`.

Root separately wrote
`run-observation-primitives-gitleaks-diagnostic-preflight-lossless-root-oct07.json`
with an exact-text representation of the preflight and confirmed that decoding
reproduces the original 857 bytes. Earlier root raw-copy attempts named
`run-observation-primitives-gitleaks-diagnostic-preflight-oct07.raw` and
`run-observation-primitives-gitleaks-diagnostic-preflight-exact-root-oct07.raw`
lost 14 CR bytes; retain them as invalid raw copies and do not overwrite them.

## Scope and review status

Only `.gitleaksignore`, `docs/security/gitleaks-exceptions.md`, this new result,
and the CW-18/CW-30 worktree debt text changed. The DEBT index blob was preserved;
no staging or commit was done. A different independent critic must confirm the
benign source-hash classification and exact fingerprint scope before root runs
the ordinary security gate and retries the authorized normal push. The
original Gitleaks failure remains a failed gate until that ordinary retry
passes.
