import { Link, useLocation } from "@tanstack/react-router";
import styles from "./Sidebar.module.css";

export interface NavItemDef {
  path: string;
  label: string;
  icon: string;
  shortcut?: string;
  badge?: number;
}

export const OPERATE_NAV: NavItemDef[] = [
  { path: "/readiness", label: "Readiness", icon: "◈", shortcut: "R" },
  { path: "/thoth", label: "Thoth", icon: "◇" },
  { path: "/assets", label: "Assets", icon: "▱" },
  { path: "/models", label: "Domain Models", icon: "⌘" },
  { path: "/cases", label: "Cases", icon: "▤" },
  { path: "/inbox", label: "Inbox", icon: "▾", badge: 1 },
  { path: "/operations", label: "Operations", icon: "▶" },
];

export const INSPECT_NAV: NavItemDef[] = [
  { path: "/evidence", label: "Evidence", icon: "◎" },
  { path: "/memory", label: "Memory", icon: "≋" },
  { path: "/capabilities", label: "Capabilities", icon: "△" },
  { path: "/artifacts", label: "Artifacts", icon: "□" },
  { path: "/federation", label: "Federation", icon: "⌁" },
];

export const ADMIN_NAV: NavItemDef[] = [
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
          className={`${styles.navItem} ${isActive ? styles.navItemActive : ""}`}
          data-route={item.label}
          aria-current={isActive ? "page" : undefined}
        >
          <span className={styles.navIcon} aria-hidden="true">
            {item.icon}
          </span>
          <span>{item.label}</span>
          {item.badge !== undefined && <span className={styles.badge}>{item.badge}</span>}
          {item.shortcut && <kbd className={styles.kbd}>{item.shortcut}</kbd>}
        </Link>
      );
    });

  return (
    <nav className={styles.nav} aria-label="Primary navigation" data-testid="primary-navigation">
      <div className={styles.sectionLabel}>Operate</div>
      {renderNavGroup(OPERATE_NAV)}

      <div className={styles.sectionLabel}>Inspect</div>
      {renderNavGroup(INSPECT_NAV)}

      <div style={{ flex: 1 }} />

      <div className={styles.sectionLabel}>Admin</div>
      {renderNavGroup(ADMIN_NAV)}
    </nav>
  );
}
