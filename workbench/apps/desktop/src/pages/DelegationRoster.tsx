import { Link } from "@tanstack/react-router";
import { GovernedStatusPill, ProtectedActionButton } from "@sea-forge/ui-components";
import type { DelegationRow } from "@sea-forge/contracts";
import { useDelegations } from "../hooks/useDelegations";
import { DELEGATION_STANDING_VARIANT, SETTLEMENT_VARIANT, humanize } from "./standing";
import styles from "./RunRecordPage.module.css";

/**
 * The delegation roster and its per-run cancel (epic 9.7).
 *
 * # Cancel is per-run, and the UI has to make that legible
 *
 * The epic's control claim is "cancel one delegation without cancelling its
 * siblings". The kernel already guarantees it structurally — the cancel flag
 * lives on that run's own handle — so the job here is not to enforce it but to
 * *show* it: one button per row, labelled with the run it acts on, and no
 * bulk affordance anywhere.
 *
 * # Requested is not cancelled
 *
 * A successful cancel records a control request. The episode then terminates on
 * its own terms and its settlement says what happened. So the row moves to
 * `cancellation_requested`, never to "cancelled", and the confirmation says
 * "requested". Rendering the outcome in place of the request would be exactly
 * the execution-equals-settlement conflation the epic forbids.
 *
 * # Turn counts are absent while running, not zero
 *
 * The kernel publishes no per-turn state, so `turns_used` only exists once the
 * transcript evidence is written. A running delegation shows "not observable
 * while running" rather than `0`, which would be a claim that no turn has been
 * taken.
 */

function Bounded({ used, cap }: { used?: number | null; cap?: number | null }) {
  if (used == null) {
    return (
      <span className={styles.muted}>
        {cap == null ? "no cap recorded" : `not observable while running (cap ${cap})`}
      </span>
    );
  }
  return (
    <span>
      <code>{used}</code>
      {cap == null ? null : (
        <>
          {" of "}
          <code>{cap}</code>
        </>
      )}
    </span>
  );
}

function RosterRow({
  delegation,
  onCancel,
  cancelling,
}: {
  delegation: DelegationRow;
  onCancel: (runId: string) => void;
  cancelling: boolean;
}) {
  return (
    <tr>
      <th scope="row">
        <Link to="/runs/$runId" params={{ runId: delegation.run_id }}>
          <code>{delegation.run_id}</code>
        </Link>
      </th>
      <td>
        {delegation.endpoint_ref ? (
          <code>{delegation.endpoint_ref}</code>
        ) : (
          <span className={styles.muted}>not recorded</span>
        )}
      </td>
      <td>
        <GovernedStatusPill
          variant={DELEGATION_STANDING_VARIANT[delegation.standing] ?? "unknown"}
          label={humanize(delegation.standing)}
          className="status-pill"
        />
      </td>
      <td>
        {delegation.settlement ? (
          <GovernedStatusPill
            variant={SETTLEMENT_VARIANT[delegation.settlement] ?? "unknown"}
            label={humanize(delegation.settlement)}
            className="status-pill"
          />
        ) : (
          // Unsettled is not "not accepted". The work has no verdict yet.
          <span className={styles.muted}>no verdict yet</span>
        )}
      </td>
      <td>
        <Bounded used={delegation.turns_used} cap={delegation.max_turns} />
      </td>
      <td>
        {delegation.termination ? (
          humanize(delegation.termination)
        ) : (
          <span className={styles.muted}>still open</span>
        )}
      </td>
      <td>
        <ProtectedActionButton
          label={`Cancel ${delegation.run_id}`}
          variant="danger"
          isAllowed={delegation.cancellable}
          disabledReason={
            delegation.standing === "cancellation_requested"
              ? "A cancellation has already been requested for this delegation."
              : "This delegation is not active in this server, so there is nothing to cancel."
          }
          requiresConfirmation
          confirmationMessage={`Request cancellation of ${delegation.run_id}? Its siblings are unaffected. The episode ends on its own terms and its settlement records what happened.`}
          isLoading={cancelling}
          onClick={() => onCancel(delegation.run_id)}
        />
      </td>
    </tr>
  );
}

export function DelegationRoster() {
  const {
    delegations,
    unreadable,
    isLoading,
    isFetching,
    error,
    cancelling,
    lastOutcome,
    cancel,
    refresh,
  } = useDelegations();

  return (
    <section className={styles.section} aria-label="Delegation roster" data-od-id="delegation-roster">
      <h2>Delegation roster</h2>
      <p className={styles.sectionLede}>
        Every agent delegation this cell has recorded, live and finished. Cancelling acts
        on one run and never on its siblings.{" "}
        <button type="button" className="button" onClick={refresh} disabled={isFetching}>
          {isFetching ? "Re-reading…" : "Re-read roster"}
        </button>
      </p>

      {error && (
        <div role="alert" className={styles.alert}>
          The delegation roster could not be read: {error.message}
        </div>
      )}

      {unreadable.length > 0 && (
        <div role="alert" className={styles.alert}>
          {unreadable.length} delegation run(s) exist but could not be read:{" "}
          {unreadable.join(", ")}. These are not absent delegations — they are unreadable
          ones, which is an integrity signal worth investigating.
        </div>
      )}

      {lastOutcome && (
        // Named so it is distinguishable from the per-row button's own live
        // region — an operator using a screen reader needs to know which
        // announcement is the server's answer.
        <p
          role="status"
          aria-label="Last cancellation outcome"
          className={lastOutcome.accepted ? styles.muted : styles.alert}
        >
          {lastOutcome.runId}: {lastOutcome.detail}
        </p>
      )}

      {isLoading && <p className={styles.muted}>Reading the delegation roster…</p>}

      {!isLoading && !error && delegations.length === 0 && (
        <p className={styles.muted}>
          No delegation has been recorded in this cell. That is a fact about this cell, not
          a failure to read it.
        </p>
      )}

      {delegations.length > 0 && (
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">Run</th>
              <th scope="col">Endpoint</th>
              <th scope="col">Standing</th>
              <th scope="col">Settlement</th>
              <th scope="col">Turns</th>
              <th scope="col">Termination</th>
              <th scope="col">Intervene</th>
            </tr>
          </thead>
          <tbody>
            {delegations.map((delegation) => (
              <RosterRow
                key={delegation.run_id}
                delegation={delegation}
                onCancel={(runId) => void cancel(runId)}
                cancelling={cancelling === delegation.run_id}
              />
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
