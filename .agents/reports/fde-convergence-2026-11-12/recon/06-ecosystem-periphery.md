# 06 Ecosystem periphery (read-only recon, 2026-11-12)

Method: `git -C` for remote, branch, HEAD, dirty count; README/AGENTS.md heads; manifest presence; `grep -rIl --include=*.md -i`. OBSERVED = read in a file or git output. INFERRED = from a name or manifest only.

## 1. Identity and role

| System | origin | branch @ HEAD | dirty | Role (OBSERVED from README/AGENTS unless marked) |
|---|---|---|---|---|
| godspeed-readiness | none | main @ ae90607 | 0 | Gate harness for profile `local_operator` (`scripts/readiness.sh`, `profiles.yaml`). Last commit: "local_operator GO 30/30 at 2026-10-25 heads". |
| godspeed | SPRIME01/gspeedai.git | main @ f8e7dab | 8 | Transcripts and concept notes (`Ops/`, `main_language/`, `paradigm/`). No manifests. |
| godspeed-compass | GodSpeedAI/godspeed-compass.git | main @ d52f39d | 0 | "Evidence-bearing settlement workflow" (AGENTS.md:3-4). Research dirs incl. `3-fde-market-proof-system`, `11 FDE Settlement Labs`. |
| godspeed-growth | GodSpeedAI/godspeed-growth.git | main @ a1012f0 | 0 | Agency system for brand, campaigns, site (AGENTS.md:1-3). Holds `world/godspeed_ai_fde_sop_v0_3.md`. |
| strategy-growth | same as godspeed-growth | main @ a1012f0 | 0 | Same remote and HEAD; duplicate checkout (tree equality unchecked). |
| ideal | GodSpeedAI/ideal.git | main @ dcc5ea1 | 0 | Skill that audits "structured idea systems" (README.md:1-16). Has `bin/`, `schema/`, `skills/`. |
| uscos | GodSpeedAI/USCOS.git | main @ 6982fb9 | 0 | Plugin `uscos-plugin/` (comparative-advantage, strategic-compression agents). |
| SEA | GodSpeedAI/SEA.git | dev @ ccbf6f41c | 1 | Spec and routing layer (AGENTS.md "routing and index layer"); `docs/specs/shared/`. |
| sentient-enterprise-architecture | GodSpeedAI/SEA.git | main @ ef1ca782 | 0 | Same remote as SEA, other branch and HEAD. Two checkouts of one remote. |
| NewSea | none | master, no commits | 20 | Spec-first: "Do not handwrite runtime code" (AGENTS.md:6-8). Nx monorepo. |
| XSEA | not git | n/a | n/a | "Vibes Pro"-generated spec project (AGENTS.md:1-3). Has `mcp/`. |
| hermes-agent-sea-forge | GodSpeedAI/hermes-agent-sea-forge.git | main @ b85c3dc5b7 | 0 | Fork of upstream Hermes Agent (README is upstream banner). SEA-Forge link not seen in README. |
| zeroclaw-sea-forge | GodSpeedAI/zeroclaw-sea-forge.git | master @ 5518148e3 | 0 | "Run ZeroClaw with SEA authority enforcement before high-impact actions execute" (README.md:1-2). |
| gs-term | GodSpeedAI/gs-term.git | main @ 73137e4 | 0 | "Semantic terminal / control plane built on Cognate": state -> affordances -> action -> effects -> evidence (README). |
| terminal-ui | GodSpeedAI/Terminal-UI.git | remediation/audit-2026-09-29 @ 7932154 | 0 | Clack-style component library for OpenTUI React, with a Workbench (README). |
| adlc | none | main, no commits | 4 | Design drafts only (`docs/designs/adlc.md`, `concepts_brief.md`). |
| ast-grep-mcp-rs | GodSpeedAI/ast-grep-mcp-rs.git | main @ 3c2b51a | 1 | Rust MCP server for structural code search; port of ast-grep-mcp. |
| project-topology-architect | GodSpeedAI/project-topology-architect.git | main @ eb812ac | 0 | "Project-topology compiler" Claude Code plugin (README). |
| NeatCode | GodSpeedAI/NeatCode.git | main @ de9a0b9 | 0 | Engineering-judgment plugin (`skills/neatcode`, `.claude-plugin/`). |
| Strategist | SPRIME01/StrategyNotes.git | main @ 9edae61 | 0 | "Turn uncertain situations into defensible decisions ... and proof of what worked" (README). |
| hassos-addon-agent-memory-ledger | GodSpeedAI/hassos-addon-agent-memory-ledger.git | main @ 003809c | 0 | "Local-First Governance Substrate for SEA Forge and governed agent runtimes" (README). Home Assistant add-on. |
| elephant-goldfish-model | google/elephant-goldfish-model.git | main @ 403d888 | 0 | "Design with an elephant. Review with a goldfish." Review-agent plugin; its egm:* agents are in this session. |
| tree-sitter-sea | none | main @ b851f0c | 0 | Tree-sitter grammar (`grammar.js`, `queries/`). Use as SEA syntax is INFERRED from name. |
| _wt | not git | n/a | n/a | 372 entries (section 3). |
| gsr-heads | not git | n/a | n/a | 9 detached worktrees (section 2). |

