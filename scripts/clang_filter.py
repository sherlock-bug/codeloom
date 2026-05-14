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
    "MemberExpr", "CXXDependentScopeMemberExpr",
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
    # Nodes with previousDecl linking to a project declaration are project nodes
    if isinstance(node.get("previousDecl"), str) and node["previousDecl"]:
        return False

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


def filter_node(node, project_root, fallback_file=None):
    """Recursively filter a Clang AST node. Returns filtered dict or None.

    fallback_file: file path from an ancestor (used by body nodes whose own
    loc lacks file info but whose ancestor is a project file).
    """
    if not isinstance(node, dict):
        return node

    kind = node.get("kind", "")

    # Strip system nodes for non-body nodes
    if kind not in BODY_KINDS and is_system(node, project_root):
        return None

    # Determine file for propagation: prefer own file, then fallback
    node_file = get_node_file(node)
    if not node_file:
        node_file = fallback_file or ""
    has_project_file = bool(node_file) and node_file.startswith(project_root)

    # Recurse into children FIRST (before BODY_KINDS check), passing our
    # project file as fallback so DeclRefExpr inside bodies can survive.
    filtered = {}
    for key, val in node.items():
        if key == "inner" and isinstance(val, list):
            new_inner = []
            for child in val:
                # Propagate file to children that lack loc.file
                if has_project_file and isinstance(child, dict):
                    child_loc = child.get("loc", {})
                    if isinstance(child_loc, dict) and not child_loc.get("file"):
                        child = dict(child)  # shallow copy
                        child["loc"] = dict(child_loc)
                        child["loc"]["file"] = node_file
                # Pass our project file as fallback for deeper body nodes
                child_fallback = node_file if has_project_file else fallback_file
                result = filter_node(child, project_root, child_fallback)
                if result is not None:
                    new_inner.append(result)
            filtered["inner"] = new_inner
        elif key == "inner":
            filtered[key] = val
        else:
            filtered[key] = val

    # Now check BODY_KINDS — body nodes with surviving children (e.g. DeclRefExpr)
    # get a minimal representation so the extractor can find symbol references.
    if kind in BODY_KINDS:
        inner = filtered.get("inner", [])
        if not inner:
            return None
        result = {"kind": kind}
        if inner:
            result["inner"] = inner
        return result

    return filtered


def main():
    project_root = sys.argv[1] if len(sys.argv) > 1 else os.getcwd()
    if not project_root.endswith("/"):
        project_root += "/"
    # source_file: the .cc/.cpp file being parsed — used as fallback_file
    # for body nodes that lack their own loc.file info.
    source_file = sys.argv[2] if len(sys.argv) > 2 else ""

    data = json.load(sys.stdin)

    # Only process TranslationUnitDecl
    if isinstance(data, dict) and data.get("kind") == "TranslationUnitDecl" and "inner" in data:
        filtered_inner = []
        for node in data["inner"]:
            result = filter_node(node, project_root, source_file)
            if result is not None:
                filtered_inner.append(result)
        data["inner"] = filtered_inner

    json.dump(data, sys.stdout)


if __name__ == "__main__":
    main()
