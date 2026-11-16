# Adapters

## Overview
The two sources behind the one contract port (`src/ports/contract.ts`): the live HTTP adapter for the Go casework gateway (the production path) and the local contract-conformant adapter for dev and tests, plus the shared conformance suite both are held to. The local adapter stays behind a dynamic import in `src/main.tsx` so production bundles cannot ship it.

## Subdirectories & Boundaries
- HTTP adapter (live gateway): [./http/README.md] - The production `CaseworkPort`/`SessionPort`/`TemplateSourcePort` over same-origin fetch + SSE to `apps/godspeed-casework-go`, with typed refusals, backoff reconnects and cookie/CSRF handling.
- Local contract adapter (dev/test fixture): [./local/README.md] - In-memory, contract-conformant adapter over authored Northstar data with simulated governed execution and typed refusals.
- Port conformance: [./conformance/README.md] - The shared behavioral assertions (`runCaseworkPortConformance`) run against both adapters.
