# Approved selector push capture archive

Date: 2026-10-06

This archive preserves the actual joined capture of the approved push. The raw transcript and exit marker are stored byte-for-byte in the adjacent `.raw` and `.exit` files; no transcript reconstruction or summary is substituted for the capture.

- Exact command: `e8f66a48dc9f979a208cbc08ee231ce776cb8b50:refs/heads/casework/live-wiring-resume-2026-10-05`
- Session: `21018`; joined exit status: `0`
- Raw capture: 239817 bytes; SHA-256 `10a8ad056187ad08b07c5208d983019638fe92f0e9b50275fc7d339baccf6660`
- Exit marker: 2 bytes (`0` plus newline); SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`
- Byte comparison: both archived files compare exactly equal to `/tmp/selector-approved-push-oct06.raw` and `/tmp/selector-approved-push-oct06.exit`, respectively.

The root verified the successful push and remote SHA. This manifest records the local evidence archive and does not replace that verification.
