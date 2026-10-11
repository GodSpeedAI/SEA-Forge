import { describe, expect, it } from "vitest";
import { projectHomeOrientation } from "./HomeProjection";
import { projectCaseList } from "./CaseProjection";
import { projectAffordances } from "./AffordanceProjection";
import { projectEvidenceList } from "./EvidenceProjection";
import { projectJudgment } from "./JudgmentProjection";
import { projectTimelinePosition } from "./TimelineProjection";
import {
  activityFromPendingWork,
  projectCoreVisual,
} from "./CoreVisualProjection";
import { DEFAULT_VISUAL_PARAMS } from "../core/CoreVisualState";

describe("semantic projections (no fabricated domain data)", () => {
  it("HomeProjection marks unanswered sources unknown, never zero", () => {
    const live = projectHomeOrientation({
      casesLoading: false,
      casesError: null,
      caseCount: 3,
      unreadable: 1,
      approvalsLoading: false,
      approvalsError: null,
      approvalsUnreadable: false,
      pendingApprovals: 0,
      connectionLabel: "ready",
    });
    expect(live.status.state).toBe("live");
    expect(live.pendingApprovals).toBe(0);

    const partial = projectHomeOrientation({
      casesLoading: false,
      casesError: new Error("down"),
      caseCount: 3,
      unreadable: 0,
      approvalsLoading: true,
      approvalsError: null,
      approvalsUnreadable: false,
      pendingApprovals: 5,
      connectionLabel: "…",
    });
    expect(partial.caseCount).toBeNull();
    expect(partial.pendingApprovals).toBeNull();
    expect(partial.status.state).not.toBe("live");
  });

  it("CaseProjection passes governed shapes through untouched", () => {
    expect(projectCaseList([{ case_id: "c1" }, { case_id: "c2", title: "Second" }])).toEqual([
      { caseId: "c1", title: "c1" },
      { caseId: "c2", title: "Second" },
    ]);
  });

  it("AffordanceProjection never offers actions without a ready cell", () => {
    const blocked = projectAffordances({
      surfaceId: "home",
      focusedObjectId: null,
      inboxCount: 2,
      connectionReady: false,
    });
    expect(blocked).toHaveLength(1);
    expect(blocked[0].disabled).toBe(true);
    expect(blocked[0].path).toBeUndefined();
  });

  it("AffordanceProjection binds actions to focus and real pending work", () => {
    const home = projectAffordances({
      surfaceId: "home",
      focusedObjectId: null,
      inboxCount: 2,
      connectionReady: true,
    });
    expect(home.map((a) => a.id)).toContain("review-inbox");

    const quiet = projectAffordances({
      surfaceId: "home",
      focusedObjectId: null,
      inboxCount: 0,
      connectionReady: true,
    });
    expect(quiet.map((a) => a.id)).not.toContain("review-inbox");

    const focused = projectAffordances({
      surfaceId: "case",
      focusedObjectId: "case-1",
      inboxCount: undefined,
      connectionReady: true,
    });
    expect(focused.map((a) => a.id)).toEqual(
      expect.arrayContaining(["inspect-focus", "release-focus", "go-home"]),
    );
  });

  it("EvidenceProjection passes records through; JudgmentProjection keeps the unreadable/empty distinction", () => {
    expect(projectEvidenceList([{ evidence_id: "e1", kind: "run" }])).toEqual([
      { evidenceId: "e1", kind: "run" },
    ]);
    expect(
      projectJudgment({ isLoading: false, error: null, unreadable: false, count: 0 }).pending,
    ).toBe(0);
    expect(
      projectJudgment({ isLoading: false, error: null, unreadable: true, count: 0 }).pending,
    ).toBeUndefined();
  });

  it("TimelineProjection keeps cursors opaque; null stays live head", () => {
    expect(projectTimelinePosition({ cursor: null, eventCount: null }).label).toBe("live head");
    expect(projectTimelinePosition({ cursor: "led_9", eventCount: 42 }).cursor).toBe("led_9");
  });

  it("CoreVisualProjection never touches mass/spin and stays in a narrow band", () => {
    for (const activity of [0, 0.2, 0.75, 1]) {
      const intent = projectCoreVisual({ activity, focusDisplaced: false });
      expect(intent.params.mass).toBe(DEFAULT_VISUAL_PARAMS.mass);
      expect(intent.params.spin).toBe(DEFAULT_VISUAL_PARAMS.spin);
      expect(Math.abs(intent.params.bloom - DEFAULT_VISUAL_PARAMS.bloom)).toBeLessThanOrEqual(
        0.16,
      );
      expect(Math.abs(intent.params.temp - DEFAULT_VISUAL_PARAMS.temp)).toBeLessThanOrEqual(
        0.06,
      );
    }
    const displaced = projectCoreVisual({ activity: 0.2, focusDisplaced: true });
    const centered = projectCoreVisual({ activity: 0.2, focusDisplaced: false });
    expect(displaced.params.bloom).toBeLessThan(centered.params.bloom);
  });

  it("activity derives from real pending work only", () => {
    expect(activityFromPendingWork({ pendingApprovals: null, unreadableCases: null })).toBe(0.2);
    expect(activityFromPendingWork({ pendingApprovals: 1, unreadableCases: 0 })).toBe(0.75);
    expect(activityFromPendingWork({ pendingApprovals: 0, unreadableCases: 2 })).toBe(0.75);
  });
});
