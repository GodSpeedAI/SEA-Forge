#!/usr/bin/env python3
"""Real, minimal OpenAI-compatible HTTP stub for the UX Journey Settlement Gauntlet.

Mirrors crates/sea-forge-server/tests/conformance_case_authoring.rs's
`stub_endpoint()`: answers every request identically with a valid
chat-completion body, so the real `sequential_agents@0.1.0` template's
agent-task steps genuinely dispatch and complete against a real (if
scripted) HTTP endpoint, rather than a shortcut the harness fabricates.
The governance machinery around that call — authority evaluation, approval
escalation, trace/evidence recording, settlement — is all real; only the
LLM's answer content is canned, exactly as it is in the crate's own
conformance test.
"""
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BODY = b'{"choices":[{"message":{"content":"step complete"}}],"usage":{"total_tokens":1}}'


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        self.rfile.read(length)
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(BODY)))
        self.end_headers()
        self.wfile.write(BODY)

    def log_message(self, format, *args):
        pass  # keep stdout to just the readiness line below


def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 0
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(f"stub_agent_endpoint: listening on http://127.0.0.1:{server.server_port}/", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
