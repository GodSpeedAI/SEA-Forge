import { useState } from "react";
import { Link } from "@tanstack/react-router";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import type { PendingApproval } from "@sea-forge/contracts";
import { useApprovals, VERDICT_PAST, type Verdict } from "../hooks/useApprovals";
import { useIdentity } from "../hooks/useIdentity";
import { evaluateProtectedAction, type ProtectedActionDecision } from "../guards/protectedAction";
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
  action,
  currentActorId,
  onDecide,
}: {
  approval: PendingApproval;
  busy: boolean;
  action: ProtectedActionDecision;
  currentActorId?: string;
  onDecide: (verdict: Verdict, note: string) => void;
}) {
  const [note, setNote] = useState("");
  const noteId = `note-${approval.approval_id}`;
  const requester = approval.governance?.requester;
  const isSelfRequest = !!requester && !!currentActorId && requester === currentActorId;
  const isExpired = approval.expired;
  const isDisabled = busy || !action.isAllowed || isExpired;
  const disabledReason = isExpired
    ? "Decision unavailable: the approval window has closed."
    : !action.isAllowed && action.refusal
      ? `${action.refusal.message} ${action.refusal.unchangedEffect}`
      : undefined;
  const operationKind = approval.governance?.operation_kind ?? "the requested operation";
  const resourceRef = approval.governance?.resource_ref ?? "the recorded resource";
  const purposeEntries = approval.governance
    ? Object.entries(approval.governance.purpose_context ?? {})
    : [];

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

      {approval.governance ? (
        <section className={styles.governance} aria-label="Committed governance context">
          <p>
            <strong>Why review is required:</strong> {approval.governance.reason}
          </p>
          <dl className={styles.detail}>
            <div>
              <dt>Requester</dt>
              <dd className={styles.mono}>{approval.governance.requester}</dd>
            </div>
            <div>
              <dt>Operation</dt>
              <dd>{approval.governance.operation_kind}</dd>
            </div>
            <div>
              <dt>Resource</dt>
              <dd className={styles.mono}>{approval.governance.resource_ref ?? "none recorded"}</dd>
            </div>
            <div>
              <dt>Purpose context</dt>
              <dd>
                {purposeEntries.length > 0 ? (
                  <dl className={styles.purposeList}>
                    {purposeEntries.map(([key, value]) => (
                      <div key={key}>
                        <dt>{key}</dt>
                        <dd className={styles.mono}>{String(value ?? "not recorded")}</dd>
                      </div>
                    ))}
                  </dl>
                ) : (
                  <span className={styles.mono}>not recorded</span>
                )}
              </dd>
            </div>
            <div>
              <dt>Side effect</dt>
              <dd>{approval.governance.side_effect_standing}</dd>
            </div>
            <div>
              <dt>Decision record</dt>
              <dd className={styles.mono}>{approval.governance.decision_source.entry_id}</dd>
            </div>
          </dl>
          {(approval.governance.required_next_steps ?? []).length > 0 && (
            <p>Next lawful action: {(approval.governance.required_next_steps ?? []).join("; ")}</p>
          )}
        </section>
      ) : (
        <p className={styles.expiredNote} role="alert">
          This approval has no resolvable committed governance context. Re-read the case ledger
          before deciding; the server will continue to enforce eligibility and no-side-effect
          refusal rules.
        </p>
      )}

      {approval.expired && (
        <p className={styles.expiredNote} role="status">
          This approval window has closed. It stays listed so the reason dependent
          work is parked remains visible. The kernel decides whether a late decision
          is accepted. The buttons below are disabled for this reason.
        </p>
      )}

      {isSelfRequest && (
        <p className={styles.dutyWarning} role="alert">
          Separation of duty: you requested this work. Ask a different reviewer to
          decide, or record why self approval is justified in the note.
        </p>
      )}

      <p className={styles.consequence}>
        Approving executes {operationKind} on {resourceRef}. This cannot be undone
        from here. Rejecting parks dependent work instead of failing it.
      </p>

      <label className={styles.noteLabel} htmlFor={noteId}>
        Decision note
      </label>
      <textarea
        id={noteId}
        className={styles.note}
        value={note}
        rows={2}
        placeholder="Why this decision: recorded with it."
        onChange={(event) => setNote(event.target.value)}
      />

      <div className={styles.actions}>
        <button
          type="button"
          className={styles.approve}
          disabled={isDisabled}
          aria-describedby={disabledReason ? `${noteId}-reason` : undefined}
          onClick={() => onDecide("approve", note)}
        >
          {busy ? "Recording…" : "Approve"}
        </button>
        <button
          type="button"
          className={styles.reject}
          disabled={isDisabled}
          aria-describedby={disabledReason ? `${noteId}-reason` : undefined}
          onClick={() => onDecide("reject", note)}
        >
          {busy ? "Recording…" : "Reject"}
        </button>
      </div>
      {disabledReason && (
        <p className={styles.disabledReason} id={`${noteId}-reason`} role="status">
          {disabledReason}
        </p>
      )}
    </li>
  );
}

export function ApprovalInboxPage() {
  const { identity } = useIdentity();
  const { approvals, unreadable, isLoading, error, deciding, lastOutcome, decide, refresh } =
    useApprovals();
  const action = evaluateProtectedAction(identity, undefined, {
    method: "approval.decide",
    actionLabel: "approval decision",
    requiresReadiness: false,
  });

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
          empty inbox. Nothing could be determined, so no approval shown or missing here
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

      {action.refusal && (
        <p role="alert" className={styles.alert}>
          {action.refusal.message} {action.refusal.unchangedEffect}{" "}
          <Link to={action.refusal.repairRoute}>{action.refusal.repairLabel}</Link>.
        </p>
      )}

      {isLoading && <p className={styles.muted}>Reading the approvals journal…</p>}

      {!isLoading && approvals.length === 0 && !error && !unreadable && (
        <p className={styles.muted}>
          Nothing is waiting for a decision. This is an empty queue, not an unread one.
          The journal was read successfully and contained no pending approval.
        </p>
      )}

      {approvals.length > 0 && (
        <ul className={styles.list}>
          {approvals.map((approval) => (
            <ApprovalRow
              key={approval.approval_id}
              approval={approval}
              busy={deciding === approval.approval_id}
              action={action}
              currentActorId={identity?.actor?.actorId}
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
