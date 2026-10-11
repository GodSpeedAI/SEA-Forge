# PhaseB semantic repair assignment

Fresh builder t09_run_children_fixture_builder addresses immutable independent REJECT
run-children-phase-b-independent-semantic-reject.md (SHA8404a0f35fb37e0705bad134f9d672fd06231d9335c59c3925e3c04fd4bdd8f3).
Read ORIGINAL runtime assignment, test-phase supplement, fixture clarifications and real-
kernel proof supplement. Existing passing inner race04 is explicitly rejected as approval:
it passes a false fixture assumption and blanket refusal normalization.

Own ONLY internal/projection/live.go, internal/adapters/sfwp/authority.go and the two
new scoped_live_test.go/scoped_runs_test.go fixtures. No other source/tests/golden/Ask/
live-proof changes. Root authorizes correcting the frozen fixture's false semantics,
preserving every other assertion and strengthening coverage. No dependencies or APIs.

Phase1: change ONLY the two scoped tests first; production remains frozen at rejected
hashes. Missing requested Record.Ref is a legitimate unknown case regardless of other
listed cases. Replace record-ref fixture's incorrect unavailable/one-scoped expectation
with precise Invalid/not-found and no reads after ListCases. Cover both empty and nonempty
case lists without requested case; require exactly1 ListCases and0 overview/horizon/
approval/scoped/legacy calls. Add positive list with a foreign row before the requested row
to prove exact selection. Preserve all other view/ownership/duplicate/overlap guards.

Extend refusal coverage beyond explicit class unavailable: choose existing Refusal.kind
classes for Invalid, AuthorityDenied, Unavailable and Internal, asserting exact expected
apperr.Kind and preserved RefusalClass through errors.As. Account existing server_busy
pre-admission retry if using that class; never change client retry semantics. Original
unavailable test and its required normalization remain. Map both adapter error branches.

Freeze test hashes/coverage before root transfers sole token to the independent critic
for compiling regression RED against the rejected production. No builder compiler token.
After independent RED/root verification, Phase2 source repair restores immediate Invalid
for missing exact requested Record before later reads, irrespective of list length. Do
not substitute first/foreign records or defer validation to satisfy a test. In both adapter
refusal paths normalize ONLY explicit RefusalClass unavailable, preserving error chain;
all other class/kind mappings remain intact. No generic all-refusal wrapper/global kind change.

Keep all valid-case scoped ownership/counters/clone/child behavior and original fixtures
intact. Source freeze then independent comprehensive GREEN, actual two-case proof, fresh
canonical/full Go and archive identity checks. Future persistent evidence corrections use
NEW paths plus append-only errata; never overwrite/delete even a failed transcription.
Existing actual command outputs remain authoritative; wrong archive versions are explicitly
labelled historical capture errors. Full T09 remains pending later units/global settlement.
