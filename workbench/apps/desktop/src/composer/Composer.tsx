// Composer: the persistent interaction surface for acting within the current
// context. Not a destination — mounted once beside CORE for the application
// lifetime. Available actions/context vary with Focus, Surface, authority,
// affordances, and case state (see `projections/AffordanceProjection`).
// Unsupported capabilities are never populated.

import { useNavigate } from "@tanstack/react-router";
import type { Affordance } from "../projections/AffordanceProjection";
import { focusController } from "../spatial/focus/FocusController";
import styles from "./Composer.module.css";

export interface ComposerProps {
  surfaceId: string;
  affordances: Affordance[];
}

export function Composer({ surfaceId, affordances }: ComposerProps) {
  const navigate = useNavigate();
  return (
    <div className={styles.composer} data-testid="composer" data-surface={surfaceId}>
      <div className={styles.head}>
        <span>COMPOSE</span>
        <span>{surfaceId}</span>
      </div>
      <ul>
        {affordances.map((a) => (
          <li key={a.id}>
            <button
              type="button"
              disabled={a.disabled}
              title={a.reason ?? a.label}
              onClick={() => {
                if (a.id === "release-focus" || a.id === "go-home") {
                  focusController.returnHome();
                }
                if (a.path) void navigate({ to: a.path });
              }}
            >
              {a.label}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
