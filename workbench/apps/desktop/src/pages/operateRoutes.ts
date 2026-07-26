export type OperateRouteState = "ready" | "degraded" | "blocked";

export interface OperateRouteDefinition {
  id: string;
  path: string;
  shellLabel: string;
  journeyStep: string;
  title: string;
  state: OperateRouteState;
  reason: string;
  projection: string;
  action: string;
}

export const OPERATE_ROUTES = {
  thoth: {
    id: "thoth",
    path: "/thoth",
    shellLabel: "thoth",
    journeyStep: "Thoth",
    title: "Thoth workspace",
    state: "ready",
    reason:
      "A typed question can be grounded in selected source records. Drafting an answer does not change a case.",
    projection: "grounded answer / disclosure",
    action: "Inspect question path",
  },
  assets: {
    id: "assets",
    path: "/assets",
    shellLabel: "assets",
    journeyStep: "Assets",
    title: "Asset catalog",
    state: "ready",
    reason:
      "Available assets are separate from declared and installed assets. Imported material remains inert until reviewed.",
    projection: "catalog / availability",
    action: "Inspect selected asset",
  },
  domain: {
    id: "domain_models",
    path: "/models",
    shellLabel: "domain models",
    journeyStep: "Domain",
    title: "Domain models",
    state: "degraded",
    reason:
      "The active repair model is structurally valid. Its agent endpoint projection is stale and cannot support an execution claim.",
    projection: "model / validation",
    action: "Inspect validation path",
  },
  cases: {
    id: "cases",
    path: "/cases",
    shellLabel: "cases",
    journeyStep: "Cases",
    title: "Case horizon",
    state: "degraded",
    reason:
      "One committed case is awaiting endpoint evidence. Personal arrangement may change, but authoritative state remains source-backed.",
    projection: "case reducer / board",
    action: "Inspect case draft path",
  },
  inbox: {
    id: "inbox",
    path: "/inbox",
    shellLabel: "inbox",
    journeyStep: "Inbox",
    title: "Inbox and approvals",
    state: "blocked",
    reason:
      "Approval APR-07 is pending. The exact resource, authority, expiry, and separation-of-duty condition require review.",
    projection: "approval / human task",
    action: "Inspect approval path",
  },
  operations: {
    id: "operations",
    path: "/operations",
    shellLabel: "operations",
    journeyStep: "Operations",
    title: "Operations monitor",
    state: "degraded",
    reason:
      "Execution completed. Settlement evaluation continues. The event stream is stale after EVT-1182.",
    projection: "run / event cursor",
    action: "Inspect reconnect path",
  },
} satisfies Record<string, OperateRouteDefinition>;

export const OPERATE_ROUTE_BY_PATH = Object.fromEntries(
  Object.values(OPERATE_ROUTES).map((route) => [route.path, route]),
) as Record<string, OperateRouteDefinition>;
