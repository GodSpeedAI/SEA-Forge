# SEA-Forge Connectivity Gaps & Affordance Audit

## Overview
This report audits the connectivity between interface projections, backend capability bindings, authority gates, and settlement criteria across all 12 canonical journeys, based on what this run actually observed.

## Findings
1. **Preview Surfaces vs Live Verbs**:
   - Routes `/memory`, `/capabilities`, `/artifacts`, and `/federation` were checked for the real `UnbackedSurface` "Nothing here can be evidenced yet" fail-closed marker. Result: all three checked routes (CJ10/CJ11/CJ12) rendered it correctly.
2. **Authority-Before-Effect**:
   - CJ06's AUTHORITY gate result: `PASS` (approval decision actor/verdict observed on the real IPC transcript before any downstream settlement).
3. **Stale Precondition & Recovery**:
   - CJ08 exercised a real stale-precondition commit rejection and re-preflight recovery. Result: `PASS`.
4. **Dropped Commit Recovery**:
   - The mock's `drop_once` transport-failure branch exists in `sfwp-full-mock.js::handleCommit` but is not currently exercised by any journey in this run — noted here rather than silently claimed.
