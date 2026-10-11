# Independent review: root private manager design decision

Date: 2026-10-06  
Reviewed artifact: `run-observation-manager-root-private-design-decision-resume-oct06.md`.  
Disposition: **APPROVE the authority and scope of this private design decision, with no source approval.** I found no remaining user-consent boundary or governing normative conflict for the stated private limits and test-first unit, provided the implementation stays within the decision's explicit unexported, unwired scope.

## Governing evidence

- The standing T09 contract approval in `.agents/current_status.yml` (`operator_decisions.T09-contract`) covers execution observations and the protected Ask endpoint while keeping kernel verbs, identity model, and dependencies authoritative; it does not authorize deployment or a new public contract.
- The plan protocol in `.agents/current_status.yml` (`agent_protocol.loop`, `delegation`, `stop_and_ask_when`) directs continuation through T12 without waiting, assigns semantics/architecture/final decisions to the orchestrator, and lists the actual stop conditions: redesign trigger/correction protocol, unapproved dependency, security/identity expansion, unauthorized external/destructive action, or an unfixable baseline-gate regression.
- The plan's operating contract preserves the kernel as sole authority and requires additive wire changes only. The current T09 proposal assignment expressly limits the manager to a private architecture proposal and holds public HTTP/SSE/cursor/kernel/interface/identity changes.
- The governing casework spec's `run_trace_observation` contract already specifies eight selected runs/eight initial reads, a 1 MiB **initial projected envelope**, sixteen process pollers, two concurrent `run_get` reads, a one-second interval, 1,024 retained frames, and excludes aggregate transport, upstream directory enumeration, and aggregate kernel journal reads. The root decision correctly keeps the proposed 1 MiB current-retained-value budget distinct from the accepted 1 MiB initial DTO cap and does not reinterpret the exclusions as heap bounds.
- The original manager assignment already authorizes a private manager blueprint with process-shared poller admission, exact watcher authorization, explicit bounded hydration behavior, and join-before-capacity-release; it does not define a cohort-lease cap or a retained-value image cap. Those are newly selected internal bounds rather than changes to wire/schema contracts.

## Consent and normative assessment

The 16 preparing/active/draining cohort leases, 128 maximum attachments, and deterministic 1 MiB serialized current-value image are meaningful availability/resource choices, but their stated effects are private admission and fail-closed observation availability. The root decision keeps all manager/type/helper names unexported, preserves the existing DTO and ports, and changes no persisted data, ID grammar, policy precedence, security/identity rule, dependency set, or architecture boundary. It does not add a kernel read cap, public SSE/V4 frontier, deployment authorization, or public error/interface. That places these choices within the operator's existing delegated architecture authority and the private T09 implementation scope; the draft's repeated “operator approval required” clauses were not themselves a governing ask-first rule.

I checked the listed stop conditions against the decision: no public redesign trigger or correction protocol is being crossed by the private limits; no dependency, identity/security expansion, external write, destructive action, or out-of-scope architecture change is proposed. The root decision preserves `CW-23`, the existing 32 MiB Go response-line cap, and the distinct 64 MiB kernel whole-journal and 1,024-frame projection limits. I found no requirement in the governing spec or approved T09 contract that makes these private limits operator-approval-only.

## Bounded approval boundary

Approve only the root's authority to begin the explicitly TESTONLY, unexported manager unit, starting with deterministic focused tests and an independently reviewed actual RED. This review does **not** approve manager production code, caller wiring, public API/SSE/V4 behavior, a public capacity promise, runtime GREEN, deployment, or T09 settlement. Implementations must retain the frozen design's authorization and present-context checks, shared physical admission owner, exact ledger, fail-closed overflow behavior, no lock-held calls/waits/cancellation/joins, and actual read/client-retirement/worker join before releasing capacity. Test-first evidence and an independent critic remain required before any production source release.

No tests, compiler, scanner, Git, or runtime command was run for this review. Graft was used for first-pass source context; its two calls saved approximately 43,571 and 30,175 tokens.
