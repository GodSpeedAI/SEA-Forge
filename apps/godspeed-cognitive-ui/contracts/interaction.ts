/**
 * Interaction contract.
 *
 * Interaction intents are the ONLY way the environment asks for something consequential. A local
 * interaction never mutates case truth: it is a request that crosses the application boundary and the
 * governed authority, and it is refused as readily as accepted.
 */

export type IntentKind =
  | 'focus-object'
  | 'resolve-object'
  | 'inspect-artifact'
  | 'request-explanation'
  | 'propose-consequence'
  | 'decide-approval'

export interface InteractionIntent {
  readonly id: string
  readonly kind: IntentKind
  /** Opaque target identity; the intent does not interpret it. */
  readonly target?: string
  readonly parameters?: Readonly<Record<string, string>>
}

/**
 * The result of an intent. `refused` carries a typed reason rather than a sentence, so the UI can
 * present authority denial, unavailability and invalid input distinctly.
 */
export type IntentOutcome =
  | { readonly status: 'accepted'; readonly note?: string }
  | { readonly status: 'refused'; readonly reason: IntentRefusal }

export type IntentRefusal =
  | 'authority_denied'
  | 'unavailable'
  | 'invalid'
  | 'stale_projection'

export interface InteractionAdapter {
  dispatch(intent: InteractionIntent): Promise<IntentOutcome>
}
