import type { Journey } from "../ladder";
import {
  camera, click, coreIntact, count, enterCase, flightDone, loadedRenderers, markCore, node, open, rect,
  rendererRequests, shot, sleep, state, text, until, visible, waitVisible, wake,
} from "./helpers";

// J0–J2: orientation, focus, artifact inspection. Each settles a primitive the next relies on.

export const J0: Journey = {
  id: "J0",
  title: "Boot / orientation substrate",
  depends_on: [],
  settles: "A stable CORE-centred orientation surface from which selection is possible",
  unlocks: ["J1"],
  async run(ctx) {
    await ctx.step("app opens through the contract port", async () => {
      await open(ctx);
      const s = await state(ctx);
      ctx.expect(s.focus === null && s.mode === "world" && s.surface === "orbital", "boots at Home in the orbital representation");
      ctx.expect(s.revisions.length >= 6, "history loaded through the port");
      ctx.expect((await text(ctx, ".status-label")) === "Local contract adapter", "status bar labels the local contract adapter honestly");
    });
    await ctx.step("CORE persists as a single rendered anchor", async () => {
      ctx.expect((await count(ctx, "canvas.core-canvas")) === 1, "one WebGL canvas");
      ctx.expect((await count(ctx, node("core"))) === 1, "one Core node");
      ctx.expect(await visible(ctx, node("core")), "Core visible");
      await markCore(ctx);
    });
    await ctx.step("VAR-001 quiet world stays sparse while data is hidden", async () => {
      const shown = await ctx.b.eval<number>("[...document.querySelectorAll('[data-node]')].filter(e => e.style.display !== 'none' && e.dataset.node !== 'core').length");
      const total = await ctx.b.eval<number>("Object.keys(window.__gs.worldView().snap.objects).length");
      ctx.expect(total >= 25, `substantial data exists (${total} objects)`);
      ctx.expect(shown <= 8, `idle Home shows only the regions (${shown} shown)`);
      await shot(ctx, "home-quiet");
    });
    await ctx.step("no heavy artifact renderer is loaded", async () => {
      ctx.expect((await loadedRenderers(ctx)).length === 0, "no renderer chunk requested");
      ctx.expect((await rendererRequests(ctx)).length === 0, "no renderer module fetched");
    });
    await ctx.step("VAR-002 waking surfaces the consequential object among routine ones", async () => {
      await wake(ctx);
      await sleep(1200);
      const residue = await text(ctx, ".residue-home");
      ctx.expect(/need/.test(residue), `Core says something needs you (${residue})`);
      const attention = await ctx.b.eval<string[]>("[...document.querySelectorAll('[data-node].tone-attention, [data-node].tone-critical')].filter(e => e.style.display !== 'none').map(e => e.dataset.node)");
      const routine = await ctx.b.eval<number>("[...document.querySelectorAll('[data-node].satellite')].filter(e => e.style.display !== 'none' && !e.className.match(/tone-(attention|critical)/)).length");
      ctx.expect(attention.length >= 1, `attention objects surface (${attention.join(",")})`);
      ctx.expect(routine === 0, `routine members stay hidden (${routine})`);
      await shot(ctx, "home-awake");
    });
    await ctx.step("composer and chrome are available; work objects are selectable", async () => {
      await wake(ctx);
      ctx.expect(await visible(ctx, ".composer-input"), "composer visible");
      ctx.expect(await visible(ctx, ".status-bar-container"), "status bar visible when awake");
      const role = await ctx.b.eval<string>(`document.querySelector('${node("cat-projects")}').getAttribute('role')`);
      ctx.expect(role === "button" && (await visible(ctx, node("cat-projects"))), "Projects is a visible, selectable object");
    });
  },
};

export const J1: Journey = {
  id: "J1",
  title: "Focus substrate",
  depends_on: ["J0"],
  settles: "Focus is a reusable primitive that preserves CORE as orientation and object identity",
  unlocks: ["J2", "J3"],
  async run(ctx) {
    await open(ctx);
    await markCore(ctx);
    let ids: string[] = [];
    await ctx.step("select a work object and fly into it", async () => {
      await enterCase(ctx);
      const s = await state(ctx);
      ctx.expect(s.focusStack.join("/") === "northstar", `focus path ${s.focusStack} (regions are not focus levels)`);
      ctx.expect((await ctx.b.eval<string>("window.__gs.runtime.result.center")) === "northstar", "Northstar is the local center");
    });
    await ctx.step("CORE remains the home anchor and is not recreated", async () => {
      ctx.expect(await coreIntact(ctx), "same Core canvas and node");
      ctx.expect(await visible(ctx, ".core-anchor-container[data-visible='true']"), "Core anchor offered as the way home");
    });
    await ctx.step("local relationships and details are exposed", async () => {
      for (const id of ["ns-release", "ns-secondary", "ns-impl"]) ctx.expect(await visible(ctx, node(id)), `${id} visible`);
      ids = await ctx.b.eval<string[]>("[...document.querySelectorAll('[data-node]')].filter(e => e.style.display !== 'none').map(e => e.dataset.node).sort()");
      ctx.expect((await count(ctx, ".links-layer .links path")) >= 3, "spokes drawn to the parts");
      await shot(ctx, "focused-case");
    });
    await ctx.step("unfocus returns through the hierarchy to Home", async () => {
      await ctx.b.press("Escape");
      await until(ctx, "back to Home", (s) => s.focus === null);
      await flightDone(ctx);
    });
    await ctx.step("focus again: identity and state survive", async () => {
      await enterCase(ctx);
      const again = await ctx.b.eval<string[]>("[...document.querySelectorAll('[data-node]')].filter(e => e.style.display !== 'none').map(e => e.dataset.node).sort()");
      ctx.expect(JSON.stringify(again) === JSON.stringify(ids), "same objects by identity after refocus");
      ctx.expect(await coreIntact(ctx), "Core still the same instance");
    });
    await ctx.step("the Core anchor returns Home", async () => {
      await click(ctx, ".core-anchor-container");
      await until(ctx, "Home", (s) => s.focus === null);
    });
  },
};

