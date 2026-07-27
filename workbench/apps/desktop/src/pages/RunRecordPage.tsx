import { Link, useParams } from "@tanstack/react-router";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import type { CriterionCheck, RunRecord } from "@sea-forge/contracts";
import { useRunRecord } from "../hooks/useRuns";
import {
  CRITERION_VARIANT,
  EXECUTION_VARIANT,
  SETTLEMENT_VARIANT,
  VERDICT_VARIANT,
  humanize,
} from "./standing";
import styles from "./RunRecordPage.module.css";

/**
 * One run episode, with every record that bears on it linked in one place
 * (epic 12.1).
 *
 * Every other surface in the Workbench emits run ids — the case overview, the
 * horizon's per-item episodes, settlement rows, event frames. This is what they
 * resolve to, which is what makes epic invariant 4 (*every important status
 * resolves to committed source records*) reachable rather than aspirational.
 *
 * The page renders `run.get` and adds no judgment of its own. In particular the
 * criteria table shows the settlement record's own basis tokens; where the
 * settlement recorded nothing, the row says so rather than inferring a pass.
 */

/** Sentence explaining what a criterion standing actually claims. */
const CRITERION_EXPLANATION: Record<string, string> = {
  satisfied: "The settlement record cites this criterion's passing basis.",
  unsatisfied: "The settlement record cites this criterion's failing basis.",
  recorded:
    "Observed and recorded as evidence. An evaluator score never decides acceptance on its own.",
  unavailable:
    "The settlement recorded nothing deciding this criterion. Unknown — not assumed satisfied.",
};

function CriterionRow({ check }: { check: CriterionCheck }) {
  const basis = check.basis ?? [];
  return (
    <tr>
      <th scope="row" className={styles.criterionName}>
        {humanize(check.criterion)}
      </th>
      <td className={styles.expected}>
        <code>{check.expected}</code>
      </td>
      <td>
        <GovernedStatusPill
          variant={CRITERION_VARIANT[check.standing] ?? "unknown"}
          label={humanize(check.standing)}
          className="status-pill"
        />
      </td>
      <td className={styles.basis}>
        {basis.length > 0 ? (
          basis.map((token) => (
            <code key={token} className={styles.token}>
              {token}
            </code>
          ))
        ) : (
          <span className={styles.muted}>{CRITERION_EXPLANATION[check.standing]}</span>
        )}
      </td>
    </tr>
  );
}

/**
 * Termination and settlement side by side, deliberately never merged.
 *
 * An exit code and a governed outcome are different facts, and a UI that shows
 * only one of them lets the wrong one define completion (epic 12.4). Rendering
 * both, always, is what keeps "the command exited 0 but the work was rejected"
 * a legible state instead of a contradiction.
 */
function OutcomePanel({ record }: { record: RunRecord }) {
  const settlement = record.settlement_record;
  return (
    <section className={styles.outcome} aria-label="Termination and settlement">
      <div className={styles.outcomeCard}>
        <h3>Execution termination</h3>
        <GovernedStatusPill
          variant={EXECUTION_VARIANT[record.execution] ?? "unknown"}
          label={humanize(record.execution)}
          className="status-pill"
        />
        {record.termination ? (
          <dl className={styles.grid}>
            <div>
              <dt>Ended by</dt>
              <dd>{humanize(record.termination.trace_kind)}</dd>
            </div>
            {record.termination.exit_code !== undefined &&
              record.termination.exit_code !== null && (
                <div>
                  <dt>Exit code</dt>
                  <dd>{record.termination.exit_code}</dd>
                </div>
              )}
            {record.termination.execution_status && (
              <div>
                <dt>Execution status</dt>
                <dd>{humanize(record.termination.execution_status)}</dd>
              </div>
            )}
            <div>
              <dt>At</dt>
              <dd>{record.termination.at}</dd>
            </div>
          </dl>
        ) : (
          <p className={styles.muted}>
            No terminating event recorded yet. This episode has not ended.
          </p>
        )}
        <p className={styles.note}>
          How the process ended. This is not a governed outcome on its own.
        </p>
      </div>

      <div className={styles.outcomeCard}>
        <h3>Settlement</h3>
        <GovernedStatusPill
          variant={SETTLEMENT_VARIANT[record.settlement] ?? "unknown"}
          label={humanize(record.settlement)}
          className="status-pill"
        />
        {settlement ? (
          <dl className={styles.grid}>
            <div>
              <dt>Settled at</dt>
              <dd>{settlement.settled_at}</dd>
            </div>
            <div>
              <dt>Review required</dt>
              <dd>{settlement.review_required ? "Yes" : "No"}</dd>
            </div>
            <div>
              <dt>Criteria record</dt>
              <dd>{settlement.criteria_ref ?? "not attributed"}</dd>
            </div>
          </dl>
        ) : (
          <p className={styles.muted}>
            No settlement record exists for this run. The work is unsettled — which is
            distinct from rejected.
          </p>
        )}
        <p className={styles.note}>
          The governed outcome. Only this decides whether the work was accepted.
        </p>
      </div>
    </section>
  );
}

