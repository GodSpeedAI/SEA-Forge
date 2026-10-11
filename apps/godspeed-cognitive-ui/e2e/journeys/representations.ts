import type { Journey } from "../ladder";
import {
  click, count, enterCase, flightDone, historyHash, markNode, node, nodeMark, open, shot, sleep, state, text, until, visible, waitVisible,
} from "./helpers";

// J3–J4: the same source objects in alternate representations, then time and comparison.

export const J3: Journey = {
  id: "J3",
  title: "Representation-switching substrate",
  depends_on: ["J1"],
  settles: "Orbital, causal and comparison are projections of the same objects (identity, selection and state carry across)",
  unlocks: ["J4", "J6"],
  async run(ctx) {
    await open(ctx);
    await enterCase(ctx);
    await ctx.step("mark the source objects' rendered identity", async () => {
      for (const id of ["ns-secondary", "ns-workflow", "ns-release"]) await markNode(ctx, id, `m-${id}`);
    });
    await ctx.step("orbital → causal: the same objects re-arrange", async () => {
      await click(ctx, '[data-testid="center-causal"]');
      await until(ctx, "causal surface", (s) => s.surface === "causal");
      await flightDone(ctx);
      ctx.expect((await nodeMark(ctx, "ns-secondary")) === "m-ns-secondary", "ns-secondary is the same rendered object");
      ctx.expect((await nodeMark(ctx, "ns-workflow")) === "m-ns-workflow", "ns-workflow is the same rendered object");
      const roles = await ctx.b.eval<string[]>("[...document.querySelectorAll('.role-pill')].map(e => e.textContent)");
      ctx.expect(roles.some((r) => /Intended path/.test(r)) && roles.some((r) => /Unmet requirement/.test(r)), `causal roles shown (${roles.join(" | ")})`);
      await shot(ctx, "causal");
    });
    await ctx.step("select in the causal representation", async () => {
      await click(ctx, node("ns-secondary"), { at: "orb" });
      const s = await until(ctx, "selected", (x) => x.selection === "ns-secondary");
      ctx.expect(s.surface === "causal" && s.focus === "northstar", "selection does not navigate away");
      ctx.expect((await ctx.b.eval<string>(`document.querySelector('${node("ns-secondary")}').getAttribute('aria-pressed')`)) === "true", "selected object marked");
    });
    await ctx.step("causal → orbital: the selection is the same object", async () => {
      await click(ctx, '[data-testid="center-causal"]');
      await until(ctx, "orbital", (s) => s.surface === "orbital");
      await flightDone(ctx);
      ctx.expect((await nodeMark(ctx, "ns-secondary")) === "m-ns-secondary", "same rendered object");
      ctx.expect((await ctx.b.eval<string>(`document.querySelector('${node("ns-secondary")}').getAttribute('aria-pressed')`)) === "true", "still selected in orbital");
    });
    await ctx.step("orbital → comparison of the same objects", async () => {
      await click(ctx, '[data-testid="center-compare"]');
      const s = await until(ctx, "compare", (x) => x.surface === "compare" && !!x.compare);
      await waitVisible(ctx, '[data-testid="compare-bar"]');
      ctx.expect((await nodeMark(ctx, "ns-secondary")) === "m-ns-secondary", "same rendered object in comparison");
      ctx.expect((await count(ctx, node("ns-secondary"))) === 1, "no duplicate object");
      const notes = await count(ctx, '[data-testid="compare-note"]');
      ctx.expect(notes >= 1, `differences are annotated on the objects (${notes})`);
      ctx.expect(s.selection === "ns-secondary", "selection carries into comparison");
      await shot(ctx, "compare");
    });
    await ctx.step("return without duplicate state", async () => {
      await click(ctx, '[data-testid="compare-exit"]');
      const s = await until(ctx, "compare closed", (x) => !x.compare);
      ctx.expect(s.surface === "orbital" && s.revision === s.now, "back to the live orbital world");
      const dupes = await ctx.b.eval<number>("(() => { const ids = [...document.querySelectorAll('[data-node]')].map(e => e.dataset.node); return ids.length - new Set(ids).size })()");
      ctx.expect(dupes === 0, "no duplicated objects in the DOM");
    });
  },
};

