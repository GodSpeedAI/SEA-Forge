import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { Browser } from "../browser";
import type { Ctx, Journey } from "../ladder";
import { click, waitVisible } from "../journeys/helpers";
import { kindsOf, readJsonl, readTrace, waitUntil } from "../live/durable";
import { strictSteps } from "./L9";
import { closeOutline, designCaseViaUi, ensureLoggedIn, jsEval, loginThroughUi, openOutlineAtHome, outlineAction, sleep, stackOf, uiCtx } from "./helpers";

/**
 * Console errors this journey is EXPECTED to provoke by cutting the network and killing the
 * processes under the page. Exact message text only, never a pattern. Anything else, in any session,
 * from the first journey to the last, fails the final strict step.
 */
export const EXPECTED_DEGRADED_ERRORS: readonly string[] = [];

const TEMPLATE = "e2e-sentry-chain@0.1.0";
const PREPARE = "task_prepare";
const PUBLISH = "task_publish";
const act = (item: string) => `act-execute_item-${item}`;

// In-page recorder: every connection-state change, every pill state/text and the live revision ids
// the store holds, so recovery can be judged on what the user SAW, not only on where it ended.
const RECORDER = `(() => {
  const log = { marker: Math.random().toString(36).slice(2), timeOrigin: performance.timeOrigin, conn: [], pill: [], revs: [] };
  window.__lr = log;
  const st = window.__gs.store; let lastConn = ''; let lastRevs = '';
  const snap = () => { const s = st.getState(); if (s.connection !== lastConn) { lastConn = s.connection; log.conn.push({ t: Date.now(), c: s.connection }); }
    const ids = s.history.revisions.map((r) => r.id); const k = ids.join('|'); if (k !== lastRevs) { lastRevs = k; log.revs = ids; } };
  st.subscribe(snap); snap();
  let lastPill = '';
  const pill = () => { const e = document.querySelector('[data-testid="execution-pill"]'); const p = e ? { state: e.dataset.state, text: e.innerText.trim(), visible: e.classList.contains('visible') } : {}; const k = JSON.stringify(p); if (k !== lastPill) { lastPill = k; log.pill.push({ t: Date.now(), ...p }); } };
  new MutationObserver(pill).observe(document.body, { subtree: true, childList: true, attributes: true, characterData: true }); pill();
  return log.marker;
})()`;

interface Rec {
  marker: string;
  timeOrigin: number;
  conn: { t: number; c: string }[];
  pill: { t: number; state?: string; text?: string; visible?: boolean }[];
  revs: string[];
}

const recorded = (b: Browser) => jsEval<Rec>(b, `window.__lr`);

/** Revision ids the UI holds must be unique and strictly increasing (cursors are ordered strings). */
export function revisionProblems(ids: string[]): string[] {
  const out: string[] = [];
  const seen = new Set<string>();
  ids.forEach((id, i) => {
    if (seen.has(id)) out.push(`duplicate revision ${id}`);
    seen.add(id);
    if (i > 0 && !(id > ids[i - 1]!)) out.push(`revision ${id} does not follow ${ids[i - 1]}`);
  });
  return out;
}

async function executeViaOutline(ctx: Ctx, b: Browser, item: string): Promise<void> {
  const ui = uiCtx(ctx, b);
  await openOutlineAtHome(b);
  await click(ui, outlineAction(item, act(item)));
  await waitVisible(ui, '[data-testid="judgment-panel"] [data-testid="judgment-option"]');
  await click(ui, `[data-testid="judgment-option"][data-option="${act(item)}"]`);
}

const settledDurably = (cell: string, caseId: string, item: string, since: number) =>
  waitUntil(
    `${item} settled and completed in ${caseId}`,
    () => {
      const t = readTrace(cell, caseId).slice(since);
      return t.some((e) => e.kind === "settlement_recorded" && e.plan_item_id === item) && t.some((e) => e.kind === "item_completed" && e.plan_item_id === item) ? t : false;
    },
    { timeoutMs: 90000, watch: [join(cell, "cases", caseId, "case-events.jsonl")] },
  );

