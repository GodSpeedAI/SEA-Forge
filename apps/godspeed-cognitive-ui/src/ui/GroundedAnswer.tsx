import type { JSX } from 'react';
import type { ThothAnswerView } from '../ports/contract';
import './grounded-answer.css';

export interface GroundedAnswerProps {
  answer: ThothAnswerView;
  /** The claim the narration is currently on (index into answer.claims), if any. */
  activeClaim: number | null;
  paused: boolean;
}

/**
 * The complete governed Thoth disclosure behind a narration: disposition, freshness, assurance,
 * limitations, omitted claim classes, authority notice and the exact references of every claim.
 * Everything shown is a returned field, verbatim; an answer confers no execution authority.
 */
export function GroundedAnswer({ answer, activeClaim, paused }: GroundedAnswerProps): JSX.Element {
  return (
    <aside
      className="grounded-answer"
      data-testid="grounded-answer"
      data-disposition={answer.disposition}
      data-freshness={answer.freshness}
      data-answer-id={answer.answer_id}
      data-paused={paused ? 'true' : 'false'}
      aria-label="Governed Thoth disclosure"
    >
      <header className="grounded-head">
        <span className="grounded-title">Thoth disclosure</span>
        <span className="grounded-pill" data-testid="grounded-disposition">{answer.disposition}</span>
        <span className="grounded-pill" data-testid="grounded-freshness">{answer.freshness}</span>
      </header>
      <dl className="grounded-meta">
        <dt>Assurance</dt>
        <dd data-testid="grounded-assurance">{answer.assurance}</dd>
        <dt>Snapshot</dt>
        <dd data-testid="grounded-snapshot">{answer.snapshot_ref}</dd>
        <dt>Answered</dt>
        <dd>{answer.answered_at}</dd>
      </dl>
      <ol className="grounded-claims">
        {answer.claims.map((c, i) => (
          <li key={c.claim_id} data-testid="grounded-claim" data-claim-id={c.claim_id} data-active={activeClaim === i ? 'true' : 'false'}>
            <div className="grounded-claim-head">
              <span>{c.claim_class}</span>
              <span>{c.status}</span>
              <span>{c.subject}</span>
            </div>
            <p>{c.statement}</p>
            {[...c.evidence_refs, ...c.settlement_refs, ...(c.capability_record_ref ? [c.capability_record_ref] : [])].map((ref) => (
              <code key={ref} data-testid="grounded-ref">{ref}</code>
            ))}
          </li>
        ))}
      </ol>
      {answer.omitted_claim_classes.length > 0 && (
        <p className="grounded-omitted" data-testid="grounded-omitted">
          Withheld by policy: {answer.omitted_claim_classes.join(', ')}
        </p>
      )}
      {answer.limitations.length > 0 && (
        <ul className="grounded-limits" data-testid="grounded-limitations">
          {answer.limitations.map((l) => (
            <li key={l}>{l}</li>
          ))}
        </ul>
      )}
      <p className="grounded-notice" data-testid="grounded-notice">{answer.authority_notice}</p>
    </aside>
  );
}
