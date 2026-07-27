import { useState } from "react";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import type { EventFrame, RequestRecord } from "@sea-forge/contracts";
import { useEvidenceContext } from "../shell/EvidenceContext";
import { toError } from "../hooks/bridgeError";
import {
  settlementOf,
  useOperationsStream,
  useRequestStatus,
} from "../hooks/useOperationsStream";
import styles from "./SurfacesPages.module.css";

/**
 * Event kinds the kernel actually emits, mapped to how each reads as governed
 * standing. Anything absent renders `unknown` — a kind this renderer has never
 * seen must never be presented as a success (Task 4 gate).
 */
const KNOWN_EVENT_KINDS: Record<string, { variant: string; label: string }> = {
  "case.submitted": { variant: "ready", label: "Case submitted" },
  "approval.approved": { variant: "ready", label: "Approval granted" },
  "approval.rejected": { variant: "blocked", label: "Approval rejected" },
  "agent_run.delegated": { variant: "ready", label: "Delegated" },
  "agent_run.cancellation_requested": {
    variant: "degraded",
    label: "Cancellation requested",
  },
};

function standingFor(kind: string) {
  return KNOWN_EVENT_KINDS[kind] ?? { variant: "unknown", label: "Unrecognised kind" };
}

/** Execution lifecycle, straight off the correlation record. */
const EXECUTION_STANDING: Record<RequestRecord["status"], { variant: string; label: string }> = {
  pending: { variant: "pending", label: "Execution in flight" },
  completed: { variant: "ready", label: "Execution succeeded" },
  failed: { variant: "blocked", label: "Execution failed" },
};

const SETTLEMENT_STANDING = {
  not_projected: { variant: "unknown", label: "Settlement not projected" },
  rejected: { variant: "blocked", label: "Settlement rejected" },
  settled: { variant: "ready", label: "Settled" },
} as const;

