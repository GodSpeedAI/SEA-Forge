import type { Journey } from "../ladder";
import { FORBIDDEN_TOKENS } from "../live/bundle";
import { digestOf, sessionCookieValue } from "../live/sessions";
import { jsEval, sessionOf, stackOf, userBrowser, visible, loginThroughUi, W, H } from "./helpers";

// L0 readiness and identity. The operator signs in through the real LoginScreen; the session the
// gateway verified (not anything the client asserts) names operator_local/operator. The served
// bundle must carry the HTTP adapter and no fixture adapter, proven three ways: the build on
// disk, every asset fetched over HTTP, and the scripts the browser actually loaded.
export const L0: Journey = {
  id: "L0",
  title: "Readiness and identity",
  depends_on: [],
  settles: "the stack is the real, non-fixture, authenticated one",
  unlocks: ["L1"],
  async run(ctx) {
    const stack = stackOf(ctx);
    const b = userBrowser(ctx, "operator");

    await ctx.step("served bundle has no fixture adapter (disk scan + HTTP scan)", async () => {
      ctx.expect(stack.bundle.ok, `disk bundle: ${stack.bundle.problems.join("; ")}`);
      ctx.expect(stack.servedBundle.ok, `served bundle: ${stack.servedBundle.problems.join("; ")}`);
      ctx.expect(stack.servedBundle.hasHttpAdapter, "served build carries httpCaseworkAdapter-*.js");
      ctx.delta({ step: "bundle-scan", scanned: stack.servedBundle.scanned.length, forbidden: [...FORBIDDEN_TOKENS], violations: stack.servedBundle.violations });
    });

    await ctx.step("gateway is the live non-fixture build and the kernel is ready", async () => {
      const health = await (await fetch(`${stack.base}/api/healthz`)).json() as { status?: string; provenance?: string };
      ctx.expect(health.status === "ok", `healthz status ${health.status}`);
      ctx.expect(health.provenance === "go:live:sfwp", `gateway provenance is ${health.provenance}, expected go:live:sfwp`);
      const ready = await (await fetch(`${stack.base}/api/readyz`)).json() as { status?: string };
      ctx.expect(ready.status === "ready", `readyz status ${ready.status}`);
      ctx.delta({ step: "readiness", health, ready });
    });

    await ctx.step("unauthenticated browser sees the LoginScreen (live source label)", async () => {
      await b.viewport(W, H);
      await b.open(`${stack.base}/`);
      await b.waitFor(`!!document.querySelector('[data-testid="login-screen"]')`, 30000, "login screen");
      const label = await jsEval<string>(b, `document.querySelector('.login-source')?.textContent ?? ''`);
      ctx.expect(/^Live gateway/.test(label), `login source label is "${label}"`);
      ctx.expect((await sessionOf(b)).authenticated === false, "no session before login");
      await ctx.shot("login", b);
    });

    await ctx.step("operator signs in through the LoginScreen; identity is the gateway-verified one", async () => {
      await loginThroughUi(b, stack.base, "operator");
      const s = await sessionOf(b);
      ctx.expect(s.authenticated === true, "session authenticated");
      ctx.expect(s.actor_id === "operator_local" && s.role === "operator", `session identity ${JSON.stringify({ actor_id: s.actor_id, role: s.role })}`);
      const badge = await jsEval<string>(b, `document.querySelector('[data-testid="session-badge"]').innerText`);
      ctx.expect(/operator_local/.test(badge) && /operator/i.test(badge), `badge shows "${badge.replace(/\n/g, " ")}"`);
      // L9 proves this very session (same cookie, never re-issued) carried every journey up to L8.
      ctx.shared.operatorCookieDigest = digestOf(sessionCookieValue(await b.cookiesGet()));
      ctx.delta({ step: "identity", actor_id: s.actor_id, role: s.role });
      await ctx.shot("operator-home", b);
    });

    await ctx.step("the browser loaded only live-adapter scripts (no fixture adapter at runtime)", async () => {
      const urls = await jsEval<string[]>(
        b,
        `performance.getEntriesByType('resource').map((e) => e.name).filter((n) => /\\.js(\\?|$)/.test(n))`,
      );
      ctx.expect(urls.length > 0, "the browser loaded JS assets");
      const bodies = await jsEval<{ url: string; hit: string[] }[]>(
        b,
        `Promise.all(${JSON.stringify(urls)}.map(async (u) => { const t = await (await fetch(u)).text(); return { url: u, hit: ${JSON.stringify([...FORBIDDEN_TOKENS])}.filter((k) => t.includes(k)) }; }))`,
      );
      const bad = bodies.filter((x) => x.hit.length);
      ctx.expect(bad.length === 0, `loaded scripts carry fixture tokens: ${JSON.stringify(bad)}`);
      ctx.expect(urls.some((u) => /httpCaseworkAdapter-/.test(u)), "the HTTP adapter chunk was loaded by the page");
      ctx.expect(await visible(b, "session-badge"), "session badge rendered");
      ctx.delta({ step: "runtime-scripts", loaded: urls.map((u) => u.split("/").pop()) });
    });
  },
};
