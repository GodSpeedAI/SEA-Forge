import type { Journey } from "../ladder";
import { ask, click, count, enterCase, flightDone, historyHash, node, open, shot, sleep, state, text, until, visible, waitVisible } from "./helpers";

// J7–J8: case design as a bounded proposal environment; consequential action through the one
// frontend intent path (human and agent), authority, execution, evidence and settlement.

export const J7: Journey = {
  id: "J7",
  title: "Case Design journey",
  depends_on: ["J3", "J6"],
  settles: "Case Design is a bounded thinking/proposal environment that never mutates authoritative state",
  unlocks: ["J9"],
  async run(ctx) {
    await open(ctx);
    await enterCase(ctx);
    const h0 = await historyHash(ctx);
    await ctx.step("enter Case Design from the focused case", async () => {
      await click(ctx, '[data-testid="center-design"]');
      await until(ctx, "design mode", (s) => s.mode === "case-design" && !!s.design);
      await waitVisible(ctx, '[data-testid="design-panel"]');
      ctx.expect((await count(ctx, '[data-testid="design-stage"]')) === 5, "the published template's five stages are shown");
      await flightDone(ctx);
      await shot(ctx, "design");
    });
    await ctx.step("inspect structure: select a design node", async () => {
      await click(ctx, node("tpl-stages"), { at: "orb" });
      await until(ctx, "stages selected", (s) => s.design?.selected === "tpl-stages");
    });
    await ctx.step("inspect an earlier published version", async () => {
      await click(ctx, '[data-testid="design-version"][data-version="1.0000000001"]');
      await until(ctx, "v0.1 shown", (s) => s.design?.revision === "1.0000000001");
      ctx.expect((await count(ctx, '[data-testid="design-stage"]')) === 4, "v0.1 had four stages");
      await click(ctx, '[data-testid="design-version"][data-version="1.0000000002"]');
      await until(ctx, "v0.2 shown", (s) => s.design?.revision === "1.0000000002");
    });
    await ctx.step("propose: reorder a stage and change required evidence (local draft)", async () => {
      const first = await text(ctx, '[data-testid="design-stage"]');
      await click(ctx, '[data-testid="design-stage"] [data-testid="stage-down"]');
      const s = await until(ctx, "draft", (x) => !!x.design?.hasDraft && x.design.revision === "draft");
      ctx.expect(s.design!.revision === "draft", "the proposal is a local draft");
      ctx.expect((await text(ctx, '[data-testid="design-stage"]')) !== first, "the order changed in the draft");
      await click(ctx, '[data-testid="evidence-required"]');
      ctx.expect(/Local draft/i.test(await text(ctx, '[data-testid="design-panel"]')), "the panel labels the proposal as a local draft");
    });
    await ctx.step("save the draft locally; submission is honestly unavailable", async () => {
      await click(ctx, '[data-testid="design-save"]');
      await sleep(300);
      ctx.expect(/Saved locally/i.test(await text(ctx, '[data-testid="design-panel"]')), "saved locally");
      const disabled = await ctx.b.eval<boolean>("(() => { const b = document.querySelector('[data-testid=design-submit]'); return b.disabled || b.getAttribute('aria-disabled') === 'true' })()");
      ctx.expect(disabled, "Submit proposal is not offered as if it worked");
    });
    await ctx.step("compare the proposal with the published structure", async () => {
      await click(ctx, '[data-testid="design-compare"]');
      const s = await until(ctx, "design compare", (x) => x.compare?.source === "design");
      ctx.expect(s.compare!.b === "draft", "B is the local draft");
      await waitVisible(ctx, '[data-testid="compare-bar"]');
      ctx.expect((await count(ctx, `${node("tpl-stages")} [data-testid="compare-note"], ${node("tpl-evidence")} [data-testid="compare-note"]`)) >= 1, "changed design nodes are marked");
      await flightDone(ctx);
      await shot(ctx, "design-compare");
      await click(ctx, '[data-testid="compare-exit"]');
      await until(ctx, "compare closed", (x) => !x.compare && x.mode === "case-design");
    });
    await ctx.step("authoritative template unchanged by the proposal", async () => {
      const firstPublished = await ctx.b.eval<string>("(() => { const d = window.__gs.store.getState().design; const snap = d.history.snapshots[d.history.revisions.at(-1).id]; return Object.values(snap.objects).find(o => o.icon === 'layers').designItems[0].label })()");
      ctx.expect(firstPublished === "Intake", `published v0.2 still starts with Intake (${firstPublished})`);
    });
    await ctx.step("leave Case Design back to the case, without mutation", async () => {
      await click(ctx, '[data-testid="design-close"]');
      const s = await until(ctx, "world", (x) => x.mode === "world" && !x.design);
      ctx.expect(s.focus === "northstar", "returned to the focused case");
      ctx.expect((await historyHash(ctx)) === h0, "the live world is untouched");
      await flightDone(ctx);
      ctx.expect(await visible(ctx, node("ns-release")), "the case world is back");
    });
  },
};

