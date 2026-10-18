# Stage C: exposure, injection and delegation at real boundaries — 2026-10-17

Captured regression (authority/server/sandbox/agent; isolation and
PostgreSQL required): 797 passed, 0 failed, 9 ignored.

Mechanism (no new policy engine):

- `exposure::ExposureRegistry` — append-only `exposure` ledger stream:
  `exposure_observed` (context, label, source, evidence digest),
  `exposure_context_derived` (child → parent, set once, no cycles),
  `exposure_context_revoked`. Labels and revocation are the fold over the
  context and all ancestors; nothing removes a label.
- Trusted provenance only: the MCP broker records a label when it forwards
  an accepted result of a classified tool (`McpServerConfig.classify`);
  PostgreSQL scopes with `exposure` labels record at issuance (grant-time,
  conservative — the broker cannot see query results).
- Authority: context-bound actions carry `parameters.exposure`; a rule
  matches only if `permitted_exposure` covers every label
  (`exposure_permitted` in the existing matcher, so DomainForge/OPA
  candidates and the same rule language apply). Exposed contexts fall to
  default deny elsewhere.
- Production: isolated ACP MCP sessions and database sessions use context
  `case:<case>` (spans items and resumed runs).

Observed (`exposure_flow`, upstream log is the observation; injection text
is the frozen v1 `prompt_injection.lower_trust_document`):

- pre-exposure `notify.send` allowed (positive control);
- after `ticket.read` (classified) the context is `confidential`;
  following the injection, `notify.send` with the plaintext, `report.publish`
  (alternate tool), and a plaintext-free "summary" are all denied before
  forwarding; discovery hides the sinks; `audit.file` (explicitly
  permitted for confidential) succeeds;
- a resumed session of the same context stays restricted;
- a delegated child inherits the label, its own public read does not clear
  it, and a lineage cycle is refused;
- a sibling context using only the credential-backed `metadata.read` is not
  exposed and may notify;
- revoking the parent ends the child's live session (-32003) while the
  unrelated context continues;
- upstream: 2 `notify.send` (both unexposed), 0 `report.publish`, and no sink
  call carried the confidential marker.
- `conformance_m16::t16_4d`: issuing the labelled `audit-read` scope marks
  `case:<case>` confidential.

Mutation controls (manual, restored): an always-true exposure boundary and a
lineage walk that ignores parents both fail the test.

Limits: exposure is enforced on MCP and database grant routes. Workspace
files/artifacts and the ACP transcript returned to the operator are not
exposure-gated destinations; network egress is denied for isolated agents by
default (network namespace) rather than label-gated. Production delegation
has no parent/child runs today, so `derive` is exercised by tests and
case-level scoping covers items and resumptions. Labels are coarse
(per context), by design: transformations cannot launder them.
