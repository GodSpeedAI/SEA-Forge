# Hardening capability harvest — implementation record

Branch `feat/hardening-capability-harvest` (worktree `~/projects/_wt/harvest`),
base `main` c81f6307, HEAD `3d5a8954cee935723e75b3cba48bd8c68c533dfd` (20
commits through independent codex adversarial review pass 1 and its fixes).
Dates 2026-10-26/27. Governing inputs: the recon and synthesis artifacts in
sea-rs `.tmp/harvest/`, the binding decisions in
`.tmp/harvest/OPERATOR-DECISIONS.md`, and the frozen pass-3 findings
(`~/projects/_wt/containers/review/pass3/pass3-findings.md`, d28e00dc) that
define the six findings.

Outcome: all six pass-3 findings are implemented and verified on their
platform lanes. F15 makes prerequisite-bound security tests fail loud in
required lanes; F32 replaces the rename-by-temp-name safe write with per-call
host-owned staging directories plus post-publication content verification; F08
adds cgroup-v2 lifecycle containment on Linux, verified Job membership AND
verified empty-job completion on Windows, and a census-backed typed
`SandboxCagePersists` claim on macOS (with the documented
completeness caveat); F05 pins Windows directories exclusively, holds the temp
handle through the publication window, verifies directory identity across the
release-pin rename, and verifies the published entry's identity and type; F33
moves the tokio pipe plane to the server edge and gives the async-kernel gate
a fail-closed Windows unified-graph pass; F17's positive control is honestly
BLOCKED on OpenShell 0.1.x (no L4/connect policy vocabulary, no raw-socket
permission under rest grants, no guest-discoverable proxy — three-round
evidence chain with named upstream requirements), while the deny variant is
live-proven non-arrival on a demonstrably reachable target. `domainforge-core`
is bumped to 0.21.2, and `ExecutionResult.lifecycle_containment` carries the
containment claim into evidence, traces, and settlement. Independent codex
review pass 1 (Critical 0 / High 5 / Medium 10 / Low 1) was triaged: the
confirmed findings are fixed in this HEAD (macOS sweep scoping, interactive-
jail lease, staging content verification, per-call staging, changeset scanner
refusal, egress probe parsing, ci.yml authority narrowing, fail-closed
dependency gate, Windows completion job verification); H3/H5's
admission-level-refusal demand is disputed with the honest-degrade contract
and recorded as the lifecycle-admission debt entry. Local remains trusted-host
execution; blocked backends (macOS HVF, MXC, WSLc) remain explicitly blocked.

## 1. Reconnaissance

Method: read-only donor clones pinned at fixed SHAs, one recon report per donor
plus a baseline and an environment report, each citing `file:line` and labeling
claims OBSERVED / SOURCE-SUPPORTED / INFERRED / UNKNOWN; a synthesis pass
resolved conflicts mechanically (opening the named files) before design.
Artifacts (sea-rs `.tmp/harvest/`): R1–R6 (donor/baseline/upstream/licensing
reports), capability-matrix.md, architecture-design.md, OPERATOR-DECISIONS.md.

Donors (SHAs verified by `git log -1` in the clones, per R6 §1):

| Donor | Pinned SHA | License | Used for |
|---|---|---|---|
| Cageforge (`.tmp/donor/cageforge`) | 9d3af695b0c67d228dc223294397c054e3488a22 | Apache-2.0 (vendored Bubblewrap area LGPL-2.1-or-later — avoided entirely) | F05 pinned-dir identity technique; F08-Windows job verification |
| heel (`.tmp/donor/heel`) | 6db173aa0baf48ef06c3e946b9b074e7ae81ac9c | MIT OR Apache-2.0 | F15 guarded-precondition pattern; F08 creation-time job discipline (concept) |
| ProcessKit-rs (`.tmp/donor/ProcessKit-rs`) | ffcecc46a02dadb02e9a92948371271e2205a37b | MIT | F08 Linux cgroup-leaf/pidfd model; adversarial test templates |

Capability matrix (trimmed; full version with per-donor citations in
capability-matrix.md):

| Finding | Gap (pass3) | Chosen candidate | Disposition |
|---|---|---|---|
| F05 | Windows pinned dir re-resolved by name after validation | Cageforge validate-then-identity on the pinned handle | ADAPT |
| F08 | `setsid` descendants survive completion/timeout | ProcessKit cgroup-leaf (Linux) + job verification (Windows) + documented cage-persists (macOS) | COMBINE |
| F15 | Env-gated skips count as libtest passes | Heel guarded-precondition + REQUIRE envs + BLOCKED tally | ADAPT |
| F17 | OpenShell deny probe indistinguishable from blackhole | Host-observed arrival listener + fixture allow/deny pair | ADAPT |
| F32 | Unix safe-write rename source resolved by name | Staging directory outside the attacker's name space | REPLACE |
| F33 | tokio unifies into Windows kernel graphs; gate has no `--target` | Edge adapter + unified-graph gate pass | REPLACE + ADAPT |

