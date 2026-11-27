# GodSpeed branch and worktree inventory, 2026-11-16

Read-only recon. Nothing was checked out, committed, pushed, deleted, stashed, gc'd or pruned. Only `git fetch origin` (no --prune) was run, once per GodSpeedAI remote in its primary checkout.

Method and notes:
- Discovery: `.git` entries at depth <=4 under ~/projects (directories up to depth 3), plus ~/projects/_wt/* and ~/projects/gsr-heads/*. Worktrees are detected via git-dir vs git-common-dir.
- Remote branch metrics are against `origin/<default>` (default from `gh repo view`). Local branch metrics use each clone's own `origin/<default>` ref (refs may be stale in clones that were not fetched).
- `ahead` = commits on branch not on default; `behind` = commits on default not on branch; `unique_patches` = `git cherry` '+' lines; `unpushed` = commits not reachable from any `refs/remotes/origin/*` in that clone.
- Summary counts combine remote-branch rows and local-branch rows (local rows counted per clone). A branch in several clones is counted once per clone.
- Categories are mechanical: MERGED (ahead 0 or unique 0), OPEN-PR, UNIQUE, UNPUSHED, DEPENDABOT/RELEASE-BOT, KEEP-CANDIDATE (name matches website|landing|gh-pages).
- Non-GodSpeed remotes are listed separately without branch analysis.

## Summary (per GodSpeed/local repo)

| repo | remote_branches | local_branch_entries | worktrees | MERGED | UNIQUE | UNPUSHED | OPEN-PR | dirty_checkouts | stash_entries | fetch | default |
|---|---|---|---|---|---|---|---|---|---|---|---|
| gs:Cognate | 10 | 9 | 8 | 14 | 5 | 1 | 1 | 2 | 0 | ok | main |
| gs:Cognate-Workbench | 9 | 9 | 6 | 13 | 5 | 3 | 4 | 1 | 0 | ok | main |
| gs:Context-Kernel | 0 | 1 | 1 | 0 | 1 | 1 | 0 | 1 | 0 | ok | main |
| gs:DomainForge | 0 | 5 | 7 | 1 | 4 | 1 | 0 | 1 | 0 | ok | main |
| gs:Gauntlet | 21 | 8 | 6 | 15 | 14 | 1 | 12 | 0 | 0 | ok | main |
| gs:GodSpeed-Agent | 4 | 4 | 3 | 0 | 8 | 0 | 0 | 1 | 0 | ok | main |
| gs:Horizon | 25 | 28 | 14 | 35 | 18 | 10 | 4 | 2 | 0 | ok | main |
| gs:NeatCode | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | ok | main |
| gs:RealityTrace | 5 | 11 | 5 | 12 | 4 | 7 | 0 | 5 | 0 | ok | main |
| gs:SEA | 1 | 28 | 0 | 4 | 25 | 26 | 0 | 1 | 15 | ok | main |
| gs:SEA-Forge | 30 | 37 | 24 | 12 | 55 | 12 | 6 | 1 | 25 | ok | main |
| gs:Terminal-UI | 1 | 1 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | ok | main |
| gs:USCOS | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ok | main |
| gs:ast-grep-mcp-rs | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | ok | main |
| gs:canonical-evaluation-protocol | 2 | 2 | 2 | 2 | 2 | 0 | 0 | 1 | 0 | ok | main |
| gs:domainforge-lsp | 4 | 3 | 0 | 0 | 7 | 0 | 0 | 0 | 0 | ok | main |
| gs:domainforge-vsc-extension | 6 | 1 | 0 | 0 | 7 | 0 | 5 | 0 | 0 | ok | main |
| gs:godspeed-compass | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ok | main |
| gs:godspeed-growth | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ok | main |
| gs:gs-term | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ok | main |
| gs:hassos-addon-agent-memory-ledger | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | ok | main |
| gs:hermes-agent-sea-forge | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | FAILED | main |
| gs:ideal | 0 | 1 | 0 | 0 | 1 | 1 | 0 | 0 | 0 | ok | main |
| gs:project-topology-architect | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | ok | main |
| gs:swe_seed | 3 | 3 | 1 | 0 | 6 | 0 | 2 | 1 | 0 | ok | main |
| gs:zeroclaw-sea-forge | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | FAILED | master |
| local:. | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a |  |
| local:NewSea | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | n/a | master |
| local:adlc | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | n/a | main |
| local:demo-ws | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | n/a | main |
| local:godspeed-readiness | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | main |
| local:my_LLMLingua_project | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | n/a | main |
| local:owner-KT9igc | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | main |
| local:owner-w44OuL | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | main |
| local:owner-ycBVRK | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | main |
| local:sxr-repo | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | main |
| local:tree-sitter-sea | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | main |
| local:youcom-ext | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | n/a | main |

## Checkouts (GodSpeed and local-only)

| repo | path | kind | head | dirty | stash |
|---|---|---|---|---|---|
| local:sxr-repo | .data/sxr-repo | main | main | 0 | 0 |
| gs:Context-Kernel | Context_Kernel | main | main | 1 | 0 |
| gs:Horizon | Horizon | main | horizon-desk-v2 | 5 | 0 |
| gs:Horizon | Horizon-core | worktree | stage0b-containment-core | 10 | 0 |
| gs:Horizon | Horizon-gaps | worktree | horizon-desk-gaps | 0 | 0 |
| gs:Horizon | Horizon-merge | worktree | horizon-desk-v2-merge | 0 | 0 |
| gs:Horizon | Horizon-pr1 | worktree | stage0b-containment-pr1 | 0 | 0 |
| gs:Horizon | Horizon-pr1-narrow | worktree | stage0b-containment-pr1-narrow | 0 | 0 |
| gs:NeatCode | NeatCode | main | main | 0 | 0 |
| local:NewSea | NewSea | main | master | 20 | 0 |
| gs:SEA | SEA | main | dev | 1 | 15 |
| gs:swe_seed | SWE_SEED | main | feat/native-distribution | 1 | 0 |
| gs:SEA-Forge | _wt/arm64 | worktree | feat/arm64-jetson-packaging | 0 | 1 |
| gs:SEA-Forge | _wt/consol | worktree | feat/xplat-consolidate | 0 | 1 |
| local:demo-ws | _wt/demo-ws | main | main | 1 | 0 |
| gs:Cognate | _wt/df022-cognate | worktree | chore/domainforge-0.22.0 | 0 | 0 |
| gs:Cognate-Workbench | _wt/df022-cognate-workbench | worktree | chore/domainforge-0.22.0 | 0 | 0 |
| gs:Horizon | _wt/df022-horizon | worktree | chore/domainforge-0.22.0 | 0 | 0 |
| gs:SEA-Forge | _wt/df022-sea-rs | worktree | chore/domainforge-0.22.0 | 0 | 1 |
| gs:RealityTrace | _wt/df022-sxr | worktree | chore/domainforge-0.22.0 | 3 | 0 |
| gs:DomainForge | _wt/domainforge-npm | worktree | fix/npm-platform-packages | 0 | 0 |
| gs:DomainForge | _wt/domainforge-rel | worktree | DETACHED@1f03a29 | 0 | 0 |
| gs:canonical-evaluation-protocol | _wt/fde-cep-integration | worktree | fde/integration | 0 | 0 |
| gs:Cognate | _wt/fde-cognate-df0212 | worktree | fde/domainforge-0.21.2 | 0 | 0 |
| gs:Cognate | _wt/fde-cognate-integration | worktree | fde/integration | 0 | 0 |
| gs:Cognate | _wt/fde-cognate-pin-exception | worktree | fde/domainforge-pin-exception | 0 | 0 |
| gs:Cognate | _wt/fde-cognate-rt040 | worktree | fde/realitytrace-0.4.0 | 0 | 0 |
| gs:DomainForge | _wt/fde-df-integration | worktree | fde/integration | 0 | 0 |
| gs:DomainForge | _wt/fde-df-main-3960326 | worktree | DETACHED@3960326 | 0 | 0 |
| gs:DomainForge | _wt/fde-df-release | worktree | DETACHED@7139ccc | 0 | 0 |
| gs:DomainForge | _wt/fde-df-spike | worktree | fde/spike-wp10 | 0 | 0 |
| gs:Gauntlet | _wt/fde-gauntlet-integration | worktree | fde/integration | 0 | 0 |
| gs:GodSpeed-Agent | _wt/fde-godspeed-agent-integration | worktree | fde/integration | 0 | 0 |
| gs:GodSpeed-Agent | _wt/fde-gsa-baseline | worktree | DETACHED@a41c227 | 0 | 0 |
| gs:GodSpeed-Agent | _wt/fde-gsa-loop | worktree | DETACHED@4bfed12 | 0 | 0 |
| gs:Horizon | _wt/fde-horizon-cognate-091 | worktree | fde/cognate-0.9.1 | 0 | 0 |
| gs:Horizon | _wt/fde-horizon-integration | worktree | fde/integration | 0 | 0 |
| gs:Horizon | _wt/fde-horizon-repo-race | worktree | fde/fix-obligation-repo-init-race | 0 | 0 |
| gs:SEA-Forge | _wt/fde-sea-rs-df0212 | worktree | fde/domainforge-0.21.2 | 0 | 1 |
| gs:SEA-Forge | _wt/fde-sea-rs-e2e-loop | worktree | fde/fix-e2e-world-loop | 0 | 1 |
| gs:SEA-Forge | _wt/fde-sea-rs-integration | worktree | fde/integration | 0 | 1 |
| gs:SEA-Forge | _wt/fde-sea-rs-planning | worktree | fde/planning-baseline | 0 | 1 |
| gs:SEA-Forge | _wt/fde-sea-rs-rustsec | worktree | fde/rustsec-2026-0459-0486 | 0 | 1 |
| gs:RealityTrace | _wt/fde-sxr-ingest-json | worktree | fde/fix-ingest-json-spelling | 4 | 0 |
| gs:RealityTrace | _wt/fde-sxr-integration | worktree | fde/integration | 4 | 0 |
| gs:RealityTrace | _wt/fde-sxr-release-sums | worktree | fde/fix-release-sha256sums | 3 | 0 |
| gs:Cognate-Workbench | _wt/fde-workbench-cognate-091 | worktree | fde/cognate-0.9.1 | 0 | 0 |
| gs:Cognate-Workbench | _wt/fde-workbench-integration | worktree | fde/integration | 0 | 0 |
| gs:Cognate-Workbench | _wt/fde-workbench-rt040 | worktree | fde/realitytrace-0.4.0 | 0 | 0 |
| gs:Gauntlet | _wt/gauntlet-ci | worktree | ci/complete-gates | 0 | 0 |
| gs:Gauntlet | _wt/gauntlet-native | worktree | wt/gauntlet-native | 0 | 0 |
| gs:Gauntlet | _wt/gauntlet-prime | worktree | feat/prime-agent-optional | 0 | 0 |
| gs:Gauntlet | _wt/gauntlet-qual | worktree | ci/release-qualification | 0 | 0 |
| gs:SEA-Forge | _wt/harvest | worktree | feat/hardening-capability-harvest | 0 | 1 |
| gs:Horizon | _wt/horizon-ci | worktree | feat/ci | 0 | 0 |
| gs:Horizon | _wt/horizon-desktop | worktree | feat/desktop-tauri | 0 | 0 |
| gs:SEA-Forge | _wt/macjail | worktree | feat/macos-app-sandbox-jail | 0 | 1 |
| gs:SEA-Forge | _wt/macsb | worktree | ci/macos-app-sandbox-probe | 0 | 1 |
| gs:SEA-Forge | _wt/oscap | worktree | feat/openshell-capability | 0 | 1 |
| gs:SEA-Forge | _wt/oslive | worktree | ci/openshell-live | 0 | 1 |
| gs:SEA-Forge | _wt/oswin | worktree | ci/openshell-windows | 0 | 1 |
| gs:SEA-Forge | _wt/pkg | worktree | feat/native-packages | 0 | 1 |
| gs:SEA-Forge | _wt/r1osb | worktree | fix/review1-sandbox | 0 | 1 |
| gs:SEA-Forge | _wt/r1win | worktree | fix/review1-windows | 0 | 1 |
| gs:SEA-Forge | _wt/r2s2 | worktree | fix/review2-sandbox | 0 | 1 |
| gs:SEA-Forge | _wt/r2w2 | worktree | fix/review2-windows | 0 | 1 |
| gs:DomainForge | _wt/tmp/df022-fix | main | DETACHED@ac8c421 | 0 | 0 |
| local:. | _wt/tmp/gauntlet-workspace-fs-unit-seed-basic-project-280220 | main | DETACHED@ | 0 | 0 |
| local:owner-KT9igc | _wt/tmp/owner-KT9igc | main | main | 0 | 0 |
| local:owner-w44OuL | _wt/tmp/owner-w44OuL | main | main | 0 | 0 |
| local:owner-ycBVRK | _wt/tmp/owner-ycBVRK | main | main | 0 | 0 |
| gs:SEA-Forge | _wt/wintests | worktree | feat/xplat-win-sandbox-tests | 0 | 1 |
| gs:SEA-Forge | _wt/xhard | worktree | feat/xplat-hardening | 0 | 1 |
| local:adlc | adlc | main | main | 4 | 0 |
| gs:ast-grep-mcp-rs | ast-grep-mcp-rs | main | main | 1 | 0 |
| gs:canonical-evaluation-protocol | cep | main | profiles/context-bundle | 1 | 0 |
| gs:Cognate | cognate | main | security/repo-hygiene-2026-10-31 | 0 | 0 |
| gs:Cognate | cognate-horizon | worktree | feat/realitytrace-real-snapshot | 2 | 0 |
| gs:Cognate | cognate-identity-wt | worktree | horizon/shared-identity | 1 | 0 |
| gs:Cognate-Workbench | cognate-workbench | main | feat/desktop-terminal-browser | 37 | 0 |
| gs:Cognate-Workbench | cognate-workbench-horizon | worktree | feat/horizon-connect | 0 | 0 |
| local:sxr-repo | cognate-workbench/.data/sxr-repo | main | main | 0 | 0 |
| gs:DomainForge | domainforge | main | main | 6 | 0 |
| gs:domainforge-lsp | domainforge-lsp | main | semantic-adapter | 0 | 0 |
| gs:domainforge-vsc-extension | domainforge-vsc-extension | main | dev | 0 | 0 |
| gs:Gauntlet | gauntlet | main | release/native-distribution | 0 | 0 |
| gs:godspeed-compass | godspeed-compass | main | main | 0 | 0 |
| gs:godspeed-growth | godspeed-growth | main | main | 0 | 0 |
| local:godspeed-readiness | godspeed-readiness | main | main | 0 | 0 |
| gs:GodSpeed-Agent | godspeed_agent | main | migration/cep-world-required-evidence | 2 | 0 |
| gs:gs-term | gs-term | main | main | 0 | 0 |
| gs:canonical-evaluation-protocol | gsr-heads/cep | worktree | DETACHED@5111ec3 | 0 | 0 |
| gs:Cognate | gsr-heads/cognate | worktree | DETACHED@ca74ab5 | 0 | 0 |
| gs:Cognate-Workbench | gsr-heads/cognate-workbench | worktree | DETACHED@bd4c0ea | 0 | 0 |
| gs:Context-Kernel | gsr-heads/context-kernel | worktree | DETACHED@36bd8c8 | 0 | 0 |
| gs:DomainForge | gsr-heads/domainforge | worktree | DETACHED@ac8c421 | 0 | 0 |
| gs:Gauntlet | gsr-heads/gauntlet | worktree | DETACHED@0e185aa | 0 | 0 |
| gs:Horizon | gsr-heads/horizon | worktree | DETACHED@4242d31 | 0 | 0 |
| gs:RealityTrace | gsr-heads/realitytrace | worktree | DETACHED@521d363 | 0 | 0 |
| gs:SEA-Forge | gsr-heads/sea-forge | worktree | DETACHED@7c083c86 | 0 | 1 |
| gs:swe_seed | gsr-heads/swe-seed | worktree | DETACHED@232e02d | 0 | 0 |
| gs:hassos-addon-agent-memory-ledger | hassos-addon-agent-memory-ledger | main | main | 0 | 0 |
| gs:hermes-agent-sea-forge | hermes-agent-sea-forge | main | main | 0 | 0 |
| gs:Horizon | horizon-connector | worktree | feat/github-connector | 0 | 0 |
| gs:Horizon | horizon-seed | worktree | feat/seed | 0 | 0 |
| gs:ideal | ideal | main | main | 0 | 0 |
| local:my_LLMLingua_project | my_LLMLingua_project | main | main | 13 | 0 |
| gs:project-topology-architect | project-topology-architect | main | main | 0 | 0 |
| gs:SEA-Forge | sea-rs | main | docs/egm-bootstrap-readmes | 3 | 1 |
| gs:SEA-Forge | sea-rs-fix | worktree | fix/server-ambiguous-declare-keeps-lease | 0 | 1 |
| gs:SEA-Forge | sea-rs-horizon | worktree | feat/horizon-desk-support | 0 | 1 |
| gs:SEA | sentient-enterprise-architecture | main | main | 0 | 0 |
| gs:godspeed-growth | strategy-growth | main | main | 0 | 0 |
| gs:RealityTrace | sxr | main | feat/native-binary-distribution | 1 | 0 |
| gs:Terminal-UI | terminal-ui | main | remediation/audit-2026-09-29 | 0 | 0 |
| local:tree-sitter-sea | tree-sitter-sea | main | main | 0 | 0 |
| gs:USCOS | uscos | main | main | 0 | 0 |
| local:youcom-ext | youcom-ext | main | main | 13 | 0 |
| gs:zeroclaw-sea-forge | zeroclaw-sea-forge | main | master | 0 | 0 |

## Non-GodSpeed checkouts (listed only)

| remote | path | kind | head | dirty | stash |
|---|---|---|---|---|---|
| nongs:SPRIME01/StrategyNotes | Strategist | main | main | 0 | 0 |
| nongs:SPRIME01/WarpRoom | WarpRoom | main | master | 1 | 0 |
| nongs:SPRIME01/Weople | Weople | main | main | 0 | 1 |
| nongs:SPRIME01/Actual | actual | main | main | 0 | 0 |
| nongs:SPRIME01/antigravity-claude-proxy | antigravity-claude-proxy | main | main | 1 | 0 |
| nongs:finos/architecture-as-code | architecture-as-code | main | main | 1 | 0 |
| nongs:karpathy/autoresearch | autoresearch | main | master | 0 | 0 |
| nongs:SPRIME01/documenter | documenter | main | main | 0 | 0 |
| nongs:SPRIME01/dotfiles | dotfiles-legacy-recovered | main | main | 3 | 0 |
| nongs:SPRIME01/edgeai | edgeai | main | dev | 0 | 1 |
| nongs:google/elephant-goldfish-model | elephant-goldfish-model | main | main | 0 | 0 |
| nongs:SPRIME01/Gauntlet-Game-Studio | gauntlet-game-studio | main | main | 0 | 0 |
| nongs:gitleaks/gitleaks | gitleaks | main | master | 0 | 0 |
| nongs:SPRIME01/glengarry | glengarry | main | feature/house-reconstruction-v1 | 0 | 0 |
| nongs:SPRIME01/gspeedai | godspeed | main | main | 8 | 0 |
| nongs:trailhq/Graft | graft | main | local/context-documents | 0 | 0 |
| nongs:trailhq/Graft | graft-v0.29 | worktree | local/context-documents-v0.29 | 8 | 0 |
| nongs:pgadmin-org/pgagent | hassos-addon-agent-memory-ledger/agent_memory_ledger/pgagent | main | master | 0 | 0 |
| nongs:SPRIME01/home-assistant-config | home-assistant-config | main | main | 15 | 0 |
| nongs:SPRIME01/homelab | homelab | main | dev | 9 | 0 |
| nongs:SPRIME01/local-dev-mcp | local-dev-mcp | main | main | 0 | 0 |
| nongs:SPRIME01/NexusNotes | nexusnotes | main | main | 10 | 0 |
| nongs:SPRIME01/PublishingHouse | novels | main | main | 163 | 0 |
| nongs:SPRIME01/oh-my-openagent | oh-my-openagent | main | dev | 15 | 0 |
| nongs:SPRIME01/open-design | open-design | main | main | 0 | 0 |
| nongs:nexu-io/open-design | open-design-tools-recovered | main | main | 3 | 0 |
| nongs:openclaw/openclaw | openclaw | main | main | 0 | 0 |
| nongs:SPRIME01/osiris | osiris | main | main | 0 | 0 |
| nongs:SPRIME01/plugin-orchestrator | plugin-orchestrator | main | main | 0 | 0 |
| nongs:DietrichGebert/ponytail | ponytail | main | main | 21 | 0 |
| nongs:SPRIME01/Product-Actualizer | product-actualizer | main | main | 2 | 0 |
| nongs:SPRIME01/project-template | project-template | main | main | 0 | 0 |
| nongs:Jakeschincariol/replica-skill | replica-skill | main | main | 0 | 0 |
| nongs:SPRIME01/resume-tailor | resume-tailor | main | main | 20 | 0 |
| nongs:SPRIME01/setup-jetson | setup-jetson | main | main | 0 | 0 |
| nongs:SPRIME01/You.com-Agents-Extention | youcom_vscode_extension/you-com-agents | main | main | 28 | 0 |
| nongs:zeroclaw-labs/zeroclaw | zeroclaw | main | main | 0 | 0 |

## Per-repo branches and tags

### gs:Cognate

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| chore/domainforge-0.22.0 | 2026-11-14 | Sam Prime | 24 | 1 | 0 | #82 MERGED | MERGED |
| chore/domainforge-0.22.2 | 2026-11-15 | github-actions[bot] | 4 | 0 | 0 | #85 MERGED | MERGED |
| fde/domainforge-0.21.2 | 2026-11-13 | github-actions[bot] | 29 | 1 | 0 | #77 MERGED | MERGED |
| fde/domainforge-pin-exception | 2026-11-13 | github-actions[bot] | 28 | 2 | 2 | #78 MERGED | UNIQUE |
| fde/integration | 2026-11-15 | github-actions[bot] | 7 | 0 | 0 | #83 MERGED | MERGED |
| fde/realitytrace-0.4.0 | 2026-11-13 | github-actions[bot] | 25 | 1 | 0 | #80 MERGED | MERGED |
| feat/realitytrace-real-snapshot | 2026-11-07 | github-actions[bot] | 28 | 0 | 0 | #76 MERGED | MERGED |
| horizon/shared-identity | 2026-11-02 | github-actions[bot] | 30 | 0 | 0 |  | MERGED |
| release-please--branches--main | 2026-11-16 | github-actions[bot] | 0 | 1 | 1 | #86 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| release-please--branches--main--release-notes | 2026-11-16 | github-actions[bot] | 0 | 1 | 1 |  | UNIQUE DEPENDABOT/RELEASE-BOT |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/df022-cognate | chore/domainforge-0.22.0 | yes | origin/chore/domainforge-0.22.0 | 1 | 24 | 0 | 0 | #82 MERGED | MERGED |
| _wt/df022-cognate | fde/domainforge-0.21.2 | yes | origin/fde/domainforge-0.21.2 | 1 | 29 | 0 | 0 | #77 MERGED | MERGED |
| _wt/df022-cognate | fde/domainforge-pin-exception | yes | origin/fde/domainforge-pin-exception | 2 | 28 | 2 | 0 | #78 MERGED | UNIQUE |
| _wt/df022-cognate | fde/integration | yes | origin/main | 0 | 7 | 0 | 0 | #83 MERGED | MERGED |
| _wt/df022-cognate | fde/realitytrace-0.4.0 | yes | origin/fde/realitytrace-0.4.0 | 1 | 25 | 0 | 0 | #80 MERGED | MERGED |
| _wt/df022-cognate | feat/realitytrace-real-snapshot | yes | origin/feat/realitytrace-real-snapshot | 0 | 28 | 0 | 0 | #76 MERGED | MERGED |
| _wt/df022-cognate | horizon/shared-identity | yes | - | 0 | 30 | 0 | 0 |  | MERGED |
| _wt/df022-cognate | pr-70 | no-remote-branch | - | 255 | 301 | 35 | 255 |  | UNIQUE UNPUSHED |
| _wt/df022-cognate | security/repo-hygiene-2026-10-31 | no-remote-branch | origin/security/repo-hygiene-2026-10-31 | 0 | 31 | 0 | 0 | #74 MERGED | MERGED |

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| _wt/df022-cognate | checkpoint/pre-design-20261022 | abfa417544a3 |
| _wt/df022-cognate | checkpoint/pre-readiness-merge-20261022 | a477933a1ddf |

### gs:Cognate-Workbench

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| chore/domainforge-0.22.0 | 2026-11-14 | Sam Prime | 10 | 1 | 0 | #34 MERGED | MERGED |
| fde/cognate-0.9.1 | 2026-11-13 | Sam Prime | 12 | 1 | 0 | #32 MERGED | MERGED |
| fde/integration | 2026-11-15 | Sam Prime | 1 | 0 | 0 | #35 MERGED | MERGED |
| fde/realitytrace-0.4.0 | 2026-11-13 | Sam Prime | 11 | 1 | 0 | #33 MERGED | MERGED |
| feat/desktop-terminal-browser | 2026-11-08 | Sam Prime | 20 | 121 | 121 | #28 OPEN | OPEN-PR UNIQUE |
| feat/ui-production-readiness | 2026-11-01 | Sam Prime | 20 | 49 | 49 | #27 OPEN | OPEN-PR UNIQUE |
| feature/expert-teams | 2026-10-18 | Sam Prime | 118 | 0 | 0 |  | MERGED |
| fix/live-test-domainforge-version | 2026-11-05 | Sam Prime | 13 | 1 | 0 | #31 MERGED | MERGED |
| test/red-session-workspace-task-contracts-20261020 | 2026-10-30 | Sam Prime | 80 | 0 | 0 | #16 MERGED | MERGED |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/df022-cognate-workbench | chore/domainforge-0.22.0 | yes | origin/chore/domainforge-0.22.0 | 1 | 10 | 0 | 0 | #34 MERGED | MERGED |
| _wt/df022-cognate-workbench | ci/install-pinned-domainforge | no-remote-branch | origin/ci/install-pinned-domainforge | 1 | 101 | 0 | 1 | #6 MERGED | MERGED UNPUSHED |
| _wt/df022-cognate-workbench | fde/cognate-0.9.1 | yes | origin/fde/cognate-0.9.1 | 1 | 12 | 0 | 0 | #32 MERGED | MERGED |
| _wt/df022-cognate-workbench | fde/integration | yes | origin/main | 0 | 1 | 0 | 0 | #35 MERGED | MERGED |
| _wt/df022-cognate-workbench | fde/realitytrace-0.4.0 | yes | origin/fde/realitytrace-0.4.0 | 1 | 11 | 0 | 0 | #33 MERGED | MERGED |
| _wt/df022-cognate-workbench | feat/desktop-terminal-browser | no | origin/feat/desktop-terminal-browser | 140 | 20 | 140 | 19 | #28 OPEN | OPEN-PR UNIQUE UNPUSHED |
| _wt/df022-cognate-workbench | feat/horizon-connect | no-remote-branch | - | 75 | 20 | 72 | 11 |  | UNIQUE UNPUSHED |
| _wt/df022-cognate-workbench | feat/ui-production-readiness | yes | origin/feat/ui-production-readiness | 49 | 20 | 49 | 0 | #27 OPEN | OPEN-PR UNIQUE |
| _wt/df022-cognate-workbench | fix/live-test-domainforge-version | yes | origin/fix/live-test-domainforge-version | 1 | 13 | 0 | 0 | #31 MERGED | MERGED |

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| _wt/df022-cognate-workbench | checkpoint/pre-design-20261022 | 3667c28f0d51 |
| _wt/df022-cognate-workbench | checkpoint/pre-readiness-merge-20261022 | 770b4db07973 |

### gs:Context-Kernel

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| Context_Kernel | backup/harden-pre-sync | no-remote-branch | - | 1 | 37 | 1 | 1 |  | UNIQUE UNPUSHED |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:DomainForge

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/domainforge-npm | fde/integration | yes | origin/fde/integration | 6 | 6 | 6 | 0 | #171 MERGED | UNIQUE |
| _wt/domainforge-npm | fde/spike-wp10 | yes | origin/fde/spike-wp10 | 1 | 6 | 1 | 0 |  | UNIQUE |
| _wt/domainforge-npm | fix/npm-platform-packages | yes | origin/fix/npm-platform-packages | 2 | 4 | 2 | 0 | #173 MERGED | UNIQUE |
| _wt/domainforge-npm | fix/npm-publish-local-path | yes | origin/fix/npm-publish-local-path | 1 | 2 | 0 | 0 | #175 MERGED | MERGED |
| _wt/domainforge-npm | rel-0.22.1 | no-remote-branch | origin/release-please--branches--main--components--domainforge | 2 | 3 | 2 | 2 |  | UNIQUE UNPUSHED |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:Gauntlet

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| backup/jollimemory-summaries-v3-20261023 | 2026-08-15 | Sam Prime | 287 | 7 | 7 |  | UNIQUE |
| backup/local-main-20261023 | 2026-10-11 | Sam Prime | 59 | 0 | 0 |  | MERGED |
| chore/release-0.1.1 | 2026-11-16 | Sam Prime | 1 | 1 | 0 | #24 MERGED | MERGED |
| ci/complete-gates | 2026-11-15 | Sam Prime | 20 | 0 | 0 | #8 MERGED | MERGED |
| ci/release-qualification | 2026-11-15 | Sam Prime | 3 | 0 | 0 | #22 MERGED | MERGED |
| dependabot/bun/workbench/multi-b1658bdecd | 2026-11-15 | dependabot[bot] | 19 | 1 | 1 | #20 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/bun/workbench/opentui/core-0.8.4 | 2026-11-15 | dependabot[bot] | 19 | 1 | 1 | #16 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/bun/workbench/opentui/react-0.8.4 | 2026-11-15 | dependabot[bot] | 19 | 1 | 1 | #12 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/bun/workbench/react-reconciler-0.34.0 | 2026-11-15 | dependabot[bot] | 19 | 1 | 1 | #18 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/bun/workbench/types/bun-1.4.4 | 2026-11-15 | dependabot[bot] | 19 | 1 | 1 | #13 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/cargo/sha2-0.11.0 | 2026-11-16 | dependabot[bot] | 0 | 1 | 1 | #15 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/cargo/toml-1.2.1spec-1.1.0 | 2026-11-16 | dependabot[bot] | 0 | 1 | 1 | #10 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/actions/attest-build-provenance-4.2.2 | 2026-11-15 | dependabot[bot] | 2 | 1 | 1 | #17 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/actions/checkout-7.0.1 | 2026-11-15 | dependabot[bot] | 2 | 1 | 1 | #14 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/actions/download-artifact-8.0.2 | 2026-11-15 | dependabot[bot] | 2 | 1 | 1 | #9 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/actions/upload-artifact-7.0.2 | 2026-11-15 | dependabot[bot] | 2 | 1 | 1 | #19 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/extractions/setup-just-4.0.0 | 2026-11-15 | dependabot[bot] | 19 | 1 | 1 | #11 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| fde/integration | 2026-11-13 | Sam Prime | 29 | 1 | 0 |  | MERGED |
| feat/prime-agent-optional | 2026-11-15 | Sam Prime | 14 | 0 | 0 | #21 MERGED | MERGED |
| fix/qualify-runs-after-skipped-package | 2026-11-16 | Sam Prime | 2 | 1 | 0 | #23 MERGED | MERGED |
| release/native-distribution | 2026-11-15 | Sam Prime | 25 | 0 | 0 | #7 MERGED | MERGED |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/fde-gauntlet-integration | ci/complete-gates | yes | origin/ci/complete-gates | 0 | 20 | 0 | 0 | #8 MERGED | MERGED |
| _wt/fde-gauntlet-integration | ci/release-qualification | yes | origin/ci/release-qualification | 0 | 3 | 0 | 0 | #22 MERGED | MERGED |
| _wt/fde-gauntlet-integration | fde/integration | yes | origin/fde/integration | 1 | 29 | 0 | 0 |  | MERGED |
| _wt/fde-gauntlet-integration | feat/prime-agent-optional | yes | origin/feat/prime-agent-optional | 0 | 14 | 0 | 0 | #21 MERGED | MERGED |
| _wt/fde-gauntlet-integration | fix/test-schema-teardown | no-remote-branch | origin/fix/test-schema-teardown | 1 | 42 | 0 | 1 | #6 MERGED | MERGED UNPUSHED |
| _wt/fde-gauntlet-integration | jollimemory/summaries/v3 | no-remote-branch | - | 7 | 287 | 7 | 0 |  | UNIQUE |
| _wt/fde-gauntlet-integration | release/native-distribution | no | origin/release/native-distribution | 0 | 29 | 0 | 0 | #7 MERGED | MERGED |
| _wt/fde-gauntlet-integration | wt/gauntlet-native | no-remote-branch | origin/release/native-distribution | 0 | 25 | 0 | 0 |  | MERGED |

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| _wt/fde-gauntlet-integration | checkpoint/pre-design-20261022 | 7209153625ef |
| _wt/fde-gauntlet-integration | checkpoint/pre-integrate-20261023 | 7209153625ef |

### gs:GodSpeed-Agent

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| audit-corrections/neatcode-2026-07-29 | 2026-10-05 | SPRIME01 | 0 | 33 | 32 |  | UNIQUE |
| codex/gsa-v1-settlement-events-lfs | 2026-07-02 | SPrime01 | 90 | 86 | 3 |  | UNIQUE |
| fde/integration | 2026-11-13 | Sam Prime | 0 | 34 | 32 |  | UNIQUE |
| migration/cep-world-required-evidence | 2026-10-05 | Sam Prime | 0 | 33 | 32 | #2 MERGED | UNIQUE |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/fde-godspeed-agent-integration | audit-corrections/neatcode-2026-07-29 | no | origin/audit-corrections/neatcode-2026-07-29 | 29 | 0 | 29 | 0 |  | UNIQUE |
| _wt/fde-godspeed-agent-integration | fde/integration | yes | origin/fde/integration | 34 | 0 | 32 | 0 |  | UNIQUE |
| _wt/fde-godspeed-agent-integration | feature/enterprise-runtime | no-remote-branch | - | 16 | 90 | 1 | 0 |  | UNIQUE |
| _wt/fde-godspeed-agent-integration | migration/cep-world-required-evidence | yes | origin/migration/cep-world-required-evidence | 33 | 0 | 32 | 0 | #2 MERGED | UNIQUE |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:Horizon

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| chore/cognate-0.10.0 | 2026-11-15 | Sam Prime | 12 | 0 | 0 | #42 MERGED | MERGED |
| chore/domainforge-0.22.0 | 2026-11-14 | Sam Prime | 204 | 1 | 0 | #31 MERGED | MERGED |
| chore/domainforge-0.22.2 | 2026-11-15 | Sam Prime | 14 | 0 | 0 | #41 MERGED | MERGED |
| chore/gateway-unit-killmode | 2026-11-05 | Sam Prime | 349 | 1 | 0 | #26 MERGED | MERGED |
| ci/codeql-upload | 2026-11-15 | Sam Prime | 9 | 0 | 0 | #44 MERGED | MERGED |
| ci/desktop-install-qualification | 2026-11-15 | Sam Prime | 6 | 0 | 0 | #43 MERGED | MERGED |
| dependabot/bun/mcp-b/webmcp-polyfill-6.2.0 | 2026-11-16 | dependabot[bot] | 11 | 1 | 1 | #39 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/bun/multi-75c0a8599d | 2026-11-16 | dependabot[bot] | 11 | 1 | 1 | #34 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/bun/typescript-7.0.2 | 2026-11-15 | dependabot[bot] | 28 | 1 | 1 | #38 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/bun/yaml-2.9.1 | 2026-11-15 | dependabot[bot] | 31 | 0 | 0 | #35 MERGED | MERGED DEPENDABOT/RELEASE-BOT |
| dependabot/bun/zod-4.6.7 | 2026-11-16 | dependabot[bot] | 11 | 1 | 1 | #36 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/uv/research/pytest-9.2.0 | 2026-11-15 | dependabot[bot] | 31 | 0 | 0 | #37 MERGED | MERGED DEPENDABOT/RELEASE-BOT |
| fde/cognate-0.9.1 | 2026-11-13 | Sam Prime | 205 | 1 | 0 | #30 MERGED | MERGED |
| fde/fix-obligation-repo-init-race | 2026-11-13 | Sam Prime | 206 | 1 | 0 | #29 MERGED | MERGED |
| fde/integration | 2026-11-15 | Sam Prime | 43 | 0 | 0 | #32 MERGED | MERGED |
| feat/ci | 2026-11-15 | Sam Prime | 33 | 0 | 0 | #33 MERGED | MERGED |
| feat/desktop-tauri | 2026-11-15 | Sam Prime | 21 | 0 | 0 | #40 MERGED | MERGED |
| fix/followups-19-21-24 | 2026-11-05 | Sam Prime | 350 | 4 | 4 | #25 MERGED | UNIQUE |
| fix/macos-app-asset-name | 2026-11-16 | Sam Prime | 2 | 1 | 0 | #48 MERGED | MERGED |
| horizon-cockpit-projection-review | 2026-11-05 | SPRIME01 | 305 | 3 | 2 | #18 MERGED | UNIQUE |
| horizon-desk-gaps | 2026-11-12 | Sam Prime | 56 | 0 | 0 |  | MERGED |
| horizon-desk-v2-merge | 2026-11-07 | Sam Prime | 207 | 0 | 0 | #28 MERGED | MERGED |
| release-please--branches--main--components--horizon | 2026-11-16 | Sam Prime | 1 | 2 | 1 | #49 MERGED | UNIQUE DEPENDABOT/RELEASE-BOT |
| settlement-approval-gate | 2026-11-03 | Sam Prime | 355 | 2 | 2 | #27 MERGED | UNIQUE |
| settlement-external-approval-gate | 2026-11-03 | Sam Prime | 308 | 1 | 1 | #15 CLOSED | UNIQUE |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| Horizon | chore/domainforge-0.22.0 | yes | origin/chore/domainforge-0.22.0 | 1 | 204 | 0 | 0 | #31 MERGED | MERGED |
| Horizon | chore/drop-unused-identity-tarball | no-remote-branch | origin/chore/drop-unused-identity-tarball | 0 | 350 | 0 | 0 | #23 MERGED | MERGED |
| Horizon | chore/gateway-unit-killmode | yes | origin/chore/gateway-unit-killmode | 1 | 349 | 0 | 0 | #26 MERGED | MERGED |
| Horizon | fde/cognate-0.9.1 | yes | origin/fde/cognate-0.9.1 | 1 | 205 | 0 | 0 | #30 MERGED | MERGED |
| Horizon | fde/fix-obligation-repo-init-race | yes | origin/fde/fix-obligation-repo-init-race | 1 | 206 | 0 | 0 | #29 MERGED | MERGED |
| Horizon | fde/integration | yes | origin/main | 0 | 43 | 0 | 0 | #32 MERGED | MERGED |
| Horizon | feat/ci | yes | origin/feat/ci | 0 | 33 | 0 | 0 | #33 MERGED | MERGED |
| Horizon | feat/desktop-tauri | yes | origin/feat/desktop-tauri | 0 | 21 | 0 | 0 | #40 MERGED | MERGED |
| Horizon | feat/github-connector | no-remote-branch | - | 0 | 272 | 0 | 0 |  | MERGED |
| Horizon | feat/governed-operate | no-remote-branch | origin/feat/governed-operate | 1 | 372 | 0 | 1 | #1 MERGED | MERGED UNPUSHED |
| Horizon | feat/ignition-onboarding | no-remote-branch | origin/feat/ignition-onboarding | 1 | 364 | 0 | 1 | #10 MERGED | MERGED UNPUSHED |
| Horizon | feat/seed | no-remote-branch | - | 0 | 285 | 0 | 0 |  | MERGED |
| Horizon | fix/followups-19-21-24 | yes | origin/fix/followups-19-21-24 | 4 | 350 | 4 | 0 | #25 MERGED | UNIQUE |
| Horizon | fix/gateway-socket-dir-at-boot | no-remote-branch | origin/fix/gateway-socket-dir-at-boot | 1 | 355 | 0 | 1 | #16 MERGED | MERGED UNPUSHED |
| Horizon | fix/j02r2-rig-operator-ref | no-remote-branch | origin/fix/j02r2-rig-operator-ref | 1 | 355 | 0 | 1 | #17 MERGED | MERGED UNPUSHED |
| Horizon | fix/settlement-pending-error-class | no-remote-branch | origin/fix/settlement-pending-error-class | 1 | 363 | 0 | 1 | #11 MERGED | MERGED UNPUSHED |
| Horizon | horizon-cockpit-projection-review | no | origin/horizon-cockpit-projection-review | 0 | 308 | 0 | 0 | #18 MERGED | MERGED |
| Horizon | horizon-desk-gaps | yes | origin/main | 0 | 56 | 0 | 0 |  | MERGED |
| Horizon | horizon-desk-v2 | no-remote-branch | - | 0 | 224 | 0 | 0 |  | MERGED |
| Horizon | horizon-desk-v2-merge | yes | origin/horizon-desk-v2-merge | 0 | 207 | 0 | 0 | #28 MERGED | MERGED |
| Horizon | settlement-approval-gate | yes | origin/main | 2 | 355 | 2 | 0 | #27 MERGED | UNIQUE |
| Horizon | settlement-external-approval-gate | yes | origin/settlement-external-approval-gate | 1 | 308 | 1 | 0 | #15 CLOSED | UNIQUE |
| Horizon | stage0b-containment-core | no-remote-branch | origin/main | 3 | 206 | 3 | 3 |  | UNIQUE UNPUSHED |
| Horizon | stage0b-containment-pr1 | no-remote-branch | - | 3 | 206 | 3 | 3 |  | UNIQUE UNPUSHED |
| Horizon | stage0b-containment-pr1-narrow | no-remote-branch | origin/main | 4 | 206 | 4 | 4 |  | UNIQUE UNPUSHED |
| Horizon | stage0b-pr1-broad-snapshot | no-remote-branch | - | 3 | 206 | 3 | 3 |  | UNIQUE UNPUSHED |
| Horizon | stage0b-pr1-narrowing-wip | no-remote-branch | - | 1 | 206 | 1 | 1 |  | UNIQUE UNPUSHED |
| Horizon | t1/reconcile-main | no-remote-branch | - | 2 | 305 | 1 | 0 |  | UNIQUE |

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| Horizon | checkpoint/2026-10-30-pre-economics-view | 55f1b4261a06 |
| Horizon | checkpoint/pre-design-20261022 | 8dc6c2e2ed62 |
| Horizon | checkpoint/pre-rebase-settlement-20261023 | 643c1fadea50 |
| Horizon | checkpoint/pre-spec-1.1.0-20261023 | 6fa506399577 |

### gs:NeatCode

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| release-please--branches--main--components--neatcode | 2026-09-22 | SPRIME01 | 0 | 1 | 1 | #5 MERGED | UNIQUE DEPENDABOT/RELEASE-BOT |

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:RealityTrace

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| chore/domainforge-0.22.0 | 2026-11-14 | Sam Prime | 12 | 1 | 0 | #16 MERGED | MERGED |
| fde/fix-ingest-json-spelling | 2026-11-13 | Sam Prime | 15 | 2 | 1 | #14 MERGED | UNIQUE |
| fde/fix-release-sha256sums | 2026-11-13 | Sam Prime | 13 | 1 | 0 | #15 MERGED | MERGED |
| fde/integration | 2026-11-15 | Sam Prime | 3 | 0 | 0 | #18 MERGED | MERGED |
| release-please--branches--main | 2026-11-15 | github-actions[bot] | 1 | 0 | 0 | #19 MERGED | MERGED DEPENDABOT/RELEASE-BOT |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/df022-sxr | chore/domainforge-0.21.1 | no-remote-branch | origin/chore/domainforge-0.21.1 | 1 | 23 | 0 | 1 | #7 MERGED | MERGED UNPUSHED |
| _wt/df022-sxr | chore/domainforge-0.22.0 | yes | origin/chore/domainforge-0.22.0 | 1 | 12 | 0 | 0 | #16 MERGED | MERGED |
| _wt/df022-sxr | fde/fix-ingest-json-spelling | yes | origin/fde/fix-ingest-json-spelling | 2 | 15 | 1 | 0 | #14 MERGED | UNIQUE |
| _wt/df022-sxr | fde/fix-release-sha256sums | yes | origin/fde/fix-release-sha256sums | 1 | 13 | 0 | 0 | #15 MERGED | MERGED |
| _wt/df022-sxr | fde/integration | yes | origin/fde/integration | 0 | 3 | 0 | 0 | #18 MERGED | MERGED |
| _wt/df022-sxr | feat/command-verifier | no-remote-branch | origin/feat/command-verifier | 1 | 20 | 0 | 1 | #9 MERGED | MERGED UNPUSHED |
| _wt/df022-sxr | feat/native-binary-distribution | no-remote-branch | origin/feat/native-binary-distribution | 8 | 17 | 8 | 8 | #12 MERGED | UNIQUE UNPUSHED |
| _wt/df022-sxr | feat/verifier-completed-at | no-remote-branch | origin/feat/verifier-completed-at | 1 | 19 | 0 | 1 | #10 MERGED | MERGED UNPUSHED |
| _wt/df022-sxr | fix/bundled-sqlite | no-remote-branch | origin/fix/bundled-sqlite | 1 | 21 | 0 | 1 | #8 MERGED | MERGED UNPUSHED |
| _wt/df022-sxr | fix/ledger-read-only-no-create | no-remote-branch | origin/fix/ledger-read-only-no-create | 1 | 23 | 0 | 1 | #6 MERGED | MERGED UNPUSHED |
| _wt/df022-sxr | jollimemory/summaries/v3 | no-remote-branch | - | 2 | 170 | 2 | 2 |  | UNIQUE UNPUSHED |

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| _wt/df022-sxr | checkpoint/2026-10-30-pre-agent-ingest | c239d5c67483 |
| _wt/df022-sxr | checkpoint/2026-10-30-pre-journey-check | 6c37573c60e6 |
| _wt/df022-sxr | checkpoint/2026-10-30-pre-merge-pr12 | b616335136bf |
| _wt/df022-sxr | checkpoint/2026-10-30-pre-tail-verify-fix | 60bb3c2f37e4 |
| _wt/df022-sxr | checkpoint/pre-design-20261022 | 872c2f6a26ee |
| _wt/df022-sxr | checkpoint/pre-readiness-merge-20261022 | 42879a82115b |

### gs:SEA

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| dev | 2026-11-06 | Sam Prime | 1 | 0 | 0 | #544 MERGED | MERGED |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| SEA | backup-before-scrub-20260202 | no-remote-branch | - | 753 | 203 | 551 | 753 |  | UNIQUE UNPUSHED |
| SEA | backup/cycle-cell-mesh-before-cla-rewrite | no-remote-branch | - | 35 | 203 | 33 | 35 |  | UNIQUE UNPUSHED |
| SEA | backup/pr-518-pre-cla-rewrite | no-remote-branch | - | 1886 | 203 | 1399 | 1886 |  | UNIQUE UNPUSHED |
| SEA | coderabbit-fixes | no-remote-branch | - | 1539 | 203 | 1127 | 1539 |  | UNIQUE UNPUSHED |
| SEA | copilot/pr523-ci-fix | no-remote-branch | - | 14 | 203 | 14 | 14 |  | UNIQUE UNPUSHED |
| SEA | cycle/DeclaredIntent-SEAEmission | no-remote-branch | - | 625 | 203 | 486 | 625 |  | UNIQUE UNPUSHED |
| SEA | cycle/choas-testing | no-remote-branch | - | 14 | 203 | 11 | 14 | #351 MERGED | UNIQUE UNPUSHED |
| SEA | cycle/debt-resolution | no-remote-branch | - | 749 | 203 | 551 | 749 |  | UNIQUE UNPUSHED |
| SEA | cycle/end-state-task1 | no-remote-branch | - | 618 | 203 | 481 | 618 | #318 MERGED | UNIQUE UNPUSHED |
| SEA | cycle/p11-c1A-sea-integrate-cli | no-remote-branch | origin/cycle/p11-c1A-sea-integrate-cli | 1668 | 203 | 1215 | 1668 | #459 MERGED | UNIQUE UNPUSHED |
| SEA | dev | yes | origin/dev | 0 | 1 | 0 | 0 | #544 MERGED | MERGED |
| SEA | end-state-task1 | no-remote-branch | - | 616 | 203 | 479 | 616 | #317 CLOSED | UNIQUE UNPUSHED |
| SEA | feat/compliance-sink-routing | no-remote-branch | origin/feat/compliance-sink-routing | 73 | 203 | 67 | 73 | #475 CLOSED | UNIQUE UNPUSHED |
| SEA | feat/observe-bootstrap-proposals | no-remote-branch | origin/feat/observe-bootstrap-proposals | 73 | 203 | 67 | 73 | #474 CLOSED | UNIQUE UNPUSHED |
| SEA | fix/pr-405-governance | no-remote-branch | - | 1563 | 203 | 1147 | 1563 |  | UNIQUE UNPUSHED |
| SEA | phase1-governed-spine | no-remote-branch | - | 1 | 87 | 0 | 1 |  | MERGED UNPUSHED |
| SEA | pr-323 | no-remote-branch | - | 624 | 203 | 486 | 624 |  | UNIQUE UNPUSHED |
| SEA | promo-main | no-remote-branch | origin/main | 55 | 203 | 48 | 55 |  | UNIQUE UNPUSHED |
| SEA | promo-stage | no-remote-branch | origin/stage | 52 | 203 | 46 | 52 |  | UNIQUE UNPUSHED |
| SEA | release/0.12.0 | no-remote-branch | origin/release/0.12.0 | 40 | 203 | 37 | 40 |  | UNIQUE UNPUSHED |
| SEA | release/0.8.0 | no-remote-branch | - | 733 | 203 | 536 | 733 |  | UNIQUE UNPUSHED |
| SEA | release/v0.10.3 | no-remote-branch | origin/release/v0.10.3 | 1652 | 203 | 1203 | 1652 |  | UNIQUE UNPUSHED |
| SEA | release/v0.13.0 | no-remote-branch | origin/release/v0.13.0 | 49 | 203 | 45 | 49 |  | UNIQUE UNPUSHED |
| SEA | release/v0.9.0 | no-remote-branch | origin/release/v0.9.0 | 1621 | 203 | 1182 | 1621 |  | UNIQUE UNPUSHED |
| SEA | remediation/phase-0 | no-remote-branch | - | 0 | 89 | 0 | 0 |  | MERGED |
| SEA | showcase/domain-segments | no-remote-branch | origin/showcase/domain-segments | 5 | 203 | 5 | 5 |  | UNIQUE UNPUSHED |
| SEA | task17-gap-report-ci | no-remote-branch | - | 5 | 203 | 5 | 5 |  | UNIQUE UNPUSHED |
| SEA | wip/green-check-followup-2026-07-30 | no-remote-branch | - | 1 | 21 | 1 | 1 |  | UNIQUE UNPUSHED |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:SEA-Forge

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| chore/domainforge-0.22.0 | 2026-11-14 | Sam Prime | 2 | 2 | 2 | #99 MERGED | UNIQUE |
| ci/macos-app-sandbox-probe | 2026-10-25 | Sam Prime | 23 | 3 | 3 |  | UNIQUE |
| ci/openshell-live | 2026-10-26 | Sam Prime | 23 | 9 | 9 |  | UNIQUE |
| ci/openshell-windows | 2026-10-25 | Sam Prime | 23 | 4 | 4 |  | UNIQUE |
| dependabot/cargo/compatible-minor-00a67746f6 | 2026-11-09 | dependabot[bot] | 6 | 1 | 1 | #93 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/cargo/jsonschema-0.60.0 | 2026-10-26 | dependabot[bot] | 21 | 1 | 1 | #70 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/actions-c8d4f145ad | 2026-11-09 | dependabot[bot] | 6 | 1 | 1 | #92 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| docs/egm-bootstrap-readmes | 2026-10-30 | Sam Prime | 21 | 2 | 2 |  | UNIQUE |
| fde/domainforge-0.21.2 | 2026-11-13 | Sam Prime | 5 | 3 | 2 | #94 MERGED | UNIQUE |
| fde/fix-e2e-world-loop | 2026-11-13 | Sam Prime | 4 | 2 | 2 | #97 MERGED | UNIQUE |
| fde/integration | 2026-11-15 | Sam Prime | 0 | 46 | 44 |  | UNIQUE |
| fde/planning-baseline | 2026-11-13 | Sam Prime | 3 | 2 | 1 | #98 MERGED | UNIQUE |
| fde/rustsec-2026-0459-0486 | 2026-11-13 | Sam Prime | 6 | 1 | 0 | #95 MERGED | MERGED |
| feat/arm64-jetson-packaging | 2026-10-27 | Sam Prime | 21 | 10 | 10 |  | UNIQUE |
| feat/hardening-capability-harvest | 2026-11-14 | Sam Prime | 1 | 49 | 47 | #71 MERGED | UNIQUE |
| feat/horizon-desk-support | 2026-11-07 | Sam Prime | 6 | 5 | 5 | #91 OPEN | OPEN-PR UNIQUE |
| feat/macos-app-sandbox-jail | 2026-10-25 | Sam Prime | 23 | 16 | 16 |  | UNIQUE |
| feat/native-packages | 2026-10-25 | Sam Prime | 23 | 19 | 18 |  | UNIQUE |
| feat/openshell-capability | 2026-10-25 | Sam Prime | 23 | 16 | 16 |  | UNIQUE |
| feat/xplat-consolidate | 2026-10-25 | Sam Prime | 23 | 48 | 43 |  | UNIQUE |
| feat/xplat-hardening | 2026-10-25 | Sam Prime | 23 | 25 | 25 |  | UNIQUE |
| feat/xplat-win-sandbox-tests | 2026-10-26 | Sam Prime | 23 | 49 | 44 |  | UNIQUE |
| fix/b08-domainforge-engine-fail-closed | 2026-10-24 | Sam Prime | 28 | 1 | 1 | #61 CLOSED | UNIQUE |
| fix/b12-seed-cell-and-backup | 2026-10-24 | Sam Prime | 28 | 1 | 1 | #60 CLOSED | UNIQUE |
| fix/review1-sandbox | 2026-10-26 | Sam Prime | 23 | 83 | 74 |  | UNIQUE |
| fix/review1-windows | 2026-10-26 | Sam Prime | 23 | 79 | 70 |  | UNIQUE |
| fix/review2-sandbox | 2026-10-26 | Sam Prime | 23 | 99 | 88 |  | UNIQUE |
| fix/review2-windows | 2026-10-26 | Sam Prime | 23 | 101 | 90 |  | UNIQUE |
| release-please--branches--main | 2026-11-15 | github-actions[bot] | 0 | 1 | 1 | #96 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| t1/rename-case-task-approval | 2026-11-05 | Sam Prime | 13 | 2 | 0 |  | MERGED |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/arm64 | chore/domainforge-0.22.0 | yes | origin/chore/domainforge-0.22.0 | 2 | 2 | 2 | 0 | #99 MERGED | UNIQUE |
| _wt/arm64 | ci/macos-app-sandbox-probe | yes | origin/ci/macos-app-sandbox-probe | 3 | 23 | 3 | 0 |  | UNIQUE |
| _wt/arm64 | ci/openshell-live | yes | origin/ci/openshell-live | 9 | 23 | 9 | 0 |  | UNIQUE |
| _wt/arm64 | ci/openshell-windows | yes | origin/ci/openshell-windows | 4 | 23 | 4 | 0 |  | UNIQUE |
| _wt/arm64 | ci/release-provenance | no-remote-branch | origin/ci/release-provenance | 1 | 28 | 0 | 1 | #58 MERGED | MERGED UNPUSHED |
| _wt/arm64 | docs/egm-bootstrap-readmes | yes | origin/docs/egm-bootstrap-readmes | 2 | 21 | 2 | 0 |  | UNIQUE |
| _wt/arm64 | fde/domainforge-0.21.2 | yes | origin/fde/domainforge-0.21.2 | 3 | 5 | 2 | 0 | #94 MERGED | UNIQUE |
| _wt/arm64 | fde/fix-e2e-world-loop | yes | origin/fde/fix-e2e-world-loop | 2 | 4 | 2 | 0 | #97 MERGED | UNIQUE |
| _wt/arm64 | fde/integration | yes | origin/fde/integration | 46 | 0 | 44 | 0 |  | UNIQUE |
| _wt/arm64 | fde/planning-baseline | yes | origin/fde/planning-baseline | 2 | 3 | 1 | 0 | #98 MERGED | UNIQUE |
| _wt/arm64 | fde/rustsec-2026-0459-0486 | yes | origin/fde/rustsec-2026-0459-0486 | 1 | 6 | 0 | 0 | #95 MERGED | MERGED |
| _wt/arm64 | feat/arm64-jetson-packaging | yes | origin/feat/arm64-jetson-packaging | 10 | 21 | 10 | 0 |  | UNIQUE |
| _wt/arm64 | feat/hardening-capability-harvest | yes | origin/feat/hardening-capability-harvest | 49 | 1 | 47 | 0 | #71 MERGED | UNIQUE |
| _wt/arm64 | feat/horizon-desk-support | yes | origin/feat/horizon-desk-support | 5 | 6 | 5 | 0 | #91 OPEN | OPEN-PR UNIQUE |
| _wt/arm64 | feat/macos-app-sandbox-jail | yes | origin/feat/macos-app-sandbox-jail | 16 | 23 | 16 | 0 |  | UNIQUE |
| _wt/arm64 | feat/native-packages | yes | origin/feat/native-packages | 19 | 23 | 18 | 0 |  | UNIQUE |
| _wt/arm64 | feat/openshell-capability | yes | origin/feat/openshell-capability | 16 | 23 | 16 | 0 |  | UNIQUE |
| _wt/arm64 | feat/xplat-consolidate | yes | origin/feat/xplat-consolidate | 48 | 23 | 43 | 0 |  | UNIQUE |
| _wt/arm64 | feat/xplat-hardening | yes | origin/feat/xplat-hardening | 25 | 23 | 25 | 0 |  | UNIQUE |
| _wt/arm64 | feat/xplat-win-sandbox-tests | yes | origin/feat/xplat-win-sandbox-tests | 49 | 23 | 44 | 0 |  | UNIQUE |
| _wt/arm64 | fix/b08-domainforge-engine-fail-closed | no | origin/fix/b08-domainforge-engine-fail-closed-v2 | 3 | 24 | 2 | 3 | #61 CLOSED | UNIQUE UNPUSHED |
| _wt/arm64 | fix/b12-seed-cell-and-backup | no | origin/fix/b12-seed-cell-and-backup-v2 | 1 | 25 | 0 | 1 | #60 CLOSED | MERGED UNPUSHED |
| _wt/arm64 | fix/go-jose-4.1.6 | no-remote-branch | origin/fix/go-jose-4.1.6 | 1 | 18 | 0 | 1 | #78 MERGED | MERGED UNPUSHED |
| _wt/arm64 | fix/restore-release-attestation | no-remote-branch | origin/fix/restore-release-attestation | 1 | 26 | 0 | 1 | #64 MERGED | MERGED UNPUSHED |
| _wt/arm64 | fix/review1-sandbox | yes | origin/fix/review1-sandbox | 83 | 23 | 74 | 0 |  | UNIQUE |
| _wt/arm64 | fix/review1-windows | yes | origin/fix/review1-windows | 79 | 23 | 70 | 0 |  | UNIQUE |
| _wt/arm64 | fix/review2-sandbox | yes | origin/fix/review2-sandbox | 99 | 23 | 88 | 0 |  | UNIQUE |
| _wt/arm64 | fix/review2-windows | yes | origin/fix/review2-windows | 101 | 23 | 90 | 0 |  | UNIQUE |
| _wt/arm64 | fix/server-ambiguous-declare-keeps-lease | no-remote-branch | origin/fix/server-ambiguous-declare-keeps-lease | 2 | 13 | 2 | 2 | #84 MERGED | UNIQUE UNPUSHED |
| _wt/arm64 | fix/settlement-attempt-claim-settlement | no-remote-branch | origin/fix/settlement-attempt-claim-settlement | 22 | 17 | 21 | 22 | #80 CLOSED | UNIQUE UNPUSHED |
| _wt/arm64 | fix/settlement-attempt-claim-settlement-main | no-remote-branch | origin/fix/settlement-attempt-claim-settlement-main | 1 | 16 | 0 | 1 | #81 MERGED | MERGED UNPUSHED |
| _wt/arm64 | fix/settlement-orphan-guard | no-remote-branch | origin/fix/settlement-orphan-guard | 2 | 8 | 2 | 2 | #89 MERGED | UNIQUE UNPUSHED |
| _wt/arm64 | security/identity-binding-version | no-remote-branch | origin/security/identity-binding-version | 1 | 19 | 0 | 1 | #77 MERGED | MERGED UNPUSHED |
| _wt/arm64 | t1/case-task-approval | no-remote-branch | origin/t1/case-task-approval | 1 | 12 | 0 | 1 | #87 MERGED | MERGED UNPUSHED |
| _wt/arm64 | t1/contracts-home | no-remote-branch | origin/t1/contracts-home | 1 | 13 | 0 | 0 | #85 MERGED | MERGED |
| _wt/arm64 | t1/rename-case-task-approval | yes | origin/t1/rename-case-task-approval | 2 | 13 | 0 | 0 |  | MERGED |
| _wt/arm64 | test/settlement-class-fail-closed | no-remote-branch | origin/test/settlement-class-fail-closed | 2 | 9 | 2 | 2 | #88 MERGED | UNIQUE UNPUSHED |

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| _wt/arm64 | checkpoint/pre-containers-20261025 | 4e67dfca734a |
| _wt/arm64 | checkpoint/pre-ctutils-20261023 | 17253b0774a5 |
| _wt/arm64 | checkpoint/pre-design-20261022 | 503cd63fa501 |
| _wt/arm64 | checkpoint/pre-merge-67-20261026 | b7f2d1122351 |
| _wt/arm64 | checkpoint/pre-readiness-merge-20261022 | 2a57ca54e2e2 |
| _wt/arm64 | fde-anchor/prereg/fde-s1 | ee93ee87198c |

### gs:Terminal-UI

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| remediation/audit-2026-09-29 | 2026-09-29 | Sam Prime | 0 | 1 | 1 |  | UNIQUE |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| terminal-ui | remediation/audit-2026-09-29 | yes | origin/remediation/audit-2026-09-29 | 1 | 0 | 1 | 0 |  | UNIQUE |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:USCOS

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:ast-grep-mcp-rs

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:canonical-evaluation-protocol

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| fde/integration | 2026-11-13 | Sam Prime | 0 | 2 | 2 |  | UNIQUE |
| profiles/context-bundle | 2026-10-05 | Sam Prime | 1 | 1 | 0 | #3 MERGED | MERGED |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| _wt/fde-cep-integration | fde/integration | yes | origin/fde/integration | 2 | 0 | 2 | 0 |  | UNIQUE |
| _wt/fde-cep-integration | profiles/context-bundle | yes | origin/profiles/context-bundle | 1 | 1 | 0 | 0 | #3 MERGED | MERGED |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:domainforge-lsp

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| dev | 2026-06-07 | SPrime01 | 0 | 3 | 3 | #13 MERGED | UNIQUE |
| feature/WASM-web-ext-support | 2025-12-17 | SPrime01 | 15 | 1 | 1 | #9 MERGED | UNIQUE |
| feature/fotmat-handler | 2025-12-16 | SPrime01 | 22 | 1 | 1 | #5 MERGED | UNIQUE |
| semantic-adapter | 2026-10-04 | Sam Prime | 0 | 7 | 7 |  | UNIQUE |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| domainforge-lsp | dev | yes | origin/dev | 3 | 0 | 3 | 0 | #13 MERGED | UNIQUE |
| domainforge-lsp | feature/fotmat-handler | yes | origin/feature/fotmat-handler | 1 | 22 | 1 | 0 | #5 MERGED | UNIQUE |
| domainforge-lsp | semantic-adapter | yes | origin/semantic-adapter | 7 | 0 | 7 | 0 |  | UNIQUE |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:domainforge-vsc-extension

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| dependabot/github_actions/actions/checkout-6 | 2025-12-25 | dependabot[bot] | 0 | 1 | 1 | #3 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/actions/setup-node-6 | 2025-12-25 | dependabot[bot] | 0 | 1 | 1 | #1 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/github_actions/lewagon/wait-on-check-action-1.4.1 | 2025-12-16 | dependabot[bot] | 35 | 1 | 1 | #2 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/npm_and_yarn/dev-dependencies-559030ff26 | 2026-01-12 | dependabot[bot] | 0 | 1 | 1 | #10 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dependabot/npm_and_yarn/types/node-25.0.2 | 2025-12-18 | dependabot[bot] | 14 | 1 | 1 | #5 OPEN | OPEN-PR UNIQUE DEPENDABOT/RELEASE-BOT |
| dev | 2026-02-01 | SPrime01 | 7 | 2 | 2 | #8 MERGED | UNIQUE |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| domainforge-vsc-extension | dev | yes | origin/dev | 2 | 7 | 2 | 0 | #8 MERGED | UNIQUE |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:godspeed-compass

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:godspeed-growth

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:gs-term

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:hassos-addon-agent-memory-ledger

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| dev | 2026-07-02 | SPrime01 | 6 | 0 | 0 | #4 MERGED | MERGED |

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:hermes-agent-sea-forge

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| hermes-agent-sea-forge | backup/opentui-prestrip-20260616-1950 | 9d05f3721d69 |
| hermes-agent-sea-forge | backup/precopystrip-20260616-2058 | a348fc1cccc2 |

### gs:ideal

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| ideal | jollimemory/summaries/v3 | no-remote-branch | - | 2 | 134 | 2 | 2 |  | UNIQUE UNPUSHED |

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:project-topology-architect

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### gs:swe_seed

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| deploy-prep | 2026-10-30 | Sam Prime | 5 | 1 | 1 | #10 MERGED | UNIQUE |
| feat/native-distribution | 2026-11-05 | SPRIME01 | 0 | 3 | 3 | #13 OPEN | OPEN-PR UNIQUE |
| fix/native-candidate-windows-paths-macos-runner | 2026-11-05 | Sam Prime | 0 | 13 | 13 | #14 MERGED | UNIQUE |

Local branches (all clones, excluding default):

| clone | branch | tip_eq_origin | upstream | ahead_default | behind_default | unique_patches | unpushed | pr | categories |
|---|---|---|---|---|---|---|---|---|---|
| SWE_SEED | deploy-prep | yes | origin/deploy-prep | 1 | 5 | 1 | 0 | #10 MERGED | UNIQUE |
| SWE_SEED | feat/native-distribution | no | origin/feat/native-distribution | 2 | 0 | 2 | 0 | #13 OPEN | OPEN-PR UNIQUE |
| SWE_SEED | fix/native-candidate-windows-paths-macos-runner | yes | origin/fix/native-candidate-windows-paths-macos-runner | 13 | 0 | 13 | 0 | #14 MERGED | UNIQUE |

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| SWE_SEED | checkpoint/pre-design-20261022 | 6d43dd0762c2 |
| SWE_SEED | checkpoint/pre-design-20261022-main | 3b779114b8af |
| SWE_SEED | checkpoint/pre-readiness-merge-20261022 | 7c9c530e7284 |

### gs:zeroclaw-sea-forge

Remote branches (origin/*, excluding default):

| branch | last_commit | author | behind_default | ahead_default | unique_patches | pr | categories |
|---|---|---|---|---|---|---|---|
| gh-pages | 2026-05-23 | SPRIME01 | 3062 | 1 | 1 | LOOKUP-FAILED | UNIQUE KEEP-CANDIDATE |

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:.

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:NewSea

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:adlc

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:demo-ws

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:godspeed-readiness

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

| clone | tag | target |
|---|---|---|
| godspeed-readiness | checkpoint/pre-p02-implementation | 44679dcd1f7a |

### local:my_LLMLingua_project

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:owner-KT9igc

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:owner-w44OuL

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:owner-ycBVRK

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:sxr-repo

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:tree-sitter-sea

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

### local:youcom-ext

Remote branches (origin/*, excluding default):

(none)

Local branches (all clones, excluding default):

(none)

Tags (backup*/checkpoint*/fde-anchor*):

