# Stage 8 plan
1. SEA-Forge cep.rs: criteria in decision, evidence acceptance, settlement evaluation; unit tests incl. CEP validators.
2. Server verbs `authority_evidence`, `authority_settle`; config `cep_authority.settlement`; socket tests.
3. Cognate: authority reference in recorded decision metadata; execution_trace builder; evidence packaging from sxr; SeaForgeAuthority.submitEvidence/settle.
4. Live end to end with released sxr 0.3.0: trace-only -> unsettled; verifier-supported -> settled; contradicting -> rejected; negative chain cases.
5. Gates, settlement report, debt, status, commit, push.
