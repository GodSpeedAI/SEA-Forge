// CoreTuningPanel: internal development/tuning surface for CORE visual
// parameters. Donor source: the `#hud` EVENT CONTROL block + `bindSlider`
// wiring in `public/reference/gargantua.html`. Same ranges, defaults, and
// formatters; same slider fill treatment. These renderer parameters are NOT
// product semantics — the panel renders only in dev builds and never ships in
// the settled product surface.

import { useState } from "react";
import type { CoreVisualParams } from "../core/CoreVisualState";
import { DEFAULT_VISUAL_PARAMS } from "../core/CoreVisualState";
import styles from "./CoreTuningPanel.module.css";

export interface CoreTuningPanelProps {
  params: CoreVisualParams;
  fps: number | null;
  onChange: (params: CoreVisualParams) => void;
}

function Slider({
  label,
  min,
  max,
  step,
  value,
  format,
  cool,
  onInput,
}: {
  label: string;
  min: number;
  max: number;
  step: number;
  value: number;
  format: (v: number) => string;
  cool?: boolean;
  onInput: (v: number) => void;
}) {
  const pct = ((value - min) / (max - min)) * 100;
  return (
    <div className={`${styles.ctl} ${cool ? styles.cool : ""}`}>
      <div className={styles.row}>
        <label>
          {label}
        </label>
        <output>{format(value)}</output>
      </div>
      <input
        type="range"
        aria-label={label}
        min={min}
        max={max}
        step={step}
        value={value}
        style={{ ["--fill" as string]: `${pct}%` }}
        onChange={(e) => onInput(parseFloat(e.target.value))}
      />
    </div>
  );
}

export function CoreTuningPanel({ params, fps, onChange }: CoreTuningPanelProps) {
  const [open, setOpen] = useState(false);
  if (!open) {
    return (
      <button
        type="button"
        className={styles.opener}
        data-testid="core-tuning-opener"
        onClick={() => setOpen(true)}
      >
        CORE TUNING · DEV
      </button>
    );
  }
  return (
    <div className={styles.hud} data-testid="core-tuning-panel">
      <div className={styles.hudHead}>
        <span>CORE TUNING</span>
        <span>
          dev ·{" "}
          <button type="button" onClick={() => onChange({ ...DEFAULT_VISUAL_PARAMS })}>
            reset
          </button>{" "}
          <button type="button" onClick={() => setOpen(false)}>
            hide
          </button>
        </span>
      </div>
      {/* Donor ranges/defaults/formatters preserved (bindSlider) */}
      <Slider
        label="Mass"
        min={0.4}
        max={1.35}
        step={0.001}
        value={params.mass}
        format={(v) => `${(v * 13.9).toFixed(1)} ×10⁶ M☉`}
        onInput={(mass) => onChange({ ...params, mass })}
      />
      <Slider
        label="Spin · a/M"
        min={0}
        max={0.98}
        step={0.001}
        value={params.spin}
        format={(v) => v.toFixed(3)}
        onInput={(spin) => onChange({ ...params, spin })}
      />
      <Slider
        label="Disk Temp"
        min={0}
        max={1}
        step={0.001}
        value={params.temp}
        format={(v) => `${Math.round(1200 + v * 11000).toLocaleString()} K`}
        cool
        onInput={(temp) => onChange({ ...params, temp })}
      />
      <Slider
        label="Bloom"
        min={0}
        max={2}
        step={0.001}
        value={params.bloom}
        format={(v) => `${(v * 100).toFixed(0)} %`}
        onInput={(bloom) => onChange({ ...params, bloom })}
      />
      <div className={styles.foot}>
        <span>RENDER CORE</span>
        <span>{fps === null ? "— FPS" : `${fps} FPS`}</span>
      </div>
    </div>
  );
}
