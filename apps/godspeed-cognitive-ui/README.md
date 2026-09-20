# @godspeed/cognitive-ui

The GodSpeed casework cognitive environment (plan `godspeed-casework-cognitive-environment`,
task T03): a renderer-independent UI core, a local fixture world adapter, and a small React host
that demonstrates the interaction grammar.

**What this is:** a fixture-backed interaction prototype. The core (surfaces, objects,
relationships, focus, selection, disclosure, time positions, narration state, consequential-intent
refusals) runs entirely against local fixture providers.

**What this is not:** connected to SEA Forge, GitHub, Gauntlet, CopilotKit, or any governed
authority. No case truth, settlement, or execution lives here, and nothing consequential can be
concluded locally — a consequential intent is dispatched to the interaction adapter and the fixture
adapter refuses it. The same banner is shown inside the running app.

## Layout

| Path | Role |
|---|---|
| `contracts/` | The adapter contract shapes (frozen by T01): world, interaction, temporal, artifact, agent |
| `src/core/` | The UI core: representations (`model.ts`), store, action vocabulary (`actions.ts`), engine (`engine.ts`), core-owned ports (`ports.ts`) |
| `src/adapters/fixture/` | The fixture provider set: worlds, world/temporal/artifact/catalog adapters, plus a deliberately different second provider (`alt-provider.ts`) used by the provider-swap tooth |
| `src/adapters/test/` | No-op/test adapters: unavailable agent, scripted agent, recording scene renderer |
| `src/host/` | The React 19 host: composes the fixture environment, renders state, maps mouse and keyboard onto the same action vocabulary the tests call |

The core imports nothing but the contract types: `purity.test.ts` fails the suite if `react`,
`three`, `@copilotkit`, a transport API, or a backend noun appears under `src/core` (REQ-ARCH-003).

## Commands

```sh
bun install --frozen-lockfile   # frozen install
bun run typecheck               # tsc --noEmit
bun run build                   # vite production build
bun test                        # core + interaction suites
```

`just casework-ui-check` runs all four as `GATE_UI`.

## Running the demo

From the repository root:

```sh
just casework-ui-up            # serves http://127.0.0.1:4178 (fixed port, strict)
just casework-ui-status        # up/down, pid, last log lines
just casework-ui-down          # stop; idempotent
```

In the app: click an object to focus it, double-click to select, `↑`/`↓` move focus, `←`/`→` change
surface, `e` discloses the focused object one level deeper (minimal → summary → source, content from
the fixture artifact adapter), `t` steps back in time, `n` returns to now, `x` proposes a
consequential action (refused by the fixture adapter), `Esc` clears.

## Honest limits (T03 scope)

The spatial scene, semantic zoom, real temporal projection of historical world content, durable
artifacts, and live agent narration are later tasks (T06–T09). Time positions here are metadata
state only; narration shows the honest "no agent adapter configured" state; the Go transport does
not exist yet.
