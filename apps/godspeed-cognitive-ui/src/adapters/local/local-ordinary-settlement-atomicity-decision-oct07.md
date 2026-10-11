# Settlement atomicity decision and test assignment

Date: 2026-10-07. This new record preserves the root clarification received
after the initial local cursor implementation draft. The earlier pre-edit
assignment record remains immutable and unchanged.

## Root decision, preserved verbatim

> Root DECISION: settlement is ONE logical operation requiring BOTH ordinary events. Reserve/validate two consecutive ordinals and prepare all fallible snapshot/history/sideevent material BEFORE anycommit; if fewerthan2remain, fail atomically with one safe perwatcher error and NO snapshot/sideevent/history/frontier mutation. Onsuccess commit both consecutiveallocations synchronously then enqueue notifications (callbacksneverinsidecommit), same ordinary allocator. This clarifies sequentialevent numbering vsatomic logical settlement, preserves private scope/no publicAPI. Add NEW tightly scoped supplemental testfile for near-ceilingF=9999999998 (only1remaining) atomicrejection, plusF=9999999997 (2remaining) successsnapshot9999999998/side9999999999 distinct and exact historicalgetAt; existingfrozen134ba/373a untouched. Test real public dispatch/start before seed boundary and controlledactualsettlementtimer as frozenfixture pattern. First write original rootdecision + tests source before settlement implementation change; no running RED/GREEN/compiler now. Record allmaterialproposalclarification and no baselineREDclaim fornewtests, differentcritic/root will verify.

## Chronology and frozen inputs

At the time this clarification arrived, `localAdapter.ts` had already been
modified from its original source hash
`d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34` to
`989e11a831e482087d28f52bae7560a9608e61b44cd6d54e87e6d3dbbcdf603f`. That
draft allocated the settlement snapshot and side event sequentially. This fact
is recorded candidly; the new atomic tests and the next source revision replace
that incomplete implementation. No test, compiler, Bun, typecheck, Git, or
runtime command was run.

The original implementation assignment, accepted proposal identity, source
pre-edit hash, and unchanged frozen test identities are in
`local-ordinary-cursor-implementation-record-oct07.md`. The accepted proposal
is revision 4 SHA-256
`6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`.
The frozen tests remain cursor-order SHA-256
`373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` and
cursor-bounds SHA-256
`134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a`.

## Required supplemental test cases

Add one separate `localAdapter.settlementAtomicity.test.ts`; do not edit the
frozen order/bounds fixtures or conformance. Both cases must use the actual
public dispatch path to start execution, then set the private local frontier
only after the start snapshot has been delivered. Use the frozen fixture's
controlled timer technique and select the actual settlement callback from
the execution timers by its greatest delay.

1. At frontier `1.9999999998`, only one sequence remains. Invoking the actual
   settlement timer must report one error, deliver neither settlement
   snapshot nor `settlement_recorded`, and leave history, final cursor, and
   allocator unchanged. Do not label this as existing baseline RED evidence.
2. At frontier `1.9999999997`, two sequences remain. The same real settlement
   timer must deliver the settlement snapshot at `1.9999999998`, then the
   `settlement_recorded` side event at `1.9999999999`; only the snapshot is
   retained/resolvable by exact `getSnapshotAt`.

These are new implementation tests, not a rerun or reinterpretation of the
accepted bounds RED01 capture. Any runtime observation remains held for a
different critic and root authorization.
