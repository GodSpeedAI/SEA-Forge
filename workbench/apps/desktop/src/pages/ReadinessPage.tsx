import { useMemo, useState } from "react";
import {
  WhyStatePanel,
  GovernedStatusPill,
  IntegrityIndicator,
  SourceFreshnessBadge,
  ProtectedActionButton,
  EvidenceDrawer,
  type StateCondition,
  type IntegrityStatus,
  type FreshnessMode,
  type EvidenceRecord,
} from "@sea-forge/ui-components";
import type { ReadinessItem, ReadinessView } from "@sea-forge/contracts";
import { useReadiness } from "../hooks/useReadiness";

/** Map a readiness item status onto the WhyStatePanel condition tri-state. */
function conditionStatus(status: ReadinessItem["status"]): StateCondition["status"] {
  switch (status) {
    case "ready":
      return "pass";
    case "degraded":
      return "warn";
    default:
      // "blocked" | "unknown" — both surface as a failing condition here.
      return "fail";
  }
}

function toCondition(item: ReadinessItem): StateCondition {
  return {
    id: item.id,
    name: item.name,
    status: conditionStatus(item.status),
    explanation:
      item.reason && item.reason.length > 0
        ? `${item.reason} (source: ${item.source_ref})`
        : `Satisfied (source: ${item.source_ref})`,
  };
}

/**
 * Select the worst condition status across a list of readiness items, in the
 * sense the WhyStatePanel pill renders: blocked > degraded > unknown > ready.
 * Empty list → "unknown" (no evidence either way is still not "ready").
 */
function worstStatus(items: ReadinessItem[]): ReadinessItem["status"] {
  if (items.length === 0) return "unknown";
  const rank: Record<ReadinessItem["status"], number> = {
    blocked: 3,
    degraded: 2,
    unknown: 1,
    ready: 0,
  };
  return items.reduce<ReadinessItem["status"]>(
    (worst, item) => (rank[item.status] > rank[worst] ? item.status : worst),
    "ready",
  );
}

/** Integrity indicator status derived from the self-model foundation + overall. */
function integrityStatusFor(view: ReadinessView): IntegrityStatus {
  if (view.overall === "integrity_halted") return "compromised";
  const foundation = view.foundations.find((f) => f.id === "self_model_integrity");
  if (!foundation) return "unverified";
  if (foundation.status === "ready") return "verified";
  if (foundation.status === "blocked") return "compromised";
  return "unverified";
}

/**
 * The case-creation action never functions in this slice — no `case.create`
 * SFWP verb is wired yet (Task 6). It is rendered permanently disabled with an
 * honest reason so the surface never implies a working authoring entry point.
 */
function caseCreationReason(view: ReadinessView): string {
  if (view.overall === "ready") {
    return "Case authoring is not yet available in this build";
  }
  const blocker = [...view.foundations, ...view.operational_capabilities].find(
    (i) => i.status !== "ready",
  );
  return blocker?.reason && blocker.reason.length > 0
    ? blocker.reason
    : "Readiness conditions are not met";
}

