#!/usr/bin/env python3
import re

path = '/mnt/d/RagMcpHermes/codeloom/src/indexer/clang/ast.rs'
with open(path) as f:
    lines = f.readlines()

# Remove all lines containing EDEBUG-VMETH or EDEBUG-OVERRIDE
new_lines = [l for l in lines if 'EDEBUG-VMETH' not in l and 'EDEBUG-OVERRIDE' not in l]
print(f"Removed {len(lines) - len(new_lines)} debug lines")
assert len(new_lines) < len(lines)  # sanity

with open(path, 'w') as f:
    f.writelines(new_lines)
print("Done")
