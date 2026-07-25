import { setup } from "xstate";

// ponytail: proof-stage placeholder machine — Task 5 replaces this with the
// real readiness preflight/commit machines wired to SFWP.
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
