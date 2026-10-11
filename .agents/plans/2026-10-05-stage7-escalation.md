# Stage 7 plan — escalation and durable continuation (recorded after the fact)
1. SEA-Forge: ledger-derived approval standing; issue on policy escalations; `resolve_approval` with engine SoD; revalidation in `decide`; verbs `authority_approval`, `authority_approvals`; config `approval_ttl_hours`.
2. Tests: authority-level (13 + CEP validators), socket-level.
3. Cognate: escalation/approval types; Governor passes approvals; SeaForgeAuthority carries approval + lineage; Invoker reports escalation; executor records escalation once and waits durably; `pollApprovals`.
4. Tests: scripted authority (7), live loop against the real server (3 new).
5. Gates, settlement, status, commit, push.
