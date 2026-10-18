# Gitleaks exception control — 2026-10-17 continuation evidence

Operator decision: provisionally retain a narrow exception, conditional on the
four checks below. Evidence produced by the builder; not independently reviewed.

## 1. The 22 matches are exclusively synthetic idempotency digests

Scan of full history with the committed config minus the 2026-10-17 block:
22 `generic-api-key` matches, 10 distinct values, in 3 files across commits
`c112482` (18) and `7cdee13` (4). `matches-classified.json`.

Every value was **recomputed from its public preimage**, not just shape-checked:

| Values | Derivation (code) | Recomputed |
| --- | --- | --- |
| 6 (AuthorizedInvocation, OperationalSettlement, EvidenceRecorded) | `sha256("" + "|" + event_type + "|" + canonical_payload_json(payload))` — `crates/sea-forge-server/src/governed_execution_boundary.rs:147` | 6/6 equal |
| 4 (ExecutionObservation fixtures) | `sha256(work_request_id)` — `case-security-control`, `case-security-rejection`, `case-sec12-rejection` (test fixtures; two values were already independently classified in the frozen v1 allowlist) | 4/4 equal |

A SHA-256 of a public payload cannot carry credential material.

## 2. Scope — narrowed

The first exception used `regexTarget = "line"` with a pattern
(`"idempotency_key":\s*"[0-9a-f]{64}"`). Canaries (below) showed that, once
its commit pin was satisfied, it would also suppress a new 64-hex value, an
allowed value at another listed path, and **a credential on the same line as an
allowed digest**. It was replaced with three exact tuples, matching the
existing repository precedent: one commit AND one path AND rule
`generic-api-key` AND `^<exact value>$` on the secret.

## 3. Canaries (`canary.py`, `canary-results.json`)

Throwaway repo, one commit per canary at the exact listed paths, fresh random
synthetic values. Four configs: committed (commit pin active), committed with
the 2026-10-17 tuples re-pinned to the canary commits (tests path/rule/value
conjuncts in isolation), previous line-regex config re-pinned, default ruleset.

| Canary | committed | re-pinned (new) | re-pinned (previous) |
| --- | --- | --- | --- |
| P1 exact allowed value, allowed path (positive control) | detected | suppressed | suppressed |
| C1 new 64-hex idempotency value, allowed path | detected | detected | **suppressed** |
| C2 `api_key` credential, allowed path | detected | detected | detected |
| C3 allowed value at another tuple's path | detected | detected | **suppressed** |
| C4 allowed value, unlisted path | detected | detected | detected |
| C5/C6 altered value (uppercase, 66 hex) | detected | detected | detected |
| C7 allowed value under another field name | detected | suppressed | detected |
| C8 credential on same line as allowed digest | detected | detected | **suppressed** |
| C9–C12 AWS key, GitHub PAT, PEM key, Slack token in allowed paths | detected | detected | detected |

Residual (C7): within the two pinned, immutable commits, the same public digest
value under a different field name would be suppressed. The value is a
recomputed public hash, and no new content can enter those commits.

## 4. Full gate

`just security` (cargo-deny + `gitleaks detect` over full history): exit 0,
480 commits, no leaks.

**Gap found:** `gitleaks detect` does not scan staged-but-uncommitted content
(scratch-repo test: staged AWS key → exit 0; `gitleaks git --staged` → exit 1;
after commit `detect` → exit 1), although the `justfile` comment says it
scans staged content. Committed content is scanned by `just ci` in the
pre-push hook. Candidate revision (not applied — gate change): add
`gitleaks git --staged` to the pre-commit recipe and correct the comment.
