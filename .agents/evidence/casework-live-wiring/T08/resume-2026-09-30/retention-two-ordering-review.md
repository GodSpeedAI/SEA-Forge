# Retention-two replay ordering: independent source proof

Root asked independent critic t07_live_confirmation to check whether a retry before offline
L could turn the intended retained-C replay into future live delivery. No source edit or
runtime command ran. The critic resolved the concern using existing source assertions.

PhaseTwo waits for successful actual ADD and retained Head=L, store length2, oldestC, then
asserts exactly one events request, zero active streams and zero subscribers before waiting
for the later snapshot. The Go serveAPI method records every events request synchronously
before handing it to the API. Phase-state reads the retained Head before taking the request
metadata lock. Any API retry before L would therefore be present and fail the assertion;
if it enters after that read, L is already retained before API processing. The later request
is checked to resume from C and the final store retains C and L.

This is a causal ordering proof from request/state checks, not a measured timestamped
request-versus-L record. It does not independently prove runtime success; the native gate
remains rejected at module import. Existing ordering assertions must be preserved.
