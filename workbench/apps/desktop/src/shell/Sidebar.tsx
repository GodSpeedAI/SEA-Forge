import { Link, useLocation } from "@tanstack/react-router";
import styles from "./Sidebar.module.css";

export interface NavItemDef {
  path: string;
  label: string;
  icon: string;
  shortcut?: string;
  badge?: number;
}

const OPERATE_NAV: NavItemDef[] = [
  { path: "/readiness", label: "Readiness", icon: "◈", shortcut: "R" },
  { path: "/thoth", label: "Thoth", icon: "◇" },
  { path: "/assets", label: "Assets", icon: "▱" },
  { path: "/delegate", label: "Delegation", icon: "⇥" },
  { path: "/models", label: "Domain Models", icon: "⌘" },
  { path: "/cases", label: "Cases", icon: "▤" },
  { path: "/inbox", label: "Inbox", icon: "▾", badge: 1 },
  { path: "/operations", label: "Operations", icon: "▶" },
];

const INSPECT_NAV: NavItemDef[] = [
  { path: "/evidence", label: "Evidence", icon: "◎" },
  { path: "/memory", label: "Memory", icon: "≋" },
  { path: "/capabilities", label: "Capabilities", icon: "△" },
  { path: "/artifacts", label: "Artifacts", icon: "□" },
  { path: "/federation", label: "Federation", icon: "⌁" },
];

const ADMIN_NAV: NavItemDef[] = [
  { path: "/admin", label: "Administration", icon: "⚙" },
];

export function Sidebar() {
  const location = useLocation();

  const renderNavGroup = (items: NavItemDef[]) =>
    items.map((item) => {
      const isActive = location.pathname.startsWith(item.path);
      return (
        <Link
          key={item.path}
          to={item.path}
          className={`${styles.navItem} nav-item ${
            isActive ? `${styles.navItemActive} nav-item--active` : ""
          }`}
          data-route={item.label}
          aria-current={isActive ? "page" : undefined}
        >
          <span aria-hidden="true">{item.icon}</span>
          <span>{item.label}</span>
          {item.badge !== undefined && (
            <span className={`${styles.badge} nav-count`}>{item.badge}</span>
          )}
          {item.shortcut && <kbd>{item.shortcut}</kbd>}
        </Link>
      );
    });

  return (
    <nav
      className={`${styles.nav} primary-nav`}
      aria-label="Primary navigation"
      data-od-id="primary-navigation"
      data-testid="primary-navigation"
    >
      <div className={`${styles.sectionLabel} nav-section-label`}>Operate</div>
      {renderNavGroup(OPERATE_NAV)}

      <div className={`${styles.sectionLabel} nav-section-label`}>Inspect</div>
      {renderNavGroup(INSPECT_NAV)}

      <div className={`${styles.navSpacer} nav-spacer`} />

      {renderNavGroup(ADMIN_NAV)}
    </nav>
  );
}
