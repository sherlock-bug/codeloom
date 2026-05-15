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
    "UnresolvedLookupExpr", "CXXNullPtrLiteralExpr",
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
    # loc.file is empty — fall back to includedFrom.file.
    # This captures the includer's path for header-declared functions.
    incl = loc.get("includedFrom", {})
    if isinstance(incl, dict):
        return incl.get("file", "")
    return ""


def is_system(node, project_root, fallback_file=None):
    """Check if node belongs to system headers (not under project_root)."""
    file = get_node_file(node)
    if not file:
        # Declaration nodes from system headers have no loc.file (Clang omits it
        # when the declaration comes from an included system header). Having a
        # line number does NOT mean it's a project node — C library functions
        # like strstr() have loc={line:42,col:5} but no file. We are only called
        # for non-body nodes (body nodes are handled separately in filter_node),
        # so treat missing file as external/system.
        # The only exception: previousDecl linking to a project declaration.
        return True
    # Resolve relative paths to match absolute project_root
    abs_file = os.path.abspath(file)
    if abs_file.startswith(project_root):
        return False
    return not file.startswith(project_root)


def _is_detail_template(name):
    """Check if a class template is an internal STL detail template.
    These have no semantic value for project-level impact analysis
    even when instantiated with project types."""
    # STL internal helpers: __* and _* (underscore + capital letter)
    if name.startswith("__"):
        return True
    if len(name) > 1 and name.startswith("_") and name[1].isupper():
        return True
    # Allocator internals
    if name in ("allocator", "allocator_traits", "__alloc_traits",
                "__new_allocator", "rebind"):
        return True
    # Type traits (pure compile-time machinery)
    if name.startswith(("is_", "has_", "remove_", "add_", "enable_if",
                        "conditional", "common_type", "common_reference",
                        "decay", "underlying_type", "integral_constant",
                        "bool_constant", "void_t")):
        return True
    # Iterator helpers
    if name in ("iterator_traits", "__normal_iterator", "reverse_iterator",
                "move_iterator", "__iterator_traits",
                "__gnu_cxx::__normal_iterator",
                "pointer_traits", "raw_storage_iterator"):
        return True
    # Compiler intrinsics / helpers
    if name in ("initializer_list", "numeric_limits",
                "unary_function", "binary_function",
                "__is_abstract", "__is_pod", "__is_empty",
                "__is_polymorphic", "__is_final"):
        return True
    # Char traits / locale / facets
    if name in ("char_traits", "ctype", "ctype_byname", "codecvt",
                "num_get", "num_put", "money_get", "money_put",
                "time_get", "time_put", "messages", "collate",
                "numpunct", "moneypunct", "timepunct",
                "fpos", "_Char_types"):
        return True
    return False


def _has_project_type_arg(node):
    """Check if a ClassTemplateSpecializationDecl has a template argument
    that's a project type (not std::, not built-in, not internal __).
    Returns True if the CTS should be kept for impact analysis.
    Also checks the template name: detail/internal templates are blocked."""
    name = node.get("name", "")
    if _is_detail_template(name):
        return False
    inner = node.get("inner", [])
    if not isinstance(inner, list):
        return False
    # std and builtin types to ignore as template args
    builtins = frozenset({
        "void", "bool", "char", "signed char", "unsigned char", "wchar_t",
        "char16_t", "char32_t", "short", "unsigned short", "int", "unsigned int",
        "long", "unsigned long", "long long", "unsigned long long",
        "float", "double", "long double", "__int128_t", "__uint128_t",
        "size_t", "ssize_t", "int8_t", "uint8_t", "int16_t", "uint16_t",
        "int32_t", "uint32_t", "int64_t", "uint64_t", "__mbstate_t",
        "std::byte", "std::nullptr_t", "nullptr_t",
    })
    for child in inner:
        if not isinstance(child, dict) or child.get("kind") != "TemplateArgument":
            continue
        qual_type = child.get("type", {}).get("qualType", "")
        if not qual_type or qual_type == "?":
            continue
        # Strip cv-qualifiers, references, pointers
        base = qual_type.strip()
        while base.startswith("const ") or base.startswith("volatile ") or base.startswith("constexpr "):
            for p in ("const ", "volatile ", "constexpr "):
                if base.startswith(p):
                    base = base[len(p):]
                    break
        base = base.rstrip(" *&")
        # Check if it's a known non-project type
        if base in builtins:
            continue
        if base.startswith("std::") or base.startswith("__") or base.startswith("::std::"):
            continue
        # Looks like a project type — keep the CTS
        return True
    return False