export function RunRecordPage() {
  const { runId } = useParams({ from: "/runs/$runId" });
  const { record, isLoading, error } = useRunRecord(runId);

  if (isLoading) {
    return (
      <div className={styles.page}>
        <p className={styles.muted}>Reading committed records for {runId}…</p>
      </div>
    );
  }

  if (error) {
    // A governed `not_found` and an unreadable record need different responses,
    // so the class travels with the message rather than collapsing to "error".
    const errorClass = (error as { errorClass?: string }).errorClass;
    return (
      <div className={styles.page}>
        <div role="alert" className={styles.alert}>
          <strong>{errorClass === "not_found" ? "No such run" : "Run record unreadable"}</strong>
          <p>{error.message}</p>
          {errorClass === "record_unreadable" && (
            <p>
              The directory for this run exists but holds no readable trace, plan, or
              settlement. That is an integrity signal, not an absence.
            </p>
          )}
        </div>
        <Link to="/cases" className={styles.backLink}>
          Back to the case horizon
        </Link>
      </div>
    );
  }

  if (!record) return null;

  const criteria = record.criteria ?? [];
  const evidence = record.evidence ?? [];
  const trace = record.trace ?? [];
  const declarations = record.declarations ?? [];
  const records = record.records ?? [];

  return (
    <div className={styles.page} data-od-id="run-record">
      <header className={styles.header}>
        <div className={styles.breadcrumb}>
          <Link to="/cases">Case horizon</Link>
          {record.case_id && <span> / {record.case_id}</span>}
        </div>
        <h1>{record.plan_item_name || record.run_id}</h1>
        <p className={styles.lede}>
          One run episode with every record that bears on it. Each retry is its own
          immutable run — nothing here overwrites a previous attempt.
        </p>
        <dl className={styles.grid}>
          <div>
            <dt>Run</dt>
            <dd>
              <code>{record.run_id}</code>
            </dd>
          </div>
          <div>
            <dt>Plan item</dt>
            <dd>{record.plan_item_id ?? "not recorded"}</dd>
          </div>
          <div>
            <dt>Item kind</dt>
            <dd>{record.item_kind ? humanize(record.item_kind) : "not recorded"}</dd>
          </div>
          <div>
            <dt>Started</dt>
            <dd>{record.started_at ?? "not recorded"}</dd>
          </div>
          <div>
            <dt>Finished</dt>
            <dd>{record.finished_at ?? "not recorded"}</dd>
          </div>
          <div>
            <dt>Criteria record</dt>
            <dd>{record.criteria_ref ?? "not attributed"}</dd>
          </div>
        </dl>
      </header>

      <OutcomePanel record={record} />

      <section aria-label="Authority" className={styles.section}>
        <h2>Authority</h2>
        {record.authority ? (
          <div className={styles.authority}>
            <GovernedStatusPill
              variant={VERDICT_VARIANT[record.authority.verdict] ?? "unknown"}
              label={humanize(record.authority.verdict)}
              className="status-pill"
            />
            <p className={styles.reason}>{record.authority.reason}</p>
            <dl className={styles.grid}>
              <div>
                <dt>Disposition</dt>
                <dd>{humanize(record.authority.normalized_disposition)}</dd>
              </div>
              <div>
                <dt>Matched rule</dt>
                <dd>{record.authority.matched_rule ?? "none recorded"}</dd>
              </div>
              <div>
                <dt>Decided at</dt>
                <dd>{record.authority.decided_at}</dd>
              </div>
              <div>
                <dt>Policy refs</dt>
                <dd>{(record.authority.policy_refs ?? []).join(", ") || "none recorded"}</dd>
              </div>
            </dl>
            {(record.authority.reason_codes ?? []).length > 0 && (
              <p className={styles.codes}>
                Reason codes: {(record.authority.reason_codes ?? []).join(", ")}
              </p>
            )}
            {/*
              A denial is a governed outcome with a next lawful path, not a dead
              end (epic invariants 3 and 7). When the decision names steps, they
              are the actionable part of the record and belong in the open.
            */}
            {(record.authority.required_next_steps ?? []).length > 0 && (
              <div className={styles.nextSteps}>
                <h3>Required next steps</h3>
                <ul>
                  {(record.authority.required_next_steps ?? []).map((step) => (
                    <li key={step}>{step}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ) : (
          <p className={styles.muted}>
            No authority decision record for this run. Absent is not the same as denied —
            see the record inventory below.
          </p>
        )}
      </section>

      <section aria-label="Criteria and evidence" className={styles.section}>
        <h2>Criteria against evidence</h2>
        <p className={styles.sectionLede}>
          Each immutable criterion this run was committed against, paired with the basis the
          settlement record cited for it.
        </p>
        {criteria.length === 0 ? (
          <p className={styles.muted}>
            No settlement criteria were recorded for this run, so there is nothing to pair.
          </p>
        ) : (
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Criterion</th>
                <th scope="col">Declared</th>
                <th scope="col">Standing</th>
                <th scope="col">Settlement basis</th>
              </tr>
            </thead>
            <tbody>
              {criteria.map((check, index) => (
                <CriterionRow key={`${check.criterion}-${check.expected}-${index}`} check={check} />
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section aria-label="Settlement declarations" className={styles.section}>
        <h2>Declarations</h2>
        {declarations.length === 0 ? (
          <p className={styles.muted}>
            No settlement declaration has been issued for this run. Without one, this run
            contributes no qualifying weight to any capability record.
          </p>
        ) : (
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Declarer</th>
                <th scope="col">Role</th>
                <th scope="col">Strength</th>
                <th scope="col">Independent</th>
                <th scope="col">Qualifies</th>
                <th scope="col">Weight</th>
              </tr>
            </thead>
            <tbody>
              {declarations.map((declaration) => (
                <tr key={declaration.declaration_id}>
                  <th scope="row">{declaration.declarer_actor_id}</th>
                  <td>{declaration.declarer_role}</td>
                  <td>{humanize(declaration.strength)}</td>
                  {/*
                    Self-certification is shown, not silently down-weighted
                    (epic 12.6): an operator must be able to see that the
                    declarer and the acting entity were the same.
                  */}
                  <td>{declaration.independent ? "Yes" : "No — self-declared"}</td>
                  <td>{declaration.qualifies_for_capability ? "Yes" : "No"}</td>
                  <td>{declaration.reliability_weight}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section aria-label="Evidence" className={styles.section}>
        <h2>Evidence ({evidence.length})</h2>
        {evidence.length === 0 ? (
          <p className={styles.muted}>No evidence records were captured for this run.</p>
        ) : (
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Kind</th>
                <th scope="col">URI</th>
                <th scope="col">Digest</th>
                <th scope="col">From event</th>
              </tr>
            </thead>
            <tbody>
              {evidence.map((row) => (
                <tr key={row.evidence_id}>
                  <th scope="row">{humanize(row.kind)}</th>
                  <td className={styles.expected}>
                    <code>{row.uri}</code>
                  </td>
                  <td>
                    <code className={styles.digest}>{row.sha256 ?? "not hashed"}</code>
                  </td>
                  <td>
                    <code className={styles.digest}>{row.source_event_id}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>

      <section aria-label="Trace" className={styles.section}>
        <h2>Trace ({trace.length})</h2>
        <p className={styles.sectionLede}>
          Every state transition this run went through, in committed order. This is why a
          state is what it is.
        </p>
        {trace.length === 0 ? (
          <p className={styles.muted}>No trace events were recorded for this run.</p>
        ) : (
          <ol className={styles.trace}>
            {trace.map((event) => (
              <li key={event.event_id} className={styles.traceRow}>
                <span className={styles.traceKind}>{humanize(event.kind)}</span>
                <span className={styles.traceTime}>{event.timestamp}</span>
                <span className={styles.traceActor}>{event.actor_id}</span>
              </li>
            ))}
          </ol>
        )}
      </section>

      <section aria-label="Record inventory" className={styles.section}>
        <h2>Source records</h2>
        <p className={styles.sectionLede}>
          Every human-readable claim above resolves to one of these files. A record that is
          absent is reported as absent rather than rendered as an empty panel.
        </p>
        <ul className={styles.records}>
          {records.map((entry) => (
            <li key={entry.record} data-present={entry.present}>
              <code>{entry.record}</code>
              <span>
                {entry.present
                  ? `present${entry.bytes !== undefined && entry.bytes !== null ? ` · ${entry.bytes} bytes` : ""}`
                  : "not present"}
              </span>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
