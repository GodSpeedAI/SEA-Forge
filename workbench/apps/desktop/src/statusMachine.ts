import { setup } from "xstate";

// Proof-stage status machine, still consumed by the (non-routed) ProofPage and
// its unit test. Task 5's readiness slice is a plain read-only query, so it uses
// TanStack Query directly with no XState machine (per implementation-workflow.md
// step 10: "plain queries need no machine"). The real preflight/commit machine
// belongs to a later authoring slice (case commit, Task 6+), which will replace
// this placeholder — the readiness query deliberately does not.
export const statusMachine = setup({}).createMachine({
  id: "proofStatus",
  initial: "idle",
  states: {
    idle: { on: { CHECK: "checking" } },
    checking: { on: { READY: "ready", FAIL: "degraded" } },
    ready: { on: { CHECK: "checking" } },
    degraded: { on: { CHECK: "checking" } },
  },
});
