import type { Journey } from "../ladder";
import { ask, click, count, enterCase, flightDone, node, open, shot, sleep, state, text, until, visible, waitVisible } from "./helpers";

// J5–J6: narration composes the proven primitives; causal reasoning is a working representation.

export const J5: Journey = {
  id: "J5",
  title: "Narrated semantic-beat journey",
  depends_on: ["J2", "J4"],
  settles: "Narration composes focus, arrangement, artifacts and time through the same grammar, and is interruptible",
  unlocks: ["J9"],
  async run(ctx) {
    // Beat holds ×4: this browser renders at ~5 fps, so each real-pointer interaction takes seconds.
    await open(ctx, "beatPace=4");
    await enterCase(ctx);
    let id = "";
    let pausedAt = -1;
    await ctx.step("ask a contextual question; a multi-beat explanation starts", async () => {
      await ask(ctx, "Why did the pilot fail?");
      const s = await until(ctx, "narrative playing", (x) => x.narrative?.status === "playing" && x.narrative.index >= 0);
      id = s.narrative!.id;
      await ctx.b.waitFor("(document.querySelector('.void-caption')?.textContent ?? '').length > 5", 10000, "caption");
      const shown = await text(ctx, ".void-caption");
      const captions = await ctx.b.eval<string[]>(`window.__gs.store.getState().narratives[${JSON.stringify(id)}].beats.map(b => b.caption)`);
      ctx.expect(captions.includes(shown) && shown.length < 60, `a beat's short caption is shown in the world, not a transcript (${shown})`);
    });
    await ctx.step("a beat reveals source evidence; the user interrupts by inspecting it", async () => {
      await until(ctx, "record excerpt", (s) => s.artifacts.some((a) => a.id === "evi-record"), 20000);
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-record"]');
      const s = await until(ctx, "paused + dock", (x) => x.narrative?.status === "paused" && x.artifacts.some((a) => a.phase === "expanded"));
      pausedAt = s.narrative!.index;
      ctx.expect(pausedAt < 3, `interrupted mid-explanation at beat ${pausedAt}`);
      await ctx.b.waitFor("!!document.querySelector('[data-testid=renderer-markdown]')", 15000, "markdown renderer");
      await shot(ctx, "beat-interrupted");
      await sleep(3000);
      ctx.expect((await state(ctx)).narrative!.index === pausedAt, "the explanation does not advance while paused");
    });
    await ctx.step("dismiss and return to the explanation's world", async () => {
      await ctx.b.press("Escape");
      const s = await until(ctx, "dock closed", (x) => !x.artifacts.some((a) => a.phase === "expanded"));
      ctx.expect(s.narrative?.status === "paused" && s.focus === "northstar", "still paused, in the same place");
      const ph = await ctx.b.eval<string>("document.querySelector('.composer-input').getAttribute('placeholder') ?? ''");
      ctx.expect(/continue/i.test(ph), `the composer offers to continue (${ph})`);
    });
    await ctx.step("resume continues from the checkpoint and finishes", async () => {
      await ask(ctx, "continue");
      const s1 = await until(ctx, "playing again", (x) => x.narrative?.status === "playing" || x.narrative?.status === "done");
      ctx.expect(s1.narrative!.index >= pausedAt, `resumed at ${s1.narrative!.index}, not restarted`);
      const s = await until(ctx, "explanation done", (x) => x.narrative?.status === "done", 90000);
      ctx.expect(s.narrative!.id === id, "the same explanation finished");
      ctx.expect(s.surface === "causal", "it re-arranged the world causally");
      ctx.expect(s.overrides.reveal.includes("ns-failure"), "the observed failure was revealed last");
      await flightDone(ctx);
      await shot(ctx, "beat-causal");
    });
    await ctx.step("VAR-005 two artifact kinds, source evidence preserved", async () => {
      const kinds = await ctx.b.eval<string[]>("window.__gs.store.getState().artifacts.map(a => window.__gs.artifacts.get(a.id)).filter(x => x.status === 'ready').map(x => x.model.kind)");
      ctx.expect(new Set(kinds).size >= 2, `artifact kinds used: ${kinds.join(",")}`);
      const cites = await ctx.b.eval<number[]>(`window.__gs.store.getState().narratives[${JSON.stringify(id)}].beats.map(b => (b.citations || []).length)`);
      ctx.expect(cites.length === 4 && cites.every((n) => n > 0), `every beat cites evidence (${cites})`);
    });
    await ctx.step("RECOV-003 lose the agent adapter mid-explanation; direct UI stays usable", async () => {
      await open(ctx, "agentFailAfter=1&beatPace=4");
      await enterCase(ctx);
      await ask(ctx, "Why did the pilot fail?");
      const s = await until(ctx, "agent lost", (x) => x.agent === "unavailable" && !!x.narrative?.error, 20000);
      ctx.expect(!!s.narrative, "the partial explanation remains, marked as interrupted");
      await ctx.b.press("Escape");
      await until(ctx, "narration dismissed", (x) => !x.narrative);
      await click(ctx, '[data-testid="center-history"]');
      await until(ctx, "history still works", (x) => x.timeline);
      await click(ctx, '[data-testid="time-close"]');
      await click(ctx, `${node("ns-impl")} [data-testid="artifact-pill"]`);
      await waitVisible(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-impl-graph"]');
      await click(ctx, node("ns-release"), { at: "orb" });
      await until(ctx, "focus still works", (x) => x.focus === "ns-release");
      await ask(ctx, "What's the evidence?");
      await sleep(500);
      const ph = await ctx.b.eval<string>("document.querySelector('.composer-input').getAttribute('placeholder') ?? ''");
      ctx.expect(/unavailable/i.test(ph), `the composer says honestly that the agent is unavailable (${ph})`);
    });
  },
};

export const J6: Journey = {
  id: "J6",
  title: "Causal reasoning journey",
  depends_on: ["J2", "J3"],
  settles: "The causal projection is a working thinking representation with inspectable evidence",
  unlocks: ["J7", "J9"],
  async run(ctx) {
    await open(ctx);
    await enterCase(ctx);
    await ctx.step("enter the causal representation", async () => {
      await click(ctx, '[data-testid="center-causal"]');
      await until(ctx, "causal", (s) => s.surface === "causal");
      await flightDone(ctx);
    });
    await ctx.step("expected, missing condition and observed are distinct, with causal links", async () => {
      const roles = await ctx.b.eval<string[]>("[...document.querySelectorAll('.role-pill')].map(e => e.textContent)");
      for (const r of ["Intended path", "Key assumption", "Real-world condition", "Unmet requirement", "Failure observed"]) {
        ctx.expect(roles.some((x) => x.includes(r)), `role ${r} shown`);
      }
      const labels = await ctx.b.eval<string[]>("[...document.querySelectorAll('.links-layer .link-labels text')].map(e => e.textContent)");
      ctx.expect(labels.length >= 4, `labelled causal links (${labels.join(", ")})`);
    });
    await ctx.step("select a causal node", async () => {
      await click(ctx, node("ns-failure"), { at: "orb" });
      const s = await until(ctx, "selected failure", (x) => x.selection === "ns-failure");
      ctx.expect(s.surface === "causal", "selection keeps the representation");
    });
    await ctx.step("inspect the supporting source (expected vs observed)", async () => {
      await click(ctx, `${node("ns-failure")} [data-testid="artifact-pill"]`);
      await waitVisible(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-coverage-trace"]');
      await click(ctx, '[data-testid="artifact-excerpt"][data-ref="evi-coverage-trace"]');
      await ctx.b.waitFor("!!document.querySelector('[data-testid=renderer-trace]')", 15000, "trace renderer");
      const t = await text(ctx, '[data-testid="renderer-trace"]');
      ctx.expect(/claim accepted/.test(t) && /claim rejected/.test(t), "expected and observed outcomes side by side");
      await click(ctx, '[data-testid="renderer-trace"] tr[data-selected], [data-testid="renderer-trace"] tbody tr');
      await shot(ctx, "causal-evidence");
      await click(ctx, '[data-testid="dock-close"]');
      const s = await until(ctx, "closed", (x) => !x.artifacts.some((a) => a.phase === "expanded"));
      ctx.expect(s.surface === "causal" && s.selection === "ns-failure", "causal view and selection restored");
    });
    await ctx.step("move between causal and orbital without identity loss", async () => {
      const before = await ctx.b.eval<string[]>("[...document.querySelectorAll('[data-node]')].map(e => e.dataset.node).sort()");
      await click(ctx, '[data-testid="center-causal"]');
      await until(ctx, "orbital", (s) => s.surface === "orbital");
      await click(ctx, '[data-testid="center-causal"]');
      const s = await until(ctx, "causal again", (x) => x.surface === "causal");
      ctx.expect(s.selection === "ns-failure", "selection survives the round trip");
      await flightDone(ctx);
      const after = await ctx.b.eval<string[]>("[...document.querySelectorAll('[data-node]')].map(e => e.dataset.node).sort()");
      ctx.expect(after.every((id) => before.includes(id) || id.startsWith("ns-")), "no invented causal-only objects");
      ctx.expect((await count(ctx, node("ns-failure"))) === 1, "one ns-failure object");
    });
    await ctx.step("return to the prior focus", async () => {
      await ctx.b.press("Escape");
      const s = await until(ctx, "orbital focus", (x) => x.surface === "orbital");
      ctx.expect(s.focus === "northstar", "back in the focused case");
      ctx.expect(await visible(ctx, node("ns-release")), "orbital arrangement restored");
    });
  },
};