(none)

## Non-GodSpeed remote

(none analysed; see table above)

## Cognate Workbench capability/plugin/skill/MCP branches

Name regex: plugin|capabilit|skill|mcp (case-insensitive), listed only where ahead of default > 0 in the same branch ref; subjects are `git log --oneline <default>..<b> | head -20`.

## gs:Horizon remote origin/dependabot/bun/mcp-b/webmcp-polyfill-6.2.0 (ahead 1, unique patches 1)
a199ca1 build(deps): Bump @mcp-b/webmcp-polyfill from 5.1.0 to 6.2.0

## gs:SEA-Forge remote origin/feat/hardening-capability-harvest (ahead 49, unique patches 47)
7bd0cb01 test(case-runner): the live egress allow-control sends a real HTTP GET
264c234c fix(core): the macOS jail reports GroupTerminatedDescendantsUnverified, not ProcessGroup
c187a8cd fix(sandbox): the pidfd fallback sweeps descendant cgroups and removes them bottom-up
d97aca5e Merge remote-tracking branch 'origin/main' into feat/hardening-capability-harvest
2fc5b1ab fix(sandbox): the mutation control re-admits the ack fd across exec
2fe4b236 fix(sandbox): hand the launch chain's pipes over above the low fd range
9e78c29a fix(sandbox): list the helper's descriptors when the ack reopen fails
90efe247 test(sandbox): report how the launch chain ended and mark the helper's start
965ac885 test(sandbox): capture the launcher fixture's launch-chain stderr
62bd4424 test(server): accept the object-shaped lifecycle_containment verdict and compare trace to evidence
4841aac7 fix(hardening): address PR #71 review threads
18f53788 docs(agents): restore main current-state text in CURRENT_STATUS.md and add rev 267
a7b3baf2 Merge origin/main into feat/hardening-capability-harvest
700c264e docs: consolidated final report — capability harvest + ARM64 distribution
b4a68733 chore(status): final lane triage — required gates green; 24.04 fixture defect scoped
743f1e87 fix(tests): scope the R4 unshare override and record the 24.04 fixture defect
8c80117a fix(tests): the launcher fixture captures the helper's stderr
6a5bd7bf fix(sandbox): re-admit both pipe fds in the isolated pre_exec
e7e998f5 fix(sandbox): the ack write end uses the lifeline's reopen mechanism
fa7576b2 fix(security): codex pass-3 review fixes — dedicated jail thread, CLOEXEC pipes, scoped override

