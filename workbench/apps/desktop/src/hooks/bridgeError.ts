/**
 * Normalize whatever the Tauri bridge rejected with into an `Error`.
 *
 * Older host commands reject with bare strings. Governed command refusals now
 * also carry an error class, no-side-effect standing, and a next lawful action.
 * Reading `.message` off either raw value loses the operator-visible outcome.
 */
export function toError(reason: unknown): Error {
  if (reason instanceof Error) return reason;
  if (typeof reason === "string") return new Error(reason);
  if (typeof reason === "object" && reason !== null) {
    const body = reason as {
      error?: unknown;
      error_class?: unknown;
      no_side_effect?: unknown;
      next_lawful_action?: unknown;
    };
    if (typeof body.error === "string" && typeof body.error_class === "string") {
      return new BridgeGovernedError(
        body.error,
        body.error_class,
        body.no_side_effect === true,
        typeof body.next_lawful_action === "string" ? body.next_lawful_action : undefined,
      );
    }
  }
  return new Error(typeof reason === "undefined" ? "unknown bridge failure" : String(reason));
}
export class BridgeGovernedError extends Error {
  readonly errorClass: string;
  readonly noSideEffect: boolean;
  readonly nextLawfulAction?: string;

  constructor(
    message: string,
    errorClass: string,
    noSideEffect: boolean,
    nextLawfulAction?: string,
  ) {
    super(message);
    this.name = "BridgeGovernedError";
    this.errorClass = errorClass;
    this.noSideEffect = noSideEffect;
    this.nextLawfulAction = nextLawfulAction;
  }
}
