# d100 two-fingerprint ignore repair — assignment and result

Date: 2026-10-08

## Bounded assignment

Preserve all existing `.gitleaksignore` bytes as a prefix and append exactly
the following two full fingerprints, once each:

```text
d100b9b80986cef4b7c38022b3299b198ee68a03:.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-checkpoint-gitleaks-exception-independent-review-oct08.md:generic-api-key:15
d100b9b80986cef4b7c38022b3299b198ee68a03:.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/checkpoint-two-false-positive-fingerprints-builder-oct08.md:generic-api-key:7
```

This is a narrow per-fingerprint exception based on the independent safe
classifications in `d100-copied-prose-false-positive-independent-classification-oct08.md`
and `run-observation-checkpoint-gitleaks-exception-independent-review-oct08.md`.
No rules, paths, commits, hooks, scanner configuration, or broad allowlists are
authorized. Do not include triggering source prose or scanner `Match`/`Secret`
values in this record. Do not rerun the scanner or claim a security-gate pass.

## Result and exact identities

Only `.gitleaksignore` was edited. Its preimage is 1,924 bytes, SHA-256
`88ea6db03797a0de85083fdae2ec24c163b16006734f5d321a22e4cdd17f7eee`. The
postimage is 2,297 bytes, SHA-256
`06b7160c48dc57fde28ec7adc0dae8980bf9ae3e6ebb57da1d812b7f257c5118`. The
two requested lines above are the exact diff, appended in the requested order.
I reconstructed the 1,924-byte prefix by removing only those two final lines;
its SHA-256 exactly matches the preimage. Each new fingerprint occurs exactly
once and the file ends with those two lines.

No raw diagnostic values were included in this record. No scanner rerun,
compiler, test, gate, Git command, or security-pass claim was made. Root will
perform the separately controlled security verification after independent
config review.
