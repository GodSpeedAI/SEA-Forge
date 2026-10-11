# Correction to independent review — publisher join-error attribution

Date: 2026-10-07  
Corrects review: `case-mutation-publication-critical-path-source-facts-independent-review-oct07.md`  
Corrected review SHA-256: `dea731c54e9ceefba11f77fd8db7e7dea1742b518de7b9d333b570d9bd6e3c14`  
Reviewed source-facts artifact SHA-256: `6baf5c1920efe2769e8577a8cf66747cfd9fc1ca79a7d93c3ec0378b840305bf`

## Correction

The review's fourth finding incorrectly says the source-facts note “correctly records” that the publisher task's `JoinError` is discarded. It does not. The source-facts note says the helper waits for the publisher task to drain and identifies swallowed event-append errors, but it omits the separate `let _ = publisher.await` join-error case.

Direct source at `crates/sea-forge-server/src/sfwp/case_mutations.rs:159-192` shows the helper closes the sender and discards the result of awaiting the publisher task. Thus a publisher-task panic/join failure is also swallowed and may prevent full draining. This is an independently verified, bounded omission from the source-facts note, not a material reversal of the review: the note remains approved only as a bounded inventory, with no atomicity or readiness claim.

The review's run-permit qualification and all other findings remain unchanged. This correction is a new immutable artifact; neither the source-facts note nor the earlier review was edited.

## Original review assignment

> Bounded source recon for root architecture, SOURCEONLY/no runtime/compiler/Git. Root accepted factual writerinventoryaec/039 review. Determine mutation duration and publication critical paths: do case_dispatch::submit, case_mutations::advance/run_case_mutation andcase.commit hold work through actual capability execution/settlement? Exactly when eachcase/plan/horizon filewrite occurs relative to case-localtrace notification, asynchronousglobalEventFrame append, appendretirement/drain andresponse? Inspect existing events_ledger API/locks/append sequence and notifier mechanism (case_ops_*_with callback->channel tasks). Produce concise NEW sourcefact record ORIGINALinstructions+table operation→write→notification→durableglobalappend→return, highlighting concurrentreads/long waits andexistingusabletransaction boundaries. Inventory allrelevant publishererrswallowed and whether callbacks can synchronouslyfail a write (without proposingarchitectureyet). Need directsource/callersevidence; Graftfirst, knownpaths verify. Do NOT assert broadlocking solution or change APIs/schemas/security/kernel async boundaries/deps. Root ownsdesign; no tests/services/scanners. Immutable artifact writeonce final hash; correctionsNEWfilesonly. Purpose avoid choosing cellwide lock held across longcapabilitywork without understanding effects.

## Provenance

The correction follows direct verification of the cited source lines and comparison with the full source-facts note. No source, tests, runtime state, or Git state was changed. No compiler, test, or runtime command was run.
