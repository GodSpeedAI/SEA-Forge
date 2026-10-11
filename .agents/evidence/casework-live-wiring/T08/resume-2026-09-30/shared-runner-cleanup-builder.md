# Shared live conformance runner: original repair instructions and result

Fresh Luna builder production_import_guard_builder was assigned only
apps/godspeed-cognitive-ui/e2e/live-conformance.ts after independent source rejection.

Original bounded requirements: replace the fixed binary artifact with a unique owned run
directory using existing standard library; preserve failed cell/evidence; clean only owned
kernel/gateway processes, signal gracefully then await real exit with bounded timeout and
owned-process escalation if needed; never kill an unrelated listener. Preflight hardcoded4179
and do not accept a foreign health response as proof of own startup. Preserve the identical
shared conformance body and fixtures. No dependencies, kernel/schema/interface changes,
fake adapters, build/test commands or self approval.

Builder reports source stable: mkdtemp run root owns binary/cell; Bun.listen probes4179;
health requires own gateway alive; SIGTERM then5s exit wait, owned SIGKILL then5s wait;
cleanup failures fail the run and retain the cell. Success removes only the owned run root.
Only git diff --check ran and passed. No runtime gate ran. Root inspected the diff; independent
critic must verify compilation, actual cleanup and shared live conformance before approval.
