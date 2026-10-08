import './chrome.css';

export interface BrandProps {
  visible: boolean;
}

export function Brand(props: BrandProps): JSX.Element {
  return (
    <div className="chrome-element brand-container" data-visible={props.visible}>
      <h1 className="brand-title">GodSpeed</h1>
      <div className="brand-subtitle">
        <div>Cognitive environment</div>
        <div>for real progress.</div>
      </div>
    </div>
  );
}

export interface BreadcrumbProps {
  path: string[];
  visible: boolean;
}

export function Breadcrumb(props: BreadcrumbProps): JSX.Element {
  return (
    <div
      className="chrome-element breadcrumb-container"
      data-visible={props.visible}
    >
      <div className="breadcrumb-items">
        {props.path.map((item, index) => (
          <div key={index}>
            {index > 0 && <span className="breadcrumb-separator">/</span>}
            <span className="breadcrumb-item">{item}</span>
          </div>
        ))}
      </div>
      <div className="breadcrumb-line" />
    </div>
  );
}

export interface StatusBarProps {
  state: 'live' | 'past' | 'local';
  when: string;
  visible: boolean;
}

export function StatusBar(props: StatusBarProps): JSX.Element {
  const stateLabels = {
    live: 'Live',
    past: 'In time travel',
    local: 'Local contract adapter',
  };

  return (
    <div
      className="chrome-element status-bar-container"
      data-visible={props.visible}
    >
      <div className={`status-dot ${props.state}`} />
      <span className={`status-label ${props.state}`}>
        {stateLabels[props.state]}
      </span>
      <span className="status-separator">|</span>
      <span className="status-time">{props.when}</span>
    </div>
  );
}

export interface UserMarkProps {
  visible: boolean;
}

export function UserMark(props: UserMarkProps): JSX.Element {
  return (
    <div className="chrome-element user-mark-container" data-visible={props.visible}>
      <span className="user-label">You</span>
      <div className="user-avatar" />
    </div>
  );
}

export interface CoreAnchorLabelProps {
  visible: boolean;
  onHome(): void;
}

export function CoreAnchorLabel(props: CoreAnchorLabelProps): JSX.Element {
  return (
    <button
      className="chrome-element core-anchor-container"
      data-visible={props.visible}
      onClick={props.onHome}
      aria-label="Return to Core (Home)"
    >
      <div className="core-anchor-text">
        <h2 className="core-anchor-label">Core</h2>
        <p className="core-anchor-sublabel">System</p>
      </div>
    </button>
  );
}
