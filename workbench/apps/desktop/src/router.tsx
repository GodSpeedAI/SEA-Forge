import {
  createRootRoute,
  createRoute,
  createRouter,
  Outlet,
  useSearch,
} from "@tanstack/react-router";
import { AppShell } from "./shell/AppShell";
import { ReadinessPage } from "./pages/ReadinessPage";
import { CaseCreationWorkbench } from "./pages/CaseCreationWorkbench";
import { CaseHorizonPage } from "./pages/CaseHorizonPage";
import { DelegationWorkbench } from "./pages/DelegationWorkbench";
import { ApprovalInboxPage } from "./pages/ApprovalInboxPage";
import { EvidencePage } from "./pages/EvidencePage";
import { RunRecordPage } from "./pages/RunRecordPage";
import {
  ThothPage,
  AssetsPage,
  ModelsPage,
  OperationsPage,
  MemoryPage,
  CapabilitiesPage,
  ArtifactsPage,
  FederationPage,
  AdminPage,
} from "./pages/SurfacesPages";
import { evaluateGuard, type GuardId } from "./guards/guards";
import { GovernedDenialSurface } from "./guards/GovernedDenialSurface";
import { useGuardContext, BLOCKING_GUARDS } from "./guards/useGuardContext";

interface RouterContextSearch {
  guardSimFail?: GuardId;
}

/**
 * The shell, gated on the guards this cell can actually answer.
 *
 * Only guards that reached a `failed` verdict block a route. An
 * `indeterminate` guard — one no source answered — does not, because blocking
 * on absent evidence would make surfaces unreachable without making anything
 * safer: the server refuses protected work on its own authority regardless of
 * what renders here. What it must never do is *claim* the guard passed, which
 * is what the removed `mockGuardContext` did for all ten at once.
 */
function RootComponent() {
  const search = useSearch({ strict: false }) as RouterContextSearch;
  const guardContext = useGuardContext();

  // `?guardSimFail=G9` renders the denial surface for one guard against the
  // real context, so the layout can be inspected without a broken cell.
  if (search.guardSimFail) {
    const simulated = evaluateGuard(search.guardSimFail, {
      ...guardContext,
      readinessState: "blocked",
    });
    return (
      <AppShell>
        <GovernedDenialSurface guardResult={simulated} />
      </AppShell>
    );
  }

  const blocked = BLOCKING_GUARDS.map((guard) => evaluateGuard(guard, guardContext)).find(
    (result) => result.verdict === "failed",
  );
  if (blocked) {
    return (
      <AppShell>
        <GovernedDenialSurface guardResult={blocked} />
      </AppShell>
    );
  }

  return (
    <AppShell>
      <Outlet />
    </AppShell>
  );
}

const rootRoute = createRootRoute({
  component: RootComponent,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: ReadinessPage,
});

const readinessRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/readiness",
  component: ReadinessPage,
});

const thothRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/thoth",
  component: ThothPage,
});

const assetsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/assets",
  component: AssetsPage,
});

/**
 * Configuring an agent task is its own surface rather than a panel on
 * `/assets`: the catalog answers "what does this cell hold", while this answers
 * "what would this particular delegation be governed by" — a question about a
 * request that does not exist yet, and one whose answer is only meaningful for
 * one exact set of inputs.
 */
const delegateRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/delegate",
  component: DelegationWorkbench,
});

const modelsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/models",
  component: ModelsPage,
});

const casesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/cases",
  component: CaseHorizonPage,
});

const caseCreationRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/cases/new",
  component: CaseCreationWorkbench,
});

const inboxRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/inbox",
  component: ApprovalInboxPage,
});

const operationsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/operations",
  component: OperationsPage,
});

const evidenceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/evidence",
  component: EvidencePage,
});

/**
 * The run record every other surface's run ids resolve to. Kept at the top
 * level rather than nested under `/cases/$caseId` because a run is reachable
 * from the horizon, the evidence index, and the event stream alike — and
 * because a run whose case cannot be determined (an orphan, epic 11.6) still
 * has to be openable.
 */
const runRecordRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/runs/$runId",
  component: RunRecordPage,
});

const memoryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/memory",
  component: MemoryPage,
});

const capabilitiesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/capabilities",
  component: CapabilitiesPage,
});

const artifactsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/artifacts",
  component: ArtifactsPage,
});

const federationRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/federation",
  component: FederationPage,
});

const adminRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin",
  component: AdminPage,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  readinessRoute,
  thothRoute,
  assetsRoute,
  delegateRoute,
  modelsRoute,
  casesRoute,
  caseCreationRoute,
  inboxRoute,
  operationsRoute,
  evidenceRoute,
  runRecordRoute,
  memoryRoute,
  capabilitiesRoute,
  artifactsRoute,
  federationRoute,
  adminRoute,
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