## gs:SEA-Forge remote origin/feat/openshell-capability (ahead 16, unique patches 16)
73516733 feat(sandbox): enforcement capability and workload identity for OpenShell
d83d6707 feat(core): typed WorkloadImage for OCI and native identities
55cc249f ci(portability): run the Windows lane on every feat branch
22dddbcb fix(server): refuse configured Postgres resources on Windows
2055258e fix(server): Windows test expectations and model line endings
498a8b30 fix(server): Windows test scoping, Send PipeSecurity, native example binding
722b69c6 fix(server): Windows peer moves, SID audit field, unused-import gating
0e5f8d32 ci(portability): run control-plane tests and upload Windows binaries
bac55a54 feat(cli): Windows named-pipe client that verifies the server SID first
0df13198 fix(sandbox): refuse MicroVM virtualization probe on Windows
bcecd950 feat(server): native identity keys (uid on Unix, SID on Windows) and named-pipe transport
4b1bff3a fix(os): gate process-group kill to Unix; Job Object containment on Windows
5d8fdf65 fix(changeset): scan Windows reparse points and hard links
39ff3639 feat(os): add sea-forge-os Windows FFI crate (pipe identity, pipe DACL, job objects)
19dc8b73 docs(agents): cross-platform execution record, hardening-review and runtime-probing skills
e206fe42 ci(portability): native Windows workspace check lane

