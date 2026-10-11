// JudgmentProjection: approval inbox state → judgment surface arrangement.
// The inbox count lineage is preserved: unreadable/unread is `undefined`,
// never 0. Only approvals the kernel actually listed are projected.

export interface JudgmentSummary {
  pending: number | undefined;
  hasPending: boolean;
}

export function projectJudgment(input: {
  isLoading: boolean;
  error: unknown;
  unreadable: boolean;
  count: number;
}): JudgmentSummary {
  const pending =
    input.isLoading || input.error || input.unreadable ? undefined : input.count;
  return { pending, hasPending: (pending ?? 0) > 0 };
}