def _has_body(node):
    """Check if a FunctionDecl node has a body (CompoundStmt child)."""
    inner = node.get("inner", [])
    if not isinstance(inner, list):
        return False
    return any(
        isinstance(c, dict) and c.get("kind") == "CompoundStmt"
        for c in inner
    )


def filter_node(node, project_root, fallback_file=None):
    """Recursively filter a Clang AST node. Returns filtered dict or None.

    fallback_file: file path from an ancestor (used by body nodes whose own
    loc lacks file info but whose ancestor is a project file).
    """
    if not isinstance(node, dict):
        return node

    kind = node.get("kind", "")

    # Determine file for propagation: need this before children recursion
    # for path injection into body nodes
    node_file = get_node_file(node)
    if not node_file:
        node_file = fallback_file or ""
    has_project_file = bool(node_file) and node_file.startswith(project_root)

    # Recurse into children FIRST (before system/BODY_KINDS checks), so
    # system namespaces containing CTS nodes with project-type template args
    # (e.g. std::vector<MyType>) can survive through their children.
    filtered = {}
    for key, val in node.items():
        if key == "inner" and isinstance(val, list):
            new_inner = []
            for child in val:
                child_kind = child.get("kind", "") if isinstance(child, dict) else ""
                if has_project_file and isinstance(child, dict):
                    child_loc = child.get("loc", {})
                    if isinstance(child_loc, dict) and not child_loc.get("file"):
                        if child_kind in BODY_KINDS or child_kind in ("DeclRefExpr", "MemberExpr"):
                            child = dict(child)
                            child["loc"] = dict(child_loc)
                            child["loc"]["file"] = node_file
                child_fallback = node_file if has_project_file else fallback_file
                result = filter_node(child, project_root, child_fallback)
                if result is not None:
                    new_inner.append(result)
            filtered["inner"] = new_inner
        elif key == "inner":
            filtered[key] = val
        else:
            filtered[key] = val

    # Now check: should this node be kept?
    inner = filtered.get("inner", [])
    
    # Strip system nodes for non-body nodes
    # Exception: ClassTemplateSpecializationDecl with project-type template args
    # (e.g. vector<FileMetaData>) — kept for impact analysis.
    # Exception: system containers (namespace, class template, record, linkage spec)
    # with surviving children — keeps the nesting hierarchy so CTS nodes remain reachable.
    if kind not in BODY_KINDS and is_system(node, project_root, fallback_file):
        # FunctionDecl with body but no file info — project function declared
        # in header but defined in .cc. Clang omits loc.file for these.
        # Let it through; Rust side decides based on cur_file context.
        if kind == "FunctionDecl" and _has_body(node):
            pass  # keep
        elif kind == "ClassTemplateSpecializationDecl" and _has_project_type_arg(node):
            pass  # keep
        elif kind in ("NamespaceDecl", "LinkageSpecDecl", "ClassTemplateDecl",
                      "CXXRecordDecl", "RecordDecl") and inner:
            pass  # keep with surviving children
        elif kind == "TemplateArgument":
            pass  # keep — provides type info for kept CTS nodes
        else:
            return None

    # Now check BODY_KINDS — body nodes with surviving children (e.g. DeclRefExpr)
    # get a minimal representation so the extractor can find symbol references.
    if kind in BODY_KINDS:
        inner = filtered.get("inner", [])
        if not inner:
            return None
        result = {"kind": kind}
        if inner:
            result["inner"] = inner
        # Keep type and path info for call target extraction
        if "type" in node:
            result["type"] = node["type"]
        if "path" in node:
            result["path"] = node["path"]
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
