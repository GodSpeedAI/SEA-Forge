# Safe trace-port fixture preparation

Independent read-only preparation by t09_ask_fixture_independent_critic; no source/test
writes or compiler, no interface/implementation release. Root must specify semantic port
and projection before fixtures. Existing artifact projection remains a separate contract.

Anchors: sfwp/frame.go308-313 NewRunGet emits run_get/run_id; 686-694 narrow artifact wire
view intentionally omits timestamps/status/raw payload. authority.go744-780 illustrates
request/decode/map; artifact_provenance.go11-54 owns artifact authorization joins. Reuse
client_test.go41-65 fake server/request counting or artifact_provenance_test.go16-88
deterministic net.Pipe harness. Do not widen artifact view to smuggle live observations.

Cases to specify: exact requested run/case/item identity; ten allowlisted kinds with real
event IDs/timestamps; optional command_finished payload.execution.status/exit_code only;
synthetic public markers in ignored actor/raw payload/output must never cross the semantic
return type; malformed/duplicate IDs, invalid timestamp/standing/status/trace shape,
unknown trace kinds; JS exact integer boundaries plus fractional/string/bool/i64 overflow.
Each new port method is absent today; runtime interface assertion can provide compiling
RED if declarations are needed, with any declaration dependency explicitly recorded.

Boundaries: preserve all true safe-frame counts while retaining at most1024 newest safe
rows in source order; test1024/1027/empty and exact omission/truncation, not fabricated
identity/times. This does not bound upstream journal or aggregate read work. No trace
buffers/frames enter CaseFacts, captured snapshots or existing artifact provenance DTO.

Root checked canonical types.ts323-335: execution_status/exit_code are optional NON-null
fields on the outgoing safe frame. Raw Rust payload/Option<i64> may differ; future fixture
assignment must distinguish raw absent/null representing no recorded value from canonical
wire null, which is not allowed. The preparer's nullable suggestion is not approval to
change canonical schema. Similarly unknown-kind omission versus malformed allowed-row
failure, duplicate identities and exact count semantics require explicit root decisions.

No API or projection design is approved by this preparation. Unit5 also depends on actual
blocked-read cancellation/drain proof recorded in cancellation-source-facts, effective
session fanout authorization, cohort budgets and poller/cache lifecycle design.
