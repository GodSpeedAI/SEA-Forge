import type { JSX } from 'react';
import { useEffect, useRef } from 'react';
import './outline.css';

export interface OutlineItem {
  id: string;
  title: string;
  subtitle?: string;
  status?: { label: string; tone: 'ok' | 'progress' | 'attention' | 'critical' | 'hypothesis' | 'muted' };
  depth: number;
  isCenter: boolean;
  actions: { id: string; label: string }[];
}

export interface OutlineViewProps {
  visible: boolean;
  title: string;
  caption?: string;
  timeLabel: string;
  items: OutlineItem[];
  onFocus(id: string): void;
  onAction(id: string, actionId: string): void;
  onClose(): void;
}

export function OutlineView(p: OutlineViewProps): JSX.Element {
  const sectionRef = useRef<HTMLElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const firstButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (p.visible && firstButtonRef.current) {
      firstButtonRef.current.focus();
    }
  }, [p.visible]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLUListElement>) => {
    if (!listRef.current) return;

    const buttons = Array.from(listRef.current.querySelectorAll('button'));
    const focusedButton = document.activeElement as HTMLButtonElement;
    const focusedIndex = buttons.indexOf(focusedButton);

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (focusedIndex < buttons.length - 1) {
        buttons[focusedIndex + 1].focus();
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (focusedIndex > 0) {
        buttons[focusedIndex - 1].focus();
      }
    } else if (e.key === 'Home') {
      e.preventDefault();
      buttons[0]?.focus();
    } else if (e.key === 'End') {
      e.preventDefault();
      buttons[buttons.length - 1]?.focus();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      p.onClose();
    }
  };

  return (
    <section
      ref={sectionRef}
      className="outline-view"
      role="dialog"
      aria-modal={false}
      aria-label="Outline of the world"
      style={{ display: p.visible ? 'block' : 'none' }}
    >
      <div className="outline-header">
        <h1 className="outline-title">{p.title}</h1>
        <p className="outline-time">{p.timeLabel}</p>
        {p.caption && <p className="outline-caption">{p.caption}</p>}
        <button className="outline-close" aria-label="Close outline" onClick={p.onClose}>
          ✕
        </button>
      </div>

      <ul
        ref={listRef}
        className="outline-tree"
        role="tree"
        onKeyDown={handleKeyDown}
      >
        {p.items.map((item, index) => (
          <li
            key={item.id}
            className="outline-item"
            role="treeitem"
            aria-level={item.depth + 1}
            style={{ paddingLeft: `${item.depth * 16}px` }}
          >
            <button
              ref={index === 0 ? firstButtonRef : null}
              className={`outline-item-button ${item.isCenter ? 'is-center' : ''}`}
              onClick={() => p.onFocus(item.id)}
            >
              {item.isCenter && <span className="visually-hidden">(center)</span>}
              {item.isCenter && <span className="outline-center-bar" aria-hidden="true"></span>}
              <span className="outline-item-content">
                <span className="outline-item-title">{item.title}</span>
                {item.subtitle && (
                  <span className="outline-item-subtitle">{item.subtitle}</span>
                )}
              </span>
            </button>

            {item.status && (
              <span className="outline-status">
                <span
                  className={`outline-status-dot outline-status-${item.status.tone}`}
                  aria-hidden="true"
                ></span>
                <span className="outline-status-label">{item.status.label}</span>
              </span>
            )}

            {item.actions.length > 0 && (
              <div className="outline-actions">
                {item.actions.map((action) => (
                  <button
                    key={action.id}
                    className="outline-action-button"
                    onClick={() => p.onAction(item.id, action.id)}
                  >
                    {action.label}
                  </button>
                ))}
              </div>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
