# Approved push root verification

The operator approved exact commit e8f66a48dc9f979a208cbc08ee231ce776cb8b50 and history to GodSpeedAI/SEA-Forge on new branch casework/live-wiring-resume-2026-10-05. Root ran the normal push, joined session21018 exit0 and verified the resulting remote ref with read-only ls-remote. All eleven excluded index identities remained unchanged.

Actual command:

```text
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 git push origin e8f66a48dc9f979a208cbc08ee231ce776cb8b50:refs/heads/casework/live-wiring-resume-2026-10-05
```

Fresh preflight had2688MiB available RAM and4024MiB free swap; root held exclusive compiler ownership through the joined exit. Normal pre-push CI passed, including Gitleaks509commits/no leaks, workspace all-features tests, no-async-kernel and final build. Actual capture contains125suite summaries,1110passed,0failed,4ignored. Extra Go/UI/browser/proof and other-platform gates are not covered by that claim.

Root independently compared the native archived raw transcript239817bytes and exit2bytes with their actual `/tmp` originals: both byte-exact. Hashes10a8ad056187ad08b07c5208d983019638fe92f0e9b50275fc7d339baccf6660 and9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa match the archive manifest. Clarification: the archive manifest's “Exact command” bullet contains only the refspec; the complete command is above. No hook bypass, history rewrite or force operation occurred.
