import { invoke } from "@tauri-apps/api/core";
import type { ValidateFunction } from "ajv";

/**
 * The one path from a renderer question to a validated SFWP answer.
 *
 * Every inspect hook needs the same three steps in the same order: send the
 * query, refuse the server's typed error envelope, then validate the body
 * against its generated contract. Before this module each hook re-implemented
 * all three, which had already produced three copies of `describeAjv` and eight
 * near-identical throw sites with drifting wording.
 *
 * The order matters and is the reason this is a function rather than a
 * convention. A governed error body (`{error, error_class}`) is not a view and
 * will never satisfy a view's schema, so validating first would report every
 * denial as "failed contract validation" — burying an actionable governance
 * outcome under a transport-shaped complaint. Denial is a governed outcome, not
 * a malformed response (epic invariant 3), so it is checked first, always.
 */

/** Flatten AJV's error list into one operator-readable line. */
export function describeAjv(errors: unknown): string {
  const list = errors as { instancePath?: string; message?: string }[] | null | undefined;
  if (!list?.length) return "unknown schema error";
  return list.map((e) => `${e.instancePath || "(root)"} ${e.message ?? ""}`.trim()).join("; ");
}

/**
 * A governed failure the server named, as opposed to a transport failure.
 *
 * `not_found` and `record_unreadable` need different operator responses — the
 * first means the record is not here, the second means something that should be
 * readable is not — so the class travels with the message rather than being
 * flattened into one error string.
 */
export class GovernedViewError extends Error {
  readonly errorClass: string;
  constructor(message: string, errorClass: string) {
    super(message);
    this.name = "GovernedViewError";
    this.errorClass = errorClass;
  }
}

/**
 * Historical name for {@link GovernedViewError}, kept because case views were
 * the first surface to need it. Identical type — `instanceof` works either way.
 */
export const CaseViewError = GovernedViewError;

/** Throw if the body is the server's typed error envelope rather than a view. */
export function rejectGovernedError(raw: unknown): void {
  const body = raw as { error?: unknown; error_class?: unknown } | undefined;
  if (typeof body?.error === "string") {
    throw new GovernedViewError(
      body.error,
      typeof body.error_class === "string" ? body.error_class : "unknown",
    );
  }
}

/**
 * Issue one SFWP inspect query and return its validated body.
 *
 * `method` is the dotted SFWP name used only in failure text, so an operator
 * reading an error knows which question failed rather than only that one did.
 */
export async function queryGoverned<T>(
  query: Record<string, unknown>,
  validate: ValidateFunction<T>,
  method: string,
): Promise<T> {
  const raw = await invoke<unknown>("sfwp_query", { query });
  rejectGovernedError(raw);
  if (!validate(raw)) {
    throw new Error(
      `${method} response failed contract validation: ${describeAjv(validate.errors)}`,
    );
  }
  return raw as T;
}
