import { spawnSync } from "child_process";

interface SpawnResult {
  stdout: string;
  stderr: string;
  status: number | null;
}

export class Browser {
  private readonly session: string;

  constructor(session: string, _timeoutMs: number = 30000) {
    this.session = session;
  }

  /** The agent-browser --session name this Browser drives (identity isolation is per session). */
  get sessionName(): string {
    return this.session;
  }

  /** Raw agent-browser invocation for commands this wrapper has no method for. */
  run(args: string[]): SpawnResult {
    return this.spawn(args);
  }

  private spawn(args: string[]): SpawnResult {
    const result = spawnSync("agent-browser", [
      ...args,
      "--session",
      this.session,
    ]);

    return {
      stdout: result.stdout?.toString() || "",
      stderr: result.stderr?.toString() || "",
      status: result.status,
    };
  }

  async open(url: string): Promise<void> {
    const result = this.spawn(["open", url]);
    if (result.status !== 0) {
      throw new Error(`Failed to open ${url}: ${result.stderr}`);
    }
  }

  async eval<T>(js: string): Promise<T> {
    const result = this.spawn(["eval", js]);
    if (result.status !== 0) {
      throw new Error(`eval failed: ${result.stderr}`);
    }

    const output = result.stdout.trim();
    if (!output) {
      throw new Error("eval returned empty output");
    }

    // eval output is already JSON, just needs parsing
    try {
      return JSON.parse(output) as T;
    } catch {
      throw new Error(`eval output is not valid JSON: ${output}`);
    }
  }

  async click(selector: string): Promise<void> {
    const result = this.spawn(["click", selector]);
    if (result.status !== 0) {
      throw new Error(`click ${selector} failed: ${result.stderr}`);
    }
  }

  /** Real pointer input at viewport coordinates (move, press, release). */
  async mouseClick(x: number, y: number): Promise<void> {
    for (const args of [["mouse", "move", String(Math.round(x)), String(Math.round(y))], ["mouse", "down"], ["mouse", "up"]]) {
      const r = this.spawn(args);
      if (r.status !== 0) throw new Error(`mouse ${args.join(" ")} failed: ${r.stderr}`);
    }
  }

  async mouseMove(x: number, y: number): Promise<void> {
    this.spawn(["mouse", "move", String(Math.round(x)), String(Math.round(y))]);
  }

  async press(key: string): Promise<void> {
    const result = this.spawn(["press", key]);
    if (result.status !== 0) {
      throw new Error(`press ${key} failed: ${result.stderr}`);
    }
  }

  /** Fills an input the way a user's typing would end up (fires input events). */
  async fill(selector: string, text: string): Promise<void> {
    const result = this.spawn(["fill", selector, text]);
    if (result.status !== 0 || /✗/.test(result.stdout)) throw new Error(`fill failed: ${result.stderr || result.stdout}`);
  }

  async type(text: string): Promise<void> {
    const result = this.spawn(["keyboard", "type", text]);
    if (result.status !== 0) {
      throw new Error(`type failed: ${result.stderr}`);
    }
  }

  async screenshot(path: string): Promise<void> {
    const result = this.spawn(["screenshot", path]);
    if (result.status !== 0) {
      throw new Error(`screenshot failed: ${result.stderr}`);
    }
  }

  async viewport(w: number, h: number): Promise<void> {
    const result = this.spawn(["set", "viewport", String(w), String(h)]);
    if (result.status !== 0) {
      throw new Error(`set viewport failed: ${result.stderr}`);
    }
  }

  async waitFor(
    js: string,
    timeoutMs: number = 15000,
    label: string = "condition"
  ): Promise<void> {
    const startTime = Date.now();
    const pollInterval = 250;

    while (Date.now() - startTime < timeoutMs) {
      try {
        const result = await this.eval<boolean>(js);
        if (result) {
          return;
        }
      } catch {
        // Ignore errors during poll, keep trying
      }

      await new Promise((resolve) => setTimeout(resolve, pollInterval));
    }

    throw new Error(`Timed out waiting for ${label}`);
  }

  async consoleErrors(): Promise<string[]> {
    const result = this.spawn(["errors"]);
    if (result.status !== 0) {
      return [];
    }

    const output = result.stdout.trim();
    if (!output || output.includes("✓")) {
      return [];
    }

    return output.split("\n").filter((line) => line.trim());
  }

  async clearLogs(): Promise<void> {
    // agent-browser doesn't have explicit log clearing; we track it ourselves
    // This is a no-op in the agent-browser context
  }

  async requests(filter?: string): Promise<string[]> {
    const result = this.spawn(
      filter ? ["network", "requests", "--filter", filter] : ["network", "requests"]
    );
    if (result.status !== 0) {
      return [];
    }

    const output = result.stdout.trim();
    if (output.includes("No requests captured") || !output) {
      return [];
    }

    return output.split("\n").filter((line) => line.trim() && !line.includes("✓"));
  }

  async clearRequests(): Promise<void> {
    // Network clear handled by calling network requests after this
    // agent-browser doesn't have explicit request clearing
  }

