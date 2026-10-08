import type { Camera, Vec3 } from '../model/types'

export interface Viewport {
  w: number
  h: number
  /** focal point in screen px */
  fx: number
  fy: number
}

export interface Projected {
  sx: number
  sy: number
  /** screen px per world unit at this point (includes depth) */
  scale: number
}

export const PERSPECTIVE = 1400 // world units; used for depth z

/**
 * Project a world point to screen coordinates.
 *
 * Projection math:
 *   dx = p.x - cam.x; dy = p.y - cam.y
 *   ty = dy * cos(cam.tilt)
 *   tz = dy * sin(cam.tilt)
 *   depth = PERSPECTIVE / (PERSPECTIVE - p.z * 1 + tz * 0.15)
 *   rx = dx * cos(roll) - ty * sin(roll)
 *   ry = dx * sin(roll) + ty * cos(roll)
 *   sx = vp.fx + rx * cam.zoom * depth
 *   sy = vp.fy + ry * cam.zoom * depth
 *   scale = cam.zoom * depth
 */
export function project(p: Vec3, cam: Camera, vp: Viewport): Projected {
  const dx = p.x - cam.x
  const dy = p.y - cam.y

  const cosTilt = Math.cos(cam.tilt)
  const sinTilt = Math.sin(cam.tilt)
  const ty = dy * cosTilt
  const tz = dy * sinTilt

  // z and the tilt term are in screen px so depth behaves the same at every nesting level.
  const depth = PERSPECTIVE / (PERSPECTIVE - p.z + tz * cam.zoom * 0.15)

  const cosRoll = Math.cos(cam.roll)
  const sinRoll = Math.sin(cam.roll)
  const rx = dx * cosRoll - ty * sinRoll
  const ry = dx * sinRoll + ty * cosRoll

  const sx = vp.fx + rx * cam.zoom * depth
  const sy = vp.fy + ry * cam.zoom * depth
  const scale = cam.zoom * depth

  return { sx, sy, scale }
}

/**
 * Inverse of project for z=0 (depth ignored when tilt and z are 0).
 * Converts screen coordinates back to world coordinates.
 * Used for dragging and must be an exact inverse when depth=1 (z=0, tilt small).
 */
export function unproject(
  sx: number,
  sy: number,
  cam: Camera,
  vp: Viewport
): { x: number; y: number } {
  // Translate from screen to centered coordinates
  const rx = (sx - vp.fx) / cam.zoom
  const ry = (sy - vp.fy) / cam.zoom

  // Undo roll (inverse rotation)
  const cosRoll = Math.cos(cam.roll)
  const sinRoll = Math.sin(cam.roll)
  const dx = rx * cosRoll + ry * sinRoll
  const ty = -rx * sinRoll + ry * cosRoll

  // Undo tilt (inverse: divide y by cos(tilt))
  const cosTilt = Math.cos(cam.tilt)
  const dy = ty / cosTilt

  // Add camera offset to get world coordinates
  return {
    x: cam.x + dx,
    y: cam.y + dy,
  }
}

/**
 * Pan the camera by moving the world with the pointer (inverse of project for z=0).
 * Accounts for zoom, tilt, and roll.
 * Used for drag interactions where the user's pointer movement should map to world movement.
 */
export function panBy(cam: Camera, dxScreen: number, dyScreen: number): Camera {
  // Translate screen motion to centered coordinates
  const rx = dxScreen / cam.zoom
  const ry = dyScreen / cam.zoom

  // Undo roll
  const cosRoll = Math.cos(cam.roll)
  const sinRoll = Math.sin(cam.roll)
  const dx = rx * cosRoll + ry * sinRoll
  const ty = -rx * sinRoll + ry * cosRoll

  // Undo tilt
  const cosTilt = Math.cos(cam.tilt)
  const dy = ty / cosTilt

  return {
    ...cam,
    x: cam.x + dx,
    y: cam.y + dy,
  }
}

/**
 * Zoom at a screen point, keeping the world point under that screen point fixed.
 * Clamps zoom to limits (default min 0.25, max 40).
 */
