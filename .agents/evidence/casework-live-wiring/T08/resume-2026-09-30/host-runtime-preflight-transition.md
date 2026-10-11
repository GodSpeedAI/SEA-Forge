# T08 runtime preflight: host process transition

Root performed approved direct host metadata checks before independent native testing.
Previously recorded test PIDs/listeners were no longer present: ports4178,4179,44279,44280
were all free. Historical T07 proof records remain evidence of their actual prior runs;
their PIDs must not be reused as current process identities.

The unexpected Bun PID13814 was sleeping with RSS4748kB in a different project, confirmed
using only its executable/cwd/state/memory metadata. It was preserved; no environment or
command arguments were read. Root observed available RAM1.2GiB and swap6.9GiB free.
Critic's subsequent immediate preflight reported about1.47GB available, no active Go/Rust/
Vite/browser compiler/test process, existing kernel binary and agent-browser present.

The sole compiler owner is t07_live_confirmation. Native live-tag Go race/count1/package1
test started serially; actual outcome remains pending. No process cleanup used stale PIDs
and no unrelated process was killed. This preflight is not passing runtime evidence.
