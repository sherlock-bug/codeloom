#!/usr/bin/env python3
"""Filter Clang -ast-dump=json: strip system headers and function bodies.

Reads full Clang AST JSON from stdin (piped from clang), strips noise,
outputs compact filtered JSON to stdout.

Usage: clang ... -ast-dump=json file.cpp | python3 clang_filter.py <project_root>
"""

import json
import sys
import os

# -- Node kinds to strip (statement/expression "body" content) --
BODY_KINDS = frozenset({
    "CompoundStmt", "IfStmt", "ForStmt", "WhileStmt", "DoStmt", "SwitchStmt",
    "ReturnStmt", "DeclStmt", "BreakStmt", "ContinueStmt", "GotoStmt",
    "NullStmt", "LabelStmt", "CaseStmt", "DefaultStmt",
    "GCCAsmStmt", "AttributedStmt",
    "CallExpr", "CXXMemberCallExpr", "CXXOperatorCallExpr",
    "BinaryOperator", "UnaryOperator", "ConditionalOperator",
    "ImplicitCastExpr", "CXXStaticCastExpr", "CXXDynamicCastExpr",
    "CXXReinterpretCastExpr", "CXXConstCastExpr", "CStyleCastExpr",
    "CXXFunctionalCastExpr", "ParenExpr",
    "DeclRefExpr", "MemberExpr", "CXXDependentScopeMemberExpr",
    "UnresolvedLookupExpr", "CXXThisExpr", "CXXNullPtrLiteralExpr",
    "IntegerLiteral", "FloatingLiteral", "CharacterLiteral", "StringLiteral",
    "ArraySubscriptExpr", "InitListExpr", "CXXConstructExpr",
    "MaterializeTemporaryExpr", "CXXBindTemporaryExpr",
    "CXXNewExpr", "CXXDeleteExpr", "CXXThrowExpr",
    # Attributes
    "WarnUnusedResultAttr", "AlwaysInlineAttr", "VisibilityAttr",
})


def get_node_file(node):
    """Extract the source file path from a Clang AST node. Returns '' if unknown."""
    loc = node.get("loc", {})
    if not isinstance(loc, dict) or not loc:
        return ""
    file = loc.get("file", "")
    if file:
        return file
    # Header declarations: file is empty, but includedFrom has the TU file
    incl = loc.get("includedFrom", {})
    if isinstance(incl, dict):
        return incl.get("file", "")
    return ""


def is_system(node, project_root):
    """Check if node belongs to system headers (not under project_root)."""
    file = get_node_file(node)
    if not file:
        # No file path at all. Check if node has any location info (line/col).
        # If it has line/col but no file, Clang omitted the file field
        # and the node belongs to the project — don't strip it.
        loc = node.get("loc", {})
        if isinstance(loc, dict) and loc.get("line"):
            return False  # has line info → project node with missing file
        # No location info at all → treat as system (compiler builtin)
        return True
    return not file.startswith(project_root)


def filter_node(node, project_root):
    """Recursively filter a Clang AST node. Returns filtered dict or None."""
    if not isinstance(node, dict):
        return node

    kind = node.get("kind", "")

    # 1. Strip all body/attribute content
    if kind in BODY_KINDS:
        return None

    # 2. Strip system nodes (check BEFORE recursing into children)
    if is_system(node, project_root):
        return None

    # 3. Propagate project file to children that lack loc.file, THEN recurse
    filtered = {}
    node_file = get_node_file(node)
    has_project_file = node_file and node_file.startswith(project_root)
    
    for key, val in node.items():
        if key == "inner" and isinstance(val, list):
            new_inner = []
            for child in val:
                # If parent is a project node, propagate file to child before filtering
                if has_project_file and isinstance(child, dict):
                    child_loc = child.get("loc", {})
                    if isinstance(child_loc, dict) and not child_loc.get("file"):
                        child = dict(child)  # shallow copy
                        child["loc"] = dict(child_loc)
                        child["loc"]["file"] = node_file
                result = filter_node(child, project_root)
                if result is not None:
                    new_inner.append(result)
            filtered["inner"] = new_inner
        elif key == "inner":
            filtered[key] = val
        else:
            filtered[key] = val

    return filtered


def main():
    project_root = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    if not project_root.endswith("/"):
        project_root += "/"

    data = json.load(sys.stdin)

    # Only process TranslationUnitDecl
    if isinstance(data, dict) and data.get("kind") == "TranslationUnitDecl" and "inner" in data:
        filtered_inner = []
        for node in data["inner"]:
            result = filter_node(node, project_root)
            if result is not None:
                filtered_inner.append(result)
        data["inner"] = filtered_inner

    json.dump(data, sys.stdout)


if __name__ == "__main__":
    main()