export function OperationsPage() {
  const { inspectEvidence } = useEvidenceContext();
  const { frames, cursor, stale, error, isLoading, refresh } = useOperationsStream();
  // ponytail: local state, not a route search param — the correlation id is a
  // transient inspection, not a shareable location. Promote to `?request=` if
  // operators start needing to link to one.
  const [requestId, setRequestId] = useState("");
  const [tracked, setTracked] = useState<string | undefined>(undefined);
  const status = useRequestStatus(tracked);

  const record = status.data;
  const settlement = settlementOf(record);
  const execution = record ? EXECUTION_STANDING[record.status] : undefined;

  // `stale` is checked before `error`: a failed re-read while frames are already
  // held is degraded (last-known state, still useful), not unavailable. Testing
  // `error` first would blank a view the operator can still legitimately read.
  const streamStanding = stale
    ? { variant: "degraded", label: "Last known — reconnecting" }
    : error
      ? { variant: "blocked", label: "Stream unavailable" }
      : isLoading
        ? { variant: "pending", label: "Reading ledger" }
        : { variant: "ready", label: "Live from durable ledger" };

  const inspectFrame = (frame: EventFrame) =>
    inspectEvidence({
      id: `event.${frame.cursor}`,
      kind: frame.kind,
      disclosureStatus: "permitted",
      rawPayload: JSON.stringify(frame, null, 2),
    });

  return (
    <div className={styles.page}>
      <section className="governed-focus" data-od-id="operations" data-state={streamStanding.variant}>
        <div className="focus-copy">
          <div className="title-line">
            <h1>Operations monitor</h1>
            <GovernedStatusPill
              variant={streamStanding.variant}
              label={streamStanding.label}
              className="status-pill"
            />
          </div>
          <p>
            Durable event frames read from the events ledger. The live subscription is
            only a signal to re-read — every frame shown here was read back
            authoritatively, so a dropped notification cannot silently lose one.
          </p>
          <div className="focus-meta">
            <span className="machine-value">
              {cursor ? `cursor ${cursor}` : "no cursor yet"}
            </span>
            <span className="machine-value">{frames.length} frames held</span>
          </div>
        </div>
        <div className="focus-actions">
          <button className="button" type="button" onClick={refresh}>
            Re-read ledger
          </button>
        </div>
      </section>

      <div className="content-grid">
        <div className="content-primary">
          <section className="panel" data-od-id="execution-console">
            <div className="section-header">
              <div>
                <p className="section-kicker">Governed event stream</p>
                <h2>Durable frames</h2>
              </div>
            </div>

            {error && !stale ? (
              <p className="operational-copy" role="alert">
                The events ledger could not be read: {error.message}. No frames are shown
                because none can be evidenced.
              </p>
            ) : isLoading ? (
              <p className="operational-copy">Reading the events ledger…</p>
            ) : frames.length === 0 ? (
              <p className="operational-copy">
                The events ledger holds no frames yet. Submitting a case appends the first.
              </p>
            ) : (
              <div className="event-stream">
                {stale ? (
                  <div className="event-stale">
                    <span className="machine-value">STALE</span>
                    <strong>Last authoritative read failed. Reconnecting…</strong>
                    <small>
                      Frames below remain the last confirmed durable state, not current
                      state.
                    </small>
                  </div>
                ) : null}
                {[...frames].reverse().map((frame) => {
                  const standing = standingFor(frame.kind);
                  return (
                    <button
                      key={frame.cursor}
                      type="button"
                      className="event-frame"
                      onClick={() => inspectFrame(frame)}
                    >
                      <span className="machine-value">{frame.cursor}</span>
                      <strong>{frame.kind}</strong>
                      <GovernedStatusPill
                        variant={standing.variant}
                        label={standing.label}
                        className="status-pill"
                      />
                      <small>
                        {frame.committed_at}
                        {frame.case_id ? ` · case ${frame.case_id}` : ""}
                        {frame.run_id ? ` · run ${frame.run_id}` : ""}
                      </small>
                    </button>
                  );
                })}
              </div>
            )}
          </section>

          <section className="panel" data-od-id="settlement-preview">
            <div className="section-header">
              <div>
                <p className="section-kicker">Request correlation</p>
                <h2>Execution and settlement</h2>
              </div>
            </div>

            <label className="field-label" htmlFor="requestId">
              Request id
            </label>
            <div className="correlation-row">
              <input
                id="requestId"
                className="workbench-input"
                value={requestId}
                placeholder="request id from a submitted command"
                onChange={(e) => setRequestId(e.target.value)}
              />
              <button
                className="button button--primary"
                type="button"
                disabled={requestId.trim() === ""}
                onClick={() => setTracked(requestId.trim())}
              >
                Inspect
              </button>
            </div>

            {status.isError ? (
              <p className="operational-copy" role="alert">
                No correlation record could be read: {toError(status.error).message}
              </p>
            ) : status.isLoading && tracked ? (
              <p className="operational-copy">Reading correlation record…</p>
            ) : record ? (
              <>
                {/*
                  Execution and settlement are deliberately two pills. A request can
                  execute successfully and still settle as rejected; collapsing them
                  would report the rejection as a success.
                */}
                <div className="dual-state">
                  <div>
                    <span>Execution</span>
                    <GovernedStatusPill
                      variant={execution?.variant}
                      label={execution?.label ?? "Unrecognised status"}
                      className="status-pill"
                    />
                  </div>
                  <div>
                    <span>Settlement</span>
                    <GovernedStatusPill
                      variant={SETTLEMENT_STANDING[settlement.kind].variant}
                      label={SETTLEMENT_STANDING[settlement.kind].label}
                      className="status-pill"
                    />
                  </div>
                </div>
                <p className="operational-copy">{settlement.reason}</p>
                <dl className="detail-grid">
                  <div>
                    <dt>Method</dt>
                    <dd className="machine-value">{record.method}</dd>
                  </div>
                  <div>
                    <dt>Submitted</dt>
                    <dd className="machine-value">{record.submitted_at}</dd>
                  </div>
                  {record.completed_at ? (
                    <div>
                      <dt>Completed</dt>
                      <dd className="machine-value">{record.completed_at}</dd>
                    </div>
                  ) : null}
                </dl>
                <button
                  className="button"
                  type="button"
                  onClick={() =>
                    inspectEvidence({
                      id: `request.${record.request_id}`,
                      kind: "request_record",
                      disclosureStatus: "permitted",
                      rawPayload: JSON.stringify(record, null, 2),
                    })
                  }
                >
                  Inspect correlation record
                </button>
              </>
            ) : (
              <p className="operational-copy">
                Enter the request id returned by a submitted command to read its
                authoritative correlation record.
              </p>
            )}
          </section>
        </div>

        <aside className="attention-rail" aria-label="Operations attention">
          <section className="panel" data-od-id="operation-control">
            <div className="section-header">
              <div>
                <p className="section-kicker">Recovery</p>
                <h2>Gaps are not silent</h2>
              </div>
            </div>
            <p>
              Frames are read from the durable ledger after the cursor already held, and
              paged to the tail. A missed live notification changes nothing: the next
              read returns the frames it would have carried.
            </p>
            <p className="machine-value">
              {cursor ? `last cursor ${cursor}` : "awaiting first frame"}
            </p>
          </section>
        </aside>
      </div>
    </div>
  );
}