  /** Cookies visible to this session, as agent-browser prints them (raw text). */
  async cookiesGet(): Promise<string> {
    const r = this.spawn(["cookies", "get"]);
    if (r.status !== 0) throw new Error(`cookies get failed: ${r.stderr}`);
    return r.stdout;
  }

  async cookiesClear(): Promise<void> {
    const r = this.spawn(["cookies", "clear"]);
    if (r.status !== 0) throw new Error(`cookies clear failed: ${r.stderr}`);
  }

  /** Console log lines (all levels). */
  async consoleLogs(clear = false): Promise<string[]> {
    const r = this.spawn(clear ? ["console", "--clear"] : ["console"]);
    if (r.status !== 0) throw new Error(`console failed: ${r.stderr}`);
    return r.stdout.split("\n").filter((l) => l.trim() && !l.includes("✓"));
  }

  /**
   * Every console entry this session's pages have produced (all levels, across navigations), read
   * with --json: the plain-text form prints no entries. Entries are {level, text}.
   */
  async consoleEntries(): Promise<{ level: string; text: string }[]> {
    const r = this.spawn(["console", "--json"]);
    if (r.status !== 0) throw new Error(`console --json failed: ${r.stderr}`);
    const parsed = JSON.parse(r.stdout) as {
      success?: boolean;
      data?: { entries?: { level: string; text: string }[]; messages?: { type: string; text: string }[] };
    };
    if (!parsed.success) throw new Error(`console --json unsuccessful: ${r.stdout.slice(0, 200)}`);
    // agent-browser has answered in two shapes depending on how it was launched: {entries:[{level}]}
    // and {messages:[{type}]}. Reading only one would silently see an empty console under the other.
    if (parsed.data?.entries) return parsed.data.entries;
    if (parsed.data?.messages) return parsed.data.messages.map((m) => ({ level: m.type, text: m.text }));
    throw new Error(`console --json carried neither entries nor messages: ${r.stdout.slice(0, 200)}`);
  }

  /** Uncaught page errors this session has seen (--json; the plain form prints no detail). */
  async pageErrors(): Promise<{ text: string }[]> {
    const r = this.spawn(["errors", "--json"]);
    if (r.status !== 0) throw new Error(`errors --json failed: ${r.stderr}`);
    const parsed = JSON.parse(r.stdout) as { success?: boolean; data?: { errors?: { text: string }[] } };
    if (!parsed.success) throw new Error(`errors --json unsuccessful: ${r.stdout.slice(0, 200)}`);
    if (!Array.isArray(parsed.data?.errors)) throw new Error(`errors --json carried no errors array: ${r.stdout.slice(0, 200)}`);
    return parsed.data!.errors!;
  }

  /** Discards page errors captured so far (so a later errors check starts from a known point). */
  async clearErrors(): Promise<void> {
    const r = this.spawn(["errors", "--clear"]);
    if (r.status !== 0) throw new Error(`errors --clear failed: ${r.stderr}`);
  }

  async harStart(): Promise<void> {
    const r = this.spawn(["network", "har", "start"]);
    if (r.status !== 0) throw new Error(`network har start failed: ${r.stderr}`);
  }

  async harStop(path: string): Promise<void> {
    const r = this.spawn(["network", "har", "stop", path]);
    if (r.status !== 0) throw new Error(`network har stop failed: ${r.stderr}`);
  }

  async recordStart(path: string, url?: string): Promise<void> {
    const r = this.spawn(url ? ["record", "start", path, url] : ["record", "start", path]);
    if (r.status !== 0) throw new Error(`record start failed: ${r.stderr}`);
  }

  async recordStop(): Promise<void> {
    const r = this.spawn(["record", "stop"]);
    if (r.status !== 0) throw new Error(`record stop failed: ${r.stderr}`);
  }

  async traceStart(): Promise<void> {
    const r = this.spawn(["trace", "start"]);
    if (r.status !== 0) throw new Error(`trace start failed: ${r.stderr}`);
  }

  async traceStop(path: string): Promise<void> {
    const r = this.spawn(["trace", "stop", path]);
    if (r.status !== 0) throw new Error(`trace stop failed: ${r.stderr}`);
  }

  async close(): Promise<void> {
    const result = this.spawn(["close"]);
    if (result.status !== 0) {
      throw new Error(`close failed: ${result.stderr}`);
    }
  }

  // Helper for dispatchWheel
  async dispatchWheel(
    selector: string | null,
    deltaY: number,
    x: number,
    y: number
  ): Promise<void> {
    const jsCode = `
      const el = ${
        selector
          ? `document.querySelector("${selector}")`
          : `document.elementFromPoint(${x}, ${y})`
      };
      if (el) {
        const event = new WheelEvent('wheel', {
          deltaY: ${deltaY},
          clientX: ${x},
          clientY: ${y},
          bubbles: true,
          cancelable: true
        });
        el.dispatchEvent(event);
      }
    `;
    await this.eval<void>(jsCode);
  }
}
