import type { ReactNode } from "react";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import { useEvidenceContext } from "../shell/EvidenceContext";
import { UnbackedSurface } from "./UnbackedSurface";
import {
  STANDING_PRESENTATION,
  standingOf,
  useServerContract,
} from "../hooks/useServerContract";
import styles from "./SurfacesPages.module.css";
import {
  OPERATE_ROUTES,
  type OperateRouteDefinition as RouteContext,
  type OperateRouteState as RouteState,
} from "./operateRoutes";

function routeEvidence(route: RouteContext, detail = route.reason) {
  return {
    id: `${route.id}.state`,
    kind: `operate_${route.id}_state`,
    disclosureStatus: "permitted" as const,
    rawPayload: JSON.stringify(
      {
        route: route.title,
        reason: detail,
        display: "copied specification projection",
      },
      null,
      2,
    ),
  };
}

/**
 * Every pill inside a specimen surface renders `unknown`, whatever the layout
 * asks for.
 *
 * This is the single choke point where the copied specification's illustrative
 * states would otherwise become governance claims. The layouts below are the
 * design target and are worth keeping, but the states they depict were written
 * to show what the screens will look like — not read from any record. A pill
 * saying `ready` here would be indistinguishable from one backed by evidence,
 * so the label survives to carry the layout's meaning while the variant tells
 * the truth: this cell knows nothing about it.
 */
function Status({
  state,
  label,
}: {
  /** Retained so the specimen layouts stay self-documenting; never rendered. */
  state: RouteState | "pending" | "unknown";
  label: string;
}) {
  void state;
  return <GovernedStatusPill variant="unknown" label={label} className="status-pill" />;
}

function InspectButton({
  route,
  children,
  className = "button",
  detail,
}: {
  route: RouteContext;
  children: ReactNode;
  className?: string;
  detail?: string;
}) {
  const { inspectEvidence } = useEvidenceContext();
  return (
    <button
      className={className}
      type="button"
      onClick={() => inspectEvidence(routeEvidence(route, detail))}
    >
      {children}
    </button>
  );
}

function GovernedFocus({ route }: { route: RouteContext }) {
  const { contract } = useServerContract();
  const standing = standingOf(route.method, contract);
  const presentation = STANDING_PRESENTATION[standing];

  return (
    <section className="governed-focus" data-od-id={route.id} data-state="unknown">
      <div className="focus-copy">
        <div className="title-line">
          <h1>{route.title}</h1>
          <Status state="unknown" label="○ Specification preview" />
        </div>
        <p>{presentation.explanation}</p>
        <p className="operational-copy">Reference scenario: {route.reason}</p>
        <div className="focus-meta">
          <InspectButton route={route} className="freshness-badge">
            <span className="state-dot" />
            Copied specification · not live
          </InspectButton>
          <span className="machine-value">
            requires {route.method} · {presentation.label.toLowerCase()}
          </span>
        </div>
      </div>
      <div className="focus-actions">
        <InspectButton route={route}>Why this state</InspectButton>
        <InspectButton route={route} className="button button--primary">
          {route.action}
        </InspectButton>
      </div>
    </section>
  );
}

function Panel({
  kicker,
  title,
  id,
  children,
  className = "",
}: {
  kicker: string;
  title: string;
  id: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`panel ${className}`.trim()} data-od-id={id}>
      <div className="section-header">
        <div>
          <p className="section-kicker">{kicker}</p>
          <h2>{title}</h2>
        </div>
      </div>
      {children}
    </section>
  );
}

function RouteLayout({
  route,
  primary,
  attention,
  className = "",
}: {
  route: RouteContext;
  primary: ReactNode;
  attention: ReactNode;
  className?: string;
}) {
  // `data-specimen` marks the region whose contents are illustrative. The CSS
  // watermark makes that legible at a glance, so an operator never has to infer
  // it from the header alone.
  return (
    <div className={styles.page} data-specimen="true">
      <GovernedFocus route={route} />
      <div className={`content-grid ${className}`.trim()}>
        {primary}
        <aside className="attention-rail" aria-label={`${route.title} attention`}>
          {attention}
        </aside>
      </div>
    </div>
  );
}

