import type { Journey } from "../ladder";
import {
  ask, click, clickNode, coreIntact, count, enterCase, flightDone, historyHash, markCore, node, open, rect, shot, sleep, state, text,
  until, visible, waitVisible, wake,
} from "./helpers";

// J9: the whole user-visible path in one session, and RECOVERY: bounded failures.
// Evidence runs against the local contract-conformant adapter, NOT the Go system front end.

export const J9: Journey = {
  id: "J9",
  title: "Integrated cognitive-environment journey (contract adapter)",
  depends_on: ["J4", "J5", "J6", "J7", "J8"],
  settles: "The full reference path works end to end on settled primitives, with no deep links",
  unlocks: ["RECOVERY"],
  async run(ctx) {
    // No deep links. beatPace only slows narration holds for this ~5 fps software-GL browser.
    await open(ctx, "beatPace=4");
    await markCore(ctx);
    await ctx.step("orientation → focus", async () => {
      await enterCase(ctx);
    });
    await ctx.step("progressively resolve representation (semantic zoom)", async () => {
      const before = await ctx.b.eval<string>(`document.querySelector('${node("ns-impl")}').className.match(/lod-\\d/)[0]`);
      const r = await rect(ctx, `${node("ns-impl")} .orb`);
      for (let i = 0; i < 6; i++) {
        await ctx.b.eval(`(() => { const el = document.elementFromPoint(${r!.x + r!.w / 2}, ${r!.y + r!.h / 2}); el.dispatchEvent(new WheelEvent('wheel', { deltaY: -240, clientX: ${r!.x + r!.w / 2}, clientY: ${r!.y + r!.h / 2}, bubbles: true, cancelable: true })); return true })()`);
        await sleep(250);
      }
      await sleep(3000);
      const after = await ctx.b.eval<string>(`document.querySelector('${node("ns-impl")}').className.match(/lod-\\d/)[0]`);
      const parts = await ctx.b.eval<number>("['ns-claims-api','ns-session','ns-assumption'].filter(id => document.querySelector(`[data-node=\"${id}\"]`)?.style.display !== 'none').length");
      ctx.expect(after > before || parts > 0, `zoom resolves more structure (${before} → ${after}, ${parts} parts)`);
      await shot(ctx, "semantic-zoom");
      await clickNode(ctx, "ns-impl");
      await until(ctx, "focus implementation", (s) => s.focus === "ns-impl");
      await flightDone(ctx);
      ctx.expect(await visible(ctx, node("ns-claims-api")), "the work object's own parts are its local world");
      await ctx.b.press("Escape");
      await until(ctx, "back to case", (s) => s.focus === "northstar");
      await flightDone(ctx);
    });
    await ctx.step("inspect and dismiss a source artifact", async () => {
      await click(ctx, `${node("ns-secondary")} [data-testid="artifact-pill"]`);
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-diff-491"]');
      await ctx.b.waitFor("!!document.querySelector('[data-testid=renderer-diff]')", 15000, "diff");
      await click(ctx, '[data-testid="dock-close"]');
      await until(ctx, "closed", (s) => !s.artifacts.some((a) => a.phase === "expanded"));
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-diff-491"] [data-testid="excerpt-dismiss"]');
    });
    await ctx.step("contextual question → interruptible narrated answer", async () => {
      await ask(ctx, "Why did the pilot fail?");
      await until(ctx, "evidence beat", (s) => s.artifacts.some((a) => a.id === "evi-record"), 30000);
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-record"]');
      await until(ctx, "paused on artifact", (s) => s.narrative?.status === "paused" && s.artifacts.some((a) => a.phase === "expanded"));
      await ctx.b.press("Escape");
      await ask(ctx, "continue");
      const s = await until(ctx, "answer finished", (x) => x.narrative?.status === "done", 90000);
      ctx.expect(s.surface === "causal", "answer ends in the causal arrangement");
      await flightDone(ctx);
      await shot(ctx, "answer");
      await ctx.b.press("Escape");
      await until(ctx, "narration cleared", (x) => !x.narrative && x.surface === "orbital");
    });
    const h0 = await historyHash(ctx);
    await ctx.step("temporal rewind, compare two positions, return to live", async () => {
      await click(ctx, '[data-testid="center-history"]');
      const revs = (await state(ctx)).revisions;
      for (const r of [revs[1]!, revs[2]!, revs[4]!]) {
        await click(ctx, `[data-testid="time-marker"][data-rev="${r}"]`);
        await until(ctx, `at ${r}`, (s) => s.revision === r);
      }
      await click(ctx, '[data-testid="time-mark"]');
      await click(ctx, `[data-testid="time-marker"][data-rev="${revs[1]}"]`);
      await until(ctx, "back at first", (s) => s.revision === revs[1]);
      await click(ctx, '[data-testid="time-mark"]');
      await click(ctx, '[data-testid="time-compare"]');
      await until(ctx, "comparing", (s) => !!s.compare);
      await flightDone(ctx);
      await shot(ctx, "compare");
      await click(ctx, '[data-testid="compare-exit"]');
      await click(ctx, '[data-testid="time-live"]');
      await until(ctx, "live", (s) => s.revision === s.now && !s.timeline && !s.compare);
      ctx.expect((await historyHash(ctx)) === h0, "history not mutated");
    });
    await ctx.step("causal representation and case design", async () => {
      await click(ctx, '[data-testid="center-causal"]');
      await until(ctx, "causal", (s) => s.surface === "causal");
      await click(ctx, '[data-testid="center-causal"]');
      await until(ctx, "orbital", (s) => s.surface === "orbital");
      await click(ctx, '[data-testid="center-design"]');
      await until(ctx, "design", (s) => s.mode === "case-design");
      await click(ctx, '[data-testid="design-stage"] [data-testid="stage-down"]');
      await until(ctx, "draft", (s) => !!s.design?.hasDraft);
      await click(ctx, '[data-testid="design-close"]');
      await until(ctx, "world", (s) => s.mode === "world" && s.focus === "northstar");
      await flightDone(ctx);
    });
    await ctx.step("consequential action: agent denied, human accepted (fresh attempts)", async () => {
      await click(ctx, `${node("ns-release")} [data-testid="action-chip"][data-action="approve-release"]`);
      await until(ctx, "judgment", (s) => !!s.judgment);
      await ask(ctx, "let the agent approve the release");
      await until(ctx, "agent denied", (s) => s.judgment?.outcome?.state === "refused");
      await click(ctx, '[data-testid="judgment-option"][data-option="approve-release"]');
      await until(ctx, "human accepted", (s) => s.intents.some((i) => i.actor.kind === "human" && i.state === "accepted"));
    });
    await ctx.step("execution → evidence → authoritative settlement", async () => {
      await until(ctx, "executed", (s) => ["executed", "settled"].includes(s.executions["ns-release"]?.state ?? ""), 60000);
      await until(ctx, "settled", (s) => s.executions["ns-release"]?.state === "settled", 60000);
      await click(ctx, '[data-testid="execution-pill"]');
      await waitVisible(ctx, '[data-testid="settlement"]');
      await shot(ctx, "settlement");
      await ctx.b.press("Escape");
      await until(ctx, "world", (s) => s.mode === "world");
    });
    await ctx.step("world reorganised by the settled projection; return and orient at CORE", async () => {
      await flightDone(ctx);
      const label = await ctx.b.eval<string>(`document.querySelector('${node("ns-release")}')?.getAttribute('aria-label') ?? ''`);
      ctx.expect(/Settled/.test(label), `release settled in the world (${label})`);
      await ctx.b.press("h");
      await until(ctx, "Home", (s) => s.focus === null && s.mode === "world");
      await flightDone(ctx);
      await wake(ctx);
      ctx.expect(await coreIntact(ctx), "the same persistent Core throughout the journey");
      await shot(ctx, "home-after");
    });
  },
};

