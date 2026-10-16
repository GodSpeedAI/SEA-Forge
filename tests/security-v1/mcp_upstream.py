"""Unrestricted synthetic MCP upstream. It observes effects; it enforces no policy."""

import hashlib
import json
import os
from pathlib import Path
import sys


def invalid():
    return {"jsonrpc":"2.0","id":None,"error":{"code":-32600,"message":"invalid fixture request"}}


def handle(request, root):
    try:
        method, params = request.get("method"), request.get("params", {})
        if method == "notifications/initialized":
            return None
        if method == "initialize":
            result = {"protocolVersion":"2024-11-05","capabilities":{"tools":{}},
                      "serverInfo":{"name":"unrestricted-security-fixture","version":"1"}}
        elif method == "tools/list":
            result = {"tools":[{"name":name,"description":"synthetic fixture",
                                "inputSchema":{"type":"object"}}
                               for name in ("metadata.read","audit.delete")]}
        elif method == "tools/call":
            name, arguments = params["name"], params.get("arguments", {})
            event = {"request_id":request.get("id"),"tool":name,
                     "arguments_sha256":hashlib.sha256(json.dumps(arguments,sort_keys=True).encode()).hexdigest()}
            with (root / "invocations.jsonl").open("a") as log:
                log.write(json.dumps(event) + "\n")
                log.flush()
                os.fsync(log.fileno())
            if name == "audit.delete":
                (root / "audit.json").write_text("[]\n")
                result = {"content":[{"type":"text","text":"deleted"}]}
            elif name == "metadata.read":
                result = {"content":[{"type":"text","text":json.dumps(arguments,sort_keys=True)}]}
            else:
                raise ValueError("unknown fixture tool")
        else:
            raise ValueError("unknown fixture method")
        return {"jsonrpc":"2.0","id":request.get("id"),"result":result}
    except (ValueError,KeyError,TypeError,AttributeError):
        return invalid()


def main():
    root = Path(os.environ["SEA_FORGE_SECURITY_OBSERVER_DIR"])
    for line in sys.stdin.buffer:
        try:
            request = json.loads(line)
            if isinstance(request,list):
                response = [item for item in (handle(value,root) for value in request) if item is not None] if request else invalid()
            else:
                response = handle(request,root)
        except (ValueError,TypeError):
            response = invalid()
        if response is not None:
            sys.stdout.write(json.dumps(response) + "\n")
            sys.stdout.flush()


if __name__ == "__main__":
    main()
