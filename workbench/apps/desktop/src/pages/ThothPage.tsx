import { useState } from "react";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import { useEvidenceContext } from "../shell/EvidenceContext";
import {
  QUESTION_KINDS,
  useThoth,
  type QuestionKind,
} from "../hooks/useThoth";
import {
  STANDING_PRESENTATION,
  standingOf,
  useServerContract,
} from "../hooks/useServerContract";
import styles from "./SurfacesPages.module.css";

/**
 * Grounded questions and their answers (epic journey 3, stories 3.1-3.9).
 *
 * This surface used to render a written-in answer — "Endpoint verification and
 * an allowed authority boundary are both required…" — with evidence links
 * citing the UX epic. It was indistinguishable from a real answer and cited a
 * design document as its source. `thoth.ask` is live now, so it asks.
 *
 * Story 3.9 is the one that shapes the layout: every answer must disclose its
 * freshness, assurance, limitations, omitted claim classes, and that knowledge
 * confers no execution authority. All five are rendered, none behind a
 * disclosure toggle — an operator should not have to open a panel to discover
 * that the answer they are reading is partial.
 */
export function ThothPage() {
  const { contract } = useServerContract();
  const { inspectEvidence } = useEvidenceContext();
  const { ask, answer, error, asking } = useThoth();

  const [kind, setKind] = useState<QuestionKind>("ask_available_affordances");
  const [subject, setSubject] = useState("delegation");
  const [purpose, setPurpose] = useState("operator inspection");

  const standing = standingOf("thoth.ask", contract);
  const presentation = STANDING_PRESENTATION[standing];
  const reachable = standing === "live";

  const dispositionVariant =
    answer?.disposition === "answered"
      ? "ready"
      : answer?.disposition === "partial"
        ? "degraded"
        : "blocked";

  return (
    <div className={styles.page}>
      <section
        className="governed-focus"
        data-od-id="thoth"
        data-state={answer ? dispositionVariant : "unknown"}
      >
        <div className="focus-copy">
          <div className="title-line">
            <h1>Ask this cell</h1>
            <GovernedStatusPill
              variant={reachable ? "ready" : presentation.variant}
              label={reachable ? "thoth.ask live" : presentation.label}
              className="status-pill"
            />
          </div>
          <p>
            Typed questions answered from one verified self-model snapshot. Answers
            are grounded in committed records and confer no authority to act.
          </p>
          {!reachable ? (
            <p className="operational-copy">{presentation.explanation}</p>
          ) : null}
        </div>
      </section>

      <div className="content-grid">
        <div className="content-primary">
          <section className="panel" data-od-id="thoth-question-composer">
            <div className="section-header">
              <div>
                <p className="section-kicker">Grounded question</p>
                <h2>What should be inspected?</h2>
              </div>
            </div>

            <label className="field-label" htmlFor="thothKind">
              Question kind
            </label>
            <select
              id="thothKind"
              className="workbench-input"
              value={kind}
              onChange={(event) => setKind(event.target.value as QuestionKind)}
            >
              {QUESTION_KINDS.map((option) => (
                <option key={option.kind} value={option.kind}>
                  {option.label}
                </option>
              ))}
            </select>

            <label className="field-label" htmlFor="thothSubject">
              Subject
            </label>
            {/* A concept ref or capability name, not prose. The kernel refuses
                an empty subject rather than guessing one. */}
            <input
              id="thothSubject"
              className="workbench-input"
              value={subject}
              onChange={(event) => setSubject(event.target.value)}
            />

            <label className="field-label" htmlFor="thothPurpose">
              Purpose
            </label>
            <input
              id="thothPurpose"
              className="workbench-input"
              value={purpose}
              onChange={(event) => setPurpose(event.target.value)}
            />

            <div className="focus-actions">
              <button
                type="button"
                className="button button--primary"
                disabled={asking || !reachable || !subject.trim()}
                onClick={() => void ask({ kind, subject, purpose })}
              >
                {asking ? "Asking…" : "Ask"}
              </button>
            </div>

            {error ? (
              <p className="operational-copy" role="alert">
                {error.message}
              </p>
            ) : null}
          </section>

          <section className="panel" data-od-id="thoth-answer-detail">
            <div className="section-header">
              <div>
                <p className="section-kicker">Answer</p>
                <h2>
                  {answer ? `Disposition: ${answer.disposition}` : "No answer yet"}
                </h2>
              </div>
              {answer ? (
                <GovernedStatusPill
                  variant={dispositionVariant}
                  label={answer.disposition}
                  className="status-pill"
                />
              ) : null}
            </div>

            {!answer ? (
              <p className="operational-copy">
                Nothing has been asked in this session. This surface shows no
                sample answer, because a sample would be indistinguishable from a
                grounded one.
              </p>
            ) : answer.claims.length === 0 ? (
              <p className="operational-copy">
                This answer carries no claims. That is the shape of a denial: the
                classes withheld are listed under disclosure, and no subject
                detail beyond the question itself is disclosed.
              </p>
            ) : (
              answer.claims.map((claim) => (
                <button
                  key={claim.claim_id}
                  type="button"
                  className="evidence-link"
                  onClick={() =>
                    inspectEvidence({
                      id: claim.claim_id,
                      kind: `thoth_claim.${claim.claim_class}`,
                      disclosureStatus: "permitted",
                      rawPayload: JSON.stringify(claim, null, 2),
                    })
                  }
                >
                  <strong>{claim.statement}</strong>
                  <span>
                    {claim.claim_class} · {claim.status} ·{" "}
                    {(claim.evidence_refs?.length ?? 0) + (claim.settlement_refs?.length ?? 0)} record(s)
                  </span>
                </button>
              ))
            )}
          </section>
        </div>

        <aside className="attention-rail" aria-label="Answer standing">
          <section className="panel" data-od-id="thoth-standing">
            <div className="section-header">
              <div>
                <p className="section-kicker">Answer standing</p>
                <h2>Not a decision</h2>
              </div>
            </div>

            {answer ? (
              <>
                <p>{answer.authority_notice}</p>
                <dl className="detail-grid">
                  <div>
                    <dt>Freshness</dt>
                    <dd className="machine-value">{answer.freshness}</dd>
                  </div>
                  <div>
                    <dt>Assurance</dt>
                    <dd className="machine-value">{answer.assurance}</dd>
                  </div>
                  <div>
                    <dt>Snapshot</dt>
                    <dd className="machine-value">{answer.snapshot_ref}</dd>
                  </div>
                  <div>
                    <dt>Answered at</dt>
                    <dd className="machine-value">{answer.answered_at}</dd>
                  </div>
                </dl>

                {/* Rendered whenever non-empty, including on a partial answer.
                    An operator reading a partial answer has to be able to see
                    that it is partial without opening anything. */}
                {answer.omitted_claim_classes.length > 0 ? (
                  <>
                    <p className="section-kicker">Withheld classes</p>
                    <p className="operational-copy">
                      {answer.omitted_claim_classes.join(", ")}
                    </p>
                  </>
                ) : null}

                {answer.limitations.length > 0 ? (
                  <>
                    <p className="section-kicker">Limitations</p>
                    <ul>
                      {answer.limitations.map((limitation) => (
                        <li key={limitation} className="operational-copy">
                          {limitation}
                        </li>
                      ))}
                    </ul>
                  </>
                ) : null}
              </>
            ) : (
              <p>
                An answer is an inspection aid. It does not approve, commit, or
                execute work.
              </p>
            )}
          </section>
        </aside>
      </div>
    </div>
  );
}
