import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { Browser } from "../browser";
import type { Ctx, Journey } from "../ladder";
import { kindsOf, readAskLedger, readTrace } from "../live/durable";
import { digestOf, sessionCookieValue } from "../live/sessions";
import { caseAOf, sessionOf, sleep, stackOf } from "./helpers";

/** Console levels that are errors. (agent-browser reports error | warning | log | info | debug ...) */
const isError = (level: string) => level === "error" || level === "assert";

interface Finding {
  session: string;
  where: "console" | "pageerror";
  text: string;
}

/** All error-level console output and uncaught page errors the session has produced, ever. */
export async function strictFindings(name: string, b: Browser): Promise<{ findings: Finding[]; entries: { level: string; text: string }[] }> {
  const entries = await b.consoleEntries();
  const findings: Finding[] = entries.filter((e) => isError(e.level)).map((e) => ({ session: name, where: "console" as const, text: e.text }));
  for (const e of await b.pageErrors()) findings.push({ session: name, where: "pageerror", text: e.text });
  return { findings, entries };
}

// L9: L0-L8 as ONE continuous journey. Every journey from L0 to L8 ran in the same two named
// agent-browser sessions (operator, and rso for the second principal): one login each, one case
// lifecycle, no fresh browser in between. This journey proves that continuity from outside (the
// operator's session cookie is the one L0's real sign-in minted; the gateway logged exactly two
// sign-ins; the durable files hold the whole lifecycle) and then applies the strict standard to the
// whole session: `agent-browser console` and `agent-browser errors` hold no error-level output and
// no uncaught page error, from the first page load to now. Nothing is filtered: a real console
// error is a product defect to fix.
export const L9: Journey = {
  id: "L9",
  title: "L0-L8 in one session: console and errors stay empty",
  depends_on: ["L8"],
  settles: "the whole live ladder ran in one continuous session per user with no error-level console output and no page error",
  unlocks: [],
  async run(ctx) {
    const stack = stackOf(ctx);
    const cell = stack.cell;
    const op = ctx.session("operator");
    const rso = ctx.session("rso");

    await ctx.step("continuity: the operator session is the L0 sign-in's (same cookie), the gateway logged exactly one sign-in per user", async () => {
      const now = digestOf(sessionCookieValue(await op.cookiesGet()));
      ctx.expect(!!ctx.shared.operatorCookieDigest && now === ctx.shared.operatorCookieDigest, "the operator's session cookie changed since L0: a journey signed in again or the session was replaced");
      ctx.expect((await sessionOf(op)).actor_id === "operator_local", "operator session identity");
      ctx.expect((await sessionOf(rso)).actor_id === "rso_local", "rso session identity");
      const log = readFileSync(join(stack.root, "logs", "gateway.log"), "utf8");
      const logins = log.split("\n").filter((l) => /method="POST" path="\/api\/auth\/login" status=200/.test(l)).length;
      ctx.expect(logins === 2, `the gateway logged ${logins} successful sign-ins; the continuous ladder signs each of the two users in once`);
      ctx.delta({ step: "continuity", sign_ins: logins, operator_session: op.sessionName, rso_session: rso.sessionName });
    });

    await ctx.step("durable: the one lifecycle L0-L8 drove is all in the kernel's files (create, mutate, execute, settle, sign-off, close, reopen, terminate, ask)", async () => {
      const kinds = new Set(kindsOf(readTrace(cell, caseAOf(ctx))));
      for (const k of ["case_created", "plan_mutated", "item_activated", "settlement_recorded", "item_completed", "item_enabled", "case_closed", "case_reopened"]) {
        ctx.expect(kinds.has(k), `case A trace lacks ${k}: ${[...kinds].join(",")}`);
      }
      const caseB = ctx.shared.caseB as string | undefined;
      ctx.expect(!!caseB && kindsOf(readTrace(cell, caseB)).includes("case_terminated"), "case B was terminated");
      ctx.expect(readAskLedger(cell).length >= 4, "the Thoth ask is in the ledger");
    });

    // Positive control: prove the console capture sees THIS app's pages before trusting "no errors".
    // Cutting the network makes the product log its own `[event stream]` warning; it must show up.
    await ctx.step("positive control: the operator session's console capture sees the app's own warning when its stream drops", async () => {
      const before = (await op.consoleEntries()).filter((e) => /\[event stream\]/.test(e.text)).length;
      await stack.startProxy();
      try {
        await op.open(`${stack.proxyBase}/?case=${encodeURIComponent(caseAOf(ctx))}`);
        await op.waitFor(`!!(window.__gs && window.__gs.ready) && window.__gs.store.getState().connection === 'live'`, 30000, "app live through the proxy");
        await stack.killProxy();
        await op.waitFor(`window.__gs.store.getState().connection !== 'live'`, 40000, "stream drop noticed");
        await stack.startProxy();
        // Nothing happened while the stream was down: recovery must still be shown, from the read that proves the source.
        await op.waitFor(`window.__gs.store.getState().connection === 'live'`, 60000, "stream resumed with nothing missed");
      } finally {
        await stack.killProxy();
      }
      const after = (await op.consoleEntries()).filter((e) => /\[event stream\]/.test(e.text));
      ctx.expect(after.length > before, `the capture saw no '[event stream]' warning from the app (${before} -> ${after.length}): the strict check below would be vacuous`);
      ctx.delta({ step: "console-positive-control", warnings_seen: after.length - before, level: after[0]?.level });
    });

    await strictSteps(ctx, [["operator", op], ["rso", rso]]);
  },
};

