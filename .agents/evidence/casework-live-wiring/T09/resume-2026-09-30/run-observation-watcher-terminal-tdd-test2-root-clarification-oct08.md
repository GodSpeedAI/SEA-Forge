# Recurring terminal outer-cap test boundary

Date: 2026-10-08
Applies to source-preparation assignment bfcfa5cd, test group 2.

Group 2 must reach the recurring terminal fallback rejected in ecf2e06f:
first accept a fitting nonterminal initial state through real Prepare/read,
then hold the real recurring trace call. Return a valid terminal candidate
whose inner retained image fits but complete combined poller image exceeds
the unchanged cap. Verify those sizes from the actual candidate/key/prior
state, preserving the ledger. Initial-only retention-unavailable A is a
different path and would miss the stale pre-fallback terminalStop defect.

Observe recurring entry using its real callback channel and bounded wait;
the existing cadence is permitted, sleeps and production hooks are not.
Prove safe marker/no terminal candidate disclosure, no later claim, and
terminal cancellation/drain with capacity held through actual worker JOIN.
Use only existing actual boundaries and read-only observations. Report any
proof obstacle to root before adding a seam or weakening the requirement.

This specifies the already-required rejected outer-cap branch; no extra
production field, mode, public change, algorithm repair, or execution grant.
