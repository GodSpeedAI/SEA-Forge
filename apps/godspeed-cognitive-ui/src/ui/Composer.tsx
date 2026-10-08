import { useRef, useEffect, useState } from 'react';
import './composer.css';

export interface ComposerProps {
  variant: 'home' | 'focused';
  placeholder?: string;
  onSubmit(text: string): void;
  hint?: string;
}

export function Composer(props: ComposerProps): JSX.Element {
  const inputRef = useRef<HTMLInputElement>(null);
  const [text, setText] = useState('');

  const handleSubmit = () => {
    const trimmed = text.trim();
    if (trimmed) {
      props.onSubmit(trimmed);
      setText('');
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleSubmit();
    } else if (e.key === 'Escape') {
      inputRef.current?.blur();
    }
  };

  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      // Only trigger on "/" when not in input/textarea
      if (
        e.key === '/' &&
        document.activeElement !== inputRef.current &&
        !(document.activeElement instanceof HTMLInputElement ||
          document.activeElement instanceof HTMLTextAreaElement)
      ) {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);
    return () => window.removeEventListener('keydown', handleGlobalKeyDown);
  }, []);

  return (
    <div className="composer-wrapper" data-visible="true" data-variant={props.variant}>
      {props.hint && <div className="composer-hint">{props.hint}</div>}
      <div className="composer" data-variant={props.variant}>
        <div className="composer-icon-left">
          <svg viewBox="0 0 24 24">
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.35-4.35" />
          </svg>
        </div>
        <input
          ref={inputRef}
          type="text"
          className="composer-input"
          placeholder={props.placeholder || 'Ask, find, understand, or act...'}
          value={text}
          onChange={(e) => setText(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          aria-label="Ask GodSpeed"
        />
        <div className="composer-controls">
          {props.variant === 'home' && (
            <div className="composer-icon-right" aria-hidden>
              <svg viewBox="0 0 24 24" width="24" height="24">
                <path d="M4 10v4M7.5 7v10M11 4.5v15M14.5 7v10M18 9.5v5M21 11v2" />
              </svg>
            </div>
          )}
          {props.variant === 'focused' && (
            <>
              <div className="composer-icon-right" aria-hidden>
                <svg viewBox="0 0 24 24" width="22" height="22">
                  <rect x="9" y="3" width="6" height="11" rx="3" />
                  <path d="M5.5 11a6.5 6.5 0 0 0 13 0M12 17.5V21" />
                </svg>
              </div>
              <button className="composer-submit-button" onClick={handleSubmit} type="button" aria-label="Send">
                <svg viewBox="0 0 24 24" width="20" height="20">
                  <path d="M5 12h14M13 6l6 6-6 6" />
                </svg>
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