async function strictSteps(ctx: Ctx, sessions: (readonly [string, Browser])[]): Promise<void> {
  const outDir = join(ctx.out, "console");
  mkdirSync(outDir, { recursive: true });
  for (const [name, b] of sessions) {
    await ctx.step(`strict: ${name} session, agent-browser console and errors are empty of errors for the whole run`, async () => {
      const { findings, entries } = await strictFindings(name, b);
      writeFileSync(join(outDir, `${name}-console.json`), JSON.stringify({ session: b.sessionName, entries, page_errors: findings.filter((f) => f.where === "pageerror") }, null, 2));
      ctx.delta({ step: `strict-${name}`, console_entries: entries.length, levels: Object.fromEntries([...new Set(entries.map((e) => e.level))].map((l) => [l, entries.filter((e) => e.level === l).length])), errors: findings.length });
      ctx.expect(findings.length === 0, `${findings.length} error(s) in the ${name} session: ${findings.map((f) => `[${f.where}] ${f.text}`).join(" | ").slice(0, 1500)}`);
    });
  }
}

/**
 * Tooth (L9): the strict check must really bite. A page that logs one console error and throws one
 * uncaught error MUST fail the strict step with exactly those messages. (The product's own page
 * cannot be used to inject: its CSP forbids inline script, and console calls made from
 * agent-browser's evaluation are not captured, so a local file page stands in for a misbehaving app.)
 */
export const L9Tooth: Journey = {
  id: "L9",
  title: "Tooth: the strict console check fails on an injected console error",
  depends_on: [],
  settles: "an injected console.error and uncaught error are caught by the strict check",
  unlocks: [],
  async run(ctx) {
    const b = ctx.session("tooth");
    const dir = mkdtempSync(join(tmpdir(), "l9-tooth-"));
    const file = join(dir, "noisy.html");
    writeFileSync(file, `<!doctype html><html><body>tooth<script>console.log('fine'); console.error('tooth-injected-console-error'); setTimeout(function () { throw new Error('tooth-injected-page-error') }, 0)</script></body></html>`);
    await ctx.step("a page logs one console.error and throws one uncaught error", async () => {
      await b.open(`file://${file}`);
      await b.waitFor(`document.body.innerText.includes('tooth')`, 10000, "tooth page");
      await sleep(500);
    });
    await strictSteps(ctx, [["tooth", b]]);
  },
};
