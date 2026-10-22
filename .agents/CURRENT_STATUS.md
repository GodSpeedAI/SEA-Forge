# Current status

**Status revision:** 150

**Stage:** policy-resources-filter

**Summary:** Authority policy rules on cognate_action / cognate_capability can name the exact resources they cover (PolicyRule.resources). Workbench as an external-authority client (godspeed-readiness P02) needs this: consequential agent capabilities (tool.edit, tool.bash, ...) require approval while ordinary UI capabilities are allowed, which operation_kind alone cannot express.

**Verified:**

- cargo test -p sea-forge-authority --test cep_authority: 18 passed, 0 failed, including 4 new tests (exact match, non-match falls through, invalid on other operation kinds, empty list rejected)
- mutation check: removing the resources match makes the new match test fail
- Workbench J-SEA-FORGE-AUTHORITY passes against this server binary with the fixed Cognate runtime (Cognate PR #60)

**Limits:**

- resources match exactly; no prefixes or globs
- full just ci runs in pre-push

**Next move:** Merge, then open the Workbench P02 PR once a Cognate SDK release includes PR #60.

**Evidence:**

- crates/sea-forge-authority/tests/cep_authority.rs
- ~/projects/godspeed-readiness/docs/proposals/P02-workbench-external-authority.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
