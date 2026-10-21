# Branch reconciliation — 2026-10-21

Scope: consolidate useful local/GitHub branch work into main, retain deliberate
replacements, and remove extra project worktrees. The operator explicitly chose
to fix and integrate the three failing dependency upgrades.

## Preservation and branch review

A verified Git bundle retains all 63 initial refs (including tool checkpoints)
under the common Git directory at `reconciliation-20261021/before.bundle`.
It is private, outside versioned source, and not published.

| Branch | Original tip | Review |
| --- | --- | --- |
| main | 257b622 | Fast-forwarded to round-4 squash 14a5f97 |
| archive/main-pre-round4-20261018 | c9f8b4e | Ancestor of round 3; local branch removed |
| chore/coderabbit-non-blocking | 2f3b835 | Equivalent patch in 4f0420c; local branch removed |
| security/enforcement-gaps-20261017 | 337a3a5 | Exact tree match to squash 257b622; local branch removed |
| security/round4-20261018 | 8fe1543 | Exact tree match to squash 14a5f97; local branch removed |
| frontend | d105f9a | Superseded; remote removal pending |
| dependabot/cargo/compatible-minor-b46ebe6573 | 1fc8b99 | Keep serde_json/rustix; retain intentional DomainForge 0.21.0 pin |
| dependabot/cargo/rusqlite-0.40.2 | 19b1e49 | Cherry-picked; validation pending |
| dependabot/cargo/rand-0.10.2 | 35f1741 | Cherry-picked; API adaptation under validation |
| dependabot/cargo/serde-saphyr-2.0.0 | d33cafb | Cherry-picked; strict parsing policy retained |
| dependabot/cargo/jsonschema-0.58.6 | 1160b4d | Applied with Cargo-generated lockfile and validator API adaptations |

Fetch pruned 11 remote-tracking refs whose GitHub branches were already deleted.
No GitHub branch has been deleted by this reconciliation yet.

Frontend comparison used PR #11's source tip
`26ddaae306a2c980086fcd66baaf2e181eb68910`, squash-merged as `bd661c9`.
`git cherry` identifies 104 equivalent patches out of 111 non-merge commits.
The seven others concern early crate graduation (`eb62638`), authority
(`d7072f3`), planning/provenance (`6ff7c3b`), specs/security docs (`0cfe508`),
and licensing (`5ad0fed`, `e537c09`, `2a4f907`). Their resulting modules,
tests, specifications and license files exist on main in later forms; licenses
were also published by `dd7195c`. Do not restore obsolete manifests, security
behavior, or stale status records. Exact original commits remain in the bundle.

An independent Luna audit confirmed the classification and found no useful,
non-stale frontend change absent from main. The scoped spatial UI tree at
`19730c1` matches PR #11 squash `bd661c9`, despite the original commit not being
an ancestor of main. Recheck the remote tip before deletion to detect races.

## Intentional replacements verified

- `2c10ef5` deliberately removed Tauri and desktop packaging. They stay removed.
- `19730c1` replaced the old frontend shell with the cognitive/spatial UI in
  PR #11's history. Old shell components and `mockup.css` stay retired.
- `97ac2b7` replaced serde_yaml/unsafe-libyaml with the strict YAML boundary.
  Neither dependency is reintroduced; strict YAML regressions remain binding.
- `5a73e23` pinned DomainForge to 0.21.0 together with its runtime version
  check, CLI installation and generated fixture provenance. Dependabot's 0.21.1
  library-only bump conflicts with that alignment; the pin is preserved.
- Archive-only `server/src/exposure.rs` was removed by successor `5118875`;
  round 3 incorporates that successor. It must not be resurrected.

Three extra worktrees had no tracked or untracked changes. Their ignored
contents were Cargo build output, generated Devbox toolchains/state, generated
graft data and Python bytecode. All three were removed; only main remains.
The original workspace edit and untracked security handoff prompt are untouched.

## Validation

The original upgrades fail typechecking at rand::thread_rng and warn on a
deprecated strict-parser option. Adaptations construct Ed25519 keys from the
current thread RNG bytes, retain explicit rejection of non-finite YAML floats,
and migrate RealityTrace tests to the current JSON Schema validator API without
removing assertions. Cargo regenerates the combined lockfile.
The updated validator disables default features: its embedded schema uses only
local references, so new HTTP/file resolvers and their TLS stack are unnecessary.
An independent Luna dependency audit found no other reverted replacement or
security-version downgrade. It confirmed the DomainForge pin and compatible
serde_json/rustix updates, and required restaging the final generated lockfile
so the index cannot retain the abandoned 0.21.1/HTTP-resolver snapshot.

The compiler cache stalled without starting rustc; verification uses the same
pinned compiler with `RUSTC_WRAPPER=''`. No repository check is bypassed.
A frontend deletion attempt was refused by the stale-status hook; remote refs
remain intact. Publishing and final remote cleanup are pending.

`RUSTC_WRAPPER='' just ci` passed: formatting, lint, typecheck, dependency and
secret checks, staged-secret canaries, all 174 test suites (1,534 passed,
0 failed, 23 explicitly ignored), no-async-kernel, and debug build. The existing
host limitations for native workload-isolation qualification remain unchanged.
`just secrets-staged`, `git diff --check`, and `graft build` also passed.
The API-only changes do not alter frontend contracts or persisted schemas.

The Luna compatibility check found no newly added/updated package requiring
Rust newer than the workspace MSRV 1.92.0. Its live GitHub snapshot matched
all recorded topic tips and main at `14a5f97`; deletion still uses exact leases.

`RUSTC_WRAPPER='' just proof` passed P1–P4b. The versioned verification summary
is in `../evidence/branch-reconciliation-20261021/verification.json`; raw logs
are retained beside the private bundle.

The normal pre-push CI also passed. GitHub then rejected the atomic main update
because main requires a pull request and two status checks. Atomicity preserved
all remote refs. Integration will use a temporary reconciliation PR followed by
normal protected merging and exact-tip topic cleanup; no protection is changed.
macOS tests are not run locally on this Linux host; GitHub supplies platform CI.
