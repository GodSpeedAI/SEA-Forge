import type { Browser } from "../browser";
import type { Journey } from "../ladder";
import { delegatedAsks, readAskLedger, waitUntil } from "../live/durable";
import { ensureLoggedIn, jsEval, sleep, stackOf, W, H } from "./helpers";

interface NarrState {
  id: string;
  index: number;
  status: string;
  error?: string;
}

const narr = (b: Browser) => jsEval<NarrState | null>(b, `(() => { const n = window.__gs.store.getState().narrative; return n ? { id: n.id, index: n.index, status: n.status, error: n.error } : null })()`);

async function submit(b: Browser, text: string): Promise<void> {
  await b.eval<boolean>(`(document.activeElement && document.activeElement.blur && document.activeElement.blur(), true)`);
  await b.click(".composer-input");
  await b.fill(".composer-input", text);
  await b.press("Enter");
}

/** The ask ledger entries that belong to one answer (the kernel threads the answer id through the chain). */
function entriesOf(cell: string, answerId: string) {
  return readAskLedger(cell).filter((e) => JSON.stringify(e).includes(answerId));
}

/**
 * L8: Thoth Ask narration (spec 04 section 8.1) with interrupt and resume. The composer takes a
 * typed question; the gateway forwards it to the kernel as the signed-in user; the kernel commits
 * the question and the disclosure to ledgers/thoth-asks; the UI narrates only what came back.
 * Interrupt (Space) pauses at a checkpoint; "continue" resumes from it. Neither may ask again.
 */
