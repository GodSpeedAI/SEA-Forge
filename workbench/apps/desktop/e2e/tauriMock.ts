import type { Page } from "@playwright/test";

/**
 * Install a `window.__TAURI_INTERNALS__` shim before any app script runs. Both
 * `@tauri-apps/api`'s `invoke` and `listen` route through this global, so a
 * single shim covers the whole closed bridge without the native Tauri shell.
 *
 * - `sfwp_query` returns the provided readiness view (serialized in-script).
 * - `plugin:event|listen` returns a stub subscription id (no live events are
 *   needed for the readiness journey; event-driven refetch is covered by the
 *   Vitest component test).
 */
export async function installTauriMock(
  page: Page,
  readinessView: unknown,
): Promise<void> {
  await page.addInitScript((view) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).__TAURI_INTERNALS__ = {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      invoke: (cmd: string, _args: any) => {
        if (cmd === "sfwp_query") {
          return Promise.resolve(view);
        }
        if (cmd === "plugin:event|listen") {
          return Promise.resolve(1);
        }
        if (cmd === "plugin:event|unlisten") {
          return Promise.resolve();
        }
        return Promise.resolve(null);
      },
      transformCallback: (cb: unknown) => {
        const id = Math.floor(Math.random() * 1e9);
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (window as any)[`_${id}`] = cb;
        return id;
      },
      unregisterCallback: () => {},
      // Present in the real runtime; the event module's unlisten path calls it.
      unregisterListener: () => {},
      convertFileSrc: (p: string) => p,
    };
  }, readinessView);
}

/** A degraded (ready-with-limitations) readiness view for the journey. */
export function degradedReadinessView() {
  return {
    overall: "ready_with_limitations",
    foundations: [
      {
        id: "self_model_integrity",
        name: "Self-model integrity",
        category: "foundation",
        status: "ready",
        reason: "",
        source_ref: "sea-forge-self-model/src/store.rs::validate",
      },
    ],
    operational_capabilities: [
      {
        id: "local_governed_execution",
        name: "Local governed execution",
        category: "operational_capability",
        status: "ready",
        reason: "",
        source_ref: "sea-forge-server::case_dispatch",
      },
      {
        id: "external_delegation",
        name: "External delegation",
        category: "operational_capability",
        status: "degraded",
        reason: "Endpoint verification has not been recorded",
        source_ref: "sea-forge-agent::AgentConfig::endpoints",
      },
    ],
    recent_invalidations: [],
  };
}
