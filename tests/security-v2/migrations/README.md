# Authenticated legacy workflow migrations (candidate)

The operator authorized these four migrations on 2026-10-16. Frozen security
v1 remains unchanged at `af41ea2465e38c87dfe86471c33c6b54ea8564e1`.
These tests restore legitimate regression coverage; they do not qualify the
unexecuted SEC-01–SEC-12 obligations or prove isolated OS workload identities.

| Original control | Legitimate purpose retained | Insecure assumption replaced |
|---|---|---|
| CEP escalation approval | Separate operator approves; same request replays; approved action revalidates once; later reuse refuses | One service socket could select the operator by changing an actor label |
| CEP resolution validation | Missing actor still returns identity_required; authenticated malformed resolution still returns cep_request_invalid | A public same-UID socket could select a secondary operator identity |
| CEP world transition | Source-only move allows; semantic change escalates; operator approves; revalidation allows; exact transition records list | The requester's socket could impersonate the approver |
| SFWP discretionary item | A proposes, B executes an escalating item, A cannot resolve its own proposal, approvals journal stays byte-identical | A fresh same-UID socket was sufficient evidence of B's identity |

The shared trusted driver holds the real authority issuer; returned clients
hold only one private endpoint. The driver accepts an explicitly chosen trusted
principal independently of the transmitted actor claim, evaluates and commits
an exact identity-minting decision, consumes its grant through production
`AuthenticatedConnection::new`, and runs the real `run_authenticated` server.
Production code performs authentication. The driver does not intercept or
reject attempted attacks and does not mint identities from received claims.

New attempts send the operator/executor claim over a previously used requester
connection and a fresh public socket. Production refuses both; native CEP or
case ledger bytes are checked unchanged. The SFWP SoD denial retains its
original `approvals.jsonl` byte equality. CEP replay also checks unchanged
authority journal bytes. Successful workflows verify ledger integrity and
correlate authentication to actual committed issuer decisions.

An independent fresh-context reviewer checked the originals at `dd494c1`
before the migration diff and accepted preservation within this narrow scope.
Its receipt is in `.agents/evidence/security-v2/continuation-20261016/migration-review.json`.
Hashes in `manifest.json` identify the reviewed candidates; there is no
independent supplementary qualification freeze or external pin here.

Run both complete affected suites from the repository root:

```sh
RUSTC_WRAPPER= cargo test --locked -p sea-forge-server \
  --test conformance_cep_authority --test sfwp_case_mutations -- --nocapture
```

The workload launcher, distinct host UIDs, credential isolation and governed
database sessions require separate production integration and resource-side
evidence. Private capability possession does not establish those properties.
