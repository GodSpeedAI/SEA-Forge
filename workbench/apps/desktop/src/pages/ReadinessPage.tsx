import { useMemo, useState } from "react";
import {
  GovernedStatusPill,
  IntegrityIndicator,
  ProtectedActionButton,
  SourceFreshnessBadge,
  type EvidenceRecord,
  type FreshnessMode,
  type GovernedStatusVariant,
  type IntegrityStatus,
} from "@sea-forge/ui-components";
import type { ReadinessItem, ReadinessView } from "@sea-forge/contracts";
import { useNavigate } from "@tanstack/react-router";
import { useReadiness } from "../hooks/useReadiness";
import { useEvidenceContext } from "../shell/EvidenceContext";
import styles from "./ReadinessPage.module.css";

type Operation = "local" | "agent" | "federation";

const OPERATION_METHODS: Record<Operation, string> = {
  local: "case.create",
  agent: "agent_run.start",
  federation: "federation.bundle.import",
};

const OPERATION_EXPLANATIONS: Record<Operation, string> = {
  local: "Local governed execution is spendable for this actor and policy context.",
  agent:
    "This path needs a verified endpoint and an allowed authority boundary before commitment.",
  federation:
    "Federation import inspects and stages compatible bundles; it does not adopt them automatically.",
};

function integrityStatusFor(view: ReadinessView): IntegrityStatus {
  if (view.overall === "integrity_halted") return "compromised";
  const foundation = view.foundations.find((item) => item.id === "self_model_integrity");
  if (!foundation) return "unverified";
  if (foundation.status === "ready") return "verified";
  if (foundation.status === "blocked") return "compromised";
  return "unverified";
}

function statusVariant(status: ReadinessItem["status"]): GovernedStatusVariant {
  if (status === "ready" || status === "degraded" || status === "blocked") {
    return status;
  }
  return "unknown";
}

function overallVariant(overall: ReadinessView["overall"] | "unknown"): GovernedStatusVariant {
  if (overall === "ready") return "ready";
  if (overall === "ready_with_limitations" || overall === "stale") return "degraded";
  if (overall === "blocked") return "blocked";
  if (overall === "integrity_halted") return "integrity_halted";
  return "unknown";
}

function overallLabel(overall: ReadinessView["overall"] | "unknown"): string {
  const labels: Record<typeof overall, string> = {
    ready: "Readiness ready",
    ready_with_limitations: "Review required",
    blocked: "Readiness blocked",
    integrity_halted: "Integrity halted",
    stale: "Readiness stale",
    unknown: "Readiness unknown",
  };
  return labels[overall];
}

function itemStatusLabel(item: ReadinessItem): string {
  if (item.status === "ready") {
    return item.category === "operational_capability"
      ? "Capability proven"
      : item.id === "self_model_integrity"
        ? "Integrity verified"
        : "Readiness ready";
  }
  if (item.status === "degraded") {
    return item.category === "operational_capability"
      ? "Capability degraded"
      : "Readiness degraded";
  }
  if (item.status === "blocked") return "Readiness blocked";
  return "Readiness unknown";
}

function itemDetail(item: ReadinessItem): string {
  if (item.id === "self_model_integrity") return "Ledger and local state";
  if (item.id.includes("actor")) return "Identity and sponsorship";
  if (item.id.includes("delegation") || item.id.includes("endpoint")) {
    return "External delegation";
  }
  return item.category.replace(/_/g, " ");
}

function canCreateCase(view: ReadinessView): boolean {
  return (
    view.foundations.every((item) => item.status === "ready") &&
    view.operational_capabilities.find((item) => item.id === "local_governed_execution")
      ?.status === "ready"
  );
}

function caseCreationReason(view: ReadinessView): string {
  const blocker = [...view.foundations, ...view.operational_capabilities].find(
    (item) => item.status !== "ready",
  );
  return blocker?.reason || "Readiness conditions are not met";
}

