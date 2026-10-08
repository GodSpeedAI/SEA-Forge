import { expect, test } from 'bun:test'
import type { Camera, Vec3 } from '../model/types'
import {
  project,
  unproject,
  panBy,
  zoomAt,
  clampTilt,
  lerpCamera,
  easeInOutCubic,
  flightDuration,
  sampleFlight,
  type Viewport,
  type Flight,
} from './camera'

const vp: Viewport = { w: 800, h: 600, fx: 400, fy: 300 }
const epsilon = 1e-10

test('project: camera target lands on focal point', () => {
  const cam: Camera = { x: 100, y: 200, zoom: 1, tilt: 0, roll: 0 }
  const target: Vec3 = { x: 100, y: 200, z: 0 }

  const projected = project(target, cam, vp)

  expect(Math.abs(projected.sx - vp.fx)).toBeLessThan(epsilon)
  expect(Math.abs(projected.sy - vp.fy)).toBeLessThan(epsilon)
})

test('project/unproject round-trip at tilt 0, roll 0', () => {
  const cam: Camera = { x: 50, y: 75, zoom: 2, tilt: 0, roll: 0 }
  const world: Vec3 = { x: 60, y: 85, z: 0 }

  const projected = project(world, cam, vp)
  const unprojected = unproject(projected.sx, projected.sy, cam, vp)

  expect(Math.abs(unprojected.x - world.x)).toBeLessThan(epsilon)
  expect(Math.abs(unprojected.y - world.y)).toBeLessThan(epsilon)
})

test('project/unproject round-trip at tilt 0, roll 0.2', () => {
  const cam: Camera = { x: 50, y: 75, zoom: 2, tilt: 0, roll: 0.2 }
  const world: Vec3 = { x: 60, y: 85, z: 0 }

  const projected = project(world, cam, vp)
  const unprojected = unproject(projected.sx, projected.sy, cam, vp)

  expect(Math.abs(unprojected.x - world.x)).toBeLessThan(epsilon)
  expect(Math.abs(unprojected.y - world.y)).toBeLessThan(epsilon)
})

test('zoomAt keeps world point under cursor fixed (tilt 0)', () => {
  const cam: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const screenX = 450
  const screenY = 350

  // Get the world point under the cursor
  const unprojected = unproject(screenX, screenY, cam, vp)
  const world: Vec3 = { x: unprojected.x, y: unprojected.y, z: 0 }

  // Zoom in by 2x at that point
  const zoomed = zoomAt(cam, 2, screenX, screenY, vp)

  // Project the original world point with the new camera
  const newProjection = project(world, zoomed, vp)

  // It should be under the same screen point
  expect(Math.abs(newProjection.sx - screenX)).toBeLessThan(1)
  expect(Math.abs(newProjection.sy - screenY)).toBeLessThan(1)
})

test('zoom clamping: min limit', () => {
  const cam: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const limits = { min: 0.5, max: 10 }

  // Try to zoom out below minimum
  const zoomed = zoomAt(cam, 0.1, vp.fx, vp.fy, vp, limits)

  expect(zoomed.zoom).toEqual(0.5)
})

test('zoom clamping: max limit', () => {
  const cam: Camera = { x: 0, y: 0, zoom: 10, tilt: 0, roll: 0 }
  const limits = { min: 0.5, max: 10 }

  // Try to zoom in above maximum
  const zoomed = zoomAt(cam, 3, vp.fx, vp.fy, vp, limits)

  expect(zoomed.zoom).toEqual(10)
})

test('zoom clamping: default limits', () => {
  const cam: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }

  // Try to zoom out below default minimum (0.25)
  const zoomedOut = zoomAt(cam, 0.1, vp.fx, vp.fy, vp)
  expect(zoomedOut.zoom).toEqual(0.25)

  // Try to zoom in above default maximum (40)
  const cam2: Camera = { x: 0, y: 0, zoom: 40, tilt: 0, roll: 0 }
  const zoomedIn = zoomAt(cam2, 3, vp.fx, vp.fy, vp)
  expect(zoomedIn.zoom).toEqual(40)
})

test('lerpCamera: endpoints exact', () => {
  const a: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const b: Camera = { x: 100, y: 200, zoom: 4, tilt: 0.5, roll: 0.3 }

  const at0 = lerpCamera(a, b, 0)
  expect(at0.x).toEqual(a.x)
  expect(at0.y).toEqual(a.y)
  expect(at0.zoom).toEqual(a.zoom)
  expect(at0.tilt).toEqual(a.tilt)
  expect(at0.roll).toEqual(a.roll)

  const at1 = lerpCamera(a, b, 1)
  expect(at1.x).toEqual(b.x)
  expect(at1.y).toEqual(b.y)
  expect(at1.zoom).toEqual(b.zoom)
  expect(at1.tilt).toEqual(b.tilt)
  expect(at1.roll).toEqual(b.roll)
})

test('lerpCamera: zoom geometric at t=0.5', () => {
  const a: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const b: Camera = { x: 0, y: 0, zoom: 4, tilt: 0, roll: 0 }

  const mid = lerpCamera(a, b, 0.5)

  // Geometric mean of 1 and 4 is sqrt(4) = 2
  expect(Math.abs(mid.zoom - 2)).toBeLessThan(epsilon)
})

