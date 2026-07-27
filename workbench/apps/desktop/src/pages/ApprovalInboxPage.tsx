import { useState } from "react";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import type { PendingApproval } from "@sea-forge/contracts";
import { useApprovals, VERDICT_PAST, type Verdict } from "../hooks/useApprovals";
import styles from "./ApprovalInboxPage.module.css";

/**
 * The approval inbox.
 *
 * A row leaves this list only when a re-read of `approval.list` no longer
 * returns it — never optimistically. The kernel can refuse a decision (a stale
 * precondition, a closed window, separation of duty), and a row that vanished
 * on click would tell the approver the opposite of what happened.
 */

function ApprovalRow({
  approval,
  busy,
  onDecide,
}: {
  approval: PendingApproval;
  busy: boolean;
  onDecide: (verdict: Verdict, note: string) => void;
}) {
  const [note, setNote] = useState("");
  const noteId = `note-${approval.approval_id}`;

  return (
    <li className={styles.row} data-od-id="approval-decision-panel">
      <div className={styles.rowHead}>
        <span className={styles.itemName}>{approval.plan_item_id}</span>
        <GovernedStatusPill
          variant={approval.expired ? "blocked" : "degraded"}
          label={approval.expired ? "Window closed" : "Awaiting decision"}
          className="status-pill"
        />
      </div>

      <dl className={styles.detail}>
        <div>
          <dt>Case</dt>
          <dd className={styles.mono}>{approval.case_id}</dd>
        </div>
        <div>
          <dt>Approval</dt>
          <dd className={styles.mono}>{approval.approval_id}</dd>
        </div>
        <div>
          <dt>Run</dt>
          <dd className={styles.mono}>{approval.run_id}</dd>
        </div>
        <div>
          <dt>Requested</dt>
          <dd>{approval.requested_at}</dd>
        </div>
        <div>
          <dt>Expires</dt>
          <dd>{approval.expires_at}</dd>
        </div>
        {approval.criteria_ref && (
          <div>
            <dt>Criteria</dt>
            <dd className={styles.mono}>{approval.criteria_ref}</dd>
          </div>
        )}
      </dl>

      {approval.criteria_sha256 && (
        <p className={styles.criteriaHash}>
          Judged against criteria <code>{approval.criteria_sha256}</code>. If the criteria
          on disk no longer hash to this, the decision is refused rather than applied to
          different terms than the ones shown here.
        </p>
      )}

      {approval.expired && (
        <p className={styles.expiredNote} role="status">
          This approval&rsquo;s window has closed. It stays listed so the reason dependent
          work is parked remains visible; the kernel decides whether a decision is still
          accepted.
        </p>
      )}

      <label className={styles.noteLabel} htmlFor={noteId}>
        Decision note
      </label>
      <textarea
        id={noteId}
        className={styles.note}
        value={note}
        rows={2}
        placeholder="Why this decision — recorded with it."
        onChange={(event) => setNote(event.target.value)}
      />

      <div className={styles.actions}>
        <button
          type="button"
          className={styles.approve}
          disabled={busy}
          onClick={() => onDecide("approve", note)}
        >
          {busy ? "Recording…" : "Approve"}
        </button>
        <button
          type="button"
          className={styles.reject}
          disabled={busy}
          onClick={() => onDecide("reject", note)}
        >
          {busy ? "Recording…" : "Reject"}
        </button>
      </div>
    </li>
  );
}

export function ApprovalInboxPage() {
  const { approvals, unreadable, isLoading, error, deciding, lastOutcome, decide, refresh } =
    useApprovals();

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <h1>Inbox and approvals</h1>
        <p className={styles.lede}>
          Decisions waiting on a person. Each row carries the identifiers and the criteria
          hash the decision is judged against, so approving here is the same governed act
          as approving from the CLI.
        </p>
        <button type="button" className={styles.refresh} onClick={refresh}>
          Re-read the journal
        </button>
      </header>

      {error && (
        <div role="alert" className={styles.alert}>
          The approval inbox could not be read: {error.message}
        </div>
      )}

      {unreadable && (
        <div role="alert" className={styles.alert}>
          The approvals journal exists but could not be parsed: {unreadable}. This is not an
          empty inbox — nothing could be determined, so no approval shown or missing here
          should be trusted until it is resolved.
        </div>
      )}

      {lastOutcome && (
        <div
          role="status"
          className={lastOutcome.accepted ? styles.outcomeOk : styles.alert}
        >
          {lastOutcome.accepted
            ? `${lastOutcome.approvalId}: ${lastOutcome.detail}`
            : `${lastOutcome.approvalId} was not ${VERDICT_PAST[lastOutcome.verdict]}: ${lastOutcome.detail}`}
        </div>
      )}

      {isLoading && <p className={styles.muted}>Reading the approvals journal…</p>}

      {!isLoading && approvals.length === 0 && !error && !unreadable && (
        <p className={styles.muted}>
          Nothing is waiting for a decision. This is an empty queue, not an unread one — the
          journal was read successfully and contained no pending approval.
        </p>
      )}

      {approvals.length > 0 && (
        <ul className={styles.list}>
          {approvals.map((approval) => (
            <ApprovalRow
              key={approval.approval_id}
              approval={approval}
              busy={deciding === approval.approval_id}
              onDecide={(verdict, note) =>
                void decide(approval.approval_id, approval.case_id, verdict, note || undefined)
              }
            />
          ))}
        </ul>
      )}
    </div>
  );
}
