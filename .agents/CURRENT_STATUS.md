# Current status

**Status revision:** 384

**Stage:** CognitiveOS convergence #108 settlement replay authorization

**Summary:** Reauthorize both normal and obligation final settlement replay before returning the recorded packet.

**Verified:**

- Normal and obligation replay tests fail before the change for an operator without settlement authority and pass after it.
- A service whose settlement grant is removed is refused, while existing authorized normal and obligation replay tests pass.

**Limits:**

- Server correlation same-id replay and request.get_status are not fixed by this service change; legacy correlation records do not identify the requesting actor.
- No real socket cross-actor proof or full milestone, CI, or proof gates were run.
- Cognate 0.11.0 was published in prior work; this snapshot does not imply a new release.

**Next:** Independently review service change and design authenticated, same-actor-or-auditor correlation recovery without exposing complete packets.

**Evidence:**

- crates/sea-forge-authority/tests/cep_settlement.rs
- crates/sea-forge-authority/tests/cep_obligation.rs