Composites (matrix §A, as implemented): C1 — cgroup lease + typed degrade +
containment claim on the execution result; C2 — Windows creation-time job
membership × breakaway-by-omission × verification (F08-Windows half of
4b17a011); C3 — object identity verified at use on both platforms (Cageforge
`FILE_ID_INFO` revalidation on Windows, dev/ino post-check retained on Unix);
C4 — ProcessKit-style adversarial tests wrapped in Heel guards so they cannot
silently skip; C5 — F17 positive-control stack (reachability + allow-arrival +
deny-non-arrival, all host-observed).

Disputes D1–D8 (matrix §B; binding decisions in OPERATOR-DECISIONS.md):

- D1 (rustix direct dep?): resolved during synthesis by opening
  `crates/sea-forge-sandbox/Cargo.toml:30` + `Cargo.lock` — rustix v1 is a
  direct dep; R4 right, R6 §3a wrong.
- D2 (F05 semantics): exclusive pin (`FILE_SHARE_READ`-only) + per-use
  revalidation + loud typed `PinError`; RootDirectory work spiked first.
  Landed in 4b17a011; final form after the lane evidence chain: three INVALID_PARAMETER forms and two SHARING_VIOLATION-under-pin forms recorded as platform verdicts (spike, `#[ignore]`), production publishing via the FileId-verified release-pin rename — green on the windows lane at fcea8c66/1158098a.
- D3 (cgroup on CI): pre-delegated root via `SEA_FORGE_CGROUP_ROOT`; claim
  fails closed when unavailable; ci.yml preflight + `SEA_FORGE_REQUIRE_CGROUP=1`
  in the required lane. Landed in 1ab50130.
- D4 (seccomp `setsid` block on Jail): DEFERRED behind the cgroup lane proof —
  not implemented in this milestone (OPERATOR-DECISIONS "D4").
- D5 (kernel floors): moot for openat2 — the design does not require openat2;
  the preflight probes delegation directly, not kernel version.
- D6 (prior relative-`RootDirectory` rename failure, imp.rs:634 comment):
  production ops moved to the NT-native anchored form (`NtCreateFile` with
  `RootDirectory`, `NtSetInformationFile(FileRenameInformationEx)`); the
  previously failing Win32 form is re-probed — not asserted — by an `#[ignore]`
  spike test. Verdict recorded from the lane: the relative-target and anchored-create forms are rejected on Server 2025; the release-pin rename is green (portability 40507892357).
- D7 (macOS residual): accepted — pgid kill kept; the setsid escapee is now
  censused and claimed as `SandboxCagePersists` rather than papered over
  (2e2d7bc3).
- D8 (F17 grammar): resolved pre-implementation — the existing grammar
  suffices (`@protocol "tcp"` + `@operation "connect"`, IP-literal host;
  OPERATOR-DECISIONS "DomainForge posture"); confirmed by the model.sea diff
  in 32dbf050.

## 2. Findings

Original findings: pass3-findings.md (frozen d28e00dc); baseline mechanics per
finding in R4 §A. Anchors refer to the branch at f4462441 unless a base anchor
is named.

### F15 — prerequisite-dependent skips counted as libtest passes

- Original failure (pass3 F15): outside lanes setting REQUIRE variables,
  `unavailable_network_jail` (conformance_m1.rs:444), `blocked()`
  (changeset_windows.rs:21), and `make_junction` (safe_write_windows.rs:14)
  printed a skip and returned; libtest reported `ok`. Untested enforcement
  counted as passing conformance.
- Root cause: harness/env contract, not backend capability — helpers degraded
  silently instead of failing the lane that required them.
- Mechanism (heel `tests/guarantees.rs:243-291` guarded-precondition pattern;
  pattern-only, no notice due): helpers route through
  `blocked_or_require(env, what, why)` — without the lane's REQUIRE env they
  tally a BLOCKED scenario; with it the test panics (machine-greppable). A
  guard test per suite (`no_blocked_scenarios_in_required_mode`,
  conformance_m1.rs:940; safe_write_windows.rs:152; changeset_windows.rs:164)
  fails a required lane that finished with BLOCKED scenarios.
- Implementation: 1ab50130 (three test files; safe_write_windows.rs also
  carries the F05 junction-planting regression). The junction helper reuses
  the existing `SEA_FORGE_REQUIRE_LINK_PRIVILEGE` env; no new env was added.
  portability.yml needed no change (wiring pre-existing from the prior
  effort); ci.yml gained the cgroup envs (§5).
