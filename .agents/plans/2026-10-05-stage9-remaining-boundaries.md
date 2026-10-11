# Stage 9 plan — remaining boundaries

Governing spec: `.agents/specs/cep-world-ref-migration.spec.md` § Stage 9.

Order (producer before consumer so each repo stays green):
1. Context Kernel (`harden`, branch `migration/cep-world-ref`): world_ref ingress/echo, completeness. Tests: `cargo test -p ck-mcp`.
2. GodSpeed Agent (`audit-corrections/...`, branch `migration/cep-world-ref`): world.py, `_event`, evidence gate. Tests: agentic_capability_loop pytest.
3. SWE_SEED (`deploy-prep`; dirty tree, use a git worktree): request/adjudication/context gates. Tests: `cargo test -p swe-seed-core`.
4. Gauntlet (worktree on `migration/domainforge-0.18.2`): confirmation test.
5. Cross-repo vector parity, settlement report, status, DEBT.md, commit, push each repo.

Each repo: per-repo decision note (ADR / decisions.md) for the public contract change; `just context-check` before handoff.
Risks: SWE_SEED/CK frozen convergence fixtures may pin payload shape; legacy SEA manifest path is not DomainForge.
