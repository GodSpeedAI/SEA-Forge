import { useEffect, useRef, useState } from "react";
import { CoreRenderer, type CoreFatalInfo } from "./CoreRenderer";
import type { CoreVisualState } from "./CoreVisualState";
import { RenderFailureSurface } from "./chrome/RenderFailureSurface";
import styles from "./CoreViewport.module.css";

export interface CoreViewportProps {
  visualState?: Partial<CoreVisualState>;
  chromeVisible?: boolean;
  onRendererReady?: (renderer: CoreRenderer) => void;
  onFps?: (fps: number) => void;
  className?: string;
}

/**
 * CoreViewport — the persistent CORE canvas. Mounted once for the lifetime of
 * the application (see `AppShell`); ordinary focus/surface changes never
 * remount it and never recreate the WebGL context. Visual intent arrives via
 * `visualState`; GPU internals stay inside `CoreRenderer`.
 */
export function CoreViewport({
  visualState,
  chromeVisible = true,
  onRendererReady,
  onFps,
  className,
}: CoreViewportProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<CoreRenderer | null>(null);
  const readyRef = useRef(onRendererReady);
  readyRef.current = onRendererReady;
  const fpsRef = useRef(onFps);
  fpsRef.current = onFps;
  const [fatal, setFatal] = useState<CoreFatalInfo | null>(null);
  const [hidden, setHidden] = useState(false);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const renderer = new CoreRenderer({
      canvas,
      onFatal: (info) => setFatal(info),
      onFps: (fps) => fpsRef.current?.(fps),
    });
    rendererRef.current = renderer;
    const ok = renderer.start();
    if (ok) {
      // Donor lifted the intro veil ~120ms after first frame (`body.ready`);
      // the CSS transitions on `body.core-ready` carry the same fade. The
      // product class is namespaced so legacy `ready` hooks cannot collide.
      const t = window.setTimeout(() => document.body.classList.add("core-ready"), 120);
      readyRef.current?.(renderer);
      return () => {
        window.clearTimeout(t);
        document.body.classList.remove("core-ready");
        renderer.dispose();
        rendererRef.current = null;
      };
    }
    return () => {
      renderer.dispose();
      rendererRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (visualState && rendererRef.current && !fatal) {
      rendererRef.current.setVisualState(visualState);
    }
  }, [visualState, fatal]);

  useEffect(() => {
    const onToggle = () => setHidden((v) => !v);
    window.addEventListener("core:toggle-chrome", onToggle);
    return () => window.removeEventListener("core:toggle-chrome", onToggle);
  }, []);

  const showChrome = chromeVisible && !hidden && !fatal;

  return (
    <div
      className={`${styles.viewport} ${className ?? ""}`}
      data-testid="core-viewport"
      data-chrome={showChrome ? "visible" : "hidden"}
    >
      <canvas ref={canvasRef} className={styles.canvas} data-testid="core-canvas" />
      {fatal ? <RenderFailureSurface info={fatal} /> : null}
    </div>
  );
}
