import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { DualStateIndicator, GovernedStatusPill } from "@sea-forge/ui-components";
import type { CaseSummary, HorizonItem } from "@sea-forge/contracts";
import { useCaseHorizon, useCaseList, useCaseOverview } from "../hooks/useCases";
import { useApprovals } from "../hooks/useApprovals";
import {
  CASE_STATE_VARIANT,
  EXECUTION_VARIANT,
  SETTLEMENT_VARIANT,
  humanize,
} from "./standing";
import styles from "./CaseHorizonPage.module.css";

/**
 * The case horizon: every committed case, and for the selected one, its plan
 * items with execution and settlement standing shown separately.
 *
 * Nothing here is reduced client-side from events. The kernel folds its own
 * trace events into `case.get_horizon` and this renders the answer, so a missed
 * frame costs a refetch rather than a wrong view (epic invariant 14: a
 * projection may improve usability but never become truth).
 */

function CaseRow({
  summary,
  selected,
  onSelect,
}: {
  summary: CaseSummary;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      type="button"
      className={styles.caseRow}
      aria-pressed={selected}
      data-selected={selected}
      onClick={onSelect}
    >
      <span className={styles.caseSummaryText}>{summary.summary || summary.case_id}</span>
      <span className={styles.caseMeta}>
        <GovernedStatusPill
          variant={CASE_STATE_VARIANT[summary.case_state] ?? "unknown"}
          label={humanize(summary.case_state)}
          className="status-pill"
        />
        <span className={styles.caseId}>{summary.case_id}</span>
      </span>
    </button>
  );
}

function HorizonRow({ item }: { item: HorizonItem }) {
  // These carry `#[serde(default)]` server-side, so the contract permits them
  // absent. An absent list means "none recorded", which is what an empty array
  // renders — no need to distinguish it from an explicit empty one here.
  const dependsOn = item.depends_on ?? [];
  const runIds = item.run_ids ?? [];
  return (
    <li className={styles.itemRow} data-od-id="case-horizon-item">
      <div className={styles.itemHead}>
        <span className={styles.itemName}>{item.name || item.plan_item_id}</span>
        <span className={styles.itemKind}>{humanize(item.item_kind)}</span>
      </div>
      <DualStateIndicator
        executionState={EXECUTION_VARIANT[item.execution] ?? "unknown"}
        settlementState={SETTLEMENT_VARIANT[item.settlement] ?? "unknown"}
      />
      <dl className={styles.itemDetail}>
        <div>
          <dt>Execution</dt>
          <dd>{humanize(item.execution)}</dd>
        </div>
        <div>
          <dt>Settlement</dt>
          <dd>{humanize(item.settlement)}</dd>
        </div>
        {dependsOn.length > 0 && (
          <div>
            <dt>Depends on</dt>
            <dd>{dependsOn.join(", ")}</dd>
          </div>
        )}
        {runIds.length > 0 && (
          <div>
            <dt>Episodes</dt>
            {/*
              Each retry is its own immutable run (epic 7.5), so every id is
              listed rather than counted — and each links to the record that
              proves what that attempt did (epic 12.1). A run id an operator
              can see but not open is the gap this resolves.
            */}
            <dd className={styles.episodeLinks}>
              {runIds.map((runId, index) => (
                <Link
                  key={runId}
                  to="/runs/$runId"
                  params={{ runId }}
                  className={styles.episodeLink}
                >
                  Attempt {index + 1}
                </Link>
              ))}
            </dd>
          </div>
        )}
      </dl>
    </li>
  );
}

export function CaseHorizonPage() {
  const { cases, unreadable, isLoading, error, refresh } = useCaseList();
  const [selected, setSelected] = useState<string | undefined>();

  // Default to the newest case rather than forcing a click to see anything.
  const activeCaseId = selected ?? cases[0]?.case_id;

  const { overview } = useCaseOverview(activeCaseId);
  const { horizon, error: horizonError } = useCaseHorizon(activeCaseId);
  const { approvals } = useApprovals(activeCaseId);

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <h1>Case horizon</h1>
        <p className={styles.lede}>
          Every committed case in this cell, and what each of its plan items is currently
          doing. Execution and settlement are shown as separate facts — work can finish and
          still not be accepted.
        </p>
        <button type="button" className={styles.refresh} onClick={refresh}>
          Re-read from records
        </button>
      </header>

      {error && (
        <div role="alert" className={styles.alert}>
          The case list could not be read: {error.message}
        </div>
      )}

      {unreadable.length > 0 && (
        <div role="alert" className={styles.alert}>
          {unreadable.length} case record(s) exist but could not be parsed:{" "}
          {unreadable.join(", ")}. These are not missing — they are unreadable, which is an
          integrity signal worth investigating.
        </div>
      )}

      <div className={styles.layout}>
        <section className={styles.caseList} aria-label="Committed cases" data-od-id="case-horizon-board">
          {isLoading && <p className={styles.muted}>Reading committed case records…</p>}
          {!isLoading && cases.length === 0 && !error && (
            <p className={styles.muted}>
              No case has been committed in this cell yet. That is a fact about this cell,
              not a failure to read it — commit one from the case workbench to populate
              this board.
            </p>
          )}
          {cases.map((summary) => (
            <CaseRow
              key={summary.case_id}
              summary={summary}
              selected={summary.case_id === activeCaseId}
              onSelect={() => setSelected(summary.case_id)}
            />
          ))}
        </section>

        <section className={styles.detail} aria-label="Case detail">
          {!activeCaseId && <p className={styles.muted}>Select a case to see its horizon.</p>}

          {activeCaseId && horizonError && (
            <div role="alert" className={styles.alert}>
              {horizonError.message}
            </div>
          )}

          {overview && (
            <div className={styles.overview} data-od-id="case-overview">
              <h2>{overview.summary || overview.case_id}</h2>
              <dl className={styles.overviewGrid}>
                <div>
                  <dt>Case state</dt>
                  <dd>{humanize(overview.case_state)}</dd>
                </div>
                <div>
                  <dt>Committed</dt>
                  <dd>{overview.created_at}</dd>
                </div>
                <div>
                  <dt>Plan items</dt>
                  <dd>{overview.item_count}</dd>
                </div>
                <div>
                  <dt>Template</dt>
                  <dd>{overview.template_ref ?? "not recorded"}</dd>
                </div>
                <div>
                  <dt>Runs</dt>
                  <dd>{overview.run_ids.length}</dd>
                </div>
                <div>
                  <dt>Settlements recorded</dt>
                  <dd>{(overview.settlements ?? []).length}</dd>
                </div>
              </dl>
              {overview.close_reason && (
                <p className={styles.closeReason}>Closed because: {overview.close_reason}</p>
              )}
            </div>
          )}

          {approvals.length > 0 && (
            <p className={styles.approvalNote} role="status">
              {approvals.length} approval(s) on this case are waiting for a decision. Until
              they are resolved, dependent items stay parked rather than failed.
            </p>
          )}

          {horizon && (
            <>
              <h3 className={styles.itemsHeading}>
                Plan items
                <span className={styles.folded}>
                  folded from {horizon.events_folded} trace event(s)
                </span>
              </h3>
              {horizon.items.length === 0 ? (
                <p className={styles.muted}>
                  This case has no plan items to show. The plan record may not be readable
                  from this cell.
                </p>
              ) : (
                <ul className={styles.itemList}>
                  {horizon.items.map((item) => (
                    <HorizonRow key={item.plan_item_id} item={item} />
                  ))}
                </ul>
              )}
            </>
          )}
        </section>
      </div>
    </div>
  );
}