## gs:SEA-Forge local feat/hardening-capability-harvest in _wt/arm64 (ahead 49, unique patches 47, unpushed 0)
7bd0cb01 test(case-runner): the live egress allow-control sends a real HTTP GET
264c234c fix(core): the macOS jail reports GroupTerminatedDescendantsUnverified, not ProcessGroup
c187a8cd fix(sandbox): the pidfd fallback sweeps descendant cgroups and removes them bottom-up
d97aca5e Merge remote-tracking branch 'origin/main' into feat/hardening-capability-harvest
2fc5b1ab fix(sandbox): the mutation control re-admits the ack fd across exec
2fe4b236 fix(sandbox): hand the launch chain's pipes over above the low fd range
9e78c29a fix(sandbox): list the helper's descriptors when the ack reopen fails
90efe247 test(sandbox): report how the launch chain ended and mark the helper's start
965ac885 test(sandbox): capture the launcher fixture's launch-chain stderr
62bd4424 test(server): accept the object-shaped lifecycle_containment verdict and compare trace to evidence
4841aac7 fix(hardening): address PR #71 review threads
18f53788 docs(agents): restore main current-state text in CURRENT_STATUS.md and add rev 267
a7b3baf2 Merge origin/main into feat/hardening-capability-harvest
700c264e docs: consolidated final report — capability harvest + ARM64 distribution
b4a68733 chore(status): final lane triage — required gates green; 24.04 fixture defect scoped
743f1e87 fix(tests): scope the R4 unshare override and record the 24.04 fixture defect
8c80117a fix(tests): the launcher fixture captures the helper's stderr
6a5bd7bf fix(sandbox): re-admit both pipe fds in the isolated pre_exec
e7e998f5 fix(sandbox): the ack write end uses the lifeline's reopen mechanism
fa7576b2 fix(security): codex pass-3 review fixes — dedicated jail thread, CLOEXEC pipes, scoped override

