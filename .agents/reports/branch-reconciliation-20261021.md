# Branch reconciliation — 2026-10-21

Completed through protected [PR #39](https://github.com/GodSpeedAI/SEA-Forge/pull/39).
Local main and GitHub main both point to `2a57ca54e2e282ac864b71e39706ce3694a2c4e8`.
Exactly one local branch, one remote head and the primary worktree remain.
Three extra worktrees were removed. PRs #33–#37 are closed.

## What was retained

Integrated rusqlite, serde_json/rustix, rand, serde-saphyr and jsonschema updates.
Adaptations retain the existing signing, strict YAML and evidence-contract tests.
The updated test validator uses embedded schemas with local references and
disables unnecessary HTTP/file resolver defaults.

Intentional replacements remain intact:

- `2c10ef5`: Tauri and desktop packaging stay removed.
- `19730c1`: the cognitive/spatial UI replaces the old shell and mockup styles.
- `97ac2b7`: strict YAML remains; serde_yaml/unsafe-libyaml are not reintroduced.
- `5a73e23`: DomainForge stays exactly 0.21.0, aligned with the runtime version
  check, CLI and fixture provenance. Dependabot's library-only 0.21.1 bump was
  excluded; serde_json/rustix from that group were retained.
- Archive-only server exposure.rs was removed by successor `5118875`; it was
  not resurrected.

## Branch dispositions

| Branch | Original tip | Final disposition |
| --- | --- | --- |
| main | 257b622 | Fast-forwarded to round 4, then reconciled through PR #39 |
| archive/main-pre-round4-20261018 | c9f8b4e | Ancestor of round 3; archived; local ref removed |
| chore/coderabbit-non-blocking | 2f3b835 | Equivalent to 4f0420c; local ref removed |
| security/enforcement-gaps-20261017 | 337a3a5 | Exact tree match to squash 257b622; local ref removed |
| security/round4-20261018 | 8fe1543 | Exact tree match to squash 14a5f97; local ref removed |
| frontend | d105f9a | Superseded; remote ref removed after merge |
| dependabot/cargo/compatible-minor-b46ebe6573 | 1fc8b99 | Integrated except intentional DomainForge pin; remote ref removed |
| dependabot/cargo/rusqlite-0.40.2 | 19b1e49 | Integrated; remote ref removed |
| dependabot/cargo/rand-0.10.2 | 35f1741 | Integrated with API adaptation; remote ref removed |
| dependabot/cargo/serde-saphyr-2.0.0 | d33cafb | Integrated with strict parsing retained; remote ref removed |
| dependabot/cargo/jsonschema-0.58.6 | 1160b4d | Integrated with validator API adaptation; remote ref removed |
| chore/reconcile-branches-20261021 | cc410ac | Temporary PR branch; removed locally and remotely |

Frontend comparison used PR #11's source tip
`26ddaae306a2c980086fcd66baaf2e181eb68910`, squash-merged as `bd661c9`.
Of 111 non-merge commits, 104 have equivalent patches there. The seven others
concern crate graduation (`eb62638`), authority (`d7072f3`),
planning/provenance (`6ff7c3b`), specs/security docs (`0cfe508`), and
licensing (`5ad0fed`, `e537c09`, `2a4f907`). Their resulting modules,
tests, specifications and license files exist on main in later forms.
An independent Luna audit found no useful missing implementation. The scoped
spatial UI tree matches PR #11's squash despite rebased commit identities.

## History and working changes

Three verified private bundles live under the common Git directory's
`reconciliation-20261021/` directory:

- `before.bundle`: all 63 refs present after refreshing remote metadata,
  including tool checkpoints and local topic histories.
- `integration.bundle`: the six new commits ending at cc410ac, before
  protected squash alignment.
- `pruned-remote-tips.bundle`: complete histories of all 11 cached tips whose
  GitHub branches were already deleted before this session.

The 11 cached tips were separately audited. Their implementation/configuration
changes are equivalent or superseded on main. One cosmetic historical proposal,
`d15ae23`, renames the LICENSE heading. It was authored after `dd7195c`;
it remains archived and unapplied. Current published license wording was kept,
rather than treating an unmatched documentation patch as missing code.

The three removed worktrees had no tracked or untracked changes. Ignored
contents were Cargo build output, generated Devbox toolchains/state, generated
graft data and Python bytecode. No runtime state was treated as source.

Exact squash/integration tree equality was verified before safe local alignment.
The five existing dirty paths had identical hashes before and after alignment:
the two status files, this report, the operator's workspace edit, and the
untracked security handoff prompt. The operator's two original changes remain
untouched. Completion handoff/report/evidence updates are local working changes
recorded after the protected merge; no production source remains unpublished.

## Verification and protected publication

Local full `just ci` and normal hooks passed: 174 suites, 1,534 tests passed,
zero failed, 23 explicitly ignored, plus lint/typecheck/security/boundary/build.
`just proof` passed P1–P4b. Staged secrets checks, diff checks, `graft build`
and status checks passed. The stalled compiler cache was bypassed using the
same pinned compiler with `RUSTC_WRAPPER=''`; no repository gate was bypassed.

GitHub required `gate` and `conventional-commit` passed, including macOS
verification; Ubuntu, MSRV 1.92, Workbench, OPA, Go, CodeQL and dependency review
also passed. There were zero unresolved review threads; CodeRabbit completed.
Existing native-isolation qualification limits and independent security
re-attack obligations remain unchanged; this task claims no new settlement.

Direct main publication was correctly rejected by branch protection without
changing remote refs. The temporary PR followed normal squash-only integration.
Remote deletion used atomic expected-SHA leases. GitHub auto-removed the
temporary branch during the first deletion attempt, so that stale transaction
was safely rejected; the six unchanged topic refs were then removed together.

Versioned verification and completion summaries are in
`../evidence/branch-reconciliation-20261021/`. Raw logs remain private beside
the bundles. Final live unfiltered remote inspection returned exactly main;
local main equals origin/main, with one local branch and one worktree.
