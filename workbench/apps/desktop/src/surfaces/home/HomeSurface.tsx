// HomeSurface: the canonical CORE-centered orientation state. Not a
// destination containing CORE — Home IS the CORE-centered state: no secondary
// object owns focus, globally relevant objects arrange around CORE, the
// persistent Composer is available. Only source-backed values render;
// unanswered sources read as unknown, never as zero.

import { useApprovals } from "../../hooks/useApprovals";
import { useCaseList } from "../../hooks/useCases";
import { useServerContract } from "../../hooks/useServerContract";
import { useIdentity } from "../../hooks/useIdentity";
import { describeConnection } from "../../shell/connectionState";
import { projectHomeOrientation } from "../../projections/HomeProjection";
import { Surface, SpatialObject, Relationship } from "../../spatial/primitives";
import { focusController } from "../../spatial/focus/FocusController";
import styles from "./HomeSurface.module.css";

export function HomeSurface() {
  const { cases, unreadable, isLoading: casesLoading, error: casesError } = useCaseList();
  const approvals = useApprovals();
  const contract = useServerContract();
  const { supervision } = useIdentity();
  const connection = describeConnection(supervision, contract);

  const orientation = projectHomeOrientation({
    casesLoading,
    casesError,
    caseCount: cases.length,
    unreadable: unreadable.length,
    approvalsLoading: approvals.isLoading,
    approvalsError: approvals.error,
    approvalsUnreadable: approvals.unreadable !== undefined,
    pendingApprovals: approvals.approvals.length,
    connectionLabel: connection.label,
  });

  return (
    <Surface id="home" title="Home — CORE orientation">
      <div className={styles.orbit} data-testid="home-orbit">
        {orientation.status.state === "loading" ? <p>Reading cell orientation…</p> : null}
        {orientation.status.state === "unavailable" ? (
          <p role="status">
            Orientation is partial — {orientation.status.detail} Actions stay governed by the
            kernel regardless of what renders here.
          </p>
        ) : null}
        {orientation.status.state === "live" ? (
          <ul className={styles.objects}>
            <li>
              <SpatialObject
                id="cases"
                label={`Cases · ${orientation.caseCount ?? "unknown"}`}
                onFocus={(id) => focusController.focus({ kind: "object", objectId: id })}
              />
              <Relationship fromId="core" toId="cases" label="horizon" />
            </li>
            <li>
              <SpatialObject
                id="inbox"
                label={`Judgment · ${orientation.pendingApprovals ?? "unknown"} pending`}
                onFocus={(id) => focusController.focus({ kind: "object", objectId: id })}
              />
              <Relationship fromId="core" toId="inbox" label="awaits decision" />
            </li>
            {orientation.unreadableCases !== null && orientation.unreadableCases > 0 ? (
              <li>
                <SpatialObject
                  id="integrity"
                  label={`Integrity · ${orientation.unreadableCases} unreadable`}
                  onFocus={(id) => focusController.focus({ kind: "object", objectId: id })}
                />
                <Relationship fromId="core" toId="integrity" label="needs attention" />
              </li>
            ) : null}
          </ul>
        ) : null}
      </div>
    </Surface>
  );
}
