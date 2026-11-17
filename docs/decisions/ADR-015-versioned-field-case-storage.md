# ADR-015: Atomically publish versioned Field Case state

Status: approved by the operator on 2026-11-17. Implementation pending.

## Context

A Field Case operation can change its event log, record chain, and projections
in one batch. Publishing each file separately leaves an intermediate state that
readers and recovery can observe. The supported operating systems do not have a
common primitive for replacing an existing nonempty directory in one operation.

## Decision

The case-runner owns a versioned store and one current selector per logical
Field Case. Case-runner and server operations resolve the selected generation
through the same boundary. The first publication installs the staged namespace
as a new entry. Later publications replace one regular selector file after the
staged generation and manifest have been verified and flushed. Readers pin the
resolved generation for the duration of a logical read.

Keep the logical lock and runtime paths separate from generation paths. Preserve
plain-case behavior and validate legacy Field Case input before migration.
Introduce read compatibility before enabling the versioned writer. That
checkpoint must continue to read versioned state and refuse writes to an
immutable generation. A binary that predates read compatibility cannot serve as
a rollback target after migration.

Parent-entry durability and rename behavior must be qualified on each supported
platform. A failed post-publication durability check leaves an uncertain outcome
for recovery to examine. The
[storage specification](../../.agents/specs/fde-field-case-storage.spec.md)
defines the invariants and acceptance tests.
