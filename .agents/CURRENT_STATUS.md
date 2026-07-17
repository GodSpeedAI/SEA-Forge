# Current Status

Updated: 2026-07-17

## Objective

Publish SEA Forge's source-available, commercial, and enterprise licensing
documents on `main` before changing the GitHub repository from private to
public, while preserving an accurate README for the minimum kernel currently
implemented on `main`.

## Worktree State

The licensing change is isolated on `docs/public-licensing`, based on the
freshly fetched `origin/main` at `57da02f`. The active `full-spec` worktree and
its unrelated uncommitted implementation changes were not modified.

## Changed Files

- `LICENSE` — SEA-Forge Sustainable Use License.
- `COMMERCIAL-LICENSE.md` — commercial license agreement and permitted-use
  boundary.
- `LICENSE_EE.md` — enterprise-only component notice.
- `README.md` — source-available and commercial-use summary with links to the
  controlling license documents.
- `Cargo.toml` and both crate manifests — workspace license metadata changed
  from MIT/Apache-2.0 to the inherited custom `LICENSE` file so published crate
  archives include the controlling terms.
- `.gitleaksignore` and `docs/security/gitleaks-exceptions.md` — exact,
  documented suppressions for two intentional test sentinels already committed
  on `full-spec`.
- `deny.toml` — binds `LicenseRef-SEA-Forge` to the exact custom license text
  for both workspace crates and fails closed if that text changes.
- `.agents/CURRENT_STATUS.md` — this resumable handoff.

## Completed

- Selected only the three existing licensing commits from `origin/full-spec`;
  no full-spec implementation commits were merged into `main`.
- Kept the existing main-branch README as the technical baseline because the
  full-spec README advertises commands and crates not present on `main`.
- Added the authored public licensing summary and links to all controlling
  license documents.
- Confirmed the two repository-wide gitleaks findings are deliberate,
  non-functional test sentinels on `full-spec`, then reused their existing
  commit-and-rule-specific suppressions and public rationale.

## Verification

- Clean `origin/main` baseline: `cargo build --workspace --all-targets --locked`
  passed.
- Clean `origin/main` baseline: `cargo test --workspace --all-features --locked`
  passed (35 tests, 0 failed).
- `cargo package --list --allow-dirty` for both `sea-forge-core` and
  `sea-forge-cli` includes `LICENSE`.
- `cargo deny check licenses bans sources` passed with the hash-bound custom
  license clarifications (only pre-existing unmatched-allowance warnings).
- `gitleaks detect --log-opts=--all` scanned 76 commits with no leaks after
  applying the two exact documented fixture suppressions.
- Final `devbox run -- just context-check`: passed.
- Final `devbox run -- just check`: passed (fmt, clippy, typecheck,
  cargo-deny, and gitleaks).
- Final `devbox run -- just test`: passed (35 tests, 0 failed).

## Remaining

1. Commit the atomic licensing change, integrate it into `main`, and push
   `origin/main`.
2. After the repository is public, enable the documented `main` branch
   protection/ruleset and verify the required `CI / gate` check.

## Blockers

- Branch protection was previously unavailable while the repository was
  private on its current GitHub plan. Making the repository public or upgrading
  the plan is required before protection can be activated.

## Decisions

- The canonical base-license filename remains `LICENSE` so GitHub and Cargo
  detect it automatically; README links use that exact path. Cargo uses
  `license-file` rather than a custom SPDX expression because crates.io requires
  known SPDX identifiers in `license` and supports nonstandard terms through
  `license-file`.
- The integration is selective rather than a merge of `full-spec`, which is 43
  commits ahead of `main` and contains unrelated implementation work.
- Repository visibility will not be changed in this documentation commit;
  protection should be activated immediately after a separate visibility
  change.
