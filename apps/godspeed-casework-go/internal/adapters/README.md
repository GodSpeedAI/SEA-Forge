# adapters

## Overview
Provider-side adapters for the casework application. Adapters translate INTO `internal/ports` types:
provider, transport and vendor shape stops here and never reaches the core packages
(`ports`, `config`, `preflight`, `apperr`), which `internal/boundary` enforces mechanically.

## Subdirectories & Boundaries
- [./fakeauthority/README.md](./fakeauthority/README.md) - provider-side fake used by boundary/preflight tests; proves the adapter boundary is real
- [./sfwp/README.md](./sfwp/README.md) - the live governed-kernel adapter: SFWP NDJSON-over-Unix-socket client plus `ports.CaseAuthorityPort`/`RunTracePort`/`AskPort` implementations
