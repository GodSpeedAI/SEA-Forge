"""Trusted fixture deployment only; enforcement belongs to sea-forge-mcp."""
import json
import os
from pathlib import Path
import sys
import tempfile

if len(sys.argv) != 2:
    raise SystemExit("usage: provision_mcp.py <sea-forge-mcp-binary>")
binary = Path(sys.argv[1]).resolve(strict=True)
observer = Path(os.environ["SEA_FORGE_SECURITY_OBSERVER_DIR"]).resolve(strict=True)
upstream = json.loads(os.environ["SEA_FORGE_SECURITY_UPSTREAM_COMMAND"])
root = Path(tempfile.mkdtemp(prefix="broker-", dir=observer))
workspace = root / "workspace"
workspace.mkdir(mode=0o700)
config = {
    "root": str(root), "workspace": str(workspace),
    "policy": str(Path(__file__).with_name("mcp-policy.yaml").resolve(strict=True)),
    "actor": {"actor_id": "security-metadata-workload", "role": "operator"},
    "server": "security-upstream", "upstream_argv": upstream,
    "upstream_env": {
        "PATH": os.environ.get("PATH", os.defpath), "HOME": str(workspace),
        "SEA_FORGE_SECURITY_OBSERVER_DIR": str(observer),
    },
}
path = root / "mcp.json"
path.write_text(json.dumps(config))
path.chmod(0o600)
os.execv(str(binary), [str(binary), str(path)])
