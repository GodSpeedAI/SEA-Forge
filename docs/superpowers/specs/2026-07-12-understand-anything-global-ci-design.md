# Understand Anything Global and CI Integration Design

Date: 2026-07-12
Status: Approved for implementation

## Outcome

Install one current copy of Understand Anything that works from Claude Code,
Codex CLI, Antigravity, Gemini CLI, and OpenCode. Keep that installation current
on this workstation. Add a separate sea-rs workflow that refreshes the committed
project knowledge graph after pushes to `main` without making Rust CI depend on
an LLM service.

## Current State

The workstation has a clean checkout at `~/.understand-anything/repo`. Its
`origin` still names the former `Lum1104/Understand-Anything` repository, and the
checkout reports v2.7.3. The maintained repository is now
`Egonex-AI/Understand-Anything`, whose current release is v2.9.0.

Codex, Gemini CLI, and OpenCode already discover the same per-skill symlinks in
`~/.agents/skills`. Antigravity already has the upstream folder-style link under
`~/.gemini/antigravity/skills`. Claude Code is installed, but Understand
Anything is not installed as a native Claude plugin. Gemini CLI and Antigravity
executables are not currently on `PATH`; their skill integrations can still be
prepared before those executables are installed.

The sea-rs branch already contains unrelated, uncommitted CI/CD work. This
change must preserve those edits and integrate with their established action
pinning, least-privilege, stable-gate, and `just` command conventions.

## Upstream Execution Model

Understand Anything combines deterministic source analysis with semantic LLM
analysis. Tree-sitter and bundled parsers extract files, declarations, imports,
calls, inheritance, and fingerprints. Specialized agents add summaries,
architecture, domains, tours, and other semantic data. Normalizers assemble the
batch outputs, repair identifiers, remove dangling edges, and validate the
graph.

A full run scans the project, computes semantic batches, analyzes up to five
batches concurrently, reviews the assembly, identifies architecture, builds a
tour, validates the graph, and writes durable artifacts. Later runs compare Git
history and fingerprints and analyze only changed files. New projects use
`.ua/`; projects with `.understand-anything/` retain the legacy directory.

The dashboard reads the generated graph and serves source data locally. The
graph can be committed so other users can browse it without an API key or LLM
session.

## Global Installation

Use one checkout at `~/.understand-anything/repo` as the source of truth.
Change its remote to `https://github.com/Egonex-AI/Understand-Anything.git`,
fast-forward it, install locked dependencies, build the core package, and run
the upstream tests before declaring the update healthy.

Run the official installer for `codex`, `gemini`, `opencode`, and `antigravity`.
The first three intentionally converge on the shared `~/.agents/skills`
symlinks; repeated installation is idempotent. Antigravity receives the
folder-style link expected by upstream. Install the Egonex marketplace plugin
for Claude Code so Claude receives native agent definitions and hooks in
addition to the shared plugin checkout.

Users invoke the feature with each host's native syntax. Codex uses
`$understand`; slash-command hosts use `/understand`; hosts without either form
can receive a plain-language instruction to use the `understand` skill. Every
host reads and writes the same project graph format.

## Workstation Updates

Add a user-owned update command and user-level systemd timer. The command must:

1. acquire a lock so two updates cannot overlap;
2. fetch the configured Egonex remote;
3. refuse non-fast-forward or dirty-checkout updates;
4. fast-forward to the configured upstream branch;
5. install dependencies from the lockfile;
6. build the core and skill packages;
7. run the focused upstream tests; and
8. verify every expected skill link.

The timer runs daily with randomized delay and persistent catch-up. Failures
leave the checkout inspectable and appear in the user journal. The updater does
not install missing CLI executables, change unrelated CLI configuration, or
copy skills into separate versioned trees.

## sea-rs Knowledge Graph

Generate an English graph in `.ua/` and enable incremental updates. Commit the
durable graph, metadata, fingerprints, configuration, and reviewed ignore file.
Ignore intermediate batches, temporary files, trash directories, dashboard
runtime state, and diff overlays.

The initial analysis requires an interactive review of `.ua/.understandignore`
before graph generation, as required by the upstream skill. The graph covers
the repository root and follows sea-rs instruction precedence, including
`AGENTS.md` and `.github/copilot-instructions.md`.

## Push-Triggered Refresh Workflow

Add a non-required `Understand Anything` workflow triggered by pushes to
`main` and by manual dispatch. It checks out full history, installs the Egonex
plugin in Claude Code Action automation mode, and asks Claude to perform an
incremental English `/understand` update. The workflow validates the durable
artifacts and creates or updates a dedicated graph-refresh pull request.

The workflow does not push directly to protected `main`. It skips graph-only
bot updates, cancels obsolete refreshes, and uses one stable branch so repeated
pushes update one PR. These controls prevent recursive runs and stale competing
PRs.

The workflow uses a pinned Claude Code Action commit and grants only the
repository permissions required to read contents, write the refresh branch,
and manage its pull request. Claude receives a narrow tool allowlist. The
workflow runs only for trusted pushes, never with write permissions on
untrusted pull-request code.

Repository administrators must configure one Claude credential:
`ANTHROPIC_API_KEY` or `CLAUDE_CODE_OAUTH_TOKEN`. The existing release token
must not be reused unless its documented purpose and scope explicitly expand.
The design prefers a dedicated GitHub App or fine-grained token for refresh PRs
when the default action identity cannot trigger the required CI checks.

## Failure Behavior

Rust formatting, linting, tests, builds, and the stable `CI / gate` remain
independent of graph generation. A Claude outage, exhausted API quota, upstream
plugin failure, or malformed graph fails only the graph-refresh workflow. The
last accepted graph remains available.

The refresh job must fail closed when credentials are missing, plugin checkout
or build fails, validation fails, unexpected files change, or no safe push
identity is available. Logs must exclude credentials, model payloads that may
contain sensitive data, and repository secrets.

## Validation

Global validation checks the upstream commit and release, package builds and
tests, universal plugin-root link, every platform skill link, Claude plugin
registration, and skill discovery where the installed CLI supports a
non-interactive listing command. Missing Gemini or Antigravity executables are
reported as integration prepared, runtime test skipped.

Project validation checks workflow syntax, pinned actions, least-privilege
permissions, loop prevention, artifact ignore rules, graph JSON validity,
referential integrity, recorded commit ancestry, focused project tests, and the
required sea-rs commands. Platform tests that cannot run locally are reported
as skipped with their reason.

Before handoff, run `devbox run -- just context-check`, `devbox run -- just
check`, and `devbox run -- just test`. Do not claim the remote push workflow
passed until GitHub executes it with configured credentials.

## Rollout

Implement the global migration first and verify all five discovery paths. Then
create and review the initial sea-rs graph. Finally, add the refresh workflow,
document its secrets and recovery procedure, and validate it without pushing.
The user can request a push or pull request after local verification.
