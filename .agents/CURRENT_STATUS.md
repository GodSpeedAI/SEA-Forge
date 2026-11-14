# Current status

**Status revision:** 253

**Stage:** WP-19 S1 gate: replayable derive windows

**Summary:** fde/integration: the S1 episode run pins every derive to an explicit --until ordinal so the AT-18 verifier can replay a recorded derive byte for byte (the default head moves as the ledger grows); AT-06 wrapper minimum corrected.

**Verified:**
- first full run: AT-18 verifier disagreed because derive used until=head; fixed

**Limits:**
- Bundle not yet committed

**Next:** Re-run the pipeline, proof runner, bundle

**Evidence:**
- scripts/fde-s1/run.py
- scripts/fde-proof.d/AT-06.sh
