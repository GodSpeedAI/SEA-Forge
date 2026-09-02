# Tutorial: Inspecting Evidence & Conformance Proofs

> **A guided tour of cryptographic evidence verification, Merkle proofs, and running the normative minimum proof suite (P1–P4b).**

---

## Prerequisites

* Completed [Tutorial 1: Your First Governed Run](file:///c:/Users/sprim/projects/sea-rs/docs/tutorials/01-first-governed-run.md).
* `just` command runner installed (run `just --version`).
* `jq` command-line JSON processor installed.

---

## 1. Run the Minimum Conformance Proof Suite

SEA Forge specifies four normative proofs (**P1–P4b**) defined in `.agents/specs/spec-minimum.md §12.2`. These proofs execute against the real CLI binary and verify all core invariants.

Run the proof suite from the repository root:
```sh
just proof
```

### Expected Output:
```text
[proof] running minimum conformance proof suite (P1-P4b)
[proof] P1: complete lifecycle record presence and cross-referencing ... OK
[proof] P2: deterministic plan and authority hashes ... OK
[proof] P3: fail-closed authority denial and empty workspace ... OK
[proof] P4: generated zone write denial ... OK
[proof] P4b: false success rejection ... OK
[proof] all minimum proofs passed deterministically.
```

---

## 2. Understanding What the Proofs Established

Each proof tests a specific architectural security boundary:

| Proof | Target Invariant | What It Mechanically Proven |
|---|---|---|
| **P1** | **Full Episode Records** | Runs a demo workload. Asserts that all 6 files (`plan.json`, `authority.json`, `trace.jsonl`, `evidence.jsonl`, `settlement.json`, `semantic-envelope.json`) exist, that cross-references resolve, that every artifact carries an `ifl:hash:<sha256>` pre-mint identity, and that `settlement.json` has `status == "accepted"` with a non-empty `basis`. |
| **P2** | **Determinism** | Runs identical intent twice in separate directories. Compares the SHA-256 digests of `plan.json` and `authority.json`, proving byte-for-byte determinism. |
| **P3** | **Fail-Closed Denial** | Submits an unclassifiable intent. Proves the CLI exits with code 2, logs `RunHalted`, commits `authority_decision` (Deny), and leaves `workspace/` completely empty. |
| **P4** | **Generated Zone Guard** | Attempts to write to `src/gen/model.sea`. Proves the planner/authority engine denies the write, exiting with code 3. |
| **P4b** | **False Success Rejection** | Submits `IntentPattern::FalseSuccess` (which writes to `other.sea` while criteria require `model.sea`). Proves settlement status is `Rejected` with basis `required_artifact_missing:model.sea`. |

---

## 3. Deep Dive into `evidence.jsonl`

Inspect the evidence log from your previous tutorial run:

```sh
RUN_DIR=$(ls -d /tmp/tutorial-cell/runs/run_* | tail -n 1)
cat $RUN_DIR/evidence.jsonl | jq .
```

You will see discrete records for each piece of evidence gathered during the run:

### 1. The Authority Evidence Record
```json
{
  "version": "0.1",
  "evidence_id": "evi_0001",
  "run_id": "run_...",
  "kind": "authority_decision",
  "uri": "decision:dec_...",
  "source_event_id": "tev_0003"
}
```

### 2. The Artifact Evidence Record & Pre-Mint Identity
```json
{
  "version": "0.1",
  "evidence_id": "evi_0003",
  "run_id": "run_...",
  "kind": "artifact",
  "uri": "file://artifacts/model.sea",
  "sha256": "sha256:7b243...5b",
  "metadata": {
    "artifact_descriptor": {
      "artifact_id": "art_...",
      "artifact_type": "sea_model",
      "owner": "operator_local",
      "pre_mint_identity": "ifl:hash:7b243...5b"
    }
  }
}
```
Notice the `pre_mint_identity`. This cryptographic token links the physical bits of `model.sea` directly to the run, establishing provenance before any external attestation.

---

## 4. Cryptographic Proof of Inclusion via the Ledger

SEA Forge records all governance commitments into an append-only integrity ledger. You can cryptographically verify this ledger and extract Merkle proofs.

### Verify the Ledger Stream
```sh
./target/debug/sea-forge ledger verify main --root /tmp/tutorial-cell
```
Output:
```text
Ledger stream 'main' verified:
  Entries: 4
  Hash chain: valid (sha256:...)
  MMR root: valid
```

### Prove a Specific Entry Exists (Merkle Inclusion Proof)
Extract an `entry_ulid` from the ledger:
```sh
ULID=$(head -n 1 /tmp/tutorial-cell/ledgers/main/entries.jsonl | jq -r .entry_ulid)
./target/debug/sea-forge ledger prove main $ULID --root /tmp/tutorial-cell
```

Output:
```json
{
  "ledger_id": "main",
  "entry_ulid": "01J7K...",
  "append_ordinal": 1,
  "leaf_hash": "sha256:...",
  "mmr_root": "sha256:...",
  "peaks": [
    "sha256:..."
  ],
  "audit_path": []
}
```
The resulting `MerkleProof` can be independently validated by any client without trusting the server or database.

---

## 5. Next Steps

Now that you have verified how evidence and proofs work at the CLI and ledger levels:
* Walk through the graphical experience in [Tutorial 3: Authoring Your First Workbench Case](file:///c:/Users/sprim/projects/sea-rs/docs/tutorials/03-authoring-first-workbench-case.md).
* Explore the mathematics of the ledger in the [Integrity Ledger Subsystem Guide](file:///c:/Users/sprim/projects/sea-rs/docs/subsystems/integrity-ledger.md).