## gs:SEA-Forge local feat/openshell-capability in _wt/arm64 (ahead 16, unique patches 16, unpushed 0)
73516733 feat(sandbox): enforcement capability and workload identity for OpenShell
d83d6707 feat(core): typed WorkloadImage for OCI and native identities
55cc249f ci(portability): run the Windows lane on every feat branch
22dddbcb fix(server): refuse configured Postgres resources on Windows
2055258e fix(server): Windows test expectations and model line endings
498a8b30 fix(server): Windows test scoping, Send PipeSecurity, native example binding
722b69c6 fix(server): Windows peer moves, SID audit field, unused-import gating
0e5f8d32 ci(portability): run control-plane tests and upload Windows binaries
bac55a54 feat(cli): Windows named-pipe client that verifies the server SID first
0df13198 fix(sandbox): refuse MicroVM virtualization probe on Windows
bcecd950 feat(server): native identity keys (uid on Unix, SID on Windows) and named-pipe transport
4b1bff3a fix(os): gate process-group kill to Unix; Job Object containment on Windows
5d8fdf65 fix(changeset): scan Windows reparse points and hard links
39ff3639 feat(os): add sea-forge-os Windows FFI crate (pipe identity, pipe DACL, job objects)
19dc8b73 docs(agents): cross-platform execution record, hardening-review and runtime-probing skills
e206fe42 ci(portability): native Windows workspace check lane


