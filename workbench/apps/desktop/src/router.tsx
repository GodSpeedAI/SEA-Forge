import { createRootRoute, createRoute, createRouter } from "@tanstack/react-router";
import { ProofPage } from "./ProofPage";

type ProofSearch = { filter: string };

function validateSearch(search: Record<string, unknown>): ProofSearch {
  const filter = search.filter;
  return { filter: typeof filter === "string" ? filter : "" };
}

const rootRoute = createRootRoute();

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  validateSearch,
  component: ProofPage,
});

const routeTree = rootRoute.addChildren([indexRoute]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
