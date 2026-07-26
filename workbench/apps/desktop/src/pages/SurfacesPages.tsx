import type { ReactNode } from "react";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import { useEvidenceContext } from "../shell/EvidenceContext";
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

function Status({
  state,
  label,
}: {
  state: RouteState | "pending" | "unknown";
  label: string;
}) {
  return <GovernedStatusPill variant={state} label={label} className="status-pill" />;
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
  return (
    <section
      className="governed-focus"
      data-od-id={route.id}
      data-state={route.state}
    >
      <div className="focus-copy">
        <div className="title-line">
          <h1>{route.title}</h1>
          <Status state="unknown" label="○ Specification preview" />
        </div>
        <p>Reference scenario: {route.reason}</p>
        <div className="focus-meta">
          <InspectButton route={route} className="freshness-badge">
            <span className="state-dot" />
            Copied specification · not live
          </InspectButton>
          <span className="machine-value">Reference layout: {route.projection}</span>
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
  return (
    <div className={styles.page}>
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

export function CasesPage() {
  const route = OPERATE_ROUTES.cases;
  return (
    <div className={styles.page}>
      <GovernedFocus route={route} />
      <section className="case-toolbar panel">
        <div>
          <p className="section-kicker">Case projections</p>
          <h2>Repair agent for recurring service incidents</h2>
        </div>
        <div className="projection-switch">
          <button className="segment segment--active" type="button">Board</button>
          <button className="segment" type="button">List</button>
          <button className="segment" type="button">Graph</button>
        </div>
      </section>
      <section className="horizon-board" data-od-id="case-horizon-board">
        <HorizonLane route={route} title="Spendable now" card="Local repair recovery" state="ready" status="Ready" />
        <HorizonLane route={route} title="Awaiting authority" card="External agent repair" state="degraded" status="Awaiting evidence" />
        <HorizonLane route={route} title="Blocked" card="Endpoint validation" state="blocked" status="Blocked" />
        <HorizonLane route={route} title="Future" card="Evidence review" state="unknown" status="Parked" />
      </section>
    </div>
  );
}

function HorizonLane({
  route,
  title,
  card,
  state,
  status,
}: {
  route: RouteContext;
  title: string;
  card: string;
  state: RouteState | "unknown";
  status: string;
}) {
  return (
    <section className={`horizon-lane horizon-lane--${state}`}>
      <div><h2>{title}</h2><span className="machine-value">1</span></div>
      <InspectButton route={route} className="case-card" detail={`${card}: ${status}`}>
        <strong>{card}</strong>
        <Status state={state} label={status} />
        <span>Source-backed case projection</span>
      </InspectButton>
    </section>
  );
}

export function InboxPage() {
  const route = OPERATE_ROUTES.inbox;
  return (
    <RouteLayout
      route={route}
      primary={
        <div className="content-primary">
          <Panel
            kicker="Approval APR-07"
            title="Permit external agent endpoint probe"
            id="approval-decision-panel"
          >
            <dl className="detail-grid">
              <div><dt>Requesting actor</dt><dd>Operator</dd></div>
              <div><dt>Sponsor</dt><dd>Service recovery</dd></div>
              <div><dt>Resource</dt><dd className="machine-value">endpoint:repair-agent</dd></div>
              <div><dt>Boundary</dt><dd>Probe only; no task start</dd></div>
              <div><dt>Policy reason</dt><dd>External endpoint lacks current verification</dd></div>
              <div><dt>Separation of duty</dt><dd>Requester may not approve</dd></div>
            </dl>
            <div className="decision-actions">
              <InspectButton route={route}>Inspect evidence</InspectButton>
              <InspectButton route={route} className="button button--attention">
                Escalate for decision
              </InspectButton>
            </div>
          </Panel>
        </div>
      }
      attention={
        <Panel
          kicker="Decision consequence"
          title="No automatic approval"
          id="approval-consequence"
        >
          <p>Approval does not execute the probe. It only permits the bounded request.</p>
          <p className="machine-value">Expires in 1h 58m</p>
        </Panel>
      }
    />
  );
}

export function OperationsPage() {
  const route = OPERATE_ROUTES.operations;
  return (
    <RouteLayout
      route={route}
      primary={
        <div className="content-primary">
          <Panel kicker="Run RUN-041" title="Repair recovery verification" id="execution-console">
            <div className="dual-state">
              <div><span>Execution</span><Status state="ready" label="Execution succeeded" /></div>
              <div><span>Settlement</span><Status state="degraded" label="Settlement evaluating" /></div>
            </div>
            <div className="event-stream">
              <div><span className="machine-value">EVT-1182</span><strong>Test command exited 0</strong><small>Execution evidence captured</small></div>
              <div><span className="machine-value">EVT-1183</span><strong>Settlement evaluator queued</strong><small>Awaiting criterion result</small></div>
              <div className="event-stale"><span className="machine-value">CURSOR</span><strong>Live state may be stale. Reconnecting…</strong><small>Last confirmed event remains visible.</small></div>
            </div>
          </Panel>
          <Panel kicker="Evidence inventory" title="Settlement preview" id="settlement-preview">
            <div className="criterion-row"><strong>Repair recovery test passes</strong><Status state="ready" label="Pass" /><span>JUnit report</span></div>
            <div className="criterion-row"><strong>Qualifying declaration</strong><Status state="degraded" label="Not evaluated" /><span>Human reviewer required</span></div>
          </Panel>
        </div>
      }
      attention={
        <Panel
          kicker="Scoped control"
          title="Cancellation is not required"
          id="operation-control"
        >
          <p>No active process is running. Retry would create a new episode.</p>
          <button className="button button--disabled" type="button" disabled>
            Cancel run
          </button>
        </Panel>
      }
    />
  );
}

function GenericSurfacePage({ title, kind }: { title: string; kind: string }) {
  return (
    <div className={styles.genericPage}>
      <h1>{title} Surface</h1>
      <Status state="ready" label={`${kind} Active`} />
      <p>
        Governed workspace surface for <strong>{title}</strong>. Grounded in SFWP view
        projections.
      </p>
    </div>
  );
}

export function EvidencePage() { return <GenericSurfacePage title="Evidence Browser" kind="Evidence" />; }
export function MemoryPage() { return <GenericSurfacePage title="Memory Recall" kind="Memory" />; }
export function CapabilitiesPage() { return <GenericSurfacePage title="Capability Matrix" kind="Capabilities" />; }
export function ArtifactsPage() { return <GenericSurfacePage title="Artifact Registry" kind="Artifacts" />; }
export function FederationPage() { return <GenericSurfacePage title="Federation Gateway" kind="Federation" />; }
export function AdminPage() { return <GenericSurfacePage title="Administration" kind="Admin" />; }
