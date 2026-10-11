import type { Journey } from "../ladder";
import { caseEventsFile, kindsOf, listCases, waitForNewCase } from "../live/durable";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { ensureLoggedIn, jsEval, stackOf } from "./helpers";

const TEMPLATE = "e2e-sentry-chain@0.1.0";

// L1 template -> parameters -> preflight (digest) -> commit. The durable delta is read from the
// kernel's own files: cases/<id>/case-events.jsonl must gain case_created (and the template's
// first item enabled) and cases/<id>/plan.json must hold the template's plan. NOTE on
// PlanCreated: the SFWP case.commit path records the plan as plan.json and never appends a
// plan_created trace (that TraceKind belongs to the CLI pipeline; see
// crates/sea-forge-server/tests/case_templates_live.rs), so the plan assertion reads plan.json.
export const L1: Journey = {
  id: "L1",
  title: "Template to preflight to commit",
  depends_on: ["L0"],
  settles: "a case exists in the kernel because the UI committed it",
  unlocks: ["L2"],
  async run(ctx) {
    const stack = stackOf(ctx);
    const cell = stack.cell;
    const b = await ensureLoggedIn(ctx, "operator");
    const before = listCases(cell);
    const param = (name: string) => `[data-testid="proposal-param"][data-name="${name}"]`;
    const submitDisabled = () => jsEval<boolean>(b, `document.querySelector('[data-testid="proposal-submit"]').disabled`);

    await ctx.step("home offers Design case; the picker lists the published templates", async () => {
      await b.waitFor(`!!document.querySelector('[data-testid="center-design"]')`, 15000, "Design case entry on empty home");
      await b.click('[data-testid="center-design"]');
      await b.waitFor(`document.querySelectorAll('[data-testid="proposal-template"]').length >= 2`, 15000, "templates listed");
      const refs = await jsEval<string[]>(b, `[...document.querySelectorAll('[data-testid="proposal-template"]')].map((e) => e.dataset.ref)`);
      ctx.expect(refs.includes(TEMPLATE) && refs.includes("e2e-signoff-gate@0.1.0"), `templates: ${refs.join(", ")}`);
      await ctx.shot("picker", b);
    });

    const params = { dataset_name: `l1-${Date.now().toString(36)}`, dataset_label: "L1 live", max_rows: "3", out_dir: "work" };
    await ctx.step("select template, fill parameters; submit stays disabled before preflight", async () => {
      await b.click(`[data-testid="proposal-template"][data-ref="${TEMPLATE}"]`);
      await b.waitFor(`!!document.querySelector('[data-testid="proposal-param"]')`, 10000, "parameters rendered");
      for (const [k, v] of Object.entries(params)) await b.fill(param(k), v);
      ctx.expect(await submitDisabled(), "submit is disabled before any preflight");
      await ctx.shot("parameters", b);
    });

    let digest = "";
    await ctx.step("preflight passes with a digest; only then is submit enabled", async () => {
      await b.click('[data-testid="proposal-preflight"]');
      await b.waitFor(`document.querySelector('[data-testid="proposal-preflight-result"]')?.dataset.passed === 'true'`, 20000, "preflight passed");
      digest = await jsEval<string>(b, `document.querySelector('[data-testid="proposal-digest"]').innerText`);
      ctx.expect(/^sha256:[0-9a-f]{64}$/.test(digest), `digest ${digest}`);
      ctx.expect(!(await submitDisabled()), "submit enabled after a passing preflight");
      ctx.delta({ step: "preflight", digest });
    });

    await ctx.step("editing a parameter invalidates the preflight and disables submit again", async () => {
      await b.fill(param("max_rows"), "4");
      await b.waitFor(`!document.querySelector('[data-testid="proposal-preflight-result"]')`, 5000, "preflight cleared");
      ctx.expect(await submitDisabled(), "submit disabled after editing a parameter");
      await b.fill(param("max_rows"), "3");
      await b.click('[data-testid="proposal-preflight"]');
      await b.waitFor(`document.querySelector('[data-testid="proposal-preflight-result"]')?.dataset.passed === 'true'`, 20000, "preflight passed again");
      ctx.expect(!(await submitDisabled()), "submit enabled after re-running preflight");
      digest = await jsEval<string>(b, `document.querySelector('[data-testid="proposal-digest"]').innerText`);
    });

    let caseId = "";
    await ctx.step("durable: commit writes case_created + item_enabled to the new case's case-events.jsonl and its plan.json", async () => {
      await b.click('[data-testid="proposal-submit"]');
      // Durable delta FIRST: the UI closing the panel is not evidence that the kernel wrote anything.
      const found = await waitForNewCase(cell, before, ["case_created", "item_enabled"], { timeoutMs: 30000 });
      caseId = found.caseId;
      const kinds = kindsOf(found.trace);
      ctx.expect(kinds[0] === "case_created", `first trace kind is ${kinds[0]}`);
      const planFile = join(cell, "cases", caseId, "plan.json");
      ctx.expect(existsSync(planFile), `${planFile} exists`);
      const plan = JSON.parse(readFileSync(planFile, "utf8")) as { items: { plan_item_id: string; operations: { content_hint?: string }[] }[] };
      const ids = plan.items.map((i) => i.plan_item_id);
      ctx.expect(ids.includes("task_prepare") && ids.includes("task_publish"), `plan items ${ids.join(", ")}`);
      const hint = plan.items.find((i) => i.plan_item_id === "task_prepare")?.operations[0]?.content_hint ?? "";
      ctx.expect(hint.includes(params.dataset_name), `plan carries the committed parameters (${hint})`);
      ctx.shared.caseA = caseId;
      ctx.shared.caseAParams = params;
      ctx.delta({ step: "commit", case_id: caseId, events_file: caseEventsFile(cell, caseId), kinds, plan_items: ids, digest });
    });

    await ctx.step("UI: the panel closes and the new case's world is the one shown", async () => {
      await b.waitFor(`!document.querySelector('[data-testid="proposal-panel"]')`, 15000, "panel closed");
      await b.waitFor(`window.__gs.worldView().snap.caseId === ${JSON.stringify(caseId)}`, 15000, "world shows the new case");
      const titles = await jsEval<string[]>(b, `Object.values(window.__gs.worldView().snap.objects).map((o) => o.id)`);
      ctx.expect(titles.includes("task_prepare") && titles.includes("task_publish"), `world objects ${titles.join(", ")}`);
      await ctx.shot("new-case", b);
    });
  },
};