## 2. gsr-heads and readiness

- **Pinned heads** (worktrees, `rev-parse HEAD`, OBSERVED): cep 5111ec3 · cognate bcef663 · cognate-workbench 1b6fcb4 · context-kernel 81a6fdb · gauntlet 1543cbf · horizon 55f1b42 · realitytrace 4d05cc0 · sea-forge 4e67dfca · swe-seed 232e02d.
- **Gap:** `domainforge` is not in gsr-heads, yet `scripts/readiness.sh:25,54-55` builds `$HEADS/domainforge` when present. Unresolved.
- **readiness.sh** (header 1-16): syncs each head to origin default as a detached worktree (dirty head stops the run); starts pg0; checks bubblewrap and DomainForge; runs gates one at a time with RAM waits; regenerates `evidence/REPORT.md`; exits 0 only on `local_operator` GO (line 87).
- **Gates:** a profile is GO only when every gate PASSes at pinned revisions and no open blocker remains (`profiles.yaml:1-3`). Gate IDs seen: L01, S01, C01, R01, S03, SW01, J02, J02R2, U01, U02f.
- **Blockers** (`docs/release-blockers.md`, table only partly read): B-01 Workbench session takeover (DNS rebinding); B-02 DomainForge version skew; B-03 crash repeats external effect; B-04 to B-06 and B-08 marked merged or fixed in the table; B-07 Workbench lacks SEA-Forge/SWE_SEED/Gauntlet integration. Current status of B-01 to B-03 and B-07 not confirmed.

## 3. `_wt` (372 entries, OBSERVED)

Prefixes: wb 24, b08 23, ss 18, sf 14, swe 12. Subdirs: arm64, bm*, consol, containers, harvest, logs, macjail, macsb, oscap, oslive, oswin, pkg, r1osb, r1win, r2s2, r2w2 (worktrees or scratch). Files: `*.log`, notes (`approval-fix-notes.md`, `b08-pr.md`), `Cargo.lock*`, `b12.sock`, backup manifests. `_wt/cargo.lock` is the readiness build lock (readiness.sh:5). Do not delete while gates run.

## 4. Keyword hits (markdown)

- "field case": godspeed-growth/world/godspeed_ai_fde_sop_v0_3.md; godspeed-growth/new_world/glossary.md; strategy-growth (same); project-topology-architect/.tmp/bootstap.md.
- "forward deployed": godspeed-compass (32 files, e.g. `4-GodSpeed-GTM-intell/handoff.md`); godspeed-growth/brand/voice/canonical-language.md; strategy-growth (32).
- "golden key": uscos/uscos-plugin/agents/AGENTS.md; uscos/uscos-plugin/agents/comparative-advantage.md; Strategist/.agents/specs/Strategy_Framework_tread.md.
- "playing to win": uscos/uscos-plugin/agents/strategic-compression.md; godspeed-growth/world/syntelligent-ai-coe.md; SEA/docs/specs/shared/reference/017-dialectical-emergence-engine.md.
- "JTBD" / "jobs to be done": 0 hits in all named dirs.
- "evidence gate": godspeed-compass/3-fde-market-proof-system/FDE-research.md; godspeed-compass/3-fde-market-proof-system/GodSpeed_Forward_Deployment_Playbook.md; Strategist/PLAN.md; SEA/.archive/1plans/plan-spec.md.
- "tax": mostly unrelated copy. SEA/docs/specs/shared/sds/016-semantic-debt-ledger-service.md; NeatCode/docs/subsystems/taxonomy-and-gates.md.