test('panBy moves a world point with the pointer (tilt 0)', () => {
  const cam: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const world: Vec3 = { x: 50, y: 100, z: 0 }

  // Project the world point
  const proj1 = project(world, cam, vp)

  // Pan by 30 pixels right and 40 pixels down
  const panned = panBy(cam, 30, 40)

  // Project the same world point with the panned camera
  const proj2 = project(world, panned, vp)

  // The screen coordinates should have moved by the opposite amount
  // (because the camera moved, the world point appears to move on screen)
  expect(Math.abs((proj2.sx - proj1.sx) - (-30))).toBeLessThan(epsilon)
  expect(Math.abs((proj2.sy - proj1.sy) - (-40))).toBeLessThan(epsilon)
})

test('clampTilt: clamps to [0, 1.2]', () => {
  expect(clampTilt(-0.5)).toEqual(0)
  expect(clampTilt(0.5)).toEqual(0.5)
  expect(clampTilt(1.2)).toEqual(1.2)
  expect(clampTilt(1.5)).toEqual(1.2)
  expect(clampTilt(2.0)).toEqual(1.2)
})

test('easeInOutCubic: range [0, 1] for input [0, 1]', () => {
  expect(easeInOutCubic(0)).toEqual(0)
  expect(easeInOutCubic(1)).toEqual(1)
  expect(easeInOutCubic(0.5)).toBeGreaterThan(0.4)
  expect(easeInOutCubic(0.5)).toBeLessThan(0.6)
})

test('flightDuration: basic calculation', () => {
  const a: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const b: Camera = { x: 100, y: 200, zoom: 1, tilt: 0, roll: 0 }

  const duration = flightDuration(a, b)

  // Should be clamped between 650 and 1800
  expect(duration).toBeGreaterThanOrEqual(650)
  expect(duration).toBeLessThanOrEqual(1800)
})

test('flightDuration: zoom change increases duration', () => {
  const a: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const b1: Camera = { x: 0, y: 0, zoom: 2, tilt: 0, roll: 0 }
  const b2: Camera = { x: 0, y: 0, zoom: 4, tilt: 0, roll: 0 }

  const duration1 = flightDuration(a, b1)
  const duration2 = flightDuration(a, b2)

  // Larger zoom ratio should result in longer duration
  expect(duration2).toBeGreaterThan(duration1)
})

test('sampleFlight: returns from camera at start', () => {
  const from: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const to: Camera = { x: 100, y: 100, zoom: 2, tilt: 0.5, roll: 0 }
  const flight: Flight = { from, to, start: 1000, duration: 500 }

  const result = sampleFlight(flight, 1000)

  expect(result.cam.x).toEqual(from.x)
  expect(result.cam.y).toEqual(from.y)
  expect(result.cam.zoom).toEqual(from.zoom)
  expect(result.done).toEqual(false)
})

test('sampleFlight: returns to camera at end', () => {
  const from: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const to: Camera = { x: 100, y: 100, zoom: 2, tilt: 0.5, roll: 0 }
  const flight: Flight = { from, to, start: 1000, duration: 500 }

  const result = sampleFlight(flight, 1500)

  expect(result.cam.x).toEqual(to.x)
  expect(result.cam.y).toEqual(to.y)
  expect(result.cam.zoom).toEqual(to.zoom)
  expect(result.cam.tilt).toEqual(to.tilt)
  expect(result.done).toEqual(true)
})

test('sampleFlight: returns done=false between start and end', () => {
  const from: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const to: Camera = { x: 100, y: 100, zoom: 2, tilt: 0.5, roll: 0 }
  const flight: Flight = { from, to, start: 1000, duration: 500 }

  const result = sampleFlight(flight, 1250)

  expect(result.done).toEqual(false)
})

test('sampleFlight: interpolates correctly at midpoint', () => {
  const from: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const to: Camera = { x: 100, y: 100, zoom: 2, tilt: 0.5, roll: 0 }
  const flight: Flight = { from, to, start: 1000, duration: 500 }

  const result = sampleFlight(flight, 1250)

  // At midpoint with ease-in-out-cubic, the value should be around 0.5 (but not exactly)
  expect(result.cam.x).toBeGreaterThan(0)
  expect(result.cam.x).toBeLessThan(100)
  expect(result.cam.y).toBeGreaterThan(0)
  expect(result.cam.y).toBeLessThan(100)
})

test('project with depth (z != 0)', () => {
  const cam: Camera = { x: 0, y: 0, zoom: 1, tilt: 0, roll: 0 }
  const worldFront: Vec3 = { x: 0, y: 0, z: 100 }
  const worldBack: Vec3 = { x: 0, y: 0, z: -100 }

  const projFront = project(worldFront, cam, vp)
  const projBack = project(worldBack, cam, vp)

  // Points in front should have larger scale (appear larger)
  expect(projFront.scale).toBeGreaterThan(projBack.scale)
})

test('project with tilt', () => {
  const cam: Camera = { x: 0, y: 0, zoom: 1, tilt: 0.3, roll: 0 }
  const world: Vec3 = { x: 0, y: 100, z: 0 }

  const proj = project(world, cam, vp)

  // With tilt, the projected position should be different from no tilt
  const projNoTilt = project(world, { ...cam, tilt: 0 }, vp)

  expect(Math.abs(proj.sy - projNoTilt.sy)).toBeGreaterThan(0.1)
})
