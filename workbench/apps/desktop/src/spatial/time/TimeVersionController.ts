// Time/Version boundary: the same object/system inspected through its
// developmental or historical states. Positions are opaque cursors minted by
// the kernel's event/ledger surfaces — never fabricated client-side.

export interface TimePosition {
  /** Opaque cursor (ledger entry ULID, event cursor, run id). */
  cursor: string;
  label: string;
}

export class TimeVersionController {
  private position: TimePosition | null = null;
  private readonly listeners = new Set<(position: TimePosition | null) => void>();

  get current(): TimePosition | null {
    return this.position;
  }

  /** Live head: no historical position selected. */
  isLive(): boolean {
    return this.position === null;
  }

  pin(position: TimePosition): void {
    this.position = position;
    for (const fn of this.listeners) fn(position);
  }

  returnToLive(): void {
    this.position = null;
    for (const fn of this.listeners) fn(null);
  }

  subscribe(fn: (position: TimePosition | null) => void): () => void {
    this.listeners.add(fn);
    return () => {
      this.listeners.delete(fn);
    };
  }
}

/** Singleton for the application lifetime. */
export const timeVersionController = new TimeVersionController();
