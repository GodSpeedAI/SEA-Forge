// EvidenceProjection: evidence index state → causal surface arrangement.
// Pass-through over governed evidence records; no client-side reduction.

export interface EvidenceSummary {
  evidenceId: string;
  kind: string;
}

export function projectEvidenceList(
  records: Array<{ evidence_id: string; kind: string }>,
): EvidenceSummary[] {
  return records.map((r) => ({ evidenceId: r.evidence_id, kind: r.kind }));
}