function artifactOf(cell: string, caseId: string): { file: string; digest: string; bytes: Buffer } {
  const runs = join(cell, "cases", caseId, "runs");
  for (const run of readdirSync(runs)) {
    const rows = readJsonl(join(runs, run, "evidence.jsonl")).records as { kind: string; uri: string; sha256: string | null }[];
    const row = rows.find((r) => r.kind === "artifact" && r.sha256);
    if (row) {
      const file = join(runs, run, row.uri);
      return { file, digest: row.sha256!.replace(/^sha256:/, ""), bytes: readFileSync(file) };
    }
  }
  throw new Error(`no captured artifact under ${runs}`);
}

async function newCase(ctx: Ctx, b: Browser, label: string): Promise<string> {
  const params = { dataset_name: `${label}-${Date.now().toString(36)}`, dataset_label: label, max_rows: "3", out_dir: "work" };
  const made = await designCaseViaUi(ctx, b, TEMPLATE, params);
  return made.caseId;
}

/**
 * L-RECOV (plan T10): the product under failure. (a) a stored artifact corrupted on disk is refused
 * with a typed digest-mismatch card and never rendered; (d) the SSE stream dropping while another
 * user executes shows Reconnecting (never progress) and resumes from the cursor with no duplicate
 * revision; (c) the kernel killed (-9) and restarted recovers its cases from disk and the gateway
 * re-subscribes; (b) the gateway killed (-9) and restarted forgets sessions BY DESIGN (in-memory
 * store), so the user signs in again and sees exactly what the durable files say.
 */
