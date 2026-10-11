// TimelineProjection: event-stream cursor state → temporal surface
// arrangement. Cursors are opaque kernel-minted values passed through
// untouched; the UI never invents a position in time.

export interface TimelinePosition {
  cursor: string | null;
  label: string;
}

export function projectTimelinePosition(input: {
  cursor: string | null;
  eventCount: number | null;
}): TimelinePosition {
  if (input.cursor === null) return { cursor: null, label: "live head" };
  return {
    cursor: input.cursor,
    label:
      input.eventCount === null ? `pinned at ${input.cursor}` : (
        `${input.eventCount} events · pinned at ${input.cursor}`
      ),
  };
}
