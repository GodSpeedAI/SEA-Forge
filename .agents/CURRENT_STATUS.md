# Current status

**Status revision:** 293

**Stage:** Local records committed and leftovers removed

**Summary:** Committed the uncommitted FDE convergence, CognitiveOS design and domain-modeling records from 2026-11-12 to 2026-11-26, removed a stale contracts schema copy and an identity-staging diagnostic bundle after a verified backup, and allowlisted 97 git commit SHAs that gitleaks misread as Sourcegraph tokens.

**Verified:**

- Gitleaks on every committed path with the repository config: no leaks. The 166 earlier hits were 97 unique 40-hex commit SHAs in the OD-4 branch-backup ledger; the allowlist is scoped by rule, path and exact value.
- Backup ~/projects/_backups/sea-rs-untracked-leftovers-20261127.tar.gz: 15,775 entries, equal to the source file count; sha256 verified. Only two dead socket files were skipped.
- bun run generate:contracts after removing workbench/packages/contracts/schema and generated/PendingApproval.ts leaves generated/ unchanged: the generator reads the repository-level contracts/schema, and PendingApproval was renamed to CaseTaskApproval in #87.

**Limits:**

- The contracts package README wrongly placed schema/ inside the package; corrected here.
- The deja machine-local rule cache .deja/live_rules.jsonl is now gitignored; the shareable .deja/memory.jsonl is not.
- Hosted SEA lint and Security are red on main from RUSTSEC-2026-0592 (url) and RUSTSEC-2026-0543 (chrono unmaintained); the url fix and chrono migration are separate work.

**Next:** Land the url advisory fix, then the chrono-to-time migration across DomainForge, RealityTrace and SEA.

**Evidence:**

- .agents/reports/2026-11-26-cognitiveos-108-wp23/evidence-ledger.md
- .gitleaks.toml
- workbench/packages/contracts/README.md
- .gitignore
