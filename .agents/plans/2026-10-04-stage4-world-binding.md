# Stage 4 plan — SEA-Forge semantic-world binding
1. Add `world` module + `WorldBindingError` (typed, fail-closed) to sea-forge-domainforge.
2. Add `world_ref` to `DomainModelRef`.
3. Tests: register/cache, unknown world, tampered identity, digest mismatch, invalid-model flag, model_ref mismatch, snapshot from real DomainForge envelope (positive), idempotency, name conflict.
4. `just check` + `just test`; settlement note; status files; `just context-check`; commit; push.
