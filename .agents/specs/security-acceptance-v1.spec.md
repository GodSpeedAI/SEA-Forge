# Behavioral security acceptance v1

The normative behavioral contract and seven-part SEC-01 through SEC-12 test
specifications are in [CONTRACT.md](../../tests/security-v1/CONTRACT.md). The
machine-readable inventory is [inventory.json](../../tests/security-v1/inventory.json).
This phase adds test contracts and controlled fixtures only. It does not change
production identity, authority, runtime, sandbox, database, MCP or settlement behavior.

This operator-requested contract strengthens the historical minimum slice's
`principal = actor.actor_id` / local-default identity model. An exact configured
binding plus shared UID is insufficient proof of a distinct workload identity.
Existing normative execution/authority ordering, synchronous kernel boundaries,
DomainForge world validation, OPA provenance, and evidence-based settlement remain
in force. Policy projection is not workload authentication or native enforcement.

Named gates are `python3 tests/security-v1/run.py A|B|C --output <new-directory>`
from the repository root, with observer output under `.agents/evidence/security-v1`.
Every classification except PASS makes a gate non-green. Partial executable PASS
does not satisfy uncovered obligations. PostgreSQL and MCP probes are executable
even when provisioning or production ingress is unavailable; unavailable prerequisites
produce INFRASTRUCTURE-BLOCKED. Gate B/C integration gaps remain explicitly
NOT-YET-EXECUTABLE. No assertion is reduced because a provider is unavailable.

PostgreSQL behavioral assertions are independent of provisioning. A configurable
PostgreSQL URI or local initdb may provision real PostgreSQL. Optional pg0 may supply
a URI; it is not a production dependency or a different acceptance path. Privileged
fixture setup creates unique synthetic schemas/roles; agent queries receive a
separate generated login and a scrubbed environment. Workload credential issuance
and network isolation must additionally satisfy SEC-02/06; native role tests alone
do not establish those protections.

Freeze covers this spec, the contract, inventory, assertions, provider fixtures,
attacker corpus, runner, baseline report, handoff and verifier. An implementation
must preserve v1 bytes. Missing integrations may receive separately versioned
supplementary tests; they cannot be declared satisfied merely by the v1 gap inventory.
