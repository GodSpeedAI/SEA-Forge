// CaseProjection: case-horizon domain state → case surface arrangement.
// Pure mapping over the governed `case.list` / `case.get_overview` shapes;
// fetching stays in the existing `useCases` port. No invented semantics.

export interface CaseSummary {
  caseId: string;
  title: string;
}

export function projectCaseList(cases: Array<{ case_id: string; title?: string }>): CaseSummary[] {
  return cases.map((c) => ({ caseId: c.case_id, title: c.title ?? c.case_id }));
}
