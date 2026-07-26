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
import {
  ThothPage,
  AssetsPage,
  ModelsPage,
  CasesPage,
  InboxPage,
  OperationsPage,
  EvidencePage,
  MemoryPage,
  CapabilitiesPage,
  ArtifactsPage,
  FederationPage,
  AdminPage,
} from "./pages/SurfacesPages";
import { evaluateGuard, type GuardContext, type GuardId } from "./guards/guards";
import { GovernedDenialSurface } from "./guards/GovernedDenialSurface";

interface RouterContextSearch {
  cellId?: string;
  guardSimFail?: GuardId;
}

function RootComponent() {
  const search = useSearch({ strict: false }) as RouterContextSearch;

  const mockGuardContext: GuardContext = {
    cellId: search.cellId ?? "cell_local_01",
    actorId: "actor_op_01",
    actorRole: "Operator",
    sponsorId: "sponsor_human_01",
    policyLoaded: true,
    policyHash: "sha256:policy_v1",
    integrityStatus: "verified",
    resourceFound: true,
    disclosurePermitted: true,
    compatibilityOk: true,
    readinessState: "ready",
    standingOk: true,
  };

  // Allow simulating guard failure via query param `?guardSimFail=G9`
  if (search.guardSimFail) {
    const result = evaluateGuard(search.guardSimFail, { ...mockGuardContext, readinessState: "blocked" });
    return (
      <AppShell>
        <GovernedDenialSurface guardResult={result} />
      </AppShell>
    );
  }

  // Base Guard G1 check
  const g1Result = evaluateGuard("G1", mockGuardContext);
  if (!g1Result.passed) {
    return (
      <AppShell>
        <GovernedDenialSurface guardResult={g1Result} />
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

const modelsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/models",
  component: ModelsPage,
});

const casesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/cases",
  component: CasesPage,
});

const caseCreationRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/cases/new",
  component: CaseCreationWorkbench,
});

const inboxRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/inbox",
  component: InboxPage,
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
  modelsRoute,
  casesRoute,
  caseCreationRoute,
  inboxRoute,
  operationsRoute,
  evidenceRoute,
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