export function ThothPage() {
  const route = OPERATE_ROUTES.thoth;
  return (
    <RouteLayout
      route={route}
      primary={
        <div className="content-primary">
          <Panel
            kicker="Grounded question"
            title="What should be inspected?"
            id="thoth-question-composer"
          >
            <label className="field-label" htmlFor="thothQuestion">
              Question
            </label>
            <textarea
              id="thothQuestion"
              className="workbench-input"
              rows={3}
              defaultValue="What is missing before external agent delegation is spendable?"
            />
            <div className="source-chip-row">
              <span className="machine-value">
                3 selected sources · disclosure required
              </span>
            </div>
          </Panel>
          <Panel kicker="Answer detail" title="Grounded answer" id="thoth-answer-detail">
            <p className="operational-copy">
              Endpoint verification and an allowed authority boundary are both
              required before delegation becomes spendable. The current projection
              records no endpoint-verification evidence.
            </p>
            <InspectButton route={route} className="evidence-link">
              <strong>Readiness screen contract</strong>
              <span>Primary path §3</span>
            </InspectButton>
            <InspectButton route={route} className="evidence-link">
              <strong>Authority boundary</strong>
              <span>UX epic §4.6</span>
            </InspectButton>
          </Panel>
        </div>
      }
      attention={
        <Panel kicker="Answer standing" title="Not a decision" id="thoth-standing">
          <p>
            This answer is an inspection aid. It does not approve, commit, or execute
            work.
          </p>
        </Panel>
      }
    />
  );
}

export function AssetsPage() {
  const route = OPERATE_ROUTES.assets;
  const assets = [
    ["repair.tests", "ready", "Available", "Compatible with current policy", "JUnit manifest"],
    ["agent-kit/repair", "unknown", "Installed", "Endpoint validation stale", "Install record"],
    [
      "external/triage-bundle",
      "degraded",
      "Declared",
      "Inspection required before adoption",
      "Bundle digest",
    ],
  ] as const;

  return (
    <RouteLayout
      route={route}
      primary={
        <div className="content-primary">
          <Panel kicker="Availability ladder" title="Governed assets" id="asset-catalog-table">
            <div className="asset-table" aria-label="Governed assets">
              <div className="asset-row asset-row--header" aria-hidden="true">
                <span>Name</span>
                <span>Availability</span>
                <span>Compatibility</span>
                <span>Evidence</span>
              </div>
              {assets.map(([name, state, label, compatibility, evidence]) => (
                <InspectButton
                  key={name}
                  route={route}
                  className="asset-row"
                  detail={`${name}: ${compatibility}`}
                >
                  <strong className="machine-value">{name}</strong>
                  <Status state={state} label={label} />
                  <span>{compatibility}</span>
                  <span className="machine-value">{evidence}</span>
                </InspectButton>
              ))}
            </div>
          </Panel>
          <Panel kicker="Selected asset" title="repair.tests" id="asset-detail">
            <dl className="detail-grid">
              <div><dt>Version</dt><dd className="machine-value">1.4.2</dd></div>
              <div><dt>Producer</dt><dd>Local test suite</dd></div>
              <div><dt>Compatibility</dt><dd>Policy PB-14</dd></div>
              <div><dt>Evidence</dt><dd>JUnit report required</dd></div>
            </dl>
          </Panel>
        </div>
      }
      attention={
        <Panel kicker="Import boundary" title="External bundle is inert" id="asset-boundary">
          <p>Inspection can stage the bundle. It cannot adopt it automatically.</p>
          <InspectButton route={route}>Why this boundary</InspectButton>
        </Panel>
      }
    />
  );
}