export const LRECOV: Journey = {
  id: "L-RECOV",
  title: "Recovery: corrupt artifact, SSE drop, kernel and gateway kill -9",
  depends_on: ["L8"],
  settles: "the UI is honest and consistent with the durable files across a corrupt artifact, an SSE drop and kernel/gateway crashes",
  unlocks: [],
  async run(ctx) {
    const stack = stackOf(ctx);
    const cell = stack.cell;
    const b = await ensureLoggedIn(ctx, "operator");
    const ui = uiCtx(ctx, b);

    // ---- case R1: execute prepare, then corrupt its artifact -------------------------------------------------
    let r1 = "";
    await ctx.step("setup: design case R1 through the UI and execute its first item; the artifact is captured", async () => {
      r1 = await newCase(ctx, b, "r1");
      const before = readTrace(cell, r1).length;
      await executeViaOutline(ctx, b, PREPARE);
      await settledDurably(cell, r1, PREPARE, before);
      await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 60000, "pill settled");
      await closeOutline(b);
    });

    const art = () => artifactOf(cell, r1);
    await ctx.step("(a) corrupt artifact: bytes edited inside the cell -> typed digest-mismatch card, nothing rendered from the bad bytes; restored afterwards", async () => {
      const good = art();
      const marker = "CORRUPTED-BYTES-MUST-NEVER-RENDER";
      let restored = false;
      try {
        writeFileSync(good.file, Buffer.concat([good.bytes, Buffer.from(`\n${marker}\n`)]));
        await b.open(`${stack.base}/?case=${encodeURIComponent(r1)}`);
        await b.waitFor(`!!(window.__gs && window.__gs.ready) && window.__gs.worldView().snap.caseId === ${JSON.stringify(r1)}`, 30000, "case loaded");
        await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 30000, "pill settled on a fresh load");
        await click(ui, '[data-testid="execution-pill"] button');
        await waitVisible(ui, '[data-testid="execution-evidence"]');
        await click(ui, '[data-testid="execution-evidence"]');
        await b.waitFor(`!!document.querySelector('[data-testid="artifact-error"]') || !!document.querySelector('[data-testid="dock-digest"]')`, 30000, "dock resolved");
        const card = await jsEval<{ error: boolean; text: string; dock: string; renderers: number }>(
          b,
          `(() => { const e = document.querySelector('[data-testid="artifact-error"]'); const d = document.querySelector('[data-testid="artifact-dock"]');
            return { error: !!e, text: e ? e.innerText : '', dock: d ? d.innerText : '', renderers: document.querySelectorAll('[data-testid^="renderer-"]').length } })()`,
        );
        await ctx.shot("corrupt-artifact", b);
        ctx.expect(card.error, "the dock must show the artifact error card for corrupted stored bytes");
        ctx.expect(/digest/i.test(card.text) && /(does not match|no longer matches|mismatch)/i.test(card.text), `the card names a digest mismatch (${card.text.replace(/\s+/g, " ")})`);
        ctx.expect(card.renderers === 0 && !card.dock.includes(marker), "nothing is rendered from the corrupted bytes");
        const res = await jsEval<{ status: number; body: string }>(
          b,
          `fetch('/api/artifacts/' + encodeURIComponent('sha256:${good.digest}'), { credentials: 'include' }).then(async (r) => ({ status: r.status, body: await r.text() }))`,
        );
        ctx.expect(res.status !== 200 && !res.body.includes(marker), `the gateway refuses the corrupted artifact (HTTP ${res.status})`);
        ctx.expect(/integrity_mismatch/.test(res.body), `the refusal is typed integrity_mismatch, not a generic outage: ${res.body.slice(0, 160)}`);
        ctx.delta({ step: "corrupt-artifact", file: good.file.slice(cell.length), status: res.status, card: card.text.replace(/\s+/g, " ").slice(0, 200) });
      } finally {
        writeFileSync(good.file, good.bytes);
        restored = true;
      }
      ctx.expect(restored && art().bytes.equals(good.bytes), "the artifact bytes are restored");
      await b.open(`${stack.base}/?case=${encodeURIComponent(r1)}`);
      await b.waitFor(`!!(window.__gs && window.__gs.ready) && document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 30000, "reloaded");
      await click(ui, '[data-testid="execution-pill"] button');
      await waitVisible(ui, '[data-testid="execution-evidence"]');
      await click(ui, '[data-testid="execution-evidence"]');
      await b.waitFor(`!!document.querySelector('[data-testid="dock-digest"]')`, 30000, "restored artifact opens");
      ctx.expect((await jsEval<string>(b, `document.querySelector('[data-testid="dock-digest"]').dataset.digest`)) === `sha256:${good.digest}`, "after restore the dock shows the verified digest again");
      await b.press("Escape");
    });

    // ---- (d) SSE drop while another user executes ----------------------------------------------------------
    await ctx.step("(d) SSE drop: while the stream is down another session executes; the pill says Reconnecting (no progress), then resumes from the cursor with no duplicate revision", async () => {
      // The page is served through a severable pass-through, so the stream can be cut at the network
      // (every open SSE connection drops) while the gateway, its sessions and the kernel stay up.
      await stack.startProxy();
      await b.open(`${stack.proxyBase}/?case=${encodeURIComponent(r1)}`);
      await b.waitFor(`!!(window.__gs && window.__gs.ready) && window.__gs.worldView().snap.caseId === ${JSON.stringify(r1)}`, 30000, "case loaded");
      await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 30000, "pill settled");
      const marker = await b.eval<string>(RECORDER);
      const revsBefore = (await recorded(b)).revs.length;
      await stack.killProxy();
      try {
        await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'reconnecting' || document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'interrupted'`, 40000, "pill shows Reconnecting while the stream is down");
        const mid = await recorded(b);
        ctx.expect(!mid.pill.some((p) => /%/.test(p.text ?? "")), `no percentage while reconnecting: ${JSON.stringify(mid.pill.map((p) => p.text))}`);
        await ctx.shot("sse-reconnecting", b);

        // Another user session executes the downstream item while this one is cut off.
        const other = ctx.session("operator-b");
        await loginThroughUi(other, stack.base, "operator");
        await other.open(`${stack.base}/?case=${encodeURIComponent(r1)}`);
        await other.waitFor(`!!(window.__gs && window.__gs.ready) && window.__gs.worldView().snap.caseId === ${JSON.stringify(r1)}`, 30000, "second session case loaded");
        const before = readTrace(cell, r1).length;
        await executeViaOutline(ctx, other, PUBLISH);
        await settledDurably(cell, r1, PUBLISH, before);
        const stale = await jsEval<string>(b, `document.querySelector('[data-testid="execution-pill"]')?.dataset.state`);
        ctx.expect(stale === "reconnecting" || stale === "interrupted", `while cut off the pill still reports the connection, not the new run (${stale})`);
      } finally {
        await stack.startProxy();
      }
      await b.waitFor(`window.__gs.store.getState().connection === 'live'`, 60000, "stream resumed");
      await b.waitFor(`window.__gs.worldView().snap.objects[${JSON.stringify(PUBLISH)}]?.status?.label === 'Settled accepted'`, 60000, "missed revision arrived after resume");
      const after = await recorded(b);
      ctx.expect(after.marker === marker && after.timeOrigin === (await jsEval<number>(b, `performance.timeOrigin`)), "no page reload: the same page recovered by itself");
      ctx.expect(after.conn.some((c) => c.c === "reconnecting" || c.c === "interrupted") && after.conn.at(-1)!.c === "live", `connection went down and came back: ${JSON.stringify(after.conn.map((c) => c.c))}`);
      ctx.expect(after.revs.length > revsBefore, `revisions grew after resume (${revsBefore} -> ${after.revs.length})`);
      const problems = revisionProblems(after.revs);
      ctx.expect(problems.length === 0, `revision ids ${JSON.stringify(after.revs)}: ${problems.join("; ")}`);
      ctx.expect(!after.pill.some((p) => /%/.test(p.text ?? "") || p.state === "running"), `the pill never showed progress: ${JSON.stringify(after.pill.map((p) => `${p.state}:${p.text}`))}`);
      const t = readTrace(cell, r1).filter((e) => e.kind === "settlement_recorded" && e.plan_item_id === PUBLISH);
      ctx.expect(t.length === 1, "the durable files hold exactly one settlement for the downstream item");
      ctx.delta({ step: "sse-drop-resume", conn: after.conn, revisions: after.revs, pill: after.pill.map((p) => `${p.state}:${p.text}`) });
      await ctx.shot("sse-resumed", b);
    });

    // ---- (c) kernel kill -9 and restart ---------------------------------------------------------------------
    let r2 = "";
    await ctx.step("(c) kernel kill -9 and restart: cases recover from disk, the gateway re-subscribes, the next execution reaches the UI over SSE", async () => {
      await b.open(`${stack.base}/`);
      await b.waitFor(`!!(window.__gs && window.__gs.ready)`, 30000, "app ready");
      r2 = await newCase(ctx, b, "r2");
      const durable = readTrace(cell, r2);
      const casesBefore = readdirSync(join(cell, "cases")).sort();
      await b.eval(RECORDER);
      await stack.killKernel("SIGKILL");
      await sleep(1500);
      ctx.expect(JSON.stringify(readTrace(cell, r2)) === JSON.stringify(durable), "the crash changed nothing already durable");
      await stack.startKernel();
      ctx.expect(JSON.stringify(readdirSync(join(cell, "cases")).sort()) === JSON.stringify(casesBefore), "the restarted kernel sees the same cases on disk");
      // The same open page must still work: execute through the UI, and the result arrives by SSE.
      const before = readTrace(cell, r2).length;
      await executeViaOutline(ctx, b, PREPARE);
      await settledDurably(cell, r2, PREPARE, before);
      await b.waitFor(`window.__gs.worldView().snap.objects[${JSON.stringify(PREPARE)}]?.status?.label === 'Settled accepted'`, 60000, "UI follows the recovered kernel over SSE");
      await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 60000, "pill settled");
      const rec = await recorded(b);
      ctx.expect(rec.timeOrigin === (await jsEval<number>(b, `performance.timeOrigin`)), "no page reload was needed");
      const problems = revisionProblems(rec.revs);
      ctx.expect(problems.length === 0, `revision ids ${JSON.stringify(rec.revs)}: ${problems.join("; ")}`);
      ctx.expect(kindsOf(readTrace(cell, r2)).filter((k) => k === "case_created").length === 1, "recovery did not re-create the case");
      ctx.delta({ step: "kernel-kill", conn: rec.conn, revisions: rec.revs });
      await closeOutline(b);
    });

    // ---- (b) gateway kill -9 and restart ---------------------------------------------------------------------
    await ctx.step("(b) gateway kill -9 and restart: the UI shows Reconnecting, sessions are gone by design, a fresh sign-in shows exactly the durable state with no duplicate revision", async () => {
      await b.eval(RECORDER);
      await stack.killGateway("SIGKILL");
      await b.waitFor(`['reconnecting', 'interrupted'].includes(window.__gs.store.getState().connection)`, 60000, "UI notices the stream is gone");
      const down = await recorded(b);
      ctx.expect(!down.pill.some((p) => /%/.test(p.text ?? "")), "no fake progress while the gateway is down");
      await ctx.shot("gateway-down", b);
      const trace = readTrace(cell, r2);
      await stack.startGateway();
      // The session store is in-memory by design: the old cookie no longer names a session.
      const session = await jsEval<{ authenticated: boolean }>(b, `fetch('/api/session', { credentials: 'include' }).then((r) => r.json())`);
      ctx.expect(session.authenticated === false, "a restarted gateway does not know the old session");
      await loginThroughUi(b, stack.base, "operator");
      await b.open(`${stack.base}/?case=${encodeURIComponent(r2)}`);
      await b.waitFor(`!!(window.__gs && window.__gs.ready) && window.__gs.worldView().snap.caseId === ${JSON.stringify(r2)}`, 30000, "case loaded after sign-in");
      await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 30000, "pill settled from the durable state");
      const ui2 = await jsEval<Record<string, string>>(b, `Object.fromEntries(Object.values(window.__gs.worldView().snap.objects).filter((o) => ['${PREPARE}', '${PUBLISH}'].includes(o.id)).map((o) => [o.id, o.status && o.status.label]))`);
      ctx.expect(ui2[PREPARE] === "Settled accepted" && ui2[PUBLISH] === "Ready", `UI standing after recovery ${JSON.stringify(ui2)} matches the durable trace (${kindsOf(trace).join(",")})`);
      ctx.expect(JSON.stringify(readTrace(cell, r2)) === JSON.stringify(trace), "the gateway crash changed nothing durable");
      // And the recovered stack still works end to end, with the stream carrying the result.
      await b.eval(RECORDER);
      const before = readTrace(cell, r2).length;
      await executeViaOutline(ctx, b, PUBLISH);
      await settledDurably(cell, r2, PUBLISH, before);
      await b.waitFor(`window.__gs.worldView().snap.objects[${JSON.stringify(PUBLISH)}]?.status?.label === 'Settled accepted'`, 60000, "UI follows the recovered gateway over SSE");
      const rec = await recorded(b);
      const problems = revisionProblems(rec.revs);
      ctx.expect(problems.length === 0, `revision ids ${JSON.stringify(rec.revs)}: ${problems.join("; ")}`);
      ctx.delta({ step: "gateway-kill", revisions: rec.revs, standing: ui2 });
      await closeOutline(b);
    });

    // L9 ran before this journey, so its whole-run check cannot see the crashes provoked here. Same
    // strict standard again, now covering L0..L-RECOV in full (harvested + live), with only the exact
    // messages in EXPECTED_DEGRADED_ERRORS allowed.
    await strictSteps(ctx, [["operator", ctx.session("operator")], ["rso", ctx.session("rso")], ["operator-b", ctx.session("operator-b")]], EXPECTED_DEGRADED_ERRORS, "strict-final");
  },
};
