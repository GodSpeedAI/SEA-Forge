# Drain/ref semantic conflict recon assignment — 2026-10-07

## Complete root instruction archived before findings

> Fresh source-only narrow semantic conflict analysis from priority message, since you finished: frozen manager_test.go:879-897 requires entry.refs len1 for BOTH held case_A(detached,canceled) andB, 'retainedref/liveworker'. root manager-signal-ref-ownership-root-decisions-oct07.md requires firstlease detach/drain owns removingremainingexactrefs and clearslease refmembershipatomically. Need actual fullrevision6/prereg/retentioncorrections/rootdecision/test: reconcile lease.pollers cleared versus entry.refs retaining drainownership untilJOIN/noneligible vialease.draining; is this interpretation explicitly permitted withoutaddingstate/weakeningtests? Other new failure test DetachAndSelectedAReleaseHaveOneOwner mayexpect entry.refs immediately0 contradictory. Cite exact both tests/source clauses and any actual contradiction beforePhaseB. No compiler/Graft/sourceedits/Git, native immutable instruction/result. Root keepsarchitecture, nopermissioninventstate.

## Scope

Read only the named lifecycle/retention/ref ownership records and frozen test
assertions, then record whether the proposed distinction between a lease's own
membership and a poller's reverse ref map is explicitly authorized. No source,
test, compiler, Graft, scanner, or Git operation is authorized. Root retains
architecture decisions; do not resolve ambiguity by inventing state or
weakening a frozen test.
