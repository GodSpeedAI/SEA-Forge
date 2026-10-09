# Independent review: fresh CW-41 documentation repair

## Verdict

**APPROVE the CW-41 documentation repair within its stated scope.** The two
ambiguities identified in the prior rejection are corrected. The entry keeps
the original source rejection as historical evidence, records the approved
initial-unavailable fixture disposition and its bounded replacement proof,
and makes no runtime, policy, Next-completion, or T09-settlement claim.

This is approval of the ledger wording only. It does not approve the rejected
test source, grant compiler access, or establish expected RED or source
readiness.

## Evidence reviewed

Read the full `.agents/DEBT.md`, the fresh repair builder record, the prior
independent rejection and builder record, the full original Next source review
and frozen builder result, the full assignment and all relevant addenda, and
the approved initial-unavailable correction and its independent review.

Verified these identities:

| Artifact | SHA-256 |
|---|---|
| `.agents/DEBT.md` | `be651245fd6a593b239d73775f8e2fa954e2ab04995b7ee73935cd7270a6324b` |
| `private-next-static-debt-fresh-repair-builder-oct09.md` | `21c015b8d01b87108c5e0a258dfa7c4ae083cafbbfc319d99fe4f70b70241574` |
| Approved initial-unavailable correction review | `060a5349b1e3538b98ba9c9581f19ca5a6cfdb931edf5c2c68d1898d94146842` |

I read the read-only `git diff 9efd039 -- .agents/DEBT.md`. It appends CW-40
and CW-41 after the prior ledger ending; it changes or removes no earlier
committed entry. CW-40 is the previously recorded addition and is unchanged.
The CW-41 text in that diff is the repaired version whose current file hash
matches the builder's frozen identity.

## Review findings

1. **Compile-defect count is now precise.** CW-41 records two defect classes
   across ten sites: four callbacks missing the error result and six map
   membership checks incorrectly treating a struct value as a boolean. This
   matches the rejected source review's locations and does not imply ten
   unrelated defect classes.

2. **Finding 4 remains historical and its impossible setup is superseded.**
   CW-41 preserves the original review's observation that the rejected source
   lacked an initial-no-current recovery fixture. It also cites the approved
   correction and its review hash, says the initial read failure detaches the
   poller, and expressly prohibits claiming same-entry recovery after that
   worker is detached. The replacement proof is bounded to absent initial
   watermark, no attached poller references, and actual worker JOIN, consistent
   with the approved correction and its source-backed review.

3. **Recoverability remains correctly scoped.** CW-41 requires later fitting
   worker publication and unchanged prior watermarks for transient-read and
   nonterminal-retention recovery only after an accepted `retainedCurrent`
   attachment. It does not turn initial failure into recovery, alter the
   initializer contract, or introduce a new policy.

4. **The next-step and evidence limits are complete.** The entry points the
   fresh fixture repair to the full assignment and sequencing, notifier,
   projection-failure, and initial-unavailable documents. It retains the
   requirement for independent source review before root's expected-RED work,
   preserves existing assertions and scope, and denies runtime or completion
   claims. The source rejection and frozen builder result remain properly
   characterized as static source-preparation evidence only.

## Limits

No source, status, or debt file was edited by this critic. No compiler,
formatter, test, gate, runtime command, or Git mutation was run. The read-only
base diff and artifact hashes were used only to review the documentation
change. This approval does not replace the required fresh fixture source
review or root-owned compiler and expected-RED sequence.
