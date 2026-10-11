# Independent safe classification of d100 copied-prose findings

**Classification: two prose false positives.** This record contains only safe report metadata and source-integrity evidence. It does not reproduce source lines, triggering text, scanner `Match`/`Secret` values, or the raw report.

The private d100 diagnostic reported two `generic-api-key` findings, both in ordinary prose in newly authored Markdown evidence. The matched spans were each 40 characters and fell outside inline-code or fenced-code regions. The source context was not assignment-shaped and contained no URL. I inspected only the report's rule, file, commit, line/column, and fingerprint fields, then verified each location against the committed blob. The committed blobs matched the corresponding worktree files byte-for-byte.

| Safe fingerprint | Source location | Committed source SHA-256 | Verification |
|---|---|---|---|
| `d100b9b80986cef4b7c38022b3299b198ee68a03:.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-checkpoint-gitleaks-exception-independent-review-oct08.md:generic-api-key:15` | Commit `d100b9b80986cef4b7c38022b3299b198ee68a03`, line 15, columns 33–73 | `656138fa42d7389e330d40734b32933fbd0ca8ec4d7331cf614c54654a3da6d8` | Committed blob equals worktree bytes; 40-character prose span. |
| `d100b9b80986cef4b7c38022b3299b198ee68a03:.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/checkpoint-two-false-positive-fingerprints-builder-oct08.md:generic-api-key:7` | Commit `d100b9b80986cef4b7c38022b3299b198ee68a03`, line 7, columns 256–296 | `3f23ff7c1913fa0c911c4d17e177d830da45993439f17bd62a7b8936cbbf3fbc` | Committed blob equals worktree bytes; 40-character prose span. |

Both findings recur because task evidence quotes earlier generic/rejected prose. This is not evidence of a credential in source or configuration. If a narrow scanner exception is needed, its scope should match only these two exact commit/path/rule/line fingerprints. This classification does not establish scanner exception behavior, approve a scanner configuration change, or make any runtime/security-gate claim. Earlier findings and their existing narrow dispositions are unchanged.

No scanner rerun, source/configuration edit, compiler, test, gate, or Git mutation was performed for this classification.