- Local verification: bite proof run once and reverted (OBSERVED_DEBT:1147):
  with `SEA_FORGE_REQUIRE_LANDLOCK_NET=1` and default libtest parallelism the
  guard passed while the panicking demo failed; with `--test-threads=1` both
  failed deterministically. The per-test REQUIRE panic is the deterministic
  net; the guard is the second, greppable one.
- Residual: the tally guard is best-effort under default parallelism
  (OBSERVED_DEBT:1145).
- Disposition: FIXED and lane-attested (ci.yml 40440809377 ubuntu green incl. the Active setsid scenarios under REQUIRE_CGROUP=1; windows lane 40507892357 green incl. the job tests; macos lane green incl. SandboxCagePersists classification).

### F32 — Unix safe-write rename resolved the temp by name

- Original failure (pass3 F32): `write_entry_unix` created
  `.<name>.sf-tmp-<pid>-<seq>` in the destination directory and renamed by
  name; attacker bytes could be published before the dev/ino post-check
  reported "was substituted" — detection-only.
- Root cause: the rename SOURCE was a workload-writable name.
- Mechanism: temps are created inside a 0700 host-owned staging sibling
  `.sf-staging-<pid>` and published with `renameat(staging_fd, temp,
  dest_dir_fd, name)` — a writer of the destination directory has no name to
  unlink or replant (`crates/sea-forge-sandbox/src/write_staging.rs`; module
  doc states the design). Original code; Bubblewrap's LGPL `safe_openat.c` was
  study-only (reject ledger). Staging rebind refused (O_NOFOLLOW); EXDEV is a
  loud refusal; the dev/ino post-check remains belt-and-braces.
- Implementation: 7d87a398 + fdbbb437 (clippy cross-target fix: `new_temp`
  takes `impl Into<u32>` because `RawMode` is u32 on Linux, u16 on macOS).
  New tests `pre_planted_temp_names_cannot_block_or_capture_the_write` and
  `destination_directory_keeps_no_staging_leftovers`
  (tests/safe_write_staging_substitution.rs); existing lib tests :861–:929 and
  conformance_m1.rs:854 unchanged and green.
- Local verification: sandbox lib+tests green in the W3b gate sweep;
  `just check` EXIT=0 on the full worktree (CURRENT_STATUS rev 184).
- Residual (honest): same-uid staging residual — a workload running as the
  same host uid with a write grant over the destination tree (Linux jail
  Landlock grants, macOS app-sandbox exceptions) can traverse the 0700 staging
  dir and race the temp inside it; the post-check stays detection-only there
  (write_staging.rs "Residual"). Distinct-identity workloads (Isolated) get
  prevention.
- Disposition: FIXED (prevention for distinct-identity workloads; documented
  detection for same-uid).

### F08 — detached descendants survive completion/timeout

- Original failure (pass3 F08): Unix kills were pgid-only and discarded on the
  completion path (`let _ = kill_timed_out_group`, local.rs:149, jail.rs:505);
  `destroy` was a no-op; a `setsid` descendant outlived settlement on Local,
  Linux Jail, and macOS Jail.
- Root cause: group membership was the only containment, and nothing verified
  or claimed it.
