/**
 * Temporal contract.
 *
 * Time is a view over the same objects, not a separate object model. Adapters supply positions; the
 * environment owns the comparison state.
 */

export interface TemporalPosition {
  readonly cursor: number
  readonly at: string
  readonly summary: string
}

export interface TemporalWindow {
  readonly positions: readonly TemporalPosition[]
  /** True when older history exists beyond this window; the environment must not imply otherwise. */
  readonly truncated: boolean
}

export interface TemporalAdapter {
  window(since: string | null, limit: number): Promise<TemporalWindow>
  live(): Promise<TemporalPosition>
}
