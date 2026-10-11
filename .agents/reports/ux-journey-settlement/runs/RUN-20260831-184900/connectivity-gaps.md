# SEA-Forge Connectivity Gaps & Affordance Audit

## Overview
This report audits the connectivity between interface projections, backend capability bindings, authority gates, and settlement criteria across all 12 canonical journeys.

## Findings
1. **Preview Surfaces vs Live Verbs**:
   - Routes `/memory`, `/capabilities`, `/artifacts`, and `/federation` correctly display `Specification preview · not live` and fail closed. No false availability leaks (`UNAVAILABLE_AFFORDANCE_LEAK`) were detected.
2. **Authority-Before-Effect**:
   - Authority decisions consistently precede side effects in all tested paths.
3. **Stale Precondition & Recovery**:
   - Stale template commits are rejected with explicit guidance to re-run preflight without creating orphan case state.
4. **Dropped Commit Recovery**:
   - Ambiguous commit states offer recovery via `request.get_status` while disabling duplicate commit actions.
