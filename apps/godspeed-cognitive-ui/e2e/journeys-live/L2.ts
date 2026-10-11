import type { Journey } from "../ladder";
import { badgeFor, foldStanding, readPlan, readTrace, kindsOf } from "../live/durable";
import { clickNode } from "../journeys/helpers";
import { caseAOf, closeOutline, ensureLoggedIn, jsEval, openOutlineAtHome, stackOf, uiCtx, uiObjects } from "./helpers";

const ACT_EXEC = "act-execute_item-task_prepare";

// L2 horizon standing. The standing the UI shows for the committed case must be the standing the
// kernel's durable state implies: cases/<id>/plan.json (what exists) folded with
// cases/<id>/case-events.jsonl (what has happened) by an independent fold. After commit only the
// first manual item is enabled; the sentry-gated item and the rollup milestone are pending and
// say why. Nothing is shown as ready, running or done that the durable trace does not support.
export const L2: Journey = {
  id: "L2",
  title: "Horizon standing",
  depends_on: ["L1"],
  settles: "the UI shows the case's real standing, not an invented one",
  unlocks: ["L3", "L4"],
  async run(ctx) {
    const cell = stackOf(ctx).cell;
    const caseId = caseAOf(ctx);
    const b = await ensureLoggedIn(ctx, "operator");
    const ui = uiCtx(ctx, b);
    const plan = readPlan(cell, caseId);
    const trace = readTrace(cell, caseId);
    const standing = foldStanding(plan, trace);

    await ctx.step("durable: the kernel's own files say task_prepare is enabled and the rest are pending", async () => {
      ctx.expect(kindsOf(trace).join(",") === "case_created,item_enabled", `trace kinds ${kindsOf(trace).join(",")}`);
      ctx.expect(standing.task_prepare?.execution === "enabled", `task_prepare ${JSON.stringify(standing.task_prepare)}`);
      for (const id of ["task_publish", "ms_chain_accepted"]) {
        ctx.expect(standing[id]?.execution === "pending" && standing[id]?.settlement === "unsettled", `${id} ${JSON.stringify(standing[id])}`);
      }
      ctx.delta({ step: "durable-standing", case_id: caseId, standing });
    });

    await ctx.step("UI: the world shows exactly the plan's items, each with the standing the durable fold implies", async () => {
      await b.waitFor(`window.__gs.worldView().snap.caseId === ${JSON.stringify(caseId)}`, 20000, "world shows the committed case");
      const objs = await uiObjects(b);
      const planIds = plan.items.map((i) => i.plan_item_id).sort();
      ctx.expect(JSON.stringify(Object.keys(objs).sort()) === JSON.stringify(planIds), `UI objects ${Object.keys(objs).sort().join(",")} vs plan ${planIds.join(",")}`);
      const shown: Record<string, string> = {};
      for (const item of plan.items) {
        const o = objs[item.plan_item_id]!;
        const want = badgeFor(item.item_kind, standing[item.plan_item_id]!);
        shown[item.plan_item_id] = o.status;
        ctx.expect(o.status === want, `${item.plan_item_id}: UI says "${o.status}", durable standing implies "${want}"`);
        ctx.expect(o.title === item.name, `${item.plan_item_id}: title "${o.title}" vs plan name "${item.name}"`);
      }
      ctx.delta({ step: "ui-standing", shown });
    });

    await ctx.step("UI: a pending item says why it waits; the ready one says it is ready", async () => {
      const objs = await uiObjects(b);
      ctx.expect(/entry sentries have not fired/.test(objs.task_publish!.subtitle ?? ""), `task_publish reason: ${objs.task_publish!.subtitle}`);
      ctx.expect(/ready to execute/.test(objs.task_prepare!.subtitle ?? ""), `task_prepare reason: ${objs.task_prepare!.subtitle}`);
    });

    await ctx.step("UI: only the enabled item offers Execute (role-filtered, standing-driven)", async () => {
      const objs = await uiObjects(b);
      const intents = (id: string) => objs[id]!.actions.map((a) => a.intent);
      ctx.expect(intents("task_prepare").includes("EXECUTE_ITEM"), `task_prepare offers ${intents("task_prepare")}`);
      ctx.expect(!intents("task_publish").includes("EXECUTE_ITEM"), `task_publish offers ${intents("task_publish")}`);
      ctx.expect(!intents("ms_chain_accepted").includes("EXECUTE_ITEM"), "the milestone is never executed by hand");
    });

    await ctx.step("a real pointer click focuses the item and the outline lists it with its standing and Execute", async () => {
      await clickNode(ui, "task_prepare");
      await b.waitFor(`window.__gs.store.getState().focusStack.at(-1) === 'task_prepare'`, 15000, "task_prepare focused");
      const text = await jsEval<string>(b, `document.querySelector('[data-node="task_prepare"]').innerText`);
      ctx.expect(/prepare dataset/.test(text) && /ready to execute/.test(text), `focused node text: ${text.replace(/\n/g, " | ")}`);
      await ctx.shot("focused-ready", b);
      await openOutlineAtHome(b);
      const rows = await jsEval<{ id: string; text: string; actions: string[] }[]>(
        b,
        `[...document.querySelectorAll('[data-testid="outline-item"]')].map((e) => ({ id: e.dataset.id, text: e.innerText, actions: [...e.querySelectorAll('[data-testid="outline-action"]')].map((a) => a.dataset.action) }))`,
      );
      const row = rows.find((r) => r.id === "task_prepare")!;
      ctx.expect(/Ready/.test(row.text) && row.actions.includes(ACT_EXEC), `outline row: ${row.text.replace(/\n/g, " | ")} actions ${row.actions}`);
      const blocked = rows.find((r) => r.id === "task_publish")!;
      ctx.expect(/Waiting/.test(blocked.text) && !blocked.actions.includes("act-execute_item-task_publish"), `blocked row: ${blocked.text.replace(/\n/g, " | ")} actions ${blocked.actions}`);
      await ctx.shot("outline", b);
      await closeOutline(b);
    });
  },
};