export const J4: Journey = {
  id: "J4",
  title: "Temporal + comparison substrate",
  depends_on: ["J2", "J3"],
  settles: "Time/version and comparison are trustworthy operators that never mutate source history",
  unlocks: ["J5", "J9"],
  async run(ctx) {
    await open(ctx);
    await enterCase(ctx);
    const h0 = await historyHash(ctx);
    const revs = (await state(ctx)).revisions;
    await ctx.step("enter historical inspection", async () => {
      await click(ctx, '[data-testid="center-history"]');
      await until(ctx, "timeline open", (s) => s.timeline);
      await waitVisible(ctx, '[data-testid="time-strip"]');
    });
    const positions = [revs[1]!, revs[3]!, revs[4]!];
    await ctx.step("VAR-004 traverse three historical positions", async () => {
      const seen: string[] = [];
      for (const rev of positions) {
        await click(ctx, `[data-testid="time-marker"][data-rev="${rev}"]`);
        const s = await until(ctx, `at ${rev}`, (x) => x.revision === rev);
        seen.push(await text(ctx, '[data-testid="time-current"]'));
        ctx.expect(s.revision !== s.now, "in the past");
      }
      ctx.expect(new Set(seen).size === 3, `the active position is shown (${seen.join(" | ")})`);
      await sleep(1500);
      await shot(ctx, "history");
    });
    await ctx.step("the past is the same objects at an earlier state (read-only)", async () => {
      await click(ctx, `[data-testid="time-marker"][data-rev="${positions[0]}"]`);
      await until(ctx, "earliest", (x) => x.revision === positions[0]);
      await sleep(1500);
      ctx.expect(!(await visible(ctx, '[data-testid="action-chip"]')), "no actions offered in the past");
    });
    await ctx.step("mark two positions and compare them", async () => {
      await click(ctx, '[data-testid="time-mark"]');
      await click(ctx, `[data-testid="time-marker"][data-rev="${positions[2]}"]`);
      await until(ctx, "second position", (x) => x.revision === positions[2]);
      await click(ctx, '[data-testid="time-mark"]');
      await until(ctx, "two marks", (x) => x.marks.length === 2);
      await click(ctx, '[data-testid="time-compare"]');
      const s = await until(ctx, "comparing", (x) => !!x.compare);
      ctx.expect(s.compare!.a === positions[0] && s.compare!.b === positions[2], `compares A=${s.compare!.a} B=${s.compare!.b}`);
      await waitVisible(ctx, '[data-testid="compare-bar"]');
      const bar = await text(ctx, '[data-testid="compare-bar"]');
      ctx.expect(/changed/.test(bar), `comparison summarises differences (${bar})`);
      await sleep(1500);
      await shot(ctx, "compare-history");
    });
    await ctx.step("inspect a source artifact during comparison", async () => {
      await click(ctx, `${node("ns-secondary")} [data-testid="artifact-pill"]`);
      await waitVisible(ctx, '[data-testid="artifact-excerpt"]');
      await click(ctx, '[data-testid="artifact-excerpt"]');
      await waitVisible(ctx, '[data-testid="artifact-dock"]');
      await click(ctx, '[data-testid="dock-close"]');
      const s = await until(ctx, "dock closed", (x) => !x.artifacts.some((a) => a.phase === "expanded"));
      ctx.expect(!!s.compare && s.compare.a === positions[0], "comparison restored after inspecting the artifact");
    });
    await ctx.step("exit comparison back to the single historical position", async () => {
      await click(ctx, '[data-testid="compare-exit"]');
      const s = await until(ctx, "compare closed", (x) => !x.compare);
      ctx.expect(s.revision === positions[2] && s.timeline, `back at ${positions[2]} in history (${s.revision})`);
    });
    await ctx.step("return to live", async () => {
      await click(ctx, '[data-testid="time-live"]');
      const s = await until(ctx, "live", (x) => x.revision === x.now && !x.timeline);
      ctx.expect(s.focus === "northstar", "orientation kept on return to live");
    });
    await ctx.step("no source-history mutation", async () => {
      ctx.expect((await historyHash(ctx)) === h0, "history identical after traversal and comparison");
    });
  },
};
