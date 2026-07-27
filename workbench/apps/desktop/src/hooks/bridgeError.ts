/**
 * Normalize whatever the Tauri bridge rejected with into an `Error`.
 *
 * The host commands are `Result<T, String>`, so a failed `invoke` rejects with
 * a bare string, not an `Error`. Reading `.message` off it yields `undefined`,
 * which reaches the operator as "could not be read: undefined" — the failure
 * text replaced by nothing, at exactly the moment it matters most.
 */
export function toError(reason: unknown): Error {
  if (reason instanceof Error) return reason;
  if (typeof reason === "string") return new Error(reason);
  return new Error(typeof reason === "undefined" ? "unknown bridge failure" : String(reason));
}
