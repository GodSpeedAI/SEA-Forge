# FDE Field Case storage

Status: approved design, implementation and acceptance pending.

The frozen [FDE contract](../../contracts/fde/README.md) governs the record kinds,
lifecycle, and settlement rules. This specification adds storage requirements for
WP-21B on Linux, macOS, and Windows. The storage decision is recorded in
[ADR-015](../../docs/decisions/ADR-015-versioned-field-case-storage.md).

## Invariants

1. Each read operation pins one published generation for its entire view of a
   Field Case. Case identity, lock ownership, and runtime paths use the logical
   case ID. Plain cases retain the existing storage and behavior.
2. Under the per-case lock, a Field Case commit strictly loads the published
   state, admits the declared batch, and applies exactly that batch to a private
   stage. The full staged state must satisfy the frozen lifecycle and store
   invariants before it can become visible. Published generations are immutable.
3. One atomic publication step selects a complete generation. A reader observes
   the previous complete state or the new complete state. Before publication, a
   refusal or error leaves the prior selection intact. A commit that cannot
   confirm durability after publication reports uncertainty.
4. A selector identifies the format version, logical case ID, positive generation
   number, and manifest digest. The generation number determines the directory
   name. The manifest binds the complete set of owned paths and file digests.
   Readers reject missing or corrupt selected data with store-integrity failure.
5. Existing Field Cases may use the legacy layout while the versioned namespace
   is absent. The first successful versioned publication includes the initial
   generation and its selector in one step. After that, missing or invalid
   selector data never authorizes a legacy fallback.
6. Execution workspaces and transient sandbox output are outside published case
   generations. The runner streams from pinned workspace file handles into
   private scratch files with bounded memory. Settlement uses the sealed
   contents, and the staged writer verifies the capture digest before installing
   the file. Evidence for a committed generation continues to resolve to its
   original bytes. Reclamation must preserve referenced generations and active
   readers.
7. A version 2 manifest binds the immediately preceding committed generation
   and its manifest digest, except at first publication. A previously published
   version 1 manifest is a history root. Historical artifact lookup pins the
   selector once, then follows only verified links with decreasing generation
   numbers. Directories left by an interrupted publication are outside that
   history. The artifact API retains digest-only lookup.

## Acceptance

- Concurrent readers see one complete snapshot per operation across each commit.
- Crash tests cover stage creation, staged verification, first publication,
  selector replacement, and failures after publication. Recovery tests cover a
  corrupt selector, missing generation, and mismatched manifest or file digest.
- Legacy migration and reader-compatible rollback preserve the latest published
  data and never authorize in-place writes to an immutable generation.
- Observe ordinary execution from outside the case writer. Compare filesystem
  mutations in the published case store with successful publishes and verify
  referenced bytes against the manifest. Workspace paths alone cannot prove
  this property.
- Run the S2 proof, workspace regressions, and the applicable platform gates.
  Report visibility and crash durability based on evidence from each platform.
