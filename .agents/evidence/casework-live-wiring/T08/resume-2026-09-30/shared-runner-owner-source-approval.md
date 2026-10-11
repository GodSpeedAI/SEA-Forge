# Shared runner owner repair: independent source approval

Independent Luna critic t07_live_confirmation reviewed fresh builder
production_build_environment_builder's original shared-runner-owner-review.md instructions
and resulting live-conformance.ts. **APPROVE source only, conditional on actual runtime.**

Evidence: preflight now runs after Go build immediately before gateway spawn;
runnerOwnsGatewayListener intersects only the child PID's socket descriptor inodes with
the exact IPv4 loopback4179 LISTEN inode. Missing/mismatched/restricted procfs fails closed
without emitting process tables. Health checks own process alive and listener ownership
before and after fetch. This closes the earlier foreign-response finding.

Unique temporary binary/cell, failed evidence retention, identical shared conformance and
owned SIGTERM5s/SIGKILL5s awaited cleanup remain. The residual bind-probe race is handled
by listener identity verification. Material change is proof of endpoint ownership, not a
production identity/interface change. Critic inspected source/diff and ran diff --check;
no compile or runtime command. Actual shared live and T08 native gates remain required.