export const RECOVERY: Journey = {
  id: "RECOVERY",
  title: "UI-level recovery (bounded failures)",
  depends_on: ["J2", "J4", "J5"],
  settles: "Failures stay bounded; authoritative frontend state and the persistent Core survive",
  unlocks: [],
  async run(ctx) {
    await ctx.step("RECOV-004 optional renderer fails: isolated, source reference kept", async () => {
      await open(ctx, "failRenderer=graph");
      await enterCase(ctx);
      await click(ctx, `${node("ns-impl")} [data-testid="artifact-pill"]`);
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-impl-graph"]');
      await waitVisible(ctx, '[data-testid="renderer-error"]');
      ctx.expect((await text(ctx, '[data-testid="renderer-error"]')).includes("evi-impl-graph"), "the error names the source ref");
      ctx.expect((await text(ctx, '[data-testid="dock-provenance"]')).includes("evi-impl-graph"), "provenance still shown");
      await shot(ctx, "renderer-failed");
      await click(ctx, '[data-testid="dock-tab"][data-ref="evi-checks-table"]');
      await ctx.b.waitFor("!!document.querySelector('[data-testid=renderer-table]')", 15000, "another renderer still works");
      await click(ctx, '[data-testid="dock-close"]');
      const s = await until(ctx, "closed", (x) => !x.artifacts.some((a) => a.phase === "expanded"));
      ctx.expect(s.focus === "northstar", "authoritative frontend state intact");
      await ctx.b.clearLogs();
    });
    await ctx.step("invalid payload: bounded error, no collapse", async () => {
      await open(ctx, "corruptArtifact=evi-rejections-chart");
      await enterCase(ctx);
      await click(ctx, `${node("ns-evidence")} [data-testid="artifact-pill"]`);
      await waitVisible(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-rejections-chart"] [data-testid="artifact-error"]');
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-rejections-chart"]');
      await waitVisible(ctx, '[data-testid="artifact-dock"] [data-testid="artifact-error"]');
      await ctx.b.press("Escape");
      await clickNode(ctx, "ns-release");
      await until(ctx, "still interactive", (s) => s.focus === "ns-release");
    });
    await ctx.step("unavailable source: bounded error", async () => {
      await open(ctx, "failArtifact=evi-case-timeline");
      await enterCase(ctx);
      await click(ctx, `${node("northstar")} [data-testid="artifact-pill"]`);
      await waitVisible(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-case-timeline"] [data-testid="artifact-error"]');
      ctx.expect(/unavailable/i.test(await text(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-case-timeline"]')), "says the source is unavailable");
    });
    await ctx.step("leave temporal/comparison mid-state and return", async () => {
      await open(ctx);
      await markCore(ctx);
      await enterCase(ctx);
      const h0 = await historyHash(ctx);
      await click(ctx, '[data-testid="center-compare"]');
      await until(ctx, "comparing", (s) => !!s.compare);
      await ctx.b.press("h");
      const s = await until(ctx, "home", (x) => x.focus === null);
      ctx.expect(!s.compare && !s.timeline && s.revision === s.now, "comparison and history cleanly closed");
      await enterCase(ctx);
      await ctx.b.press("t");
      await until(ctx, "history reopens", (x) => x.timeline);
      await ctx.b.press("Escape");
      ctx.expect((await historyHash(ctx)) === h0, "history uncorrupted");
    });
    await ctx.step("interrupt a transition mid-flight", async () => {
      await ctx.b.press("h");
      await until(ctx, "home", (s) => s.focus === null);
      await flightDone(ctx);
      await wake(ctx);
      await clickNode(ctx, "cat-projects");
      await ctx.b.press("Escape");
      const s = await until(ctx, "coherent", (x) => x.focus === null);
      await flightDone(ctx);
      ctx.expect(s.surface === "orbital" && s.mode === "world", "coherent Home after interrupting the flight");
      await enterCase(ctx);
    });
    await ctx.step("resize during focused, artifact and temporal states", async () => {
      await click(ctx, `${node("ns-secondary")} [data-testid="artifact-pill"]`);
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-diff-491"]');
      await waitVisible(ctx, '[data-testid="artifact-dock"]');
      await ctx.b.viewport(1280, 800);
      await sleep(1500);
      let s = await state(ctx);
      ctx.expect(s.focus === "northstar" && s.artifacts.some((a) => a.phase === "expanded"), "dock and focus survive resize");
      await ctx.b.press("Escape");
      await ctx.b.press("t");
      await until(ctx, "history", (x) => x.timeline);
      await ctx.b.viewport(1672, 941);
      await sleep(1500);
      s = await state(ctx);
      ctx.expect(s.timeline && s.focus === "northstar", "history and focus survive resize");
      await ctx.b.press("Escape");
    });
    await ctx.step("repeated open/close of the viewer leaks no state", async () => {
      await until(ctx, "live", (s) => !s.timeline);
      for (let i = 0; i < 5; i++) {
        await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-diff-491"]');
        await waitVisible(ctx, '[data-testid="artifact-dock"]');
        await ctx.b.press("Escape");
        await until(ctx, "closed", (s) => !s.artifacts.some((a) => a.phase === "expanded"));
      }
      ctx.expect((await count(ctx, '[data-testid="artifact-dock"]')) === 0, "no lingering viewer");
      ctx.expect((await count(ctx, '[data-testid="artifact-excerpt"]')) === (await state(ctx)).artifacts.length, "one excerpt per open artifact");
      const excerptEls = await ctx.b.eval<number>("window.__gs.runtime.excerptEls.size");
      ctx.expect(excerptEls === (await state(ctx)).artifacts.length, `runtime tracks exactly the open excerpts (${excerptEls})`);
      ctx.expect((await state(ctx)).focus === "northstar", "context unchanged");
    });
    await ctx.step("navigation never recreated the persistent Core", async () => {
      ctx.expect(await coreIntact(ctx), "same Core canvas and node");
    });
  },
};
