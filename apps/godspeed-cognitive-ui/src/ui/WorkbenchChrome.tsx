import type React from 'react';
import './workbench.css';

export interface RailItem {
  id: string;
  label: string;
  badge?: number;
  onSelect?: () => void;
}

export interface TabItem {
  label: string;
  count?: number;
  onSelect?: () => void;
}

export interface WorkbenchChromeProps {
  visible: boolean;
  brandSub: [string, string];
  activeRail: string;
  breadcrumb: string[];
  title: string;
  subtitle: string;
  badge?: { label: string; tone: 'progress' | 'ok' | 'attention' };
  status: { label: string; tone: 'progress' | 'ok' | 'attention' };
  rail: RailItem[];
  tabs: TabItem[];
  activeTab: string;
  onTab(label: string): void;
  onClose(): void;
  right: React.ReactNode;
}

export function WorkbenchChrome(p: WorkbenchChromeProps): JSX.Element {
  return (
    <div
      className="workbench-chrome"
      data-visible={p.visible}
      data-theme={typeof document !== 'undefined' ? document.documentElement.getAttribute('data-theme') : 'light'}
    >
      {/* LEFT RAIL */}
      <div className="workbench-left-rail">
        <div className="workbench-brand">
          <div className="workbench-brand-title">GodSpeed</div>
          <div className="workbench-brand-sub-line">{p.brandSub[0]}</div>
          <div className="workbench-brand-sub-line">{p.brandSub[1]}</div>
        </div>

        <nav className="workbench-nav">
          {p.rail.map((item) => (
            <button
              key={item.id}
              className="workbench-nav-item"
              data-testid="rail-item"
              data-id={item.id}
              data-active={item.id === p.activeRail}
              disabled={!item.onSelect}
              aria-disabled={!item.onSelect}
              title={!item.onSelect ? 'Not available in this build' : undefined}
              onClick={item.onSelect}
            >
              <span className="workbench-nav-label">{item.label}</span>
              {item.badge !== undefined && (
                <span className="workbench-nav-badge">{item.badge}</span>
              )}
            </button>
          ))}
        </nav>

        <div className="workbench-bottom">
          <div className="workbench-avatar">SP</div>
          <div className="workbench-user-info">
            <div className="workbench-user-name">Sam Prime</div>
            <div className="workbench-user-status">Personal</div>
            <div className="workbench-online-status">● Online</div>
          </div>
        </div>
      </div>

      {/* HEADER */}
      <div className="workbench-header">
        <div className="workbench-breadcrumb">
          {p.breadcrumb.map((item, i) => (
            <span key={i}>
              {i > 0 && ' > '}
              {item}
            </span>
          ))}
        </div>

        <div className="workbench-header-title-row">
          <div className="workbench-avatar-large"></div>
          <div className="workbench-title-group">
            <div className="workbench-title">
              {p.title}
              {p.badge && (
                <span className={`workbench-badge workbench-badge-${p.badge.tone}`}>
                  {p.badge.label}
                </span>
              )}
            </div>
            <div className="workbench-subtitle">{p.subtitle}</div>
          </div>
          <div className="workbench-header-right">
            <span className={`workbench-status-pill workbench-status-${p.status.tone}`}>
              <span className="workbench-status-dot"></span>
              {p.status.label}
            </span>
            <button className="workbench-menu-button" aria-label="More options">
              ···
            </button>
          </div>
        </div>

        <div className="workbench-tabs-row">
          {p.tabs.map((tab) => (
            <button
              key={tab.label}
              className="workbench-tab"
              data-active={tab.label === p.activeTab}
              disabled={!tab.onSelect}
              aria-disabled={!tab.onSelect}
              title={!tab.onSelect ? 'Not available in this build' : undefined}
              onClick={() => {
                if (tab.onSelect) {
                  tab.onSelect();
                }
                p.onTab(tab.label);
              }}
            >
              {tab.label}
              {tab.count !== undefined && (
                <span className="workbench-tab-count">{tab.count}</span>
              )}
            </button>
          ))}
        </div>
      </div>

      {/* CENTER (transparent for world rendering) */}
      <div className="workbench-center-transparent"></div>

      {/* RIGHT RAIL */}
      <div className="workbench-right-rail">
        <button className="workbench-close-button" onClick={p.onClose} aria-label="Close workbench">
          ✕
        </button>
        {p.right}
      </div>
    </div>
  );
}
