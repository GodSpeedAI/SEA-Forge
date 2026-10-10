// `bun e2e/run.ts --live`: the harness owns the stack. Fresh temp cell -> production UI build +
// bundle scan -> kernel + gateway (serving dist) -> live ladder -> stack down, evidence preserved.
// `--tooth stub-gateway` runs the negative self-check instead (see runTooth).

import { existsSync, lstatSync, mkdirSync, readFileSync, rmSync, symlinkSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { runLadder } from "../ladder";
import { liveJourneys } from "../journeys-live";
import { bootStack, repo, type Stack, type Tooth } from "./stack";

interface LiveOpts {
  out: string;
  outGiven: boolean;
  only?: string[];
  tooth?: string;
  skipBuild: boolean;
}

const evidenceRoot = join(repo, ".agents", "evidence", "casework-live-wiring", "T10");

function runDir(): string {
  const ts = new Date().toISOString().replace(/[:.]/g, "-");
  return join(evidenceRoot, `run-${ts}`);
}

function pointLatest(dir: string) {
  const link = join(dirname(dir), "latest");
  try {
    if (existsSync(link) || lstatSync(link).isSymbolicLink()) rmSync(link, { force: true });
  } catch {
    // no previous link
  }
  try {
    symlinkSync(dir, link);
  } catch {
    // best effort; the run directory itself is the evidence
  }
}

export async function runLive(o: LiveOpts): Promise<void> {
  if (o.tooth && o.tooth !== "stub-gateway" && o.tooth !== "shared-session") {
    console.error(`unknown --tooth ${o.tooth} (known: stub-gateway, shared-session)`);
    process.exit(2);
  }
  // Only the stub-gateway tooth changes the stack; shared-session changes how the ladder hands out browsers.
  const tooth = o.tooth === "stub-gateway" ? (o.tooth as Tooth) : undefined;
  const sharedSession = o.tooth === "shared-session";
  const out = o.outGiven ? resolve(o.out) : runDir();
  mkdirSync(join(out, "screenshots"), { recursive: true });
  console.log(`live ladder${o.tooth ? ` [tooth: ${o.tooth}]` : ""}; evidence: ${out}`);

  let stack: Stack | null = null;
  let ok = false;
  const teardown = async (clean: boolean) => {
    if (!stack) return;
    const s = stack;
    stack = null;
    await s.down({ clean });
  };
  const onSignal = (sig: string) => () => {
    console.error(`received ${sig}; tearing the stack down`);
    teardown(false).finally(() => process.exit(130));
  };
  process.on("SIGINT", onSignal("SIGINT"));
  process.on("SIGTERM", onSignal("SIGTERM"));

  try {
    stack = await bootStack({ evidenceDir: out, tooth, skipBuild: o.skipBuild });
    console.log(`stack up: base=${stack.base} cell=${stack.cell}`);
    console.log(`bundle: ${stack.servedBundle.scanned.length} JS assets scanned (disk + HTTP), no fixture adapter`);
    const only = o.only ?? (tooth === "stub-gateway" ? ["L1"] : sharedSession ? ["L5"] : undefined);
    ok = await runLadder(liveJourneys, {
      base: stack.base,
      out,
      only,
      sessionPrefix: `gs-live-${Date.now().toString(36)}`,
      exitOnFailure: false,
      live: stack,
      sharedSession,
    });
    if (tooth === "stub-gateway") ok = judgeStubGatewayTooth(out);
    if (sharedSession) ok = judgeSharedSessionTooth(out);
  } catch (e) {
    console.error("live ladder error:", e instanceof Error ? e.message : e);
    ok = false;
  } finally {
    try {
      await teardown(ok);
    } catch (e) {
      console.error("stack teardown error:", e instanceof Error ? e.message : e);
      ok = false;
    }
    pointLatest(out);
  }
  process.exit(ok ? 0 : 1);
}

/**
 * Tooth (1): with the gateway stubbed to accept intents without calling the kernel, the L1
 * ladder MUST fail, and it must fail at the DURABLE step (not earlier, not for another reason).
 * Returns true only when that exact failure was observed.
 */
function judgeStubGatewayTooth(out: string): boolean {
  const results = JSON.parse(readFileSync(join(out, "results.json"), "utf8")) as {
    journey_id: string;
    status: string;
    steps: { name: string; ok: boolean; error?: string }[];
  }[];
  const l1 = results.find((r) => r.journey_id === "L1");
  const failed = l1?.steps.find((s) => !s.ok);
  const durableFailed = !!failed && failed.name.startsWith("durable:") && /timed out/.test(failed.error ?? "");
  if (l1?.status === "FAIL" && durableFailed) {
    console.log(`TOOTH stub-gateway: PASS - L1 failed at "${failed!.name}": ${failed!.error}`);
    return true;
  }
  console.error(`TOOTH stub-gateway: FAIL - expected L1 to fail at the durable step; got status=${l1?.status} failedStep=${failed?.name} error=${failed?.error}`);
  return false;
}

/**
 * Tooth (2): with both users forced into ONE agent-browser session, L5 MUST fail, and it must
 * fail at the shared-session guard (not at a login, a timeout or anything incidental).
 */
function judgeSharedSessionTooth(out: string): boolean {
  const results = JSON.parse(readFileSync(join(out, "results.json"), "utf8")) as {
    journey_id: string;
    status: string;
    steps: { name: string; ok: boolean; error?: string }[];
  }[];
  const l5 = results.find((r) => r.journey_id === "L5");
  const failed = l5?.steps.find((s) => !s.ok);
  const guarded = !!failed && failed.name.startsWith("guard:") && /shared session identity/.test(failed.error ?? "");
  if (l5?.status === "FAIL" && guarded && l5.steps.length === 1) {
    console.log(`TOOTH shared-session: PASS - L5 failed at "${failed!.name}": ${failed!.error}`);
    return true;
  }
  console.error(`TOOTH shared-session: FAIL - expected L5 to fail at the shared-session guard before doing anything else; got status=${l5?.status} steps=${l5?.steps.length} failedStep=${failed?.name} error=${failed?.error}`);
  return false;
}