export function ReadinessPage() {
  const navigate = useNavigate();
  const [selectedOperation, setSelectedOperation] = useState<Operation>("agent");
  const intendedOperation = useMemo(
    () => ({ method: OPERATION_METHODS[selectedOperation] }),
    [selectedOperation],
  );
  const { query } = useReadiness(intendedOperation);
  const { inspectEvidence } = useEvidenceContext();

  const view = query.data;
  const foundations = view?.foundations ?? [];
  const capabilities = view?.operational_capabilities ?? [];
  const overall = view?.overall ?? "unknown";
  const limitation = [...foundations, ...capabilities].find(
    (item) => item.status !== "ready",
  );

  const freshnessMode: FreshnessMode = useMemo(() => {
    if (query.isFetching && view) return "cached";
    if (query.isFetching && !view) return "checking";
    if (query.isError && view) return "stale";
    if (query.isError && !view) return "offline";
    return "live";
  }, [query.isError, query.isFetching, view]);

  const integrityStatus: IntegrityStatus = view
    ? integrityStatusFor(view)
    : query.isFetching
      ? "checking"
      : "unverified";

  const readyFoundationCount = foundations.filter((item) => item.status === "ready").length;
  const foundationLimitationCount = foundations.length - readyFoundationCount;

  const focusReason = view
    ? limitation?.reason ||
      "All projected foundations and operational capabilities are currently ready."
    : query.isFetching
      ? "Loading the source-backed readiness projection."
      : "Readiness projection unavailable. Reconnect to inspect current conditions.";

  function inspectSource(sourceRef: string) {
    const evidence: EvidenceRecord = {
      id: sourceRef,
      kind: "readiness_source_citation",
      disclosureStatus: "restricted",
    };
    inspectEvidence(evidence);
  }

  function handleWhyThisState() {
    const reasonText =
      view?.overall === "ready"
        ? "All critical foundations and required capabilities have passed verification."
        : foundations.find((f) => f.status !== "ready")?.reason ||
          "External agent delegation requires endpoint verification before commitment.";

    inspectEvidence({
      id: "readiness.get",
      kind: "readiness_evaluation_summary",
      disclosureStatus: "permitted",
      rawPayload: JSON.stringify(
        {
          overall,
          intended_operation: selectedOperation,
          primary_limitation: reasonText,
          foundations_summary: foundations.map((f) => ({
            name: f.name,
            status: f.status,
            reason: f.reason,
            source: f.source_ref,
          })),
        },
        null,
        2
      ),
    });
  }

  return (
    <div
      className={`${styles.page} readiness-page`}
      data-testid="readiness-workspace"
    >
      <section
        className="governed-focus"
        aria-labelledby="readiness-title"
        data-od-id="readiness-governed-focus"
        data-state={overall === "ready" ? "ready" : "degraded"}
      >
        <div className="focus-copy">
          <div className="title-line">
            <h1 id="readiness-title">Readiness Overview</h1>
            <GovernedStatusPill
              variant={overallVariant(overall)}
              label={overallLabel(overall)}
              className="status-pill focus-status"
            />
          </div>
          <p id="readinessReason">{focusReason}</p>
          <div className="focus-meta">
            <SourceFreshnessBadge
              mode={freshnessMode}
              sourceName="readiness.get"
              className="freshness-badge"
              onClick={() => void query.refetch()}
            />
            <span className="machine-value">
              Projection: readiness / {OPERATION_METHODS[selectedOperation]}
            </span>
            <button
              className="freshness-badge"
              type="button"
              onClick={() => inspectSource("readiness.get")}
            >
              Inspect source
            </button>
          </div>
        </div>
        <div className="focus-actions">
          <button
            className="button button--secondary"
            id="inspectReasonButton"
            type="button"
            onClick={handleWhyThisState}
          >
            Why this state
          </button>
          <button
            className="button button--primary"
            id="runChecksButton"
            type="button"
            onClick={() => void query.refetch()}
          >
            Run checks
          </button>
        </div>
      </section>

      <div className="content-grid">
        <div className="content-primary">
          <section
            className="panel operation-selector"
            aria-labelledby="operation-title"
            data-od-id="intended-work-selector"
          >
            <div className="section-header">
              <div>
                <p className="section-kicker">Operation-sensitive readiness</p>
                <h2 id="operation-title">What do you intend to do?</h2>
              </div>
              <kbd>I</kbd>
            </div>
            <div className="segmented-control" role="radiogroup" aria-label="Intended operation">
              <button
                className={`segment ${selectedOperation === "local" ? "segment--active" : ""}`}
                type="button"
                role="radio"
                aria-checked={selectedOperation === "local"}
                onClick={() => setSelectedOperation("local")}
              >
                Run local governed work
              </button>
              <button
                className={`segment ${selectedOperation === "agent" ? "segment--active" : ""}`}
                type="button"
                role="radio"
                aria-checked={selectedOperation === "agent"}
                onClick={() => setSelectedOperation("agent")}
              >
                Delegate to external agent
              </button>
              <button
                className={`segment ${selectedOperation === "federation" ? "segment--active" : ""}`}
                type="button"
                role="radio"
                aria-checked={selectedOperation === "federation"}
                onClick={() => setSelectedOperation("federation")}
              >
                Import governed bundle
              </button>
            </div>
            <p id="operationExplanation" className="operation-explanation">
              {OPERATION_EXPLANATIONS[selectedOperation]}
            </p>
          </section>

          <section className="panel" aria-labelledby="foundations-title" data-od-id="critical-foundations">
            <div className="section-header">
              <div>
                <p className="section-kicker">Required conditions</p>
                <h2 id="foundations-title">Critical foundations</h2>
              </div>
              <span className="section-summary">
                {readyFoundationCount} verified · {foundationLimitationCount}{" "}
                {foundationLimitationCount === 1 ? "limitation" : "limitations"}
              </span>
            </div>
            <div className="condition-table" role="table" aria-label="Critical foundations">
              <div className="condition-row condition-row--header" role="row">
                <span role="columnheader">Foundation</span>
                <span role="columnheader">State</span>
                <span role="columnheader">Reason</span>
                <span role="columnheader">Source</span>
              </div>
              {foundations.map((f) => {
                const isDegraded = f.status === "degraded";
                const pillClass =
                  f.status === "ready"
                    ? "status-pill--ready"
                    : f.status === "degraded"
                    ? "status-pill--degraded"
                    : "status-pill--blocked";
                return (
                  <div
                    key={f.id}
                    className={`condition-row ${isDegraded ? "condition-row--attention" : ""}`}
                    role="row"
                  >
                    <span role="cell">
                      <button
                        className={styles.conditionTrigger}
                        type="button"
                        aria-label={`Inspect evidence for ${f.name}`}
                        onClick={() => inspectSource(f.source_ref)}
                      >
                        <strong>{f.name}</strong>
                        <small>{itemDetail(f)}</small>
                      </button>
                    </span>
                    <span role="cell">
                      {f.id === "self_model_integrity" ? (
                        <IntegrityIndicator
                          status={integrityStatus}
                          details={f.reason || undefined}
                          className="status-pill"
                        />
                      ) : (
                        <GovernedStatusPill
                          variant={statusVariant(f.status)}
                          label={itemStatusLabel(f)}
                          className={`status-pill ${pillClass}`}
                        />
                      )}
                    </span>
                    <span role="cell">{f.reason}</span>
                    <span role="cell" className="machine-value">
                      {f.source_ref}
                    </span>
                  </div>
                );
              })}
            </div>
          </section>

          <section className="panel" aria-labelledby="capabilities-title" data-od-id="operational-capabilities">
            <div className="section-header">
              <div>
                <p className="section-kicker">Current affordances</p>
                <h2 id="capabilities-title">Operational capabilities</h2>
              </div>
              <button className="text-button" type="button">
                Inspect all capabilities
              </button>
            </div>
            <div className="capability-list">
              {capabilities.map((c) => {
                const pillClass = c.status === "ready" ? "status-pill--ready" : "status-pill--degraded";
                const pillText = c.status === "ready" ? "Capability proven" : "Capability degraded";
                return (
                  <div key={c.id} className="capability-row">
                    <div>
                      <strong>{c.name}</strong>
                      <span>{c.reason}</span>
                    </div>
                    <GovernedStatusPill
                      variant={statusVariant(c.status)}
                      label={pillText}
                      className={`status-pill ${pillClass}`}
                    />
                    <button
                      className="row-action"
                      type="button"
                      aria-label={`Inspect evidence for ${c.name}`}
                      onClick={() => inspectSource(c.source_ref)}
                    >
                      {c.status === "degraded" ? "Repair path" : "Inspect"}
                    </button>
                  </div>
                );
              })}
            </div>
          </section>
        </div>

        <aside className="attention-rail" aria-labelledby="attention-title" data-od-id="current-affordance-rail">
          <section className="panel panel--attention">
            <p className="section-kicker">Current limitation</p>
            <h2 id="attention-title">Endpoint verification required</h2>
            <p>
              {limitation?.reason ||
                "No active limitation is present in the current readiness projection."}
            </p>
            <dl className="key-values">
              <div>
                <dt>Affects</dt>
                <dd>{selectedOperation === "agent" ? "Agent task start" : "Intended operation"}</dd>
              </div>
              <div>
                <dt>Authority</dt>
                <dd>Operator may probe</dd>
              </div>
              <div>
                <dt>Settlement</dt>
                <dd>Not yet applicable</dd>
              </div>
            </dl>
            <button
              className="button button--attention"
              type="button"
              onClick={() => inspectSource(limitation?.source_ref ?? "readiness.get")}
            >
              Open limitation evidence
            </button>
          </section>

          <section className="panel action-panel">
            <p className="section-kicker">Next lawful action</p>
            <h2>Create case</h2>
            <ProtectedActionButton
              label="Create case"
              onClick={() => void navigate({ to: "/cases/new" })}
              isAllowed={view ? canCreateCase(view) : false}
              disabledReason={
                view ? caseCreationReason(view) : "Readiness projection unavailable"
              }
              variant="primary"
              className="lawful-action"
            />
          </section>

          <section className="panel recent-panel">
            <div className="section-header">
              <h2>Recent invalidations</h2>
              <span className="status-pill status-pill--neutral">
                {view?.recent_invalidations.length ?? 0}
              </span>
            </div>
            <p>
              {view?.recent_invalidations.length
                ? `${view.recent_invalidations.length} recent invalidation records require inspection.`
                : "No verified state has been invalidated in this projection."}
            </p>
          </section>
        </aside>
      </div>
    </div>
  );
}
