# Stage 6 plan — CEP authority loop
1. sea-forge-authority: add reserved types `cognate_action`, `cognate_capability`; `cep.rs` parse/decide/envelope + unit tests (engine real, ledger tempdir).
2. sea-forge-server: `authority_request` verb (protected, durable locator = request_id), config `cep_authority` (disabled by default; worlds list), dispatch.
3. Validate emitted envelopes with cep's own validators (gated by CEP_REPO env) — profile conformance.
4. Cognate: `SeaForgeAuthority` + constraint enforcer registry in Governor + tests vs fake socket server.
5. Live loop test: real sea-forge-server binary <-> Cognate over a Unix socket.
6. Settlement, status, commit, push (sea-rs, cognate).