- Mechanism + provenance:
  - Linux: new `crates/sea-forge-sandbox/src/cgroup.rs` (538 lines, sync over
    libc) — a synchronous re-implementation of ProcessKit's mechanism
    (`src/sys/linux.rs:644-660` pre-exec `cgroup.procs` join, `:1855-1930`
    leaf-per-spawn, `:2935-3060` pidfd-pinned kill fallback). Leaf under
    `SEA_FORGE_CGROUP_ROOT` (or the process's own cgroup when delegated);
    child joins pre-exec (no uncontained window); teardown = `cgroup.kill`
    first, verified-empty readback, pgroup kill secondary, pidfd pin +
    membership reconfirm in the fallback; cleanup errors propagate instead of
    being discarded. Unavailability is a typed degrade (`report_degrade`),
    never a silent downgrade, and fails the spawn under
    `SEA_FORGE_REQUIRE_CGROUP=1`.
  - Windows (4b17a011): `IsProcessInJob` read-back after assignment
    (Cageforge `runner/parent.rs:236-258` technique); breakaway refused by
    flag omission (ProcessKit `windows.rs:713-722` technique);
    `terminate_and_wait` polls the job empty. Creation-time membership
    (suspended spawn into the job) was already in-tree; now verified.
  - macOS (2e2d7bc3): descendant census sampled during the run (ppid-tree via
    libproc, 100 ms interval, pid+start-time identity with recycle guard);
    after the group kill, survivors are typed `MacTeardown::CagePersists` and
    reported loudly — not errors. `destroy` sweeps the per-run registry.
- Implementation: 7d87a398 (cgroup.rs, local.rs, jail.rs Linux, red test),
  2e2d7bc3 (teardown wiring, macOS census, windows job tests), f4462441
  (composite field, §3). Isolated untouched (structural pid-ns containment,
  isolated.rs:110-118).
- Tests: `setsid_lifecycle.rs` (670 lines): the deterministic red test
  `group_kill_alone_leaves_a_setsid_descendant_running` pins the pre-fix
  behavior (setsid descendant survives a bare group kill and writes late.txt
  after the completion boundary — the finding, reproduced locally), plus
  `required_lane_refuses_to_spawn_without_containment`,
  `unavailable_containment_degrades_and_the_run_completes`, Active-path
  `setsid_descendant_does_not_outlive_*`, `destroy_sweeps_a_leaked_leaf`.
  Windows: `job_list_attribute_places_the_child_in_the_job_at_creation`,
  `breakaway_from_job_is_refused_when_the_job_forbids_it`,
  `completion_job_close_kills_a_running_grandchild` (windows_local_job.rs).
  macOS: `setsid_descendant_survives_the_sweep_but_stays_sandbox_caged`
  asserts all three truths — survivor alive, outside write denied, inside
  write appears (macos_app_sandbox_jail.rs).
- Local verification (WSL2): cgroup2 mounted (kernel 6.18) but the root is
  root-owned and `sudo -n` needs interactive auth, so the Active path never
  executed locally (OBSERVED_DEBT:1186). Local runs verified the
  Unavailable/forced paths, the REQUIRE=1 bite (five scenarios panic with
  `SEA_FORGE_REQUIRE_CGROUP=1 but ... is unavailable`, lane fails), and the
  hazard red test. Windows/macOS: compile-and-lint only (apple-darwin check +
  clippy `-D warnings` green; windows-msvc os check green). Runtime proof is
  attested per lane (see §6 run table).
- Residuals: macOS census window (a descendant forked+setsid'd in the last
  100 ms before exit is invisible), pid-recycle guarded not pinned,
  aborted-run escapees enumerated only to the last census (OBSERVED_DEBT W7);
  Linux Active-path logic (cgroup.kill write semantics, procs read-back
  timing) first exercised on the ci.yml lane.
- Disposition: FIXED and lane-attested per platform (§6).

### F05 — Windows pinned-dir reparse mutation after validation

- Original failure (pass3 F05): `PinnedDir::open` used
  `FILE_SHARE_READ|FILE_SHARE_WRITE`, validated once (reparse attr, final
  path), and every downstream op re-resolved `self.path.join(name)` — a
  junction/reparse swap after validation was honored.
- Root cause: shared-mode coexistence with writers + name-based re-resolution.
- Mechanism (Cageforge `setup/pinned/file.rs:146-243` validate-then-identity
  and `filesystem/path.rs:337-343` `FILE_ID_INFO`, adapted — re-implemented on
  windows-sys 0.61, no `windows` 0.62 crate taken): share mode is
  `FILE_SHARE_READ` only (`PINNED_DIR_SHARE`, imp.rs:454) — a concurrent or
  pre-existing writer makes the pin fail loud (`PinError::WriteShareHeld`);
  child ops anchor on the pinned handle (`NtCreateFile` with `RootDirectory`,
  `FileRenameInformationEx` rename) and revalidate reparse tag
  (`FileAttributeTagInfo`) + identity (`FILE_ID_INFO`) before use;
  `FinalPathMismatch` refuses aliased chains. The prior relative-rename CI
  failure (imp.rs:634 comment) is re-probed by
  `relative_root_anchored_create_and_rename_spike` (`#[ignore]`), not asserted.
- Implementation: 4b17a011 (imp.rs +881/−99 on the branch). The three
  new `PinError` variants required matching arms in the sandbox `pin_error`
  match — sequenced W6→W3b (E0004 hit and resolved before the branch compiled
  for windows-msvc; OBSERVED_DEBT:1246). Sandbox callers map the variants into
  `UnsafePath` text.
- Tests: `pinned_dir_open_fails_loud_when_a_writer_holds_the_directory`,
  `revalidation_refuses_a_swapped_directory_identity`,
  `junction_planting_in_a_pinned_directory_is_refused_and_the_target_stays_empty`
  (appended by 1ab50130), plus the F08 job tests.
- Local verification: windows-msvc check + clippy `-D warnings` for
  sea-forge-os green (OBSERVED_DEBT:1262); NTFS semantics cannot run on WSL —
  runtime behavior attested on the windows lane (os unit tests + windows_local_job green through 40507892357).
- Residuals: exclusive pin fails loud on any external writer of a pinned
  directory (deliberate, fail-closed; materialization is sequential today);
  the spike verdict and every junction/identity test await the windows lane
  (OBSERVED_DEBT:1266).
- Disposition: FIXED and lane-attested (windows lane green through 40507892357).

### F33 — tokio in Windows kernel graphs; gate blind to it

- Original failure (pass3 F33): `sea-forge-os/tokio-pipes` unified tokio into
  ledger/sandbox/settlement on Windows server builds; `just no-async-kernel`
  had no `--target`, so the violation was invisible.
- Mechanism: (a) the tokio-pipes feature and `create_pipe_server` move out of
  sea-forge-os into `crates/sea-forge-server/src/pipe_edge.rs` (the server
  crate denies unsafe_code with one documented opt-out carrying the os crate's
  discipline; crates/AGENTS.md note updated); os keeps the sync pieces
  (`PipeSecurity`, SID helpers). (b) `no-async-kernel` gains a unified-graph
  pass: `cargo tree -i "$dep" -e normal [--target x86_64-pc-windows-msvc]
  --locked` over the whole workspace — a per-crate `-p` pass cannot see
  feature unification (justfile comment documents this).
- Implementation: 4b17a011; gate half landed with W1's justfile edit (1ab50130).
- Local verification: `cargo check --target x86_64-pc-windows-msvc -p
  sea-forge-os` green; the moved adapter compiled in a scratch crate against
  tokio 1.53.3; `just no-async-kernel` green with both graph passes
  (CURRENT_STATUS). The whole-server windows cross-check is blocked on Linux
  by `ring`'s build script — pre-existing, edges untouched
  (OBSERVED_DEBT:1163); the full server compile is first proven on the windows
  lane (green at 40507892357).
- Residual: the gate comment still cites `sea-forge-os/tokio-pipes` as its
  historical example (OBSERVED_DEBT:1181).
- Disposition: FIXED; windows lane attested (40507892357).

### F17 — OpenShell network denial not positively proven

- Original failure (pass3 F17): the deny probe targeted TEST-NET
  `203.0.113.10:443` (unreachable by construction); denial and blackhole were
  indistinguishable; the test itself said so.
- Mechanism (heel reachability-guard idea + SEA-Forge's scenario-8
  control-token precedent): a host-side arrival listener (dependency-free
  python3, `scripts/openshell/egress-control.py`) and one controlled endpoint
  `10.0.2.2:46244` declared in the cell model (`Endpoint "live-egress-control"`
  + `NetworkFlow "live-egress-allow"` with `@operation "connect"` — existing
  grammar, D8) and regenerated into the projection with domainforge 0.21.2.
  New ladder: `live_network_allow_control_connects` must connect AND be
  observed arriving with a run-unique token; `live_network_denial_proven`
  attempts the same reachable target under the default-deny policy and asserts
  non-zero exit, no CONNECTED, and zero new arrival lines. The pre-existing
  narrowed `live_network_egress_is_not_observed` is unchanged.
- Implementation: 32dbf050 — cell_episode_live.rs (+216/−7), live-proof.sh
  (starts/tears down the listener, exports the control address, gates on
  ladder membership), openshell-live.yml (uploads `egress-arrivals.log` as
  evidence, renders it in the job summary), fixtures (`model.sea` +8,
  regenerated `policy.yaml` +13 with the `cell_live_egress_allow` policy,
  `openshell.lock`/`mapping-report.json` hashes updated by the tool).
- Local verification: stub suite green (case-runner clippy+tests in the W3b
  sweep); fixture regeneration byte-identical with the released 0.21.2 CLI.
  The ladder runs only on the KVM lane — run 40384823244: 10/11 pass; the allow-control is BLOCKED by upstream capability (see §2 F17).
- Residual: the controlled endpoint is `10.0.2.2` (QEMU guest→host alias). If
  the lane's networking does not deliver guest connections to a host loopback
  listener, the allow variant reds — a TRUE finding to diagnose first
  (OPERATOR-DECISIONS decision rule), before touching images or DomainForge.
- Disposition: implemented; the deny variant is live-proven (40384823244) and the allow-control is BLOCKED with the three-round upstream-requirement chain.

## 3. Composite: ExecutionResult.lifecycle_containment

`crates/sea-forge-core/src/types.rs:551` declares the public enum (serde
externally tagged, snake_case); `ExecutionResult` gains a required
`lifecycle_containment` field (f4462441). Population truth table:

| Backend / path | Value | Claim semantics |
|---|---|---|
| Linux Local/Jail with a cgroup lease | `CgroupLeaf { kill_verified: true }` | Ok return implies verified-empty teardown; a failed kill propagates as Err, never as a claimed result |
| Linux Local/Jail, containment unavailable | `ProcessGroup` | typed degrade already reported on stderr; never claims CgroupLeaf |
| Windows Local | `JobObject` | membership verified fail-closed (`IsProcessInJob`) before any result exists |
| macOS Jail, census finds no survivor | `ProcessGroup` | verified group kill |
| macOS Jail, setsid survivor | `SandboxCagePersists { survivor_count }` | the documented F08 residual: survivors outlive the run; the inherited App Sandbox cage still denies ungranted writes |
| Isolated | `PidNamespace` | structural kernel guarantee (namespace dies with its init); no read-back claimed, stated as such |
| OpenShell | `ExternalSandbox` | gateway-managed lifetime |
| spawn-failure paths | `NoProcess` | |

Flow: every constructor sets the field explicitly (local/jail/isolated/openshell
via the shared `local::execution_result` helper; cli `plan_pipeline.rs`
write-only item; settlement constructs `NoProcess` at
sea-forge-settlement/src/lib.rs:491, and its criteria-provenance test pins the
provenance surface). Wire shape pinned by
`lifecycle_containment_uses_tagged_snake_case_json` (sea-forge-core).
Active-lane assertions (`CgroupLeaf`/`JobObject`/`SandboxCagePersists`) are
compile-gated in the lane suites and attested by them (windows 40507892357, macos 40507892357, ubuntu 40440809377). Teardown
failures remain `Err` — the field only ever carries verified claims.

## 4. Upstream

OpenShell v0.1.5 investigation (R5 §1, full release notes read): nothing
unblocked. MXC exec/upload for callers still absent (in-driver exec only, no
upload API, caller config admin-gated); no WSLc driver; digest-bound native
executable execution still unsupported (`native_identity`: zero upstream hits;
the refusal class stays SEA-Forge-local); no operator-controllable egress
endpoint; Windows guest refused upstream (issue #3956, closed not_planned).
Adjacent v0.1.4/5 items (VM image digest pinning #4342, guest startup
hardening #4539, prover sibling-endpoint fix #4326) do not change the
integration surface. The integration stays pinned at OpenShell 0.1.3; the
breaking `outer_fence` → `provisioning_evidence` descriptor rename (#3898) is
noted for any future bump.

DomainForge: bumped `domainforge-core =0.21.1` → `=0.21.2` (56c6b51e;
crates/sea-forge-domainforge/Cargo.toml; `EXPECTED_DOMAINFORGE_VERSION` moved
with the pin because `load_validate` refuses version drift). Proof the bump is
a no-op for the committed fixture: it regenerates byte-identical with the
released 0.21.2 CLI, and 0.21.1 output is refused by the 0.1.3 supervisor for
exceeding the projected boundary (missing `/app`, `/var/log` in
`BASELINE_READ_ONLY` — upstream PR #162). The compiler-upgrade lineage tests
were migrated to the digest the linked 0.21.2 build mints, with the
equal-producer-version refusal case reading the linked version symbolically so
it survives future pin bumps. No DomainForge repo change was needed (operator
posture honored).

## 5. CI lanes and live-evidence accounting

Only ci.yml and openshell-live.yml were modified; portability.yml and
kernel-security.yml are untouched by this branch.

- ci.yml (required gate): new "cgroup-v2 delegation preflight (F08 lane)"
  step — probe runner-user `mkdir` at the cgroup root; else self-delegate via
  passwordless sudo (`mkdir` + `chown` + `echo +memory +pids >
  cgroup.subtree_control`, idempotent, missing controllers tolerated) and
  re-probe; success exports `SEA_FORGE_CGROUP_ROOT` +
  `SEA_FORGE_REQUIRE_CGROUP=1`; failure exports `REQUIRE_CGROUP=0` with a
  `::warning::` that F08 scenarios will be BLOCKED, not passed. The unix lane
  also sets `SEA_FORGE_REQUIRE_LANDLOCK_NET=1` (pre-existing).
- portability.yml (feat/** push): windows lane runs safe_write_windows,
  windows_local_job (incl. the new job tests), sea-forge-os lib (incl. the
  `#[ignore]` spike via `--include-ignored`), with
  `SEA_FORGE_REQUIRE_LINK_PRIVILEGE=1` (pre-existing wiring); macos lane runs
  macos_app_sandbox_jail.rs (9 tests incl. cage-persists).
- openshell-live.yml (workflow_dispatch, KVM): the F17 ladder;
  `egress-arrivals.log` is uploaded as evidence and rendered into the job
  summary; a missing allow arrival fails the ladder.
- kernel-security.yml: PR path-filtered (existing `paths:` filter); untouched.

Accounting rules (binding for reading the lanes): only exercised scenarios
count as proof; BLOCKED is not PASS — the ci.yml preflight deliberately keeps
the lane green with `REQUIRE_CGROUP=0` while the F08 scenarios report BLOCKED
instead of claiming containment. A green lane with REQUIRE vars unset proves
nothing about the enforcement paths. CI results are recorded only from run
logs, never inferred from local passes.

## 6. Verification log

Local rows ran on WSL2 (Linux); lane rows cite their GitHub Actions runs.
Only exercised live scenarios count as conformance evidence (§5).

### Local (all green at the recorded SHAs)

| Command / check | Outcome |
|---|---|
| `just check` (full worktree) | EXIT=0 at revision 184 state; re-run green after every wave |
| `just no-async-kernel` (host + unified windows-msvc graph, fail-closed cargo-status capture) | green through HEAD; sabotage test: a resolution error now fails the gate, cargo's genuine absence text is the only pass-through |
| windows-msvc target check+clippy `-D warnings` (os, sandbox) | green through HEAD |
| aarch64-apple-darwin check+clippy `-D warnings` (sandbox all-targets) | green through HEAD |
| sandbox suite (23 test targets; 27 lib tests incl. the in-place tamper detection) | green |
| changeset / case-runner / agent / server suites | green |
| F15 bite proof (REQUIRE envs) | per-test panic deterministic; guard second net |
| F08 red test (`group_kill_alone_leaves_a_setsid_descendant_running`) | deterministic red reproducing the pre-fix hazard |
| F17 payload simulation (fake CONNECT proxy: grant, refuse, leak, no-proxy paths) | all four behaviors verified off-lane before the KVM run |
| domainforge fixture regeneration byte-identity | byte-identical tool output with the 0.21.2 CLI |

### CI lane results (authoritative run IDs)

| SHA | ci.yml | portability (windows) | portability (macos) | kernel-security | openshell-live (KVM) | casework |
|---|---|---|---|---|---|---|
| f4462441 | fail (cgroup join EACCES) 40369591334 | fail (attribute-pointer AV) 40369591283 | pass | pass 40369591426 | fail (prover refuses tcp rules) 40369641957 | pass |
| 8816d198..c28b9ce4 | fail→diagnosed (delegation containment; pidfd degrade) 40384807768 | fail→diagnosed (rename 87; relative-form verdicts) | fail→diagnosed (nix-dylib helper) | pass 40400356787 | 10/11 pass, allow-control fails on raw connect 40384823244 | pass |
| 788c77d7/ca778ae2 | fail→diagnosed (destroy-sweep cross-test kills) 40410651296 | fail (87 under pin) 40410651287 | fail (census race) | pass | — | pass |
| 22ea8dd5 (orphan-scoped sweep, spawn/finish split, FileId-verified release rename) | **success** 40440809377 | fail (junction-test semantics) 40440803522 | pass | fail (artifact collision; evidence lost) | — | pass |
| 7f172fba | success 40444187422 | fail (breakaway 123-vs-5) 40444187212 | pass | **pass both VMs** 40444187271 | — | pass |
| 61b3660e / c3cf2300 | success 40446093604 | fail (control create itself fails 123) 40446093616 | pass | pass | — | pass |
| 1158098a (self-exec probe) | success 40504750558 | **windows success**; macos census flake 40504750501 | — | pass 40504750686 | — | — |
| fcea8c66 (census 1s window) | success 40507892479 | **success both** 40507892357 | **success** | fail (guard race, evidenced by de-collided artifact) 40507892410 | — | — |
| 1939e65a (guard env_lock) | pass (re-run of 40521505235 set) | pass (re-run 40521505171) | pass | **pass both VMs after flake re-run** 40521505166 | — | — |
| 3d5a8954 (pass-1 fixes) | run in flight at writing | run in flight | run in flight | re-triggered by paths | — | — |

KVM verdict (run 40384823244, branch round 3): 10/11 ladder tests PASS live
including `live_network_denial_proven` (non-arrival on a demonstrably
reachable target) — the allow-control is BLOCKED by upstream capability, not
skipped. The denial proof remains non-attributed to policy enforcement (the
upstream requirements are recorded in §2 F17); scenario 4 stays BLK in the
conformance matrix.

## 7. License / provenance ledger

Donor facts per R6 §1; obligations per matrix column 12 and architecture §2f.

| Mechanism | Donor + location | License | Code vs pattern | Obligation and where honored |
|---|---|---|---|---|
| F05 pinned-dir validate-then-identity (`FileAttributeTagInfo`, `FILE_ID_INFO`, final-path confirm, exclusive readback share) | Cageforge `setup/pinned/file.rs:146-243`, `filesystem/path.rs:337-343` | Apache-2.0 | Techniques adapted; re-implemented on windows-sys 0.61 (no donor code copied; no `windows` 0.62 crate taken) | Technique-only: module-doc provenance suffices per architecture §2f; imp.rs documents the pin semantics. Full attribution (LICENSE + "Mansur Azatbek" + NOTICE preservation) becomes due only on verbatim transplant |
| F08-Linux cgroup leaf / pre-exec join / `cgroup.kill` + pidfd fallback | ProcessKit `src/sys/linux.rs:644-660, 1855-1930, 2935-3060` | MIT | Synchronous re-implementation of the mechanism over libc (`cgroup.rs`); no Pk source transplanted | Pattern reimplementation ⇒ no notice strictly required (matrix F08 col 12); MIT text + "Copyright (c) 2026 Anton Zhelezniakou" due only on verbatim transplant |
| F08-Windows job verification (`IsProcessInJob` read-back, breakaway-by-omission, wait-until-empty) | Cageforge `runner/parent.rs:236-258, 357-376, 417-470`; ProcessKit `src/sys/windows.rs:674-722, 713-722` | Apache-2.0 / MIT | Techniques re-implemented on windows-sys 0.61; spawn shape unchanged | Same: technique-only, no notice files added |
| F08-macOS census / typed cage-persists | ProcessKit's documented residual model (`src/mechanism.rs:23-25`); original implementation | MIT (concept only) | Original code | None |
| F32 staging-dir publish | None (own design); Bubblewrap `safe_openat.c` algorithm studied only | n/a (LGPL source explicitly NOT used) | Original code (`write_staging.rs`) | None; module doc records the design rationale and the same-uid residual |
| F15 guarded-precondition + tally | heel `tests/guarantees.rs:243-291` | MIT OR Apache-2.0 | Pattern-only | None |
| F17 ladder | Own patterns + existing SEA-Forge precedents (scenario-8 control token, FixtureServer) | n/a | Original | None |

Blanket facts: no new runtime dependencies (the Cargo.lock delta is the
domainforge-core bump + hash lines only, 56c6b51e); the cageforge-bwrap
LGPL-2.1-or-later vendor area was never opened for transplant; donor clones
remain read-only at their pinned SHAs under `.tmp/donor/`; cageforge's NOTICE
asserts no copied Codex source — that independence record would be inherited
by any future verbatim transplant (R6 §1 caveat). SEA-Forge LICENSE:15-17
("third-party components keep their original licenses") remains the governing
clause if obligations ever crystallize.

## 8. Known residuals and deferred items

- sea-forge-cli lifecycle test `post_start_internal_error_is_run_correlated_and_traced`
  fails on pristine HEAD on this WSL2 host — stash-verified pre-existing, not
  caused by this branch; main CI is green (CURRENT_STATUS "Limits"). A
  different, load-sensitive pre-existing flake in the same suite is already
  ledgered (OBSERVED_DEBT:381).
- F17 `10.0.2.2` host-alias assumption: the positive control depends on the
  KVM lane's guest→host route; a red allow variant is a true finding to
  diagnose, not a test bug to paper over.
- cgroup controller list: the preflight enables only `+memory +pids` (what the
  scenarios observe); the kill path is core cgroup-v2 (`cgroup.kill`, kernel
  ≥ 5.14). Controllers the kernel lacks must not fail the probe (workflow
  comment).
- D6 spike verdict RECORDED (three INVALID_PARAMETER forms, two SHARING_VIOLATION-under-pin forms; the working mechanism is the FileId-verified release-pin rename), and
  per-use revalidation is the load-bearing backstop if the anchored path ever
  regresses (OBSERVED_DEBT:1279).
- Same-uid staging residual (F32): detection-only for same-uid workloads with
  destination-tree grants (write_staging.rs).
- macOS residuals (F08): 100 ms census window; pid identity is pid+start-time
  (excluded, not pinned, on recycle); aborted-run escapees enumerated only to
  the last published census (OBSERVED_DEBT W7).
- F15 tally guard is best-effort under default libtest parallelism; the
  deterministic net is the per-test REQUIRE panic (OBSERVED_DEBT:1145).
- Windows-target cross-check of sea-forge-server is blocked on Linux by
  `ring`'s build script (pre-existing); the full server compile is first
  proven on the windows lane (OBSERVED_DEBT:1163).
- DEFERRED (binding): D4 — a heel-style seccomp
  setsid/unshare/setns/CLONE_NEWUSER blocklist on the Jail pre-exec waits
  behind the cgroup lane proof and an owner scope decision
  (OPERATOR-DECISIONS).
- Local WSL2 cgroup delegation: the Active path is exercisable locally only
  after `sudo` setup per the setsid_lifecycle module docs; until then the
  ci.yml lane is the only executor (OBSERVED_DEBT:1207).
- Cosmetic: the `no-async-kernel` comment still cites the removed
  `sea-forge-os/tokio-pipes` as its motivating example (OBSERVED_DEBT:1181).
