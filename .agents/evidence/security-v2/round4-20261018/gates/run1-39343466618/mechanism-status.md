## Kernel-security mechanism status

- Landlock ABI on this runner: 7
- Native Landlock RESOLVE_UNIX path: NOT-YET-EXECUTABLE here (ABI 7 < 9); the ABI-9 ruleset code is compiled but unreachable, and this run exercises only the secured fallback
- seccomp user-notify fallback: installable on this runner; gate verdicts below are executable

### Gate verdicts
```
FAIL sandbox-suite-no-require (see .agents/evidence/security-v2/round4-20261018/gates/sandbox-suite-no-require.log)
PASS postgres-broker-no-require
FAIL sandbox-suite-no-require (see .agents/evidence/security-v2/round4-20261018/gates/sandbox-suite-no-require.log)
PASS postgres-broker-no-require
```
