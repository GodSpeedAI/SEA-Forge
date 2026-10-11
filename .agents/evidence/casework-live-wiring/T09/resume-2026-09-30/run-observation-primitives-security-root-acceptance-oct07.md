# Source-integrity fingerprint repair: root acceptance

2026-10-07. Root approves the one historical fingerprint exception and its
documentation. The independent review `run-observation-primitives-gitleaks-exception-independent-review-oct07.md`
SHA-256 `3ef3f92e59c89734ef54c9f802b8bd8cf328ffd7ff1a29515237c75b29b784fb`
independently proves the same committed blob/evidence-line relationship.
Root inspected the exact diff: one `.gitleaksignore` fingerprint, specifically
commit617dac3 / policy-format evidence / generic-api-key / line83, plus its
documentation. Rules, scanner configuration, hooks, history and source remain
unchanged. This uses the existing exact-fingerprint mechanism.

Root recomputed SHA-256 from the committed private three-string key type and
proved line83 contains exactly its filename label and source digest (90 bytes).
The record is source-integrity metadata, not credential/authentication data.
No redacted placeholder was used as a substitute for the actual source proof.
The private diagnostic JSON remains outside repository evidence.

The unchanged canonical command, from the primary repository, was
`env JUST_TEMPDIR=/tmp CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 devbox run -- just security`.
It exited0. The actual output reports advisories, bans, licenses and sources
OK, 516 commits / approximately84.29 MB scanned, and no leaks found. Its
113321-byte output SHA-256 is
`0fbafea688e00c0a28a97f526c40527ead18a98495a2fdddd5e54bc4e9eba238`.
All three originals under `/tmp/sea-primitives-root-security-jjFycY` compare
exactly with `run-observation-primitives-root-security-{preflight,output,exit}-oct07.raw`.

The failed first push remains failed, with all three original captures exact.
The diagnostic's agent shell-copy archive deviation is disclosed in CW-18;
those four raw copies are byte-correct, not claimed as native tool writes.
Root's two attempted raw preflight archives normalized14 carriage returns and
remain invalid/excluded. The native lossless JSON preflight archive decodes
exactly to the untouched857-byte original. The other three root diagnostic
captures compare byte-for-byte. No report containing an unverified Match field
was published, and no matched candidate value was copied into these records.

The new exception must be committed and pass the normal push hooks before
publication succeeds. This acceptance is not a full CI/push pass, manager
lifecycle approval, or T09 settlement. Preserve eleven foreign staged entries
and all unrelated operator work; stage explicit checkpoint paths only.
