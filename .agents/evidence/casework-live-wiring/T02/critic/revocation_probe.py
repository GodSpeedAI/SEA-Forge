#!/usr/bin/env python3
"""Probe the ADR revocation claim: "Bindings and allowlist entries are read
from the live server config on every request... Removing a binding or
allowlist entry takes effect on the NEXT request".

Empirically: edit server.yaml to shrink the allowlist, then send delegated
inspect (identity_get) requests — which do NOT go through commit_plan, the
only runtime reload_config() caller — and observe whether the old allowlist
is still honoured, then trigger one commit and observe again.
"""
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from attack_driver import (  # noqa: E402
    Client, Server, UID, OTHER_UID, seed_gateway_cell, install_e2e_fixtures,
    delegated, commit_request,
)


def main():
    root = Path(tempfile.mkdtemp(prefix="t02-critic-revocation-"))
    seed_gateway_cell(root)
    install_e2e_fixtures(root)
    server = Server(root, "revocation-cell")
    try:
        client = Client(server.socket)

        v1 = client.call(**delegated({"verb": "identity_get"}, "operator_b", "operator"))
        log1 = f"before revoke: effective_actor={v1.get('effective_actor', {}).get('actor_id')}"
        print(f"[revoke] {log1}", flush=True)

        # Shrink the allowlist: drop operator_b entirely.
        cfg = (root / "server.yaml").read_text()
        cfg2 = cfg.replace(
            "delegable_actors: [operator_a, operator_b, security_officer]",
            "delegable_actors: [operator_a, security_officer]",
        )
        assert cfg2 != cfg, "allowlist edit failed to apply"
        (root / "server.yaml").write_text(cfg2)

        # Non-commit delegated request: does the revocation bite?
        v2 = client.call(**delegated({"verb": "identity_get"}, "operator_b", "operator"))
        print(f"[revoke] identity_get after edit, NO commit since: "
              f"effective_actor={(v2.get('effective_actor') or {}).get('actor_id')} "
              f"refusal={(v2.get('refusal') or {}).get('error_class')}", flush=True)

        # A delegated protected commit forces reload_config (commit_plan).
        committed = client.call(**delegated(
            commit_request("operator_a"), "operator_a", "operator"))
        err = committed.get("error_class")
        print(f"[revoke] delegated commit as operator_a (forces reload): "
              f"{'ok' if not err else err}", flush=True)

        # Same non-commit delegated request again, after the reload.
        v3 = client.call(**delegated({"verb": "identity_get"}, "operator_b", "operator"))
        print(f"[revoke] identity_get after a commit-triggered reload: "
              f"effective_actor={(v3.get('effective_actor') or {}).get('actor_id')} "
              f"refusal={(v3.get('refusal') or {}).get('error_class')}", flush=True)
    finally:
        server.stop()
    return 0


if __name__ == "__main__":
    sys.exit(main())
