#!/usr/bin/env bun
import { resolve, dirname } from "path";
import { mkdir } from "fs/promises";
import { runLadder } from "./ladder";
import { journeys } from "./journeys/index";

async function parseArgs(): Promise<{
  base: string;
  out: string;
  only?: string[];
}> {
  const args = process.argv.slice(2);
  const opts: {
    base: string;
    out: string;
    only?: string[];
  } = {
    base: "http://127.0.0.1:4178",
    out: resolve(
      dirname(import.meta.dir),
      "../../.agents/evidence/godspeed-casework-cognitive-environment/ui-journeys/latest"
    ),
  };

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--base") {
      opts.base = args[++i];
    } else if (args[i] === "--out") {
      opts.out = args[++i];
    } else if (args[i] === "--only") {
      opts.only = args[++i].split(",");
    }
  }

  return opts;
}

async function checkServerRunning(url: string): Promise<boolean> {
  try {
    const response = await fetch(url, {
      method: "HEAD",
      redirect: "follow",
    });
    return response.ok || response.status < 500;
  } catch {
    return false;
  }
}

async function main() {
  const opts = await parseArgs();

  // Check if server is running
  console.log(`Checking if dev server is running at ${opts.base}`);
  const serverReady = await checkServerRunning(opts.base);

  if (!serverReady) {
    console.error(
      `Dev server is not responding at ${opts.base}`
    );
    console.error("Start the dev server: just casework-ui-up");
    process.exit(2);
  }

  console.log(`Dev server is ready. Running tests with base=${opts.base}`);
  console.log(`Output directory: ${opts.out}`);

  // Ensure output directory exists
  await mkdir(opts.out, { recursive: true });

  // Set viewport
  await runLadder(journeys, {
    base: opts.base,
    out: opts.out,
    only: opts.only,
  });
}

main().catch((e) => {
  console.error("Fatal error:", e instanceof Error ? e.message : e);
  process.exit(1);
});
