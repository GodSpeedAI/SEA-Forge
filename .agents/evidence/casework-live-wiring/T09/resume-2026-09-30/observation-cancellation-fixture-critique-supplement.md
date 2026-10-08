# Cancellation fixture critique and required supplement

Independent source critique by t09_ask_fixture_independent_critic found the root proposal
within the current client boundary, but not yet sufficient for fixture release. Original
observation-cancellation-test-first-assignment.md remains binding plus this supplement.
No Go writes or compilers were released; this is a preimplementation scope repair.

Material gap: generic conn.call cancellation affects record-writing Ask/mutations too.
Existing Ask EOF/timeout/overflow/busy tests(ask_transport_test.go72-177) and mutation
correlation tests(client_test.go196+,response_limit_test.go284-332) do not prove manual
cancellation behavior. Add synchronized in-flight Ask cancellation: exactly1Ask send,
no resend and no request_get_status. Add correlated mutation cancellation: exactly1
mutation send, no mutation resend and no status recovery request admitted under canceled
caller context. Require bounded cancellation/join, correct existing uncertainty/error
classification, and preserve context cancellation in the error chain where applicable.
Preserve a VALID received record-writing response completed BEFORE cancellation (Ask or
correlated mutation), plus exact idle connection identity/reuse. Keep all original busy,
ambiguity, correlation and cap tests unchanged in final gates.

Fake-peer caveat: newFakeServer handler(client_test.go41-90) runs synchronously; a handler
blocked awaiting test release cannot observe EOF until it resumes. Use net.Pipe pattern
(artifact_provenance_test.go20-66) or a peer goroutine that completes actual request read
then waits in a read and signals EOF. A later released handler write error is not prompt
connection-close proof. No guessed sleeps. Callback retirement/reuse must be established
with exact peer identity and dial counts, not merely absence of panic.

Source concerns: conn.close183-186 writes dead; call191-228 and pool release339-349 have
different locking. Cancellation callback cannot race this field. Affect only owned
net.Conn I/O, synchronize/retire callbacks before pool return, and join owned worker/peer
lifetimes. Root retains implementation design; independent critic must recheck these
ORIGINAL+supplemented fixture instructions before Go write release. No runtime approval.
