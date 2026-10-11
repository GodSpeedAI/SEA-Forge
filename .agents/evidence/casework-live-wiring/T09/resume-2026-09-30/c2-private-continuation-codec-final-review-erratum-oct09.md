# Erratum: private continuation codec review receipt

This erratum applies only to the two factual attributions below in
`c2-private-continuation-codec-final-independent-review-oct09.md` (SHA-256
`0b2102d948104aa0f9f396526ba780689c64654ef6eb8ecc083b4cf102b11f8c`). The
original receipt remains unchanged; its remaining bounded verdict and evidence
are unchanged.

1. The frontier test correction changed the invalid acknowledged `end_offset`
   from `100` to `101`. The positive adjacent case retains `end_offset = 100`.
   It did not change the pinned start.
2. The docs60 writer first encountered a failed Python/context subprocess write,
   then used an escalated subprocess write. It did not first attempt a native
   `apply_patch` write.
