import { Browser } from "./browser";
import { mkdir } from "fs/promises";
import { appendFileSync, writeFileSync } from "fs";
import { join } from "path";

export type Step = {
  journey: string;
  name: string;
  ok: boolean;
  ms: number;
  error?: string;
};

export type Journey = {
  id: string;
  title: string;
  depends_on: string[];
  settles: string;
  unlocks: string[];
  run(ctx: Ctx): Promise<void>;
};

export type Ctx = {
  b: Browser;
  base: string;
  step(name: string, fn: () => void | Promise<void>): Promise<void>;
  expect(cond: unknown, msg: string): void;
  /** Screenshot into out/screenshots; `b` selects another session's page (default: the journey's own browser). */
  shot(name: string, b?: Browser): Promise<void>;
  out: string;
  /**
   * A named, run-wide browser session (separate agent-browser --session, so separate cookies and
   * identity). Created on first use, shared by every journey of the run, closed when the ladder
   * ends. Used by the live ladder (one session per user); fixture journeys never call it.
   */
  session(name: string): Browser;
  /** Cross-journey scratch (e.g. the case id L1 created for L2+). */
  shared: Record<string, unknown>;
  /** Append one line to out/durable-delta.jsonl: what durable state a step observed. */
  delta(entry: Record<string, unknown>): void;
  /** Live stack handle (cell path etc.); absent for the fixture ladder. */
  live?: unknown;
};

interface JourneyResult {
  journey_id: string;
  title: string;
  status: "PASS" | "FAIL" | "BLOCKED";
  depends_on: string[];
  settles: string;
  unlocks: string[];
  blocked_by?: string;
  steps: Step[];
  total_ms: number;
  error?: string;
}

