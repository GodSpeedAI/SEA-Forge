# workbench/apps

## Overview
Workspace applications under the `workbench` Bun workspace (`workspaces: ["apps/*", "packages/*"]` in `/home/sprime01/projects/sea-rs/workbench/package.json`). This directory currently contains a single application, `desktop`, which is the entire user-facing Workbench frontend.

## Subdirectories & Boundaries
- desktop: [./desktop/README.md] - React 19 + TypeScript + Vite Workbench renderer (`desktop` package): spatial UI around a persistent WebGL2 CORE viewport, governed data layer over the Tauri host bridge, guard-blocked routing, and E2E evidence setup.
