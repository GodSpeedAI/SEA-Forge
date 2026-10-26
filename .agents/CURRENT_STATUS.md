# Current status

**Status revision:** 186

**Stage:** capability-harvest: windows FFI round-1 failure fixed (UpdateProcThreadAttribute pointer fault, backup-intent opens); cgroup Active-path and macOS census fixes in progress

**Summary:** Windows lane forensics: the STATUS_ACCESS_VIOLATION was PROC_THREAD_ATTRIBUTE_PARENT_PROCESS being handed the handle VALUE where Windows requires a pointer TO the handle (deterministic kernel32 read fault); fixed, plus FILE_OPEN_FOR_BACKUP_INTENT added to the anchored directory opens (existing child dirs would have failed STATUS_FILE_IS_A_DIRECTORY) and the revalidation test made robust to the \\?\ device prefix. The hard-link test failure is instrumented to print the exact failing call and raw NTSTATUS on the next lane run. The coordinator's inline triage established the cgroup cleanup uses plain rmdir on an empty leaf and the userns mount repro is impossible on this host, so the cgroup/macOS triage continues via a subagent with CI-log mining.

**Verified:**

- cargo check --target x86_64-pc-windows-msvc -p sea-forge-os --all-targets and clippy -D warnings green on the fix
- sandbox windows-target check green (blast radius limited to sea-forge-os + safe_write_windows.rs)

**Limits:**

- The hard-link replace failure is not yet reproduced; its instrumented failure output arrives with the next lane run
- cgroup EACCES root cause and the macOS TimedOut remain under triage

**Next move:** Land the cgroup/macOS triage fixes, commit both slices, push, re-dispatch openshell-live, collect round 2.

**Evidence:**

- .tmp/harvest/
- gh run 40369591283 / 40369591334 logs
