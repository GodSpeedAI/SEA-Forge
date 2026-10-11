# Manager source inventory — root correction

The immutable `run-observation-manager-source-inventory-oct06.md` correctly
identifies the existing unprotected run read models and lack of per-watcher
actor attribution. Its statement that a `(case, run)` sharing key is not
approved is incorrect. The operator-approved proposal already requires
16 process-shared `(case_id, run_id)` pollers:
`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:35,39,45`.

Keep that approved key. The unresolved implementation concerns are watcher
admission and repeated perspective/session checks, delivery isolation, cancelled
watcher detachment, and cancellation/join before poller removal. Existing
SessionStore.Current establishes session lifetime, not continuing kernel
delegation/role validity. No manager implementation or identity change is
released by this correction. The original inventory remains immutable; its
other source facts are useful inputs, not a completed architecture decision.