export function makeL8(opts: { id?: string; depends_on?: string[]; beatPace?: number } = {}): Journey {
  return {
    id: opts.id ?? "L8",
    title: "Thoth ask narration: interrupt and resume",
    depends_on: opts.depends_on ?? ["L7"],
    settles: "a typed question is answered by the real kernel, narrated from the returned disclosure only, interruptible and resumable without a second ask",
    unlocks: ["L9"],
    async run(ctx) {
      const stack = stackOf(ctx);
      const cell = stack.cell;
      const b = await ensureLoggedIn(ctx, "operator");
      const pace = opts.beatPace ?? 6;
      await b.viewport(W, H);
      await b.open(`${stack.base}/?beatPace=${pace}`);
      await b.waitFor(`!!(window.__gs && window.__gs.ready) && !!document.querySelector('[data-testid="session-badge"][data-state="signed-in"]')`, 30000, "signed-in app");

      // Thoth answers from the self-model snapshot, which the operator rebuilds (see stack.ts).
      const askSubject = stack.seedSelfModel();
      // Observation, not an assertion: on a registry-less cell the kernel discloses the rebuilt snapshot as stale.
      ctx.delta({ step: "readiness-after-self-model-rebuild", readyz: await (await fetch(`${stack.base}/api/readyz`)).json() });
      const baseLedger = readAskLedger(cell).length;
      const baseAsks = delegatedAsks(cell).length;
      let answerId = "";
      let narrativeId = "";
      let claimCount = 0;
      let pausedAt = -1;

      await ctx.step("the live app offers narration (agent available) and is not the scripted local agent", async () => {
        const agent = await jsEval<string>(b, `window.__gs.store.getState().agent`);
        ctx.expect(agent === "available", `agent availability is ${agent}`);
      });

      await ctx.step("ask a typed question; the real kernel answers and the disclosure card shows the returned fields", async () => {
        await submit(b, `ask capability ${askSubject}`);
        await b.waitFor(`!!document.querySelector('[data-testid="grounded-answer"]')`, 60000, "grounded disclosure card");
        const card = await jsEval<{ id: string; disposition: string; freshness: string; assurance: string; notice: string; claims: string[]; refs: string[] }>(
          b,
          `(() => { const c = document.querySelector('[data-testid="grounded-answer"]'); const t = (s) => c.querySelector(s)?.textContent ?? ''; return {
            id: c.dataset.answerId, disposition: c.dataset.disposition, freshness: c.dataset.freshness, assurance: t('[data-testid=grounded-assurance]'),
            notice: t('[data-testid=grounded-notice]'), claims: [...c.querySelectorAll('[data-testid=grounded-claim] p')].map((p) => p.textContent),
            refs: [...c.querySelectorAll('[data-testid=grounded-ref]')].map((r) => r.textContent) } })()`,
        );
        answerId = card.id;
        claimCount = card.claims.length;
        ctx.expect(card.disposition === "answered", `disposition ${card.disposition} (the e2e cell grants operator_local self-disclosure)`);
        ctx.expect(claimCount > 0, "an answered disclosure carries claims");
        ctx.expect(/no execution authority/i.test(card.notice), `authority notice shown verbatim (${card.notice})`);
        ctx.expect(card.freshness === "current" || card.freshness === "stale", `freshness ${card.freshness}`);
        ctx.expect(card.assurance.length > 0, "assurance shown");
        const st = await narr(b);
        ctx.expect(!!st && st.status === "playing", `narration is playing (${JSON.stringify(st)})`);
        narrativeId = st!.id;
        const beats = await jsEval<{ caption: string; citations: string[]; grounded: string | null }[]>(
          b,
          `window.__gs.store.getState().narratives[${JSON.stringify(narrativeId)}].beats.map((x) => ({ caption: x.caption, citations: x.citations || [], grounded: x.grounded ? x.grounded.answer_id : null }))`,
        );
        ctx.expect(beats.length === claimCount + 1, `${claimCount} claim beats plus the terms beat (got ${beats.length})`);
        for (let i = 0; i < claimCount; i++) ctx.expect(beats[i]!.caption === card.claims[i], `beat ${i} is the returned claim statement verbatim`);
        ctx.expect(beats.every((x) => x.grounded === answerId), "every beat is grounded in the one returned answer");
        const cited = beats.flatMap((x) => x.citations);
        ctx.expect(cited.every((r) => card.refs.includes(r)), `citations ${JSON.stringify(cited)} are only refs the answer returned`);
        await ctx.shot("ask-narration", b);
      });

      await ctx.step("durable: the kernel committed the question and disclosure to the thoth-asks ledger as operator_local, via a delegated ask", async () => {
        const entry = await waitUntil(
          "thoth-asks ledger entries for this answer",
          () => (entriesOf(cell, answerId).length >= 1 ? entriesOf(cell, answerId) : undefined),
          { timeoutMs: 15000 },
        );
        const all = readAskLedger(cell);
        ctx.expect(all.length === baseLedger + 4, `one Ask commits a 4-entry chain: ${baseLedger} -> ${all.length}`);
        const fresh = all.slice(baseLedger);
        ctx.expect(fresh.every((e) => e.writer_identity_ref === "operator_local"), `ledger writer ${fresh.map((e) => e.writer_identity_ref).join(",")}`);
        ctx.expect(entry.length >= 1, "the answer id the UI shows is in the ledger");
        const asks = delegatedAsks(cell);
        ctx.expect(asks.length === baseAsks + 1, `one delegated ask recorded (${baseAsks} -> ${asks.length})`);
        const last = asks[asks.length - 1]!;
        ctx.expect(last.actor === "operator_local" && last.role === "operator" && last.gateway === "gateway", `delegation ${JSON.stringify(last)}`);
        ctx.delta({ step: "ask", answerId, ledgerEntries: all.length - baseLedger, delegation: last });
      });

      await ctx.step("interrupt: Space pauses at a checkpoint and the narration does not advance", async () => {
        await b.eval<boolean>(`(document.activeElement && document.activeElement.blur && document.activeElement.blur(), true)`);
        await b.press("Space");
        await b.waitFor(`window.__gs.store.getState().narrative?.status === 'paused'`, 10000, "paused");
        const st = (await narr(b))!;
        pausedAt = st.index;
        ctx.expect(st.id === narrativeId && !st.error, `paused the same narration (${JSON.stringify(st)})`);
        ctx.expect(pausedAt < claimCount, `interrupted before the last beat (index ${pausedAt} of ${claimCount + 1})`);
        await sleep(2500);
        ctx.expect((await narr(b))!.index === pausedAt, "the narration does not advance while paused");
        ctx.expect(await jsEval<boolean>(b, `!!document.querySelector('[data-testid="grounded-answer"][data-paused="true"]')`), "the disclosure stays on screen while paused");
        const ph = await jsEval<string>(b, `document.querySelector('.composer-input').getAttribute('placeholder') ?? ''`);
        ctx.expect(/continue/i.test(ph), `the composer offers to continue (${ph})`);
        await ctx.shot("ask-interrupted", b);
      });

      await ctx.step("resume: continue picks up from the checkpoint and finishes without asking again", async () => {
        await submit(b, "continue");
        await b.waitFor(`['playing', 'done'].includes(window.__gs.store.getState().narrative?.status ?? 'done') || !window.__gs.store.getState().narrative`, 15000, "resumed");
        const mid = await narr(b);
        ctx.expect(!mid || mid.index >= pausedAt, `resumed at ${mid?.index}, not restarted (paused at ${pausedAt})`);
        await b.waitFor(`!window.__gs.store.getState().narrative || window.__gs.store.getState().narrative.status === 'done'`, 120000, "narration finished");
        const n = await jsEval<{ complete: boolean; beats: number }>(b, `(() => { const n = window.__gs.store.getState().narratives[${JSON.stringify(narrativeId)}]; return { complete: n.complete, beats: n.beats.length } })()`);
        ctx.expect(n.complete && n.beats === claimCount + 1, `the same explanation completed (${JSON.stringify(n)})`);
        const all = readAskLedger(cell);
        ctx.expect(all.length === baseLedger + 4, `interrupt/resume made no second ask: ledger ${all.length}, expected ${baseLedger + 4}`);
        ctx.expect(delegatedAsks(cell).length === baseAsks + 1, "no second delegated ask");
      });

      await ctx.step("a free-text question is answered honestly as unsupported and asks nothing", async () => {
        await submit(b, "Why did the pilot fail?");
        await b.waitFor(`(() => { const n = window.__gs.store.getState().narrative; if (!n) return false; const x = window.__gs.store.getState().narratives[n.id]; return n.id !== ${JSON.stringify(narrativeId)} && x.beats.length === 1 })()`, 20000, "unsupported narration");
        const cap = await jsEval<string>(b, `(() => { const s = window.__gs.store.getState(); return s.narratives[s.narrative.id].beats[0].caption })()`);
        ctx.expect(/typed questions/i.test(cap) && !/pilot/.test(cap), `honest unsupported copy (${cap})`);
        ctx.expect(!(await jsEval<boolean>(b, `!!document.querySelector('[data-testid="grounded-answer"]')`)), "no disclosure card for an unsupported question");
        await sleep(1000);
        ctx.expect(readAskLedger(cell).length === baseLedger + 4, "an unsupported question wrote nothing to the ledger");
        await b.press("Escape");
      });

      await ctx.step("a different principal (R-SO, separate session) gets the kernel's governed denial, recorded under rso_local", async () => {
        const rso = await ensureLoggedIn(ctx, "rso");
        await rso.open(`${stack.base}/?beatPace=${pace}`);
        await rso.waitFor(`!!(window.__gs && window.__gs.ready) && !!document.querySelector('[data-testid="session-badge"][data-state="signed-in"]')`, 30000, "rso app");
        await submit(rso, `ask capability ${askSubject}`);
        await rso.waitFor(`!!document.querySelector('[data-testid="grounded-answer"]')`, 60000, "rso disclosure card");
        const got = await jsEval<{ disposition: string; id: string; claims: number; omitted: string }>(
          rso,
          `(() => { const c = document.querySelector('[data-testid="grounded-answer"]'); return { disposition: c.dataset.disposition, id: c.dataset.answerId, claims: c.querySelectorAll('[data-testid=grounded-claim]').length, omitted: c.querySelector('[data-testid=grounded-omitted]')?.textContent ?? '' } })()`,
        );
        ctx.expect(got.disposition === "denied" && got.claims === 0, `rso (no self-disclosure grant) is denied: ${JSON.stringify(got)}`);
        ctx.expect(/Withheld by policy/.test(got.omitted), "the withheld claim classes are shown");
        const all = readAskLedger(cell);
        ctx.expect(all.length === baseLedger + 8, `second Ask chain committed: ${all.length}`);
        ctx.expect(all.slice(baseLedger + 4).every((e) => e.writer_identity_ref === "rso_local"), "the denial is attributed to rso_local, not to the operator or the gateway");
        const asks = delegatedAsks(cell);
        ctx.expect(asks.length === baseAsks + 2 && asks[asks.length - 1]!.actor === "rso_local", `delegation ${JSON.stringify(asks.slice(-2))}`);
        await rso.press("Escape");
        await ctx.shot("ask-denied", rso);
      });
    },
  };
}

export const L8 = makeL8();