export function ReadinessPage() {
  const { query } = useReadiness();
  const [evidence, setEvidence] = useState<EvidenceRecord | null>(null);

  // Last-known-good view: TanStack Query keeps `data` across a failed refetch,
  // so a mid-view server disconnect renders the stale view instead of blanking.
  const view = query.data;

  const freshnessMode: FreshnessMode = useMemo(() => {
    if (query.isFetching && view) return "cached"; // refetch in flight over prior data
    if (query.isFetching && !view) return "checking"; // initial fetch, no view yet
    if (query.isError && view) return "stale"; // last-known view, source unreachable
    if (query.isError && !view) return "offline"; // never loaded and unreachable
    return "live"; // fresh successful read
  }, [query.isError, query.isFetching, view]);

  const overall = view?.overall ?? "unknown";
  const integrityStatus: IntegrityStatus = view
    ? integrityStatusFor(view)
    : query.isFetching
      ? "checking" // initial fetch in progress, no view yet
      : "unverified"; // fetch finished (failed or unavailable) without a view

  const foundationConditions = view?.foundations.map(toCondition) ?? [];
  const capabilityConditions = view?.operational_capabilities.map(toCondition) ?? [];

  const summary = view
    ? `Overall readiness: ${overall.replace(/_/g, " ")}. ` +
      `${view.operational_capabilities.length} operational capabilities projected` +
      (view.intended_operation
        ? ` for intended operation "${view.intended_operation.method}".`
        : " (no intended operation).")
    : query.isLoading
      ? "Loading readiness projection…"
      : "Readiness projection unavailable.";

  function inspectSource(sourceRef: string) {
    // This slice has no content-addressed evidence records — `source_ref` is a
    // citation string. We surface it in the drawer as a restricted-disclosure
    // citation rather than fabricating a retrievable payload.
    setEvidence({
      id: sourceRef,
      kind: "readiness_source_citation",
      disclosureStatus: "restricted",
    });
  }

  return (
    <div
      data-testid="readiness-workspace"
      style={{
        padding: "var(--space-6, 24px)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--space-4, 16px)",
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 style={{ font: "var(--text-page-title, 24px / 1.2 system-ui, sans-serif)", margin: 0 }}>
          Readiness Console
        </h1>
        <div style={{ display: "flex", gap: "var(--space-3, 12px)", alignItems: "center" }}>
          <SourceFreshnessBadge
            mode={freshnessMode}
            sourceName="readiness.get"
            onClick={() => void query.refetch()}
          />
          <GovernedStatusPill variant={overall} />
        </div>
      </div>

      <IntegrityIndicator
        status={integrityStatus}
        details={view?.foundations.find((f) => f.id === "self_model_integrity")?.reason || undefined}
      />

      <section
        aria-labelledby="readiness-conditions-heading"
        style={{ display: "flex", flexDirection: "column", gap: "var(--space-4, 16px)" }}
      >
        {/* h2 keeps a valid h1 → h2 → h3 heading order (the panels render h3). */}
        <h2
          id="readiness-conditions-heading"
          style={{ font: "var(--text-section-title, 18px / 1.3 system-ui, sans-serif)", margin: 0 }}
        >
          Readiness conditions
        </h2>

        <WhyStatePanel
          title="Foundations"
          statusVariant={view ? worstStatus(view.foundations) : "unknown"}
          summary={summary}
          conditions={foundationConditions}
          evidenceRef={view?.foundations[0]?.source_ref}
          onInspectEvidence={inspectSource}
        />

        <WhyStatePanel
          title="Operational capabilities"
          statusVariant={view ? worstStatus(view.operational_capabilities) : "unknown"}
          summary={
            view
              ? "Capabilities are ordered by operation-sensitivity; the capability the intended operation depends on is foregrounded first."
              : summary
          }
          conditions={capabilityConditions}
          evidenceRef={view?.operational_capabilities[0]?.source_ref}
          onInspectEvidence={inspectSource}
        />
      </section>

      {view && view.recent_invalidations.length === 0 && (
        <p
          data-testid="recent-invalidations-empty"
          style={{ color: "var(--fg-secondary, #94A3B8)", margin: 0, fontSize: "0.875rem" }}
        >
          No readiness invalidations recorded yet.
        </p>
      )}

      <div>
        <ProtectedActionButton
          label="Create case"
          onClick={() => {
            /* Intentionally inert: no case-authoring verb exists in this slice. */
          }}
          isAllowed={false}
          disabledReason={view ? caseCreationReason(view) : "Readiness projection unavailable"}
          variant="primary"
        />
      </div>

      <EvidenceDrawer
        isOpen={evidence !== null}
        onClose={() => setEvidence(null)}
        evidence={evidence}
      />
    </div>
  );
}