## UNPUSHED local branches (commits not on any origin ref)

| key | clone | branch | unpushed | ahead_vs_default | categories |
|---|---|---|---|---|---|
| gs:Cognate | _wt/df022-cognate | pr-70 | 255 | 255 | UNIQUE UNPUSHED |
| gs:Cognate-Workbench | _wt/df022-cognate-workbench | ci/install-pinned-domainforge | 1 | 1 | MERGED UNPUSHED |
| gs:Cognate-Workbench | _wt/df022-cognate-workbench | feat/desktop-terminal-browser | 19 | 140 | OPEN-PR UNIQUE UNPUSHED |
| gs:Cognate-Workbench | _wt/df022-cognate-workbench | feat/horizon-connect | 11 | 75 | UNIQUE UNPUSHED |
| gs:Context-Kernel | Context_Kernel | backup/harden-pre-sync | 1 | 1 | UNIQUE UNPUSHED |
| gs:DomainForge | _wt/domainforge-npm | rel-0.22.1 | 2 | 2 | UNIQUE UNPUSHED |
| gs:Gauntlet | _wt/fde-gauntlet-integration | fix/test-schema-teardown | 1 | 1 | MERGED UNPUSHED |
| gs:Horizon | Horizon | feat/governed-operate | 1 | 1 | MERGED UNPUSHED |
| gs:Horizon | Horizon | feat/ignition-onboarding | 1 | 1 | MERGED UNPUSHED |
| gs:Horizon | Horizon | fix/gateway-socket-dir-at-boot | 1 | 1 | MERGED UNPUSHED |
| gs:Horizon | Horizon | fix/j02r2-rig-operator-ref | 1 | 1 | MERGED UNPUSHED |
| gs:Horizon | Horizon | fix/settlement-pending-error-class | 1 | 1 | MERGED UNPUSHED |
| gs:Horizon | Horizon | stage0b-containment-core | 3 | 3 | UNIQUE UNPUSHED |
| gs:Horizon | Horizon | stage0b-containment-pr1 | 3 | 3 | UNIQUE UNPUSHED |
| gs:Horizon | Horizon | stage0b-containment-pr1-narrow | 4 | 4 | UNIQUE UNPUSHED |
| gs:Horizon | Horizon | stage0b-pr1-broad-snapshot | 3 | 3 | UNIQUE UNPUSHED |
| gs:Horizon | Horizon | stage0b-pr1-narrowing-wip | 1 | 1 | UNIQUE UNPUSHED |
| gs:RealityTrace | _wt/df022-sxr | chore/domainforge-0.21.1 | 1 | 1 | MERGED UNPUSHED |
| gs:RealityTrace | _wt/df022-sxr | feat/command-verifier | 1 | 1 | MERGED UNPUSHED |
| gs:RealityTrace | _wt/df022-sxr | feat/native-binary-distribution | 8 | 8 | UNIQUE UNPUSHED |
| gs:RealityTrace | _wt/df022-sxr | feat/verifier-completed-at | 1 | 1 | MERGED UNPUSHED |
| gs:RealityTrace | _wt/df022-sxr | fix/bundled-sqlite | 1 | 1 | MERGED UNPUSHED |
| gs:RealityTrace | _wt/df022-sxr | fix/ledger-read-only-no-create | 1 | 1 | MERGED UNPUSHED |
| gs:RealityTrace | _wt/df022-sxr | jollimemory/summaries/v3 | 2 | 2 | UNIQUE UNPUSHED |
| gs:SEA | SEA | backup-before-scrub-20260202 | 753 | 753 | UNIQUE UNPUSHED |
| gs:SEA | SEA | backup/cycle-cell-mesh-before-cla-rewrite | 35 | 35 | UNIQUE UNPUSHED |
| gs:SEA | SEA | backup/pr-518-pre-cla-rewrite | 1886 | 1886 | UNIQUE UNPUSHED |
| gs:SEA | SEA | coderabbit-fixes | 1539 | 1539 | UNIQUE UNPUSHED |
| gs:SEA | SEA | copilot/pr523-ci-fix | 14 | 14 | UNIQUE UNPUSHED |
| gs:SEA | SEA | cycle/DeclaredIntent-SEAEmission | 625 | 625 | UNIQUE UNPUSHED |
| gs:SEA | SEA | cycle/choas-testing | 14 | 14 | UNIQUE UNPUSHED |
| gs:SEA | SEA | cycle/debt-resolution | 749 | 749 | UNIQUE UNPUSHED |
| gs:SEA | SEA | cycle/end-state-task1 | 618 | 618 | UNIQUE UNPUSHED |
| gs:SEA | SEA | cycle/p11-c1A-sea-integrate-cli | 1668 | 1668 | UNIQUE UNPUSHED |
| gs:SEA | SEA | end-state-task1 | 616 | 616 | UNIQUE UNPUSHED |
| gs:SEA | SEA | feat/compliance-sink-routing | 73 | 73 | UNIQUE UNPUSHED |
| gs:SEA | SEA | feat/observe-bootstrap-proposals | 73 | 73 | UNIQUE UNPUSHED |
| gs:SEA | SEA | fix/pr-405-governance | 1563 | 1563 | UNIQUE UNPUSHED |
| gs:SEA | SEA | phase1-governed-spine | 1 | 1 | MERGED UNPUSHED |
| gs:SEA | SEA | pr-323 | 624 | 624 | UNIQUE UNPUSHED |
| gs:SEA | SEA | promo-main | 55 | 55 | UNIQUE UNPUSHED |
| gs:SEA | SEA | promo-stage | 52 | 52 | UNIQUE UNPUSHED |
| gs:SEA | SEA | release/0.12.0 | 40 | 40 | UNIQUE UNPUSHED |
| gs:SEA | SEA | release/0.8.0 | 733 | 733 | UNIQUE UNPUSHED |
| gs:SEA | SEA | release/v0.10.3 | 1652 | 1652 | UNIQUE UNPUSHED |
| gs:SEA | SEA | release/v0.13.0 | 49 | 49 | UNIQUE UNPUSHED |
| gs:SEA | SEA | release/v0.9.0 | 1621 | 1621 | UNIQUE UNPUSHED |
| gs:SEA | SEA | showcase/domain-segments | 5 | 5 | UNIQUE UNPUSHED |
| gs:SEA | SEA | task17-gap-report-ci | 5 | 5 | UNIQUE UNPUSHED |
| gs:SEA | SEA | wip/green-check-followup-2026-07-30 | 1 | 1 | UNIQUE UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | ci/release-provenance | 1 | 1 | MERGED UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/b08-domainforge-engine-fail-closed | 3 | 3 | UNIQUE UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/b12-seed-cell-and-backup | 1 | 1 | MERGED UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/go-jose-4.1.6 | 1 | 1 | MERGED UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/restore-release-attestation | 1 | 1 | MERGED UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/server-ambiguous-declare-keeps-lease | 2 | 2 | UNIQUE UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/settlement-attempt-claim-settlement | 22 | 22 | UNIQUE UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/settlement-attempt-claim-settlement-main | 1 | 1 | MERGED UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | fix/settlement-orphan-guard | 2 | 2 | UNIQUE UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | security/identity-binding-version | 1 | 1 | MERGED UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | t1/case-task-approval | 1 | 1 | MERGED UNPUSHED |
| gs:SEA-Forge | _wt/arm64 | test/settlement-class-fail-closed | 2 | 2 | UNIQUE UNPUSHED |
| gs:ideal | ideal | jollimemory/summaries/v3 | 2 | 2 | UNIQUE UNPUSHED |

