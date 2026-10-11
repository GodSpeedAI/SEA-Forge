# Shared runner independent ownership review and fresh repair instructions

Independent critic t07_live_confirmation approved the unique directory and bounded owned
cleanup code in source, but **REJECTED runtime readiness**: port4179 is probed before Go
build, so another process can bind during compilation. Health accepts any200 while the
own child is briefly alive, allowing a foreign response during startup failure. No gates ran.

Root assigned fresh builder production_build_environment_builder (not author of the rejected
runner result), source only in live-conformance.ts. Move the probe immediately before gateway
spawn and bind health evidence to that exact child: existing Linux /proc own-PID fd socket
inodes must match the127.0.0.1:4179 LISTEN entry. Read only own socket descriptors and filtered
network metadata; no unrelated command/environment output. Missing/mismatched ownership fails
closed, and health success must check own process alive/port ownership before and after fetch.
Use existing standard APIs, no dependency/public production change. Preserve conformance
fixtures, unique artifacts, failed evidence and awaited owned-process cleanup.

The compile token remains with root until all fresh source repairs are stable, then independent
critic receives it explicitly. This note is an original instruction record, not implementation
approval, runtime evidence or T08 settlement.
