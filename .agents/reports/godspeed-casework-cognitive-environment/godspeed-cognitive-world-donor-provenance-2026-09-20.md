# GodSpeed Cognitive World donor provenance and inventory

Updated: 2026-09-20

This note is new evidence for the GodSpeed Cognitive World UI. It is separate
from the immutable older T02 evidence that discussed Open MCT/OpenMontage.
The five selected mechanic donors were inspected in `.tmp/donor/`; they are
not package dependencies and their product ontology does not enter the UI
contracts.

## Mechanical inventory

The revisions below were read from each checkout with `git remote -v`,
`git rev-parse HEAD`, and `git show -s --format='%H%n%ad%n%s' HEAD`.

| checkout | canonical remote | inspected HEAD | license / notice | use in this UI seam |
| --- | --- | --- | --- | --- |
| `tycho` | `https://github.com/jshor/tycho` | `3bf5e4224ef321ecd46df18f2dd42d54817e58f0` | `LICENSE.md`: MIT, Copyright (c) 2018-present Tycho.io by J. P. Shor | **Independently adapted affordances:** orbital/satellite placement and camera focus/zoom continuity. No Tycho source file is copied. |
| `racing-game` | `https://github.com/pmndrs/racing-game` | `7816a5d954b75e6ad853ae4e4f0cbbd628072643` | `LICENSE.md`: MIT, Copyright 2021 pmdrs, contributors | **Copied and tailored in `SceneCanvas.tsx`:** the ambient-fill plus directional-key R3F scene-light slice from `src/App.tsx:34-45`; GodSpeed adds a point fill and removes racing-specific layers, shadows, and game objects. |
| `blackhole-ts` | `https://github.com/rmarchet/blackhole-ts` | `ddc1cd9fd9bf669090fa3c291cd8cbb8538d244a` | `LICENSE`: MIT, Copyright (c) 2025 Roberto Marchetti | **Adapted in `CoreObject.tsx`:** ray setup, leapfrog geodesic stepping, Schwarzschild-style bending, photon/ring signal. Existing `COREOBJECT_PROVENANCE.md` retains the donor MIT notice. Donor textures/media are not used. |
| `BLACK-HOLE` | `https://github.com/frag2win/BLACK-HOLE` | `fb49f586560d77d1feb52b368f09b8a0644ff769` | README claims MIT and links `LICENSE`; `find` and `git ls-files` show no tracked `LICENSE`/`COPYING`/`NOTICE` file at this revision | **Inspected only. No code copied from it.** Its lensing, render-loop, particle/accretion, and leapfrog files were reviewed, but none is an extraction boundary for this UI. The README claim is not treated as a verified license file or dependency approval. |
| `threejs-galaxy-shader` | `https://github.com/AmitDigga/threejs-galaxy-shader` | `fd06407f390640f699949c26e2da68e788a67d34` | `LICENSE`: MIT, Copyright 2025 AMIT DIGGA | **Adapted in `StarDust.tsx`:** indexed time-phase drift and restrained inward orbital motion from `GalaxyShader.ts`'s spiral coordinate path. The UI keeps its own sparse shell and points material; no donor shader or geometry is copied. |

## Target seams and exact source evidence

### Spatial and camera affordances

Tycho's candidate affordances were read from:

* `.tmp/donor/tycho/src/modules/orbital.tsx:22-29,98-116,166` (orbital
  placement, satellite identity, focus callback, frame update);
* `.tmp/donor/tycho/src/components/orbital/orbital.tsx:61-87` (nested orbital
  groups and path/body composition);
* `.tmp/donor/tycho/src/elements/controls.ts:23-78,234-` (look-at target
  handoff, camera focus interpolation, and zoom tween); and
* `.tmp/donor/tycho/src/modules/scene.tsx:45,67-70,123-131` (R3F scene
  loop, recursive satellites, and semantic zoom input).

The GodSpeed-owned rederivation is in:

* `apps/godspeed-cognitive-ui/src/host/scene/layout.ts:42-50,58-154`:
  deterministic projection-angle orbit placement, parent-tied attention
  satellites, stable child phases, and semantic system/local/detail
  arrangements;
* `apps/godspeed-cognitive-ui/src/host/scene/CameraRig.tsx:1-64`:
  typed `CameraIntent`, renderer-owned journey arrival, zoom-band
  standpoints, reduced-motion snap, and eased `lookAt`; and
* `apps/godspeed-cognitive-ui/src/host/scene/journey.ts:1-76`:
  the GodSpeed-owned physical rebase/return-home continuity seam.

These are independently implemented mechanics behind GodSpeed contracts. No
Tycho product types, imports, or source text are present in the target seam.

### Core rendering

The adapted source was narrowed to:

* `.tmp/donor/blackhole-ts/src/shaders/fragmentShader/main.glsl:19-96`,
  including ray generation, leapfrog geodesic loop, disk hit, and escaped-ray
  star sampling;
* `.tmp/donor/blackhole-ts/src/shaders/vertexShader.glsl`; and
* `.tmp/donor/blackhole-ts/src/components/BlackHole.tsx:45-57,89-106,294-330`
  for the surrounding R3F shader/camera update context.

The receiving seam is
`apps/godspeed-cognitive-ui/src/host/scene/CoreObject.tsx:1-170`. Its own
comments and shader identify the adaptation at lines 38-40 and 58-104. It
uses a GodSpeed camera-facing quad, procedural sky/disk signal, quiet uniforms,
and reduced-motion behavior. No donor image, texture, product component, or
application ontology is imported. The existing
`apps/godspeed-cognitive-ui/src/host/scene/COREOBJECT_PROVENANCE.md` carries
the required blackhole-ts MIT notice.

