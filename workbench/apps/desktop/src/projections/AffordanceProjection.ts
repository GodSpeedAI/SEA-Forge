// AffordanceProjection: what the Composer may offer in the current context.
// Available actions vary with Focus, Surface, authority, affordances, and
// case state. Unsupported capabilities are never populated: an action
// appears only when its backing port exists and its preconditions hold.

export interface Affordance {
  id: string;
  label: string;
  /** Route path the affordance navigates to; undefined = intent-only. */
  path?: string;
  disabled?: boolean;
  reason?: string;
}

export function projectAffordances(input: {
  surfaceId: string;
  focusedObjectId: string | null;
  inboxCount: number | undefined;
  connectionReady: boolean;
}): Affordance[] {
  const affordances: Affordance[] = [];
  if (!input.connectionReady) {
    return [
      {
        id: "no-cell",
        label: "No cell is running — actions unavailable",
        disabled: true,
        reason: "supervision unavailable",
      },
    ];
  }
  if (input.focusedObjectId) {
    affordances.push({
      id: "inspect-focus",
      label: `Inspect ${input.focusedObjectId}`,
      path: `/cases`,
    });
    affordances.push({ id: "release-focus", label: "Return to CORE" });
  } else {
    affordances.push({ id: "new-case", label: "Compose new case", path: "/cases/new" });
    if (input.inboxCount !== undefined && input.inboxCount > 0) {
      affordances.push({
        id: "review-inbox",
        label: `Review ${input.inboxCount} pending approval${input.inboxCount === 1 ? "" : "s"}`,
        path: "/inbox",
      });
    }
  }
  if (input.surfaceId !== "home") {
    affordances.push({ id: "go-home", label: "Return to CORE", path: "/" });
  }
  return affordances;
}
