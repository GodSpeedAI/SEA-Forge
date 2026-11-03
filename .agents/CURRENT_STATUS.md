# Current status

**Status revision:** 197

**Stage:** Security: identical racing settlement requests re-read the winner's record on conflict

**Summary:** Identical racing requests for one settlement return one pending record. A conflicting commit (the loser's payload carries a different timestamp) now re-reads for the winner's record over a bounded interval (50 reads, 2 ms apart) before deciding it is absent. The race test was already green: 300 repeated runs of identical_racing_requests_yield_one_pending_id with zero failures, so its red state is the new re-read helper's tests, which failed to compile before the helper existed and pass after. The four-thread race test is unchanged.

**Verified:**
- cargo test -p sea-forge-authority --lib race_read: 2 pass (red first: E0425 before implementation)
- cargo test -p sea-forge-authority: 267 pass, 0 fail (cep_settlement includes identical_racing_requests_yield_one_pending_id)
- stress: identical_racing_requests_yield_one_pending_id run 300 times, 0 failures
- cargo test -p sea-forge-cli --test conformance_m8_artifact: both m8 CLI tests pass
- cargo fmt --all -- --check: clean

**Limits:**
- The race is not reproducible in-process without an injected fault; the bounded re-read is defence for a conflicting loser that reads before the winner's record is visible
- adversarial review not done

**Next:** Run the independent adversarial review, fix material findings in this worktree, rerun the verification, and record the results.