export const J2: Journey = {
  id: "J2",
  title: "Artifact inspection substrate",
  depends_on: ["J0", "J1"],
  settles: "Artifacts disclose progressively (pill → excerpt → viewer) without destroying spatial context",
  unlocks: ["J4", "J5", "J6", "J7"],
  async run(ctx) {
    await open(ctx);
    await enterCase(ctx);
    let before: { x: number; y: number; zoom: number } = { x: 0, y: 0, zoom: 0 };
    await ctx.step("a source-backed preview is disclosed from its object", async () => {
      await click(ctx, `${node("ns-secondary")} [data-testid="artifact-pill"]`);
      await waitVisible(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-diff-491"]');
      await ctx.b.waitFor("!document.querySelector('[data-testid=artifact-excerpt][data-ref=evi-diff-491]').textContent.includes('Resolving')", 10000, "excerpt resolved");
      ctx.expect((await loadedRenderers(ctx)).length === 0, "the excerpt is light: no source renderer loaded yet");
      await shot(ctx, "excerpts");
    });
    await ctx.step("selecting it opens the right-side viewer with the diff renderer", async () => {
      before = await camera(ctx);
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-diff-491"]');
      await waitVisible(ctx, '[data-testid="artifact-dock"]');
      await ctx.b.waitFor("!!document.querySelector('[data-testid=renderer-diff]')", 15000, "diff renderer");
      const r = await rect(ctx, '[data-testid="artifact-dock"]');
      ctx.expect(r && r.x > 900, "viewer docks on the right");
      const loaded = await loadedRenderers(ctx);
      ctx.expect(loaded.includes("diff") && !loaded.includes("graph"), `only the needed renderer loads (${loaded})`);
      const s = await state(ctx);
      ctx.expect(s.focus === "northstar", "the world stays in context (no navigation away)");
      await shot(ctx, "dock-diff");
    });
    await ctx.step("interact: search highlights, a line can be selected", async () => {
      await ctx.b.fill('[data-testid="dock-search"]', "coverage");
      await sleep(600);
      const marks = await count(ctx, '[data-testid="renderer-diff"] mark');
      ctx.expect(marks > 0, `search highlights matches (${marks})`);
      await click(ctx, '[data-testid="renderer-diff"] [role="option"]');
      ctx.expect((await count(ctx, '[data-testid="renderer-diff"] [data-selected="true"]')) >= 1, "a line is selected");
    });
    await ctx.step("source and provenance are preserved", async () => {
      const prov = await text(ctx, '[data-testid="dock-provenance"]');
      ctx.expect(prov.includes("evi-diff-491") && /sha256/.test(prov) && prov.includes("491"), `provenance shown (${prov.slice(0, 120)})`);
    });
    await ctx.step("a second artifact kind on the same object (tab)", async () => {
      await click(ctx, '[data-testid="dock-tab"][data-ref="evi-coverage-trace"]');
      await ctx.b.waitFor("!!document.querySelector('[data-testid=renderer-trace]')", 15000, "trace renderer");
      const t = await text(ctx, '[data-testid="renderer-trace"]');
      ctx.expect(/Expected/.test(t) && /Observed/.test(t) && /claim rejected/.test(t), "expected vs observed comparison renders");
    });
    await ctx.step("close returns to the exact prior context", async () => {
      await click(ctx, '[data-testid="dock-close"]');
      await until(ctx, "dock closed", (s) => !s.artifacts.some((a) => a.phase === "expanded"));
      await flightDone(ctx);
      const after = await camera(ctx);
      ctx.expect(Math.abs(after.x - before.x) < 1e-6 * Math.max(1, Math.abs(before.x)) + 0.5 && Math.abs(after.zoom / before.zoom - 1) < 1e-3, `camera restored (${JSON.stringify(before)} → ${JSON.stringify(after)})`);
      const s = await state(ctx);
      ctx.expect(s.focus === "northstar" && s.surface === "orbital", "focus and representation restored");
      ctx.expect((await count(ctx, '[data-testid="artifact-dock"]')) === 0, "viewer removed");
    });
    await ctx.step("reopen, pin (local), close: the pinned artifact stays available", async () => {
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-diff-491"]');
      await waitVisible(ctx, '[data-testid="dock-pin"]');
      await click(ctx, '[data-testid="dock-pin"]');
      ctx.expect((await ctx.b.eval<string>("document.querySelector('[data-testid=dock-pin]').getAttribute('aria-pressed')")) === "true", "pin pressed");
      await ctx.b.press("Escape");
      await until(ctx, "collapsed", (s) => !s.artifacts.some((a) => a.phase === "expanded"));
      const s = await state(ctx);
      ctx.expect(s.artifacts.some((a) => a.id === "evi-diff-491" && a.pinned), "pinned artifact kept");
      await ctx.b.press("Escape");
      await ctx.b.press("h");
      const h = await until(ctx, "home", (x) => x.focus === null);
      ctx.expect(h.artifacts.some((a) => a.id === "evi-diff-491" && a.pinned), "pin survives going Home");
    });
  },
};