### Cognitive object body and scene-light extraction

The object body now uses a source slice from Tycho's actual orbital composition rather than
rederiving the receiving structure from scratch. The copied receiving seam follows
`.tmp/donor/tycho/src/modules/orbital.tsx:121-135`: a body-centered group containing a sphere
geometry and a separately mounted shell/cloud layer. In `ObjectNode.tsx`, that source structure
is retained as the body sphere plus an independent atmospheric shell; the material is replaced
with a GodSpeed-owned procedural surface so cognitive objects need no donor textures, maps, or
astronomy data. The surrounding group, stable semantic identity, callbacks, labels, and artifact
signals remain GodSpeed-owned.

The scene lighting slice is copied from `.tmp/donor/racing-game/src/App.tsx:34-45`, specifically
its explicit `<ambientLight>` and `<directionalLight>` children directly under `<Canvas>`. It is
tailored here to the light surface with a warm directional key, a low neutral fill, and one cool
point fill; racing physics, controls, assets, and UI are not copied.

MIT notice retained for the copied Tycho source slice (from `.tmp/donor/tycho/LICENSE.md`):

> Copyright (c) 2018-present Tycho.io by J. P. Shor.
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of this software
> and associated documentation files (the "Software"), to deal in the Software without
> restriction, including without limitation the rights to use, copy, modify, merge, publish,
> distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
> Software is furnished to do so, subject to the following conditions: the above copyright notice
> and this permission notice shall be included in all copies or substantial portions of the Software.

MIT notice retained for the copied racing-game scene slice (from `.tmp/donor/racing-game/LICENSE.md`):

> Copyright 2021 pmdrs, contributors
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of this software
> and associated documentation files (the 'Software'), to deal in the Software without
> restriction, including without limitation the rights to use, copy, modify, merge, publish,
> distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
> Software is furnished to do so, subject to the following conditions: the above copyright notice
> and this permission notice shall be included in all copies or substantial portions of the Software.

### Inspection-only donors

Racing Game candidate files inspected:

* `.tmp/donor/racing-game/src/App.tsx:5-10,51-82` (R3F scene and UI/control
  composition);
* `.tmp/donor/racing-game/src/controls/Keyboard.ts:1-31` (keyboard lifecycle);
* `.tmp/donor/racing-game/src/effects/Cameras.tsx:1-12`; and
* `.tmp/donor/racing-game/src/effects/Dust.tsx:1-40` (frame-driven effect).

No Racing Game mechanic beyond the scene-light slice above is claimed as adapted in
`layout.ts`, `CameraRig.tsx`, or `CoreObject.tsx`.

BLACK-HOLE candidate files inspected:

* `.tmp/donor/BLACK-HOLE/src/core/RenderLoop.js` and `SceneManager.js`;
* `.tmp/donor/BLACK-HOLE/src/physics/integrators/Leapfrog.js`;
* `.tmp/donor/BLACK-HOLE/src/rendering/LensingRenderer.js` and
  `AccretionDiskRenderer.js`; and
* `.tmp/donor/BLACK-HOLE/README.md:10-20,73-85`.

The README advertises MIT at lines 6 and 83-85, but this checkout has no
tracked license file. No code is copied from BLACK-HOLE, and its README claim
does not approve dependency use.

Galaxy-shader candidate files inspected:

* `.tmp/donor/threejs-galaxy-shader/src/GalaxyGeometry.ts:2-20`; and
* `.tmp/donor/threejs-galaxy-shader/src/GalaxyShader.ts:3-88,92-247`
  (indexed spiral coordinates, time offset, distance fade, and black-hole
  boundary).

`StarDust.tsx` adapts only the indexed time-phase and restrained inward-orbit
mechanic from `GalaxyShader.ts`; its geometry, palette, and R3F point material
remain GodSpeed-owned. No GalaxyShader/GalaxyGeometry code is copied into the
target seam. The donor's MIT license and copyright notice are recorded in the
mechanical inventory above.

## Notice and scope decision

The donor mechanics currently explicitly retained in implementation are the
blackhole-ts adaptation, the narrow threejs-galaxy-shader particulate adaptation,
the copied-and-tailored Tycho body composition, and the copied-and-tailored
racing-game scene-light slice. The blackhole-ts full MIT notice remains in
`COREOBJECT_PROVENANCE.md`; object and scene-light notices are retained in this
report with their exact source paths. No donor textures or media are copied.
If a future change extracts source from any of those checkouts, the exact
corresponding MIT notice must be retained before that change is accepted. The
BLACK-HOLE README-only claim requires a separately verified license artifact
before any extraction; none is approved here.

## Reproduction commands

The inventory was established with:

```sh
for d in tycho racing-game blackhole-ts BLACK-HOLE threejs-galaxy-shader; do
  git -C .tmp/donor/$d remote -v
  git -C .tmp/donor/$d rev-parse HEAD
  git -C .tmp/donor/$d show -s --format='%H%n%ad%n%s' --date=iso-strict HEAD
  git -C .tmp/donor/$d ls-files
done
find .tmp/donor -maxdepth 2 -type f \( -iname 'LICENSE*' -o -iname 'COPYING*' -o -iname 'NOTICE*' \)
```

The target search found no `BLACK-HOLE` source path/import and no donor
dependency entry in `apps/godspeed-cognitive-ui/package.json`.
