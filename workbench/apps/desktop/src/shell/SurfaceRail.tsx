// SurfaceRail: the sparse surface switcher. This replaces the legacy page-nav
// sidebar, not its authority: it only changes arrangement + focus. Surfaces
// are meaningful places (home/case/beat/artifacts/temporal/causal/judgment/
// execution/design), never dashboard widgets. Positioned along the left edge
// so CORE keeps the canonical center.

import { Link, useLocation } from "@tanstack/react-router";
import styles from "./SurfaceRail.module.css";

const SURFACES = [
  { path: "/", label: "CORE", title: "Home — CORE orientation" },
  { path: "/cases", label: "Case", title: "Case surface" },
  { path: "/thoth", label: "Beat", title: "Semantic beat" },
  { path: "/artifacts", label: "Artifacts", title: "Artifacts" },
  { path: "/operations", label: "Time", title: "Temporal" },
  { path: "/evidence", label: "Causal", title: "Causal" },
  { path: "/inbox", label: "Judge", title: "Judgment" },
  { path: "/delegate", label: "Execute", title: "Execution" },
  { path: "/cases/new", label: "Design", title: "Case design" },
];

export function SurfaceRail({
  surfaceId,
  inboxCount,
}: {
  surfaceId: string;
  inboxCount?: number;
}) {
  const location = useLocation();
  return (
    <nav className={styles.rail} aria-label="Surfaces" data-testid="surface-rail">
      {SURFACES.map((s) => {
        const isActive =
          s.path === "/"
            ? location.pathname === "/" || surfaceId === "home"
            : location.pathname.startsWith(s.path);
        return (
          <Link
            key={s.path}
            to={s.path}
            className={`${styles.item} ${isActive ? styles.active : ""}`}
            title={s.title}
            aria-current={isActive ? "page" : undefined}
          >
            {s.label}
            {s.path === "/inbox" && inboxCount !== undefined && inboxCount > 0 ? (
              <span className={styles.badge}>{inboxCount}</span>
            ) : null}
          </Link>
        );
      })}
    </nav>
  );
}
