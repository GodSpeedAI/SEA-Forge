---
name: sea-forge-hardening-review
description: Run an independent adversarial review of a frozen SEA-Forge diff with a different model CLI (Codex first, else a fresh non-interactive Claude session). Trigger when a sandbox, identity, IPC, OpenShell, projection, packaging or platform change is ready to merge, or when a task demands "independent review" or "adversarial hardening".
---

# SEA-Forge hardening review

Use this to get a review whose conclusions do not come from the builder's own
reasoning. The reviewer attacks the change; the builder fixes or disputes with
evidence.

## Steps

1. Freeze the build. Commit only the change under review (leave unrelated
   work untouched), record the commit SHA and `git diff --stat <base>...HEAD`,
   and check that SHA out read-only (`git worktree add --detach`). Do not
   push or publish for the review; that needs the operator's separate intent.
   Never review an uncommitted tree.
2. Build the review packet in a scratch directory (not the repo):
   - `diff.patch`: `git diff <base>...HEAD` (complete, not truncated).
   - `spec.md`: the outcome, hard constraints and invariants being claimed,
     copied verbatim from the assignment.
   - `evidence/`: the capability matrix, conformance results with CI run URLs,
     footprint measurements, and every changed `.agents/skills/*/SKILL.md`.
   - `prompt.md`: the attacker prompt below plus the packet file list.
3. Pick the reviewer, which must be a different model from the builder:
   - `codex exec -s read-only` when `codex --version` works; record the
     model it reports in its session header.
   - Otherwise `claude -p --model <a model other than the builder's>` started
     in the packet directory with no session resume.
   - If only the builder's model is available, run a fresh session and record
     the review as "same model, fresh session", never as independent.
   Never pass the builder's transcript, notes or conclusions.
4. Attacker prompt (keep its substance):
   "Attempt to falsify the claim in spec.md. Find correctness failures, unsafe
   platform assumptions, identity spoofing paths, filesystem escapes
   (symlink, junction, reparse, hard link, rename races), network bypasses
   (non-TCP, DNS, loopback, host routes), credential leaks, authority
   downgrades or silent fallbacks, fake conformance (skips counted as passes,
   tests that assert return codes not side effects), untested code paths,
   new dependencies, codebase expansion, and convention deviations. Inspect
   changed skills for vague triggers and success-without-evidence. Rank
   findings critical/high/medium/low with file:line, violated invariant,
   concrete failure mechanism, smallest correction. Do not accept the
   builder's statements as proof."
5. Run it non-interactively with the repo checked out read-only at the frozen
   SHA. Save the exact command, reviewer model, exit status, raw log and
   final output as `review-pass-N.*` next to the packet. A non-zero exit, a
   timeout, or output without a per-severity count is an incomplete pass:
   rerun it; never read it as "no findings".
6. Triage each finding:
   - Confirmed → fix in a new commit with a test that fails before the fix.
   - Disputed → attach a reproducible command and its output that refutes it.
   - Never mark a finding resolved by editing documentation alone.
7. Re-run the narrow tests per fix, then the full verify suite.
8. Repeat from step 1 with a new pass number until a pass returns zero high
   and zero medium findings, or three passes are done.
9. Record every pass (reviewer, model, counts per severity, resolution per
   finding, commit SHAs) in the implementation record under `.agents/plans/`.

## Failure modes

- Reviewing a different revision than the one shipped: the packet SHA must
  equal the merged head.
- Summarizing the diff for the reviewer: give the full patch.
- Running `codex exec` with an open stdin (e.g. backgrounded from a shell):
  it waits on "Reading additional input from stdin" until the timeout.
  Redirect `< /dev/null`.
- Treating "reviewer found nothing" from a truncated or failed run as clean:
  require exit status 0 and the per-severity count line.
- Claiming independence when the same session reviewed its own work.

## Verify

For every pass, confirm the saved command, reviewer model (different from
the builder, or labelled "same model, fresh session"), exit status 0, the
frozen SHA, and a per-severity count. The record lists each finding's
resolution and either zero unresolved high/medium findings or states plainly
that the three-pass limit was reached with findings open.
