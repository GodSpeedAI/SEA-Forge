import { Link } from "@tanstack/react-router";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import { useRunList } from "../hooks/useRuns";
import { EXECUTION_VARIANT, SETTLEMENT_VARIANT, humanize } from "./standing";
import styles from "./RunRecordPage.module.css";

/**
 * The evidence index: every run episode in this cell, as the entry point to the
 * records that prove what happened (epic journey 12).
 *
 * This is deliberately a *run* index rather than a free-text evidence search.
 * Evidence in SEA Forge is not a standalone corpus — every evidence record
 * belongs to a run and derives its meaning from that run's criteria, authority,
 * and settlement. Indexing evidence directly would present it detached from the
 * things that make it interpretable, and would need a search method the kernel
 * does not have. Listing runs uses `run.list`, which exists.
 */
export function EvidencePage() {
  const { runs, unreadable, isLoading, error } = useRunList();

  return (
    <div className={styles.page} data-od-id="evidence-index">
      <header className={styles.header}>
        <h1>Evidence and settlement</h1>
        <p className={styles.lede}>
          Every run episode recorded in this cell. Open one to see its criteria, authority
          decision, trace, evidence, and settlement linked together. Execution and settlement
          are separate columns because they are separate facts.
        </p>
      </header>

      {error && (
        <div role="alert" className={styles.alert}>
          The run list could not be read: {error.message}
        </div>
      )}

      {unreadable.length > 0 && (
        <div role="alert" className={styles.alert}>
          {unreadable.length} run director(ies) exist but hold no readable trace or
          settlement: {unreadable.join(", ")}. These are not missing runs — they are
          unreadable ones, which is an integrity signal worth investigating.
        </div>
      )}

      <section className={styles.section} aria-label="Run episodes">
        {isLoading && <p className={styles.muted}>Reading committed run records…</p>}

        {!isLoading && runs.length === 0 && !error && (
          <p className={styles.muted}>
            No run has been recorded in this cell yet. That is a fact about this cell, not a
            failure to read it.
          </p>
        )}

        {runs.length > 0 && (
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Run</th>
                <th scope="col">Case</th>
                <th scope="col">Plan item</th>
                <th scope="col">Execution</th>
                <th scope="col">Settlement</th>
                <th scope="col">Evidence</th>
                <th scope="col">Started</th>
              </tr>
            </thead>
            <tbody>
              {runs.map((run) => (
                <tr key={run.run_id}>
                  <th scope="row">
                    <Link to="/runs/$runId" params={{ runId: run.run_id }}>
                      <code>{run.run_id}</code>
                    </Link>
                  </th>
                  {/*
                    A run no case claims is an orphan. Epic 11.6 requires those
                    stay visible — process death must not be able to hide work —
                    so the row renders with the gap named rather than filtered
                    out of the list.
                  */}
                  <td>{run.case_id ?? <span className={styles.muted}>unclaimed</span>}</td>
                  <td>{run.plan_item_id ?? "—"}</td>
                  <td>
                    <GovernedStatusPill
                      variant={EXECUTION_VARIANT[run.execution] ?? "unknown"}
                      label={humanize(run.execution)}
                      className="status-pill"
                    />
                  </td>
                  <td>
                    <GovernedStatusPill
                      variant={SETTLEMENT_VARIANT[run.settlement] ?? "unknown"}
                      label={humanize(run.settlement)}
                      className="status-pill"
                    />
                  </td>
                  <td>{run.evidence_count}</td>
                  <td>{run.started_at ?? "not recorded"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}
