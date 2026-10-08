import { describe, expect, it } from "vitest";
import { CoreCamera, HOME_CAMERA_STATE } from "./CoreCamera";

describe("CoreCamera (donor orbit behavior preserved)", () => {
  it("starts at the canonical Home orbit", () => {
    const cam = new CoreCamera();
    expect(cam.state).toEqual(HOME_CAMERA_STATE);
  });

  it("drag impulses match donor factors and clamp phi", () => {
    const cam = new CoreCamera();
    cam.setDragging(true);
    cam.addImpulse(10 * 0.0032, -5 * 0.0032);
    expect(cam.state.theta).toBeCloseTo(-0.6 + 0.032, 12);
    expect(cam.state.vTheta).toBeCloseTo(0.032, 12);
    // hard upward drag clamps at the donor pole guard
    cam.addImpulse(0, -100);
    expect(cam.state.phi).toBeGreaterThanOrEqual(0.12);
    cam.addImpulse(0, 100);
    expect(cam.state.phi).toBeLessThanOrEqual(Math.PI - 0.12);
  });

  it("wheel zoom is exponential within donor bounds", () => {
    const cam = new CoreCamera();
    cam.applyWheelZoom(1000);
    expect(cam.state.radius).toBeLessThanOrEqual(20);
    cam.applyWheelZoom(-10000);
    expect(cam.state.radius).toBeGreaterThanOrEqual(2.6);
    const before = cam.state.radius;
    cam.applyWheelZoom(100);
    expect(cam.state.radius).toBeCloseTo(
      Math.min(20, Math.max(2.6, before * Math.exp(100 * 0.0011))),
      12,
    );
  });

  it("idle auto-orbit advances theta; inertia decays when released", () => {
    const cam = new CoreCamera();
    const t0 = cam.state.theta;
    cam.update(0.016, 10_000, 0); // idle > 3500ms
    expect(cam.state.theta).toBeGreaterThan(t0);

    cam.setDragging(true);
    const held = cam.state.theta;
    cam.update(0.016, 10_001, 10_000); // recently touched: no drift
    expect(cam.state.theta).toBe(held);
    cam.setDragging(false);

    cam.state.vTheta = 0.1;
    cam.update(0.016, 10_002, 10_000);
    expect(cam.state.vTheta).toBeCloseTo(0.094, 12);
  });

  it("update builds an orthonormal basis aimed at the origin", () => {
    const cam = new CoreCamera();
    cam.update(0.016, 100, 0);
    expect(cam.fwd.length()).toBeCloseTo(1, 6);
    expect(cam.right.length()).toBeCloseTo(1, 6);
    expect(cam.upv.length()).toBeCloseTo(1, 6);
    // forward points from camera back to the origin
    expect(cam.fwd.dot(cam.camPos)).toBeLessThan(0);
  });

  it("projectOrigin centers CORE at Home and reports lens radius", () => {
    const cam = new CoreCamera();
    cam.update(0.016, 100, 0);
    const proj = cam.projectOrigin(16 / 9, 0.72);
    expect(proj).not.toBeNull();
    expect(proj!.u).toBeCloseTo(0.5, 6);
    expect(proj!.v).toBeCloseTo(0.5, 6);
    expect(proj!.lensR).toBeGreaterThan(0);
  });

  it("resetToHome restores the canonical framing", () => {
    const cam = new CoreCamera();
    cam.addImpulse(1, 1);
    cam.applyWheelZoom(500);
    cam.resetToHome();
    expect(cam.state).toEqual(HOME_CAMERA_STATE);
  });
});
