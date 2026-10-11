# Private observer policy operator approval

Date: 2026-10-08.

The operator answered the separate private-policy approval request with
“Approve the recommended private policy”. This approves16 active cohorts,
128 lease-to-run attachments and a1MiB serialized current-state limit per
poller. Nonterminal overflow returns unavailable with no watermark advance,
retaining the lease for recovery from a later complete fitting candidate.
Terminal overflow stops new reads and retains counted capacity until teardown
actually joins. These limits are counts and serialized bytes, not an RSS bound.

This closes the authorization omission recorded as CW36. It is separate from
the earlier six public C2 recommendations. The pure projection independent
review24c7f2c5 and Next integration root-decision reviewf261af24 remain bounded
to their stated scopes. Approval permits a separately scoped integration
builder grant; it does not prove implementation, gates or T09 completion.
