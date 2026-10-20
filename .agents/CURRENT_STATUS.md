# Current status

**Status revision:** 136

**Stage:** security-round4-helper-self-probe

**Summary:** The standalone trap probe shows kernel 6.17 does trap connect (bare and namespaced), so the 24.04 bypass is in the launch chain, not the kernel. In debug mode the helper now connects from a thread that inherits the workload's filter chain, splitting a broken chain before exec from a broken delivery after it.

**Verified:**

- run 39476189517: trap probe reports RECV ok and EACCES on both 22.04 and 24.04; the 24.04 sandbox suite still fails the same two connect tests

**Limits:**

- Cause of the untrapped connect in the launched workload on 24.04 is not yet established; the containment claim does not hold on 24.04 until it is. See revision 113 for the round-4 open limits.

**Next move:** Read the helper self-probe errno from the next 24.04 sandbox-suite log.

**Evidence:**

- .agents/evidence/security-v2/round4-20261018/DESIGN.md

**Spec:** .agents/specs/security-implementation-v2.spec.md

**Ledger:** .agents/OBSERVED_DEBT.md
