# Current status

**Status revision:** 397

**Stage:** CognitiveOS #108 receipt reference binding

**Summary:** Bind settlement receipts to exact operation lookups and the committed ALLOW record, verified against one pinned ledger snapshot.

**Verified:**

- Operation lookups match the record's designated operation field exactly; the fixed cognate marker no longer selects unrelated decisions or evidence.
- A receipt is projected only after the settlement row, the committed ALLOW decision and every evidence packet it relies on verify in one locked snapshot; world and decision references come from the ALLOW record and are format-checked.
- Operation ids are capped so every generated reference fits the projection limit; replay and nonfinal outcomes carry distinct typed refusal codes, nonfinal refused before a settlement packet is committed.

**Limits:**

- Server wiring, error text and the effect flag follow in the next commit.
- Full CI, proofs and cross-platform tests have not run.

**Next:** Wire the receipt verb to the typed projection with fixed error text and a truthful effect flag.

**Evidence:**

- crates/sea-forge-authority/src/cep.rs
- crates/sea-forge-authority/src/cep/obligation.rs
