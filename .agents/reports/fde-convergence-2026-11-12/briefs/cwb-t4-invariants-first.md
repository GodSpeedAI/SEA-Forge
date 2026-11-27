# Cognate Workbench T4 + T2 leftovers: invariants-first brief (2026-11-17)

**Prerequisite:** T3 (PR #39) is merged. Work from `origin/main` in new worktrees.
**Sources:**
- `.agents/plans/capability-bundles/design.md`: G8, CUJ 6, CUJ 7, §7.2, §8 and restart step 8.
- `HANDOFF.md`: "What remains".
- The root-cause brief, `root-cause-classes-2026-11-17.md` (same folder): state invariants, test them with generated inputs, and use
  construction-level guarantees.

The work is in two parts, run one after the other on this 8 GB host. Each part starts by writing its invariants and the property
tests for them, and shows the tests red where code already exists. Only then does implementation start.

## Part A: presentation slots and safe mode (`T4a`)

**Files:**
- `src/ui/FragmentSlot.tsx`
- `src/ui/workbench/CapabilityDetail.tsx`, plus the `Workbench.tsx` tab
- `src/workbench/safeMode.ts`, as one module
- `test/J-CAP-safemode.test.ts`

**Root-cause choices (construction, not checks):**
- **Closed descriptor type.** A slot registration accepts only the §7.1 closed schema (`{title ≤ 80, summary ≤ 200, capabilityId,
  viewRef}` with `list` or `record` views), parsed once at the registry boundary into a branded `RegisteredFragment`. Render code
  accepts only that type. Content is data references, never markup or directives.
- **Governance surfaces are not addressable.** The slot-id type is an enum of non-governance slots, so a governance override cannot
  even be expressed downstream. The registry boundary refuses it, records a core violation and escalates to safe mode.
- **Exclusion precedes effect.** Boot reads the safe-mode exclusion set once, before any render or mount. Mount and render take that
  read as a parameter, so neither can run without it.
- **One error boundary per slot,** wrapping the fragment and its whole subtree. A fault is recorded once (deduplicated by fragment
  version and fault class), and the core fallback renders. Recovery never renders the broken component.

**Invariants (property tests with fast-check; random descriptors, random fault points, random boot sequences):**
- **A1:** a fragment that throws during render, effect or event handling, at any depth, never unmounts the shell. The core fallback
  renders, and exactly one fault event is recorded.
- **A2:** only schema-valid descriptors register. Every other one is refused and recorded, and a governance id gives a core violation
  plus safe mode.
- **A3:** in safe mode, no non-core fragment renders and no bundle handler mounts.
- **A4:** two failed boots with the same extension set keep extensions suspended until the operator-only clear event. A non-operator
  clear is refused and recorded.
- **A5:** safe mode writes only fault and safemode events. The durable registry is otherwise byte-identical (compare a log hash
  before and after).

## Part B: restart and persistence, plus the T2 leftovers (`T4b`)

**Files:**
- `server.ts` boot (the active-set remount)
- the reducer and projection folds
- `test/J-CAP-restart.test.ts`, `test/J-CAP-persist.test.ts`
- `test/workbench-call-sites.test.ts`
- subagent session routing
- per-capability deadlines
- approval parking (D2)

**Root cause of the known bug:** the workbench fold loses a grant's principal, permissions and expiry on rebuild. The class is
**two representations drifting**: the live index and the rebuilt fold each compute state their own way.
- **Construction fix:** one pure fold, `(state, event) -> state`, used by both live updates and rebuild. The live index becomes
  `fold(log)`, applied incrementally.
- Every field of a grant, activation and suspension row comes only from that fold.

**Invariants (stateful property test; random op sequences of install, activate, deactivate, grant, revoke, suspend, resume,
uninstall, session archive and crash points, against a reference model):**
- **B1, replay equality:** for every prefix `p` of the log, `fold(log) == fold(rest, fold(p))`, and the live index equals
  `rebuild(log)` field by field (principal, permissions, expiry included).
- **B2, restart:** after a crash at any point and a restart, the mounted set equals the model's active set. Session-bound rows are
  expired before mount, and the boot spawn sweep's invariants from T3 still hold.
- **B3, a call site cannot skip authorization:** an inventory test enumerates every capability entry point (registry plus grep) and
  asserts each routes through `authorizeCapabilityOp`. A new unrouted entry point fails the test.
- **B4, session lineage:** a run's session is its root run's session, which subagent children inherit. No call resolves as
  session-unknown when a root session exists, and an unknown session still fails closed.
- **B5, deadlines:** a capability may declare a deadline of at most 300 s. Anything over is refused at registration, and the
  effective deadline is `min(declared, the platform cap)`.
- **B6, approval parking (D2):** an activation beyond the agent's grant creates a pending approval, never an activation. It activates
  only on an operator approval bound to the exact request digest, and is refused on expiry or digest change. Reuse SEA-Forge's
  approval digest pattern if one exists in Workbench; otherwise record why not.

## Docs and enablement (after A and B)

- Update `src/README.md`, `src/capabilities/README.md`, `src/workbench/README.md`, `docs/known-gaps.md`, `CURRENT_STATUS`,
  `AGENTS.md` (the enablement gate note).
- **Enablement:** write `workbench.capability_runtime.enabled` only after CI's full gate and the property suites pass (CI
  numRuns ≥ 200; local ≥ 2,000, recorded). It stays off by default for fresh installs; enablement is the operator's flag. Record
  the decision in HANDOFF.md.

## Review

One Codex round per part, briefed to break the named invariants. If the findings come in sibling classes, apply the three-round
rule from the root-cause brief.
