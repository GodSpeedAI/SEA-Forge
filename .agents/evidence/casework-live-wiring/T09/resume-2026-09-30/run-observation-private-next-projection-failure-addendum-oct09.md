# Private Next unexpected projection failure

Root semantic decision, 2026-10-09. This supplements the frozen original Next
assignment and its sequencing/notifier proof addenda.

Only explicitly classified nonterminal read-unavailable and retention-unavailable
markers preserve attachment and permit recovery from a later real fitting
publication. A pure projector failure for an otherwise captured current image
is an unexpected integrity/assembly failure: return typed unavailable, zero
aggregate and no watermark commit, then schedule the single terminal drain
owner. Release the call's operation before any teardown wait. Do not reinterpret
that failure as a recoverable marker or retry the same terminal image without
a wake. This follows the accepted terminal unavailable ownership rule and does
not alter source continuity or retention policies.

For the all-or-nothing proof, a test wrapper may inject one typed pure-projector
failure after an earlier genuinely advanced candidate was assembled. Production
still supplies exactly buildRunObservationRunDelta. Both paths use the same
capture/aggregate/commit/lifecycle helper. The fault injection proves the error
branch; it is not worker-publication or recovery evidence. Require non-vacuous
candidate high-water marks from real worker reads, preserve every prior
watermark after the failure, and observe actual terminal teardown.

Real read/retention-marker recovery remains separately proved by later actual
worker publication. No reset, fake watermark advance, implicit wake, unchecked
retry, additional manager hook or public error contract is authorized.
