#!/usr/bin/env python3
import json, subprocess, os

CODELOOM_BIN = os.path.join(os.path.dirname(__file__), "..", "target", "debug", "codeloom")

req = {
    "jsonrpc": "2.0", "id": 1,
    "method": "tools/call",
    "params": {
        "name": "codeloom_get_call_graph",
        "arguments": {
            "name": "initialize_logging",
            "repo": "expert-test",
            "branch": "main",
            "direction": "callees",
            "max_depth": 2
        }
    }
}
proc = subprocess.run(
    [CODELOOM_BIN, "mcp"],
    input=json.dumps(req).encode(),
    capture_output=True,
    timeout=30
)
resp = json.loads(proc.stdout)
print(json.dumps(resp, indent=2, ensure_ascii=False))