## 5. Relevance and exposed interfaces

| System | Relevance | Interfaces (OBSERVED unless marked) |
|---|---|---|
| godspeed-readiness | YES: release gate for pinned heads | `scripts/readiness.sh` env GSR_HEADS, GSR_BUILD_LOCK, GSR_PG0, GSR_DOMAINFORGE_BIN; evidence/REPORT.md; profiles.yaml |
| godspeed-compass | YES: FDE market research and settlement workflow | AGENTS.md read order; `.agents/CURRENT_STATUS.yml` (not read) |
| zeroclaw-sea-forge | YES: authority enforcement before high-impact actions | README "Three guarantees"; Rust crate |
| gs-term | YES: affordance/effect/evidence control plane | README; built on Cognate |
| hassos-addon-agent-memory-ledger | YES: governance, memory, event bridge substrate | README; pyproject.toml |
| SEA / sentient-enterprise-architecture | YES: semantic model and governance specs | docs/specs; `.sea/skills`; routing AGENTS.md |
| godspeed-growth / strategy-growth | MAYBE: FDE SOP and brand; strategy-growth duplicate | Markdown only |
| Strategist | MAYBE: decision and evidence records | Cargo + package.json; not read |
| ideal | MAYBE: audit method for idea artifacts | bin/ CLI, schema/, skills/ (not run) |
| uscos | MAYBE: strategy plugin | uscos-plugin agents (Markdown) |
| NeatCode, project-topology-architect, elephant-goldfish-model | MAYBE: planning and review plugins | .claude-plugin manifests, skills, agents |
| terminal-ui | MAYBE: terminal UI kit | package.json; registry.json |
| tree-sitter-sea | MAYBE: grammar for a `.sea` model (INFERRED) | grammar.js, queries/ |
| godspeed, NewSea, adlc | MAYBE: notes, generator, drafts; no clear lineage | none observed |
| hermes-agent-sea-forge | MAYBE: agent runtime fork; SEA link unverified | upstream Hermes (not read) |
| ast-grep-mcp-rs | NO: developer code-search tool | MCP server |
| XSEA | NO: unversioned generated snapshot | mcp/ dir |

## 6. Systems not named in the mission that matter

| System | Why |
|---|---|
| sea-rs (SEA-Forge, this repo) | Authority and governed-execution core. gsr sea-forge head 4e67dfca. |
| Cognate (+ cognate-workbench, cognate-horizon, cognate-identity-wt) | Pinned heads; gs-term is built on it; Workbench approval gate (blocker B-07). |
| Horizon (+ horizon-*, sea-rs-horizon) | Pinned head; "Horizon inspectability" lives here. Several branches. |
| DomainForge (`~/projects/domainforge`, main f90905a; domainforge-lsp, -vsc-extension) | Declared semantic model producer. Readiness expects it; not pinned in gsr-heads (B-02). |
| RealityTrace (`sxr`) | Pinned head. Name suggests observed-vs-declared comparison (INFERRED, not read). |
| Gauntlet, SWE_SEED (swe_seed) | Pinned heads; full-stack gates; digest fix (B-06). |
| Context-Kernel, CEP (canonical-evaluation-protocol) | Pinned heads; roles not read. |
| plugin-orchestrator, documenter | Tooling; relevance unverified. |

## Unknowns
- Whether strategy-growth and godspeed-growth working trees match.
- Whether gsr-heads should contain domainforge.
- Current status of blockers B-01 to B-03 and B-07.
- Interfaces of Strategist, uscos, ideal, Context-Kernel, RealityTrace not read beyond README or manifest.
