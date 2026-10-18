# Implementation handoff

Stop this phase after freezing. Production mechanisms were not changed. Use
[README.md](README.md) for exact commands and [CONTRACT.md](CONTRACT.md) for
the immutable observable expectations. Verify the external pin before running
or changing implementation. Never rewrite v1 to obtain green output.

The observed failing obligations are:

* **SEC-01:** changing the actor on an existing same-UID connection must not
  resolve the protected approval as a different workload. The baseline journal
  changes to `resolved_by=operator_b`. Legitimate submit, self-approval refusal,
  missing/unbound identity refusal and role refusal controls execute. Separate
  authenticated workload/approver controls and fresh-session impersonation
  coverage are still required; the narrow probe is not the whole identity claim.
* **SEC-12:** caller-claimed success must not settle an effect that an independent
  filesystem observation rejected. The baseline records EACCES, byte-identical
  protected data and an accepted operational settlement. A real successful
  authorized write settles successfully as the positive control. Preserve
  native identity/run/action/decision/invocation/outcome correlation and ledger
  integrity; do not substitute an allow verdict or zero exit code for evidence.
* **SEC-03/04:** provision the real product MCP ingress and run the frozen
  discovery, explicit-call, reconnect and raw protocol corpus against its
  unrestricted observed upstream. No ingress is currently provisioned. Both
  valid mixed-batch orders must preserve authorized functionality while refusing
  prohibited calls. A fixture or deployment wrapper must not enforce policy.
* **SEC-05:** provision real disposable PostgreSQL with a URI or local binaries.
  The provider-neutral assertions require native privilege failures and unchanged
  protected rows/digest, plus same-session metadata reads and an owner write
  control. Then establish the separate workload-to-database access obligation;
  native fixture permissions alone do not prove product credential mediation.
* **SEC-02/06/07/08:** complete observed issuance/renewal failure, filesystem and
  alternate-credential paths, separately authenticated scoped approval/expiry,
  and revocation of existing PostgreSQL/MCP sessions. Only the explicit child
  environment/bounded-shell subcase currently passes; no broad isolation claim.
* **SEC-09/10/11:** integrate trusted plaintext exposure observation, denied and
  permitted destinations, deterministic injection attempts and child/continuation
  context propagation. The complete seven-part specifications and malicious
  action corpus are frozen; these integrations are NOT-YET-EXECUTABLE.

Harness observation gaps are explicit: independent workload authentication for
same-UID actors, product credential issuance, active session revocation barriers,
production observer provenance, all network/output paths, sensitive-context
lineage, arbitrary same-UID host tampering with the observer, and hosted PostgreSQL
execution on this host. MCP coverage is stdio; other transports need supplemental
coverage. Native PostgreSQL fixture setup is outside the agent's authority and
does not establish the runtime's network or host-filesystem restrictions.

The v1 gap inventory is a frozen statement of initial executable coverage, not
an unconditional failing stub and not a substitute for missing tests. Discharge
those obligations with separately reviewed and frozen supplementary versions;
report their results alongside v1 without changing the original inventory or
claiming that the unexecuted v1 gaps passed. No v1 gate is currently green.

Record unresolved regressions and prerequisites honestly. Do not weaken existing
regressions, modify production behavior in the freeze phase, accept synthetic
gateway enforcement as product evidence, or treat an unavailable database as PASS.
