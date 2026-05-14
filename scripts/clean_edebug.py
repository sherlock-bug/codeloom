#!/usr/bin/env python3
"""Remove all remaining EDEBUG eprintln! lines from ast.rs"""
path = '/mnt/d/RagMcpHermes/codeloom/src/indexer/clang/ast.rs'
with open(path) as f:
    content = f.read()

# Remove the extract_calls debug
content = content.replace(
    '                        eprintln!(\n                            "EDEBUG: extract_calls for {} (is_external={})",\n                            &qname, is_external\n                        );\n',
    ''
)

# Remove the MemberExpr debug (single line + multi-line)
content = content.replace(
    '                        eprintln!("EDEBUG: extract_calls for {} (is_external={})", &qname, is_external);\n',
    ''
)

# Single-line MemberExpr
content = content.replace(
    '            eprintln!("EDEBUG: found MemberExpr, name={:?}", n.get("name").and_then(|v| v.as_str()));\n',
    ''
)

# Multi-line MemberExpr
content = content.replace(
    '            eprintln!(\n                "EDEBUG: found MemberExpr, name={:?}",\n                n.get("name").and_then(|v| v.as_str())\n            );\n',
    ''
)

# MemberExpr RESOLVED
content = content.replace(
    '                    eprintln!("EDEBUG: MemberExpr call RESOLVED: {}::{}", cls, method_name);\n',
    ''
)
content = content.replace(
    '                    eprintln!(\n                        "EDEBUG: MemberExpr call RESOLVED: {}::{}",\n                        cls, method_name\n                    );\n',
    ''
)

# MemberExpr UNRESOLVED
content = content.replace(
    '                    eprintln!("EDEBUG: MemberExpr call UNRESOLVED: {} (no class found)", method_name);\n',
    ''
)
content = content.replace(
    '                    eprintln!(\n                        "EDEBUG: MemberExpr call UNRESOLVED: {} (no class found)",\n                        method_name\n                    );\n',
    ''
)

with open(path, 'w') as f:
    f.write(content)

print("Done - all EDEBUG removed")
