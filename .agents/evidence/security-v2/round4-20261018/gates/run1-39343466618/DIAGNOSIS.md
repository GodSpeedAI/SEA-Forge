Kernel-security run 1 (39343466618, merge SHA 0a950fa4): job failed, and the
failure is a runner-environment defect now fixed in the launcher.

Preflight worked as designed: Landlock ABI 7 (native RESOLVE_UNIX
NOT-YET-EXECUTABLE there, as predicted), the Ubuntu 24.04 AppArmor userns
restriction relaxed via sysctl with recorded before/after (userns AVAILABLE
after), and - the key result - "user-notify listener install: AVAILABLE
(fd 3)" in a disposable child: the supervisor mechanism is exercisable on
GitHub runners.

With isolation available the suite ran (not skipped) and one test failed:
the helper binary was refused execve inside the mapped namespace
("unshare: failed to execute .../sea-forge-workload-exec: Permission
denied"). Root cause: the helper built under `devbox run` carried a
restrictive mode, and a single-UID mapped namespace sees the file's owner
as unmapped (nobody), so only the other bits decide execve. Reproduced
locally: mode 700 -> exactly that EACCES; 755 -> executes.

Fix (this commit): `ensure_helper_reachable` in the launcher - the helper
is verified other-rx before launch, repaired when the launcher owns the
file (bits it hands to untrusted code anyway), refused otherwise. The
gates script's UNAVAILABLE path and NOT-EXECUTABLE mutations behaved
exactly as designed on this run, so no verdict was fabricated.
