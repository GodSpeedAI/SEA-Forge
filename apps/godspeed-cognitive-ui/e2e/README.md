# E2E Test Harness

A small, repeatable browser E2E testing framework for godspeed-cognitive-ui using the `agent-browser` CLI.

## Running Tests

```bash
# Run all journeys against default dev server (http://127.0.0.1:4178)
bun e2e/run.ts

# Run against custom URL
bun e2e/run.ts --base http://localhost:5173

# Save results to custom directory
bun e2e/run.ts --out /tmp/test-results

# Run only specific journeys
bun e2e/run.ts --only SMOKE,J0,J1

# Shortcut
npm run e2e
```

Before running tests, ensure the dev server is running.

## Architecture

### Browser (`e2e/browser.ts`)

Wraps the `agent-browser` CLI with a TypeScript class. All commands use a dedicated session `gs-e2e-<journey-id>` for isolation.

**Public API:**
- `open(url)` - Navigate to URL
- `eval<T>(js: string): T` - Evaluate JS and return parsed result
- `click(selector)` - Click element by CSS selector
- `press(key)` - Press keyboard key
- `type(text)` - Type text into focused element
- `screenshot(path)` - Save screenshot
- `viewport(w, h)` - Set viewport size
- `waitFor(js, timeoutMs?, label?)` - Poll JS expression every 250ms until true
- `consoleErrors(): string[]` - Get page errors and console error messages
- `clearLogs()` - Clear log buffer (internal)
- `requests(filter?): string[]` - Get network request URLs
- `clearRequests()` - Clear request buffer
- `dispatchWheel(selector | null, deltaY, x, y)` - Dispatch wheel event
- `close()` - Close browser session

**Output Formats:**
- `eval` returns JSON-parsed result from `JSON.stringify(expr)`
- `click/press/type` return on success, throw on failure
- `consoleErrors()` returns string array or empty array
- `requests()` returns URL strings or empty array

### Journey & Ladder (`e2e/ladder.ts`)

Defines test structure and execution framework.

**Interface: Journey**
```typescript
interface Journey {
  id: string;              // Unique identifier
  title: string;           // Human-readable name
  depends_on: string[];    // Journey IDs that must PASS first
  settles: string;         // Capability this journey establishes
  unlocks: string[];       // Capabilities that depend on this
  run(ctx: Ctx): Promise<void>;
}
```

**Interface: Ctx**
```typescript
interface Ctx {
  b: Browser;
  base: string;            // Dev server base URL
  out: string;             // Output directory
  step(name, fn): Promise<void>;  // Record a named step
  expect(cond, msg): void;        // Assertion (throws on false)
  shot(name): void;               // Save screenshot to out/screenshots/{id}-{name}.png
}
```

**Dependencies & Blocking:**
- A journey whose `depends_on` contains an unresolved dependency is marked `BLOCKED` (not run)
- Steps record success, duration, and error message
- Console errors detected after journey completes cause FAIL status
- `ctx.b.clearLogs()` clears the error buffer when an error is expected

**Result Files:**
- `results.json` - Detailed test results with steps and timings
- `results.md` - Markdown table summary
- `screenshots/{journey-id}-{step-name}.png` - Test artifacts

### CLI (`e2e/run.ts`)

```bash
bun e2e/run.ts [--base URL] [--out DIR] [--only J1,J2]
```

- Checks dev server availability (HEAD request)
- Creates output directory
- Runs all journeys, exits with code 1 if any fail
- Default output: `.agents/evidence/godspeed-casework-cognitive-environment/ui-journeys/latest`

## Smoke Test

The `SMOKE` journey verifies the app boots and renders:
1. Open app at `/?source=local`
2. Wait for `#root` element to have children
3. Take screenshot
4. Assert `#root` exists

## Adding New Journeys

Create `e2e/journeys/your-journey.ts`:

```typescript
import { Journey, Ctx } from "../ladder.ts";

export const myJourney: Journey = {
  id: "J0",
  title: "User can log in",
  depends_on: ["SMOKE"],  // Requires SMOKE to pass first
  settles: "user-logged-in",
  unlocks: ["J1"],
  async run(ctx: Ctx) {
    await ctx.step("navigate", async () => {
      await ctx.b.open(`${ctx.base}/login`);
    });

    await ctx.step("fill form", async () => {
      await ctx.b.eval(`
        document.querySelector('input[name=email]').value = 'test@example.com';
        document.querySelector('input[name=password]').value = 'password';
      `);
    });

    await ctx.step("submit", async () => {
      await ctx.b.click('button[type=submit]');
    });

    await ctx.step("wait for dashboard", async () => {
      await ctx.b.waitFor(
        `window.location.pathname === '/dashboard'`,
        10000,
        "redirect to dashboard"
      );
    });

    await ctx.shot("logged-in");
  },
};
```

Then add to `e2e/journeys/index.ts`:

```typescript
import { myJourney } from "./your-journey.ts";

export const journeys: Journey[] = [smoke, myJourney];
```

## agent-browser Integration

The harness calls `agent-browser` directly via `Bun.spawnSync()`. Each journey uses its own isolated session:

```bash
agent-browser open URL --session gs-e2e-SMOKE
agent-browser eval 'expression' --session gs-e2e-SMOKE
agent-browser click selector --session gs-e2e-SMOKE
agent-browser screenshot path --session gs-e2e-SMOKE
agent-browser close --session gs-e2e-SMOKE
```

**Key outputs from agent-browser:**
- `eval`: Returns JSON string (parsed by wrapper)
- `click/press/type`: Returns "✓ Done" on success, error message on failure
- `errors`: Returns "✓ Done" if empty, error messages otherwise
- `network requests`: Returns "No requests captured" or URL list
- `screenshot`: Saves file and returns "✓ Screenshot saved to <path>"

## TypeScript

The harness includes e2e in `tsconfig.json` for type checking. Verify with:

```bash
bun run typecheck
```

Strict mode is enabled; all Browser methods are fully typed.

## Dependencies

- `Bun` (builtin `spawnSync`)
- `agent-browser` CLI v0.38+ on PATH
- No npm dependencies required