## Dirty checkouts (uncommitted entries > 0)

| repo | path | dirty | stash | head |
|---|---|---|---|---|
| gs:Context-Kernel | Context_Kernel | 1 | 0 | main |
| gs:Horizon | Horizon | 5 | 0 | horizon-desk-v2 |
| gs:Horizon | Horizon-core | 10 | 0 | stage0b-containment-core |
| local:NewSea | NewSea | 20 | 0 | master |
| gs:SEA | SEA | 1 | 15 | dev |
| gs:swe_seed | SWE_SEED | 1 | 0 | feat/native-distribution |
| nongs:SPRIME01/WarpRoom | WarpRoom | 1 | 0 | master |
| local:demo-ws | _wt/demo-ws | 1 | 0 | main |
| gs:RealityTrace | _wt/df022-sxr | 3 | 0 | chore/domainforge-0.22.0 |
| gs:RealityTrace | _wt/fde-sxr-ingest-json | 4 | 0 | fde/fix-ingest-json-spelling |
| gs:RealityTrace | _wt/fde-sxr-integration | 4 | 0 | fde/integration |
| gs:RealityTrace | _wt/fde-sxr-release-sums | 3 | 0 | fde/fix-release-sha256sums |
| local:adlc | adlc | 4 | 0 | main |
| nongs:SPRIME01/antigravity-claude-proxy | antigravity-claude-proxy | 1 | 0 | main |
| nongs:finos/architecture-as-code | architecture-as-code | 1 | 0 | main |
| gs:ast-grep-mcp-rs | ast-grep-mcp-rs | 1 | 0 | main |
| gs:canonical-evaluation-protocol | cep | 1 | 0 | profiles/context-bundle |
| gs:Cognate | cognate-horizon | 2 | 0 | feat/realitytrace-real-snapshot |
| gs:Cognate | cognate-identity-wt | 1 | 0 | horizon/shared-identity |
| gs:Cognate-Workbench | cognate-workbench | 37 | 0 | feat/desktop-terminal-browser |
| gs:DomainForge | domainforge | 6 | 0 | main |
| nongs:SPRIME01/dotfiles | dotfiles-legacy-recovered | 3 | 0 | main |
| nongs:SPRIME01/gspeedai | godspeed | 8 | 0 | main |
| gs:GodSpeed-Agent | godspeed_agent | 2 | 0 | migration/cep-world-required-evidence |
| nongs:trailhq/Graft | graft-v0.29 | 8 | 0 | local/context-documents-v0.29 |
| nongs:SPRIME01/home-assistant-config | home-assistant-config | 15 | 0 | main |
| nongs:SPRIME01/homelab | homelab | 9 | 0 | dev |
| local:my_LLMLingua_project | my_LLMLingua_project | 13 | 0 | main |
| nongs:SPRIME01/NexusNotes | nexusnotes | 10 | 0 | main |
| nongs:SPRIME01/PublishingHouse | novels | 163 | 0 | main |
| nongs:SPRIME01/oh-my-openagent | oh-my-openagent | 15 | 0 | dev |
| nongs:nexu-io/open-design | open-design-tools-recovered | 3 | 0 | main |
| nongs:DietrichGebert/ponytail | ponytail | 21 | 0 | main |
| nongs:SPRIME01/Product-Actualizer | product-actualizer | 2 | 0 | main |
| nongs:SPRIME01/resume-tailor | resume-tailor | 20 | 0 | main |
| gs:SEA-Forge | sea-rs | 3 | 1 | docs/egm-bootstrap-readmes |
| gs:RealityTrace | sxr | 1 | 0 | feat/native-binary-distribution |
| local:youcom-ext | youcom-ext | 13 | 0 | main |
| nongs:SPRIME01/You.com-Agents-Extention | youcom_vscode_extension/you-com-agents | 28 | 0 | main |

