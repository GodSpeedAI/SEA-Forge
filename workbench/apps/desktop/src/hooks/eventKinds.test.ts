import { describe, expect, it } from "vitest";
import {
  KNOWN_EVENT_KINDS,
  affectsApprovals,
  affectsCases,
  affectsDelegations,
  affectsReadiness,
} from "./eventKinds";

/**
 * The taxonomy had drifted before this suite existed: the server had been
 * emitting `agent_run.delegated` and `agent_run.cancellation_requested` for
 * some time while `KNOWN_EVENT_KINDS` still listed three kinds and claimed to
 * be verified against the publish sites.
 *
 * Nothing rendered stale, because the narrowing is asymmetric and unknown kinds
 * invalidate — which is exactly why nobody noticed. These tests pin both halves:
 * the roster of kinds, and the asymmetry that makes an omission safe.
 */

/**
 * Every kind the server's `publish_event` call sites emit, transcribed from
 * `crates/sea-forge-server/src/`. Update this list and `KNOWN_EVENT_KINDS`
 * together when the server grows a kind.
 */
const SERVER_EVENT_KINDS = [
  "case.submitted",
  "approval.approved",
  "approval.rejected",
  "agent_run.delegated",
  "agent_run.cancellation_requested",
];

const PREDICATES = [affectsReadiness, affectsCases, affectsApprovals, affectsDelegations];

describe("event kind taxonomy", () => {
  it("lists exactly the kinds the server emits", () => {
    expect([...KNOWN_EVENT_KINDS].sort()).toEqual([...SERVER_EVENT_KINDS].sort());
  });

  /**
   * The asymmetry, stated as a test. An unrecognized kind must produce an extra
   * read, never a skipped one: a wasted round trip is recoverable, a stale
   * render the operator cannot detect is not.
   */
  it("invalidates every surface on an unrecognized or shapeless frame", () => {
    for (const affects of PREDICATES) {
      expect(affects({ kind: "some.future.kind" })).toBe(true);
      expect(affects({})).toBe(true);
      expect(affects(undefined)).toBe(true);
      expect(affects({ kind: 42 })).toBe(true);
    }
  });

  it("narrows only kinds it knows cannot matter", () => {
    // Readiness projects the self-model and endpoint config; neither moves when
    // a case is committed or a delegation runs.
    expect(affectsReadiness({ kind: "case.submitted" })).toBe(false);
    expect(affectsReadiness({ kind: "agent_run.delegated" })).toBe(false);

    // The approval inbox does not move when a delegation starts.
    expect(affectsApprovals({ kind: "agent_run.delegated" })).toBe(false);
    expect(affectsApprovals({ kind: "approval.approved" })).toBe(true);

    // Delegations move on both agent_run kinds, and on a commit that may
    // dispatch an agent task.
    expect(affectsDelegations({ kind: "agent_run.delegated" })).toBe(true);
    expect(affectsDelegations({ kind: "agent_run.cancellation_requested" })).toBe(true);
    expect(affectsDelegations({ kind: "case.submitted" })).toBe(true);
    expect(affectsDelegations({ kind: "approval.approved" })).toBe(false);

    // A delegation runs inside a case and moves its horizon.
    expect(affectsCases({ kind: "agent_run.delegated" })).toBe(true);
  });
});
