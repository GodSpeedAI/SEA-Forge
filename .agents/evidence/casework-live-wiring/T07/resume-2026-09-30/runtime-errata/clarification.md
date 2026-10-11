# Runtime confirmation errata — 2026-09-30

This note supplements the immutable `final-independent-runtime-2026-09-30/confirmation.md`; it does not rewrite that record.

1. The line saying the verifier “held no compile token” is wrong. The verifier held the exclusive compile token. No gates were rerun in this runtime pass because no production source was changed and the assigned task was independent live confirmation. Prior unchanged-source gate evidence is referenced in the report.
2. The report's failing probe descriptions preserve the actual expectation errors, but the tool wrapper did not persist every `exec_command.exit_code` or full stdout to a file. `command-captures.md` below records exact commands, exact result text available from the session, and explicitly marks unretained output/exit data. No missing output or status is reconstructed.
3. After revocation, the restarted task kernel is PID 848392; `/proc` checks confirmed the expected executable and the fresh-cell lock descriptors. Gateway PID 800034 is alive. Preexisting Vite PID 310487 remains listening on 127.0.0.1:4178. No cleanup was performed.
4. The production-mode dev-auth negative startup was run with an HTTPS-valid test origin and exited 2 with only the intended dev-auth refusal. Port 44180 was not listening after the refusal.
5. `new_cursor` race evidence is tied to the retained trajectory: the immediate request failed for cursor `01M3SAPGD6W1CZYNPCPAZS58M3`; later the trajectory API returned that exact cursor as its six-point base. The cursor had not been evicted.
