// Provenance: camera math extracted from the Gargantua donor (`bhMain` /
// `updateCamera` in `public/reference/gargantua.html`). Donor constants and
// orbit/inertia/idle behavior preserved verbatim; globals converted to class
// state. See `src/core/PROVENANCE.md`.
//
// Donor behavior preserved:
// - initial orbit { theta: -0.6, phi: 1.22, radius: 9.4 }
// - idle cinematic auto-orbit after 3500ms: theta += dt * 0.028
// - inertial decay 0.94 on vTheta/vPhi when not dragging
// - phi clamp [0.12, PI - 0.12]
// - forward/right/up basis from world origin (CORE stays centered)
// - BH-centre screen projection + apparent lensing radius for CA/flare

import * as THREE from "three";

export interface CoreCameraState {
  theta: number;
  phi: number;
  radius: number;
  vTheta: number;
  vPhi: number;
}

export const HOME_CAMERA_STATE: CoreCameraState = {
  theta: -0.6,
  phi: 1.22,
  radius: 9.4,
  vTheta: 0,
  vPhi: 0,
};

/** Donor `FOV = 60` and its derived half-angle tangent. */
export const CORE_FOV = 60;
export const CORE_TAN_HALF_FOV = Math.tan(((CORE_FOV * 0.5) * Math.PI) / 180);

const PHI_MIN = 0.12;
const PHI_MAX = Math.PI - 0.12;
const IDLE_MS = 3500;
const AUTO_ORBIT_RATE = 0.028;
const INERTIA_DECAY = 0.94;
const UP = new THREE.Vector3(0, 1, 0);

export class CoreCamera {
  readonly state: CoreCameraState = { ...HOME_CAMERA_STATE };
  private dragging = false;

  readonly camPos = new THREE.Vector3();
  readonly basis = new THREE.Matrix3();
  readonly fwd = new THREE.Vector3();
  readonly right = new THREE.Vector3();
  readonly upv = new THREE.Vector3();
  private readonly origin = new THREE.Vector3();

  setDragging(dragging: boolean): void {
    this.dragging = dragging;
  }

  isDragging(): boolean {
    return this.dragging;
  }

  addImpulse(vTheta: number, vPhi: number): void {
    this.state.vTheta = vTheta;
    this.state.vPhi = vPhi;
    this.state.theta += vTheta;
    this.state.phi = Math.min(PHI_MAX, Math.max(PHI_MIN, this.state.phi + vPhi));
  }

  applyWheelZoom(deltaY: number): void {
    this.state.radius = Math.min(
      20,
      Math.max(2.6, this.state.radius * Math.exp(deltaY * 0.0011)),
    );
  }

  setRadius(radius: number): void {
    this.state.radius = Math.min(20, Math.max(2.6, radius));
  }

  resetToHome(): void {
    this.state.theta = HOME_CAMERA_STATE.theta;
    this.state.phi = HOME_CAMERA_STATE.phi;
    this.state.radius = HOME_CAMERA_STATE.radius;
    this.state.vTheta = 0;
    this.state.vPhi = 0;
  }

  /**
   * Donor `updateCamera(dt, t)` camera portion, preserved: idle auto-orbit,
   * inertial decay, spherical placement, orthonormal basis, uniform writes.
   * Screen projection lives in `projectOrigin` (donor's second half).
   */
  update(dt: number, nowMs: number, lastInputMs: number): void {
    const s = this.state;
    // gentle cinematic auto-orbit when idle
    if (!this.dragging && nowMs - lastInputMs > IDLE_MS) s.theta += dt * AUTO_ORBIT_RATE;
    // inertial decay
    if (!this.dragging) {
      s.theta += s.vTheta;
      s.vTheta *= INERTIA_DECAY;
      s.phi = Math.min(PHI_MAX, Math.max(PHI_MIN, s.phi + s.vPhi));
      s.vPhi *= INERTIA_DECAY;
    }
    const sp = Math.sin(s.phi);
    const cp = Math.cos(s.phi);
    this.camPos.set(
      s.radius * sp * Math.cos(s.theta),
      s.radius * cp,
      s.radius * sp * Math.sin(s.theta),
    );
    this.fwd.copy(this.camPos).multiplyScalar(-1).normalize();
    this.right.crossVectors(this.fwd, UP).normalize();
    this.upv.crossVectors(this.right, this.fwd);
    this.basis.set(
      this.right.x,
      this.upv.x,
      this.fwd.x,
      this.right.y,
      this.upv.y,
      this.fwd.y,
      this.right.z,
      this.upv.z,
      this.fwd.z,
    );
  }

  /**
   * Donor screen-space projection of the BH centre (world origin) for the
   * composite pass's chromatic aberration and flare ghosts, preserved.
   */
  projectOrigin(aspect: number, rs: number): { u: number; v: number; lensR: number } | null {
    const v = this.origin.copy(this.camPos).multiplyScalar(-1);
    const z = v.dot(this.fwd);
    if (z <= 0.001) return null;
    const x = v.dot(this.right) / (z * CORE_TAN_HALF_FOV * aspect);
    const y = v.dot(this.upv) / (z * CORE_TAN_HALF_FOV);
    return {
      u: x * 0.5 + 0.5,
      v: y * 0.5 + 0.5,
      lensR: ((2.6 * rs) / z / CORE_TAN_HALF_FOV) * 0.5,
    };
  }
}
