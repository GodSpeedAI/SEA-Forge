# Current status

**Status revision:** 206

**Stage:** Security: review fixes for approval claim and strength classification

**Summary:** Review fixes on the settlement gate: approved-but-unclaimed records refuse declarations (authorization_unclaimed), and Strong declarations classify as external (fail closed). The request-hash binding to the actual declaration content remains open.

**Verified:**
- cargo test -p sea-forge-authority -p sea-forge-core -p sea-forge-capability: all pass

**Limits:**
- Approval request hash binds the gated ingress content, not every field of the declaration sent (delegation.rs, resume.rs). Open.
- A claim whose resume errors after claiming stays open until the next resume settles it as failed (fail closed, stability only).

**Next:** Build the declaration before the approval gate so the approved hash covers the declared content.
