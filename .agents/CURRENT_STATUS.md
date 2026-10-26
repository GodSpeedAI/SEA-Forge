# Current status

**Status revision:** 182

**Stage:** capability-harvest: F15/F17/F32/F33 and Windows F05+F08 code-complete on feat/hardening-capability-harvest; macOS F08 wave and lifecycle wiring in flight

**Summary:** Harvest wave from Cageforge/Heel/ProcessKit recon (.tmp/harvest/): F15 prerequisite skips now BLOCKED-tally and hard-fail required lanes, ci.yml gains the cgroup-v2 delegation preflight; F33 resolved by moving the async pipe adapter to the server edge (tokio leaves the Windows kernel graph; no-async-kernel gained a unified-graph windows-target pass); F05 PinnedDir opens FILE_SHARE_READ-only with per-use reparse-tag and FILE_ID_INFO revalidation and anchored NtCreateFile/rename child ops; F08-Windows gains IsProcessInJob read-back and breakaway/grandchild containment tests; F32 safe writes stage in a 0700 host-owned sibling staging dir and publish by dirfd rename (destination-visible temp eliminated); F08-Linux adds cgroup-v2 leaf-per-spawn containment (pre-exec join, cgroup.kill, pidfd-pinned fallback) with typed degrade; F17 live ladder gains an allowed-control and a denial-proven test against one host-observed arrival listener; domainforge-core bumped =0.21.1 to =0.21.2 (fixture regenerates byte-identical).

**Verified:**

- cargo test -p sea-forge-sandbox: 26 lib + conformance_m1 19 + staging 2 + setsid suite green locally (Unavailable/BLOCKED paths; Active path deferred to the CI delegation lane)
- cargo test -p sea-forge-case-runner: openshell_stub 35, cell_episode 24, opa_candidate 29, cep suites green after the 0.21.2 bump
- just no-async-kernel: green on host and x86_64-pc-windows-msvc unified graphs
- cargo check --target x86_64-pc-windows-msvc and aarch64-apple-darwin green for sea-forge-os and sea-forge-sandbox
- F08 red evidence: deterministic setsid-descendant survival under group kill pinned in setsid_lifecycle.rs

**Limits:**

- F08 cgroup Active-path unproven until the ci.yml delegation lane runs (no sudo locally)
- F17 denial proof unproven until the openshell-live KVM dispatch runs (10.0.2.2 host alias unverified)
- F05 anchored-rename spike and NTFS/job tests need the portability windows lane
- macOS F08 cage-persists wave and the ExecutionResult lifecycle-capability wiring are not yet landed
- Local remains trusted-host execution on every OS

**Next move:** Land the macOS wave and the lifecycle-capability wiring, commit the F08 jail/local lifecycle slice, push the branch, run ci.yml + portability.yml + the openshell-live dispatch, then the independent codex review.

**Evidence:**

- .tmp/harvest/
- .agents/OBSERVED_DEBT.md