export const J8: Journey = {
  id: "J8",
  title: "Consequential-action UI journey (contract adapter)",
  depends_on: ["J2", "J7"],
  settles: "Human and agent consequences share one intent path; authority, execution, evidence and settlement are distinct",
  unlocks: ["J9"],
  async run(ctx) {
    await open(ctx);
    await enterCase(ctx);
    const revs0 = (await state(ctx)).revisions.length;
    await ctx.step("a consequential action opens judgment (backend-offered choices)", async () => {
      await click(ctx, `${node("ns-release")} [data-testid="action-chip"][data-action="approve-release"]`);
      await until(ctx, "judgment", (s) => s.judgment?.object === "ns-release" && s.surface === "judgment");
      await waitVisible(ctx, '[data-testid="judgment-panel"]');
      const opts = await ctx.b.eval<string[]>("[...document.querySelectorAll('[data-testid=judgment-option]')].map(b => b.dataset.option)");
      ctx.expect(opts.join(",") === "approve-release,request-changes,escalate", `choices come from the contract actions (${opts})`);
      const focused = await ctx.b.eval<string>("document.activeElement?.dataset?.testid ?? document.activeElement?.tagName");
      ctx.expect(focused !== "judgment-option", "no option is auto-focused (a stray Enter cannot decide)");
      await flightDone(ctx);
      await shot(ctx, "judgment");
    });
    await ctx.step("VAR-007 agent attempts the same consequence through the same path → denied", async () => {
      await ask(ctx, "let the agent approve the release");
      const s = await until(ctx, "agent refused", (x) => x.intents.some((i) => i.actor.kind === "agent" && i.state === "refused"));
      const agentIntent = s.intents.find((i) => i.actor.kind === "agent")!;
      ctx.expect(agentIntent.actionName === "APPROVE_HUMAN_TASK" && agentIntent.code === "AUTHORITY_DENIED", `authority denied the agent (${agentIntent.code})`);
      await until(ctx, "outcome shown", (x) => x.judgment?.outcome?.state === "refused");
      await waitVisible(ctx, '[data-testid="judgment-outcome"][data-state="refused"]');
      ctx.expect(/GodSpeed agent/.test(await text(ctx, '[data-testid="judgment-outcome"]')), "the refusal names the agent actor");
      ctx.expect((await state(ctx)).revisions.length === revs0, "denial caused zero consequential effects");
      await shot(ctx, "agent-denied");
    });
    await ctx.step("a justification-required choice is validated before sending", async () => {
      const n = (await state(ctx)).intents.length;
      await click(ctx, '[data-testid="judgment-option"][data-option="escalate"]');
      await sleep(600);
      ctx.expect((await state(ctx)).intents.length === n, "no intent sent without a reason");
    });
    await ctx.step("the human approves on a fresh attempt → accepted", async () => {
      await click(ctx, '[data-testid="judgment-option"][data-option="approve-release"]');
      const s = await until(ctx, "accepted", (x) => x.judgment?.outcome?.state === "accepted" || x.intents.some((i) => i.actor.kind === "human" && i.state === "accepted"));
      const human = s.intents.find((i) => i.actor.kind === "human")!;
      const agent = s.intents.find((i) => i.actor.kind === "agent")!;
      ctx.expect(human.actionName === agent.actionName, "human and agent used the same action through the same intent path");
    });
    const seen: string[] = [];
    await ctx.step("execution is visible and completion does not read as settlement", async () => {
      await until(ctx, "running", (s) => !!s.executions["ns-release"], 15000);
      const t0 = Date.now();
      let executedLabel = "";
      let violations = 0;
      while (Date.now() - t0 < 60000) {
        // One atomic read: execution state, the live snapshot's settlement, and the pill text.
        const r = await ctx.b.eval<{ st?: string; label: string; settled: boolean; pill: string }>(`(() => {
          const s = window.__gs.store.getState();
          const o = s.history.snapshots[s.history.revisions.at(-1).id].objects['ns-release'];
          return { st: s.executions['ns-release']?.state, label: o.status.label, settled: !!o.settlement, pill: document.querySelector('[data-testid=execution-pill]')?.textContent ?? '' };
        })()`);
        if (r.st && seen[seen.length - 1] !== r.st) seen.push(r.st);
        if (r.st === "executed") {
          if (r.settled) violations++;
          if (!executedLabel) {
            executedLabel = r.label;
            ctx.expect(/awaiting settlement/.test(r.pill), `the pill says awaiting settlement (${r.pill})`);
            await shot(ctx, "executed-awaiting-settlement");
          }
        }
        if (r.st === "settled") break;
        await sleep(200);
      }
      ctx.expect(violations === 0, "never shown as executed while a settlement exists");
      ctx.expect(seen.includes("executed") && seen.indexOf("executed") < seen.indexOf("settled"), `execution → executed → settled (${seen.join(" → ")})`);
      ctx.expect(/awaiting settlement/i.test(executedLabel), `completion shown as awaiting settlement (${executedLabel})`);
    });
    await ctx.step("evidence appears and settlement is authoritative", async () => {
      const s = await state(ctx);
      ctx.expect(s.revisions.length >= revs0 + 3, `new contract snapshots appended (${s.revisions.length - revs0})`);
      const settled = await ctx.b.eval<{ label: string; decision: string }>("(() => { const s = window.__gs.store.getState(); const o = s.history.snapshots[s.history.revisions.at(-1).id].objects['ns-release']; return { label: o.status.label, decision: o.settlement && o.settlement.decision } })()");
      ctx.expect(settled.decision === "ACCEPTED" && /Settled/.test(settled.label), `settled by authority (${JSON.stringify(settled)})`);
      ctx.expect(await ctx.b.eval<boolean>("!!window.__gs.store.getState().history.snapshots[window.__gs.store.getState().history.revisions.at(-1).id].objects['evo-ns-release-1']"), "evidence object recorded in the world");
    });
    await ctx.step("inspect execution, evidence and settlement", async () => {
      await click(ctx, '[data-testid="execution-pill"]');
      await until(ctx, "execution inspect", (s) => s.mode === "execution-inspect");
      await waitVisible(ctx, '[data-testid="execution-panel"]');
      ctx.expect(/Settled/i.test(await text(ctx, '[data-testid="settlement"]')), "settlement shown separately from execution");
      await shot(ctx, "execution-inspect");
      await click(ctx, '[data-testid="execution-evidence"]');
      await ctx.b.waitFor("!!document.querySelector('[data-testid=renderer-table]') || !!document.querySelector('[data-testid=renderer-text]')", 15000, "evidence renderer");
      await shot(ctx, "evidence");
      await ctx.b.press("Escape");
    });
    await ctx.step("the world reorganises from the settled projection", async () => {
      await until(ctx, "back in world", (s) => s.mode === "world" && !s.artifacts.some((a) => a.phase === "expanded"));
      await flightDone(ctx);
      const label = await ctx.b.eval<string>(`document.querySelector('${node("ns-release")}')?.getAttribute('aria-label') ?? ''`);
      ctx.expect(/Settled/.test(label), `release reads as settled in the world (${label})`);
      await shot(ctx, "settled-world");
    });
  },
};