export function ModelsPage() {
  const route = OPERATE_ROUTES.domain;
  return (
    <RouteLayout
      route={route}
      className="workbench-grid"
      primary={
        <>
          <aside className="panel side-list" aria-label="Domain models">
            <p className="section-kicker">Models</p>
            <button className="list-item list-item--active" type="button">
              Repair domain <span>current</span>
            </button>
            <button className="list-item" type="button">Service incident</button>
            <button className="list-item" type="button">Escalation policy</button>
          </aside>
          <div className="content-primary">
            <Panel kicker=".sea source" title="Repair domain" id="domain-model-workbench">
              <pre className="model-editor">{`domain repair {
  outcome recovery_test: all_pass
  evidence junit_report: required
  delegate repair_agent: endpoint_verified
}`}</pre>
            </Panel>
            <Panel kicker="Validation" title="Projection checks" id="model-validation">
              <div className="validation-list">
                <ValidationRow route={route} name="Schema" state="ready" label="Validated">
                  Current model schema is valid.
                </ValidationRow>
                <ValidationRow
                  route={route}
                  name="Concept references"
                  state="ready"
                  label="Resolved"
                >
                  Desired outcome and evidence types are linked.
                </ValidationRow>
                <ValidationRow route={route} name="Agent endpoint" state="degraded" label="Stale">
                  Endpoint configuration requires refresh before execution.
                </ValidationRow>
              </div>
            </Panel>
          </div>
        </>
      }
      attention={
        <Panel
          kicker="Model consequence"
          title="Execution unavailable"
          id="model-consequence"
        >
          <p>The model can be edited and reviewed. It cannot authorize delegation.</p>
          <InspectButton route={route}>Inspect validation evidence</InspectButton>
        </Panel>
      }
    />
  );
}

function ValidationRow({
  route,
  name,
  state,
  label,
  children,
}: {
  route: RouteContext;
  name: string;
  state: RouteState;
  label: string;
  children: ReactNode;
}) {
  return (
    <InspectButton route={route} className="validation-row" detail={`${name}: ${children}`}>
      <strong>{name}</strong>
      <Status state={state} label={label} />
      <span>{children}</span>
    </InspectButton>
  );
}

// Cases and Inbox are no longer specimens. `case.list`/`case.get_overview`/
// `case.get_horizon` and `approval.list`/`approval.decide` are implemented by
// the kernel, so both surfaces read committed records instead of illustrating
// what they would look like. Re-exported under their original names so the
// route table and any importer are unchanged.
export { CaseHorizonPage as CasesPage } from "./CaseHorizonPage";
export { ApprovalInboxPage as InboxPage } from "./ApprovalInboxPage";

// The operations console reads the durable events ledger for real; it lives in
// its own module and is re-exported here so the route table is unchanged.
export { OperationsPage } from "./OperationsPage";

/*
 * Surfaces below have no SFWP method behind them in this cell. Each names the
 * method from the target catalog that would back it and resolves its own
 * standing from `system.hello`, so none of them asserts a state the kernel
 * cannot evidence — and none of them keeps claiming absence after the kernel
 * implements the method.
 */

export function MemoryPage() {
  return (
    <UnbackedSurface
      title="Memory recall"
      purpose="Developmental memory recalled under authority, with the records that constrain what may be retrieved."
      method="memory.recall"
    />
  );
}

export function CapabilitiesPage() {
  return (
    <UnbackedSurface
      title="Capability matrix"
      purpose="Demonstrated capability with its qualifying settlements — what has held under variation, not what was declared."
      method="capability.list"
    />
  );
}

export function ArtifactsPage() {
  return (
    <UnbackedSurface
      title="Artifact registry"
      purpose="Artifact identity, lineage, and maturity, with the producing evidence behind each stage transition."
      method="artifact.list"
    />
  );
}

export function FederationPage() {
  return (
    <UnbackedSurface
      title="Federation gateway"
      purpose="Peer cells, the bundles exchanged with them, and the verification standing of each import before adoption."
      method="federation.preview_export"
    />
  );
}

/**
 * Administration resolves to the one self-description surface the kernel does
 * implement. `system.describe` is live, so this route is genuinely backed
 * rather than a placeholder waiting on a projection.
 */
export { SystemContractPage as AdminPage } from "./SystemContractPage";