export function zoomAt(
  cam: Camera,
  factor: number,
  sx: number,
  sy: number,
  vp: Viewport,
  limits?: { min: number; max: number }
): Camera {
  const minZoom = limits?.min ?? 0.25
  const maxZoom = limits?.max ?? 40

  // Get the world point under the cursor
  const unprojected = unproject(sx, sy, cam, vp)
  const world: Vec3 = { x: unprojected.x, y: unprojected.y, z: 0 }

  // Calculate new zoom
  let newZoom = cam.zoom * factor
  newZoom = Math.max(minZoom, Math.min(maxZoom, newZoom))

  // Calculate the offset needed to keep the world point under the cursor
  const projected = project(world, { ...cam, zoom: newZoom }, vp)

  // The world point moved on screen after zoom; compute camera offset to fix it.
  // When world point moves by offset_s on screen, camera needs to move offset_s / newZoom
  // in world space, in the same direction as the screen offset.
  const offset_sx = projected.sx - sx
  const offset_sy = projected.sy - sy

  const dx = offset_sx / newZoom
  const dy = offset_sy / newZoom

  return {
    ...cam,
    zoom: newZoom,
    x: cam.x + dx,
    y: cam.y + dy,
  }
}

/**
 * Clamp tilt to [0, 1.2] radians.
 */
export function clampTilt(t: number): number {
  return Math.max(0, Math.min(1.2, t))
}

/**
 * Linear interpolation between two cameras.
 * x, y, tilt, roll: linear interpolation.
 * zoom: geometric interpolation (a.zoom * (b.zoom/a.zoom)^t).
 * For smooth screen-space motion when zoom changes significantly (|log(b.zoom/a.zoom)| > 0.05),
 * uses weighted x,y interpolation where the weight transitions from 0 at t=0 to 1 at t=1,
 * accounting for the change in world-to-screen scale.
 */
export function lerpCamera(a: Camera, b: Camera, t: number): Camera {
  // Linear interpolation for tilt and roll
  const tilt = a.tilt + (b.tilt - a.tilt) * t
  const roll = a.roll + (b.roll - a.roll) * t

  // Geometric interpolation for zoom
  const zoomRatio = b.zoom / a.zoom
  const zoom = a.zoom * Math.pow(zoomRatio, t)

  // Smooth x,y interpolation accounting for zoom change
  let w = t
  const logZoomRatio = Math.log(zoomRatio)
  if (Math.abs(logZoomRatio) > 0.05) {
    // Weight based on change in 1/zoom (screen-to-world scaling)
    const invZoomA = 1 / a.zoom
    const invZoomB = 1 / b.zoom
    const invZoomT = 1 / zoom
    w = (invZoomT - invZoomA) / (invZoomB - invZoomA)
  }

  const x = a.x + (b.x - a.x) * w
  const y = a.y + (b.y - a.y) * w

  return { x, y, zoom, tilt, roll }
}

/**
 * Ease function: ease-in-out cubic.
 */
export function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2
}

export interface Flight {
  from: Camera
  to: Camera
  start: number
  duration: number
}

/**
 * Calculate the duration in ms for a camera flight.
 * Formula: 700 + 260*|log2(b.zoom/a.zoom)| + 0.35*screen-distance-at-a.zoom
 * Clamped to [650, 1800].
 */
export function flightDuration(a: Camera, b: Camera): number {
  const zoomRatio = b.zoom / a.zoom
  const logZoomTerm = Math.abs(Math.log2(zoomRatio))

  // Screen distance at zoom a (without depth)
  const screenDx = (b.x - a.x) * a.zoom
  const screenDy = (b.y - a.y) * a.zoom
  const screenDistance = Math.sqrt(screenDx * screenDx + screenDy * screenDy)

  const duration = 700 + 260 * logZoomTerm + 0.35 * screenDistance

  return Math.max(650, Math.min(1800, duration))
}

/**
 * Sample a flight at a given time.
 * Returns the camera at that time and whether the flight is done.
 */
export function sampleFlight(f: Flight, now: number): { cam: Camera; done: boolean } {
  const elapsed = now - f.start
  if (elapsed <= 0) {
    return { cam: f.from, done: false }
  }
  if (elapsed >= f.duration) {
    return { cam: f.to, done: true }
  }

  const t = elapsed / f.duration
  const eased = easeInOutCubic(t)
  const cam = lerpCamera(f.from, f.to, eased)

  return { cam, done: false }
}