export async function runLadder(
  journeys: Journey[],
  opts: {
    base: string;
    out: string;
    only?: string[];
    /** agent-browser session-name prefix; sessions are `${prefix}-${journey id}` (default gs-e2e). */
    sessionPrefix?: string;
    /** Exit the process non-zero on failure (default true; the live harness must tear down first). */
    exitOnFailure?: boolean;
    live?: unknown;
    /**
     * Self-check only (live tooth 2): every ctx.session(name) returns the SAME browser, i.e. all
     * users share one agent-browser session. The two-principal journeys must detect this and fail.
     */
    sharedSession?: boolean;
  }
): Promise<boolean> {
  const { base, out, only } = opts;
  const prefix = opts.sessionPrefix ?? "gs-e2e";
  const sessions = new Map<string, Browser>();
  const shared: Record<string, unknown> = {};
  const session = (requested: string): Browser => {
    const name = opts.sharedSession ? "shared" : requested;
    let s = sessions.get(name);
    if (!s) {
      s = new Browser(`${prefix}-${name}`, 30000);
      sessions.set(name, s);
    }
    return s;
  };

  // Ensure output directory exists
  await mkdir(join(out, "screenshots"), { recursive: true });

  const journeyResults: JourneyResult[] = [];
  const passedIds = new Set<string>();

  for (const journey of journeys) {
    if (only && !only.includes(journey.id)) {
      continue;
    }

    const startTime = Date.now();
    const steps: Step[] = [];
    let status: "PASS" | "FAIL" | "BLOCKED" = "PASS";
    let error: string | undefined;
    let blockedBy: string | undefined;

    // Check dependencies
    // With --only, dependencies outside the selection are assumed settled by an earlier full run.
    const blockedDep = journey.depends_on.find((dep) => !passedIds.has(dep) && (!only || only.includes(dep)));
    if (blockedDep) {
      status = "BLOCKED";
      blockedBy = blockedDep;
    } else {
      // Run the journey
      const browser = new Browser(`${prefix}-${journey.id}`, 30000);
      const ctx: Ctx = {
        b: browser,
        base,
        out,
        session,
        shared,
        live: opts.live,
        delta: (entry: Record<string, unknown>) => {
          appendFileSync(join(out, "durable-delta.jsonl"), JSON.stringify({ journey: journey.id, at: new Date().toISOString(), ...entry }) + "\n");
        },
        step: async (name: string, fn: () => void | Promise<void>) => {
          const stepStart = Date.now();
          try {
            await fn();
            steps.push({
              journey: journey.id,
              name,
              ok: true,
              ms: Date.now() - stepStart,
            });
          } catch (e) {
            const stepError = e instanceof Error ? e.message : String(e);
            steps.push({
              journey: journey.id,
              name,
              ok: false,
              ms: Date.now() - stepStart,
              error: stepError,
            });
            throw e;
          }
        },
        expect: (cond: unknown, msg: string) => {
          if (!cond) {
            throw new Error(msg);
          }
        },
        shot: (name: string, target?: Browser) => {
          const path = join(
            out,
            "screenshots",
            `${journey.id}-${name}.png`
          );
          return (target ?? browser).screenshot(path).catch((e) => {
            console.error(
              `Failed to save screenshot ${path}: ${e instanceof Error ? e.message : e}`
            );
          });
        },
      };

      try {
        await journey.run(ctx);
        // Check for console errors after journey completes
        const consoleErrors = await browser.consoleErrors();
        // Named sessions are checked too (empty set for the fixture ladder).
        for (const [name, sb] of sessions) {
          for (const line of await sb.consoleErrors()) consoleErrors.push(`[${name}] ${line}`);
          await sb.clearErrors().catch(() => {});
        }
        if (consoleErrors.length > 0) {
          status = "FAIL";
          error = `Console errors: ${consoleErrors.join(", ")}`;
        }
      } catch (e) {
        status = "FAIL";
        error = e instanceof Error ? e.message : String(e);
      } finally {
        try {
          await browser.close();
        } catch {
          // Ignore close errors
        }
      }
    }

    if (status === "PASS") {
      passedIds.add(journey.id);
    }

    const total_ms = Date.now() - startTime;
    journeyResults.push({
      journey_id: journey.id,
      title: journey.title,
      status,
      depends_on: journey.depends_on,
      settles: journey.settles,
      unlocks: journey.unlocks,
      blocked_by: blockedBy,
      steps,
      total_ms,
      error,
    });

    // Print one-line summary
    const stepSummary =
      steps.length > 0
        ? `${steps.filter((s) => s.ok).length}/${steps.length}`
        : "—";
    console.log(
      `[${status}] ${journey.id}: ${journey.title} (${stepSummary} steps, ${total_ms}ms)`
    );
    if (error) {
      console.log(`  Error: ${error}`);
    }
  }

  for (const sb of sessions.values()) {
    try {
      await sb.close();
    } catch {
      // Ignore close errors
    }
  }

  // Write results.json
  writeFileSync(
    join(out, "results.json"),
    JSON.stringify(journeyResults, null, 2)
  );

  // Write results.md table
  const mdLines = ["# E2E Test Results", ""];
  mdLines.push("| Journey | Status | Depends On | Settles | Steps | Notes |");
  mdLines.push("|---------|--------|-----------|---------|-------|-------|");

  for (const result of journeyResults) {
    const stepInfo =
      result.steps.length > 0
        ? `${result.steps.filter((s) => s.ok).length}/${result.steps.length}`
        : "—";
    const failingStep = result.steps.find((s) => !s.ok);
    const notes = result.status === "BLOCKED"
      ? `Blocked by ${result.blocked_by}`
      : failingStep
      ? `Failed: ${failingStep.name}`
      : result.error || "—";

    mdLines.push(
      `| ${result.journey_id} | ${result.status} | ${result.depends_on.join(", ") || "—"} | ${result.settles} | ${stepInfo} | ${notes} |`
    );
  }

  writeFileSync(join(out, "results.md"), mdLines.join("\n"));

  // Exit with error if any journey failed
  const anyFailed = journeyResults.some((r) => r.status !== "PASS");
  if (anyFailed && opts.exitOnFailure !== false) {
    process.exit(1);
  }
  return !anyFailed;
}
