#!/usr/bin/env python3
"""CodeLoom MCP 工具断言式测试套件

基于 tests/fixtures/comprehensive/ 测试库，所有预期输出是确定的。
运行方式: python3 tests/run-mcp-assertions.py
前置条件: codeloom 已编译，fixture 已索引 (make test-fixture-index)
"""

import json
import subprocess
import sys
import os

CODELOOM_BIN = os.path.join(os.path.dirname(__file__), "..", "target", "debug", "codeloom")
REPO = "test-fixture"
BRANCH = "main"

errors = 0
passed = 0

def run_mcp(tool: str, args: dict) -> dict:
    """调用 codeloom mcp 并返回 result.content[0].text 的解析结果"""
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool,
            "arguments": args
        }
    }
    proc = subprocess.run(
        [CODELOOM_BIN, "mcp"],
        input=json.dumps(req).encode(),
        capture_output=True,
        timeout=30
    )
    resp = json.loads(proc.stdout)
    if "error" in resp:
        print(f"  ⚠️  MCP 错误: {resp['error']}")
        return {}
    content = resp.get("result", {}).get("content", [])
    text = content[0]["text"] if content else ""
    if not text:
        return {}
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return {"_raw_text": text}

def check(name: str, actual, expected, comparator=None):
    global errors, passed
    if comparator:
        ok = comparator(actual, expected)
    else:
        ok = actual == expected
    if ok:
        passed += 1
        print(f"  ✅ {name}")
    else:
        errors += 1
        print(f"  ❌ {name}")
        print(f"     期望: {expected}")
        print(f"     实际: {actual}")

def contains(haystack, needle):
    return needle in haystack

def has_key(d, key):
    return key in d

# ================================================================
# 1.  schema - 元数据验证
# ================================================================
print("\n═══ 1. schema ═══")
schema = run_mcp("codeloom_schema", {"repo": REPO, "branch": BRANCH})
check("schema 包含 node_kinds", has_key(schema, "node_kinds"), True)
check("schema 包含 edge_types", has_key(schema, "edge_types"), True)
check("node_kinds 含 class", any(n["name"]=="class" for n in schema.get("node_kinds",[])), True)
check("edge_types 含 inherits", any(e["prefix"]=="inherits" for e in schema.get("edge_types",[])), True)

# ================================================================
# 2.  inheritance_tree - 继承树验证
# ================================================================
print("\n═══ 2. inheritance_tree ═══")

# 2a. DerivedClass → up → 应找到 BaseClass
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "DerivedClass", "repo": REPO, "branch": BRANCH,
    "direction": "up", "max_depth": 3
})
check("DerivedClass up → root=DerivedClass", tree.get("root"), "DerivedClass")
children = tree.get("children", [])
check("DerivedClass up → 有父类", len(children) > 0, True)
check("DerivedClass up → 父类是 BaseClass", any(c["symbol"]=="BaseClass" and c["relation"]=="parent" for c in children), True)

# 2b. BaseClass → down → 应找到 DerivedClass
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "BaseClass", "repo": REPO, "branch": BRANCH,
    "direction": "down", "max_depth": 3
})
check("BaseClass down → root=BaseClass", tree.get("root"), "BaseClass")
children = tree.get("children", [])
check("BaseClass down → 有子类", len(children) > 0, True)
check("BaseClass down → 子类是 DerivedClass", any(c["symbol"]=="DerivedClass" and c["relation"]=="child" for c in children), True)

# 2c. UIWidget → up → 应找到两个父类 (多继承)
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "UIWidget", "repo": REPO, "branch": BRANCH,
    "direction": "up", "max_depth": 3
})
check("UIWidget up → root=UIWidget", tree.get("root"), "UIWidget")
parents = [c["symbol"] for c in tree.get("children", [])]
check("UIWidget up → 有父类 Renderable", "Renderable" in parents, True)
check("UIWidget up → 有父类 Clickable", "Clickable" in parents, True)

# 2d. Point → down → 应无子类（叶子类）
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "Point", "repo": REPO, "branch": BRANCH,
    "direction": "down", "max_depth": 3
})
check("Point down → 无子类", len(tree.get("children",[])), 0)

# 2e. UIWidget → down → 应无子类
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "UIWidget", "repo": REPO, "branch": BRANCH,
    "direction": "down", "max_depth": 3
})
check("UIWidget down → 无子类", len(tree.get("children",[])), 0)

# ================================================================
# 3.  inspect - 符号信息验证
# ================================================================
print("\n═══ 3. inspect ═══")

# 3a. DerivedClass (class)
info = run_mcp("codeloom_inspect", {
    "name": "DerivedClass", "repo": REPO, "branch": BRANCH
})
check("DerivedClass 是 class", info.get("kind"), "class")
check("DerivedClass 有 members", has_key(info, "members"), True)
check("DerivedClass 有 methods", has_key(info, "methods"), True)

# 3b. Status (enum)
info = run_mcp("codeloom_inspect", {
    "name": "Status", "repo": REPO, "branch": BRANCH
})
check("Status 是 enum", info.get("kind"), "enum")
check("Status 有 file 路径", info.get("file","").endswith("fixture.h"), True)

# 3c. Point (struct)
info = run_mcp("codeloom_inspect", {
    "name": "Point", "repo": REPO, "branch": BRANCH
})
check("Point 是 struct", info.get("kind"), "struct")

# 3d. clamp (function)
info = run_mcp("codeloom_inspect", {
    "name": "clamp", "repo": REPO, "branch": BRANCH
})
check("clamp 是 function", info.get("kind"), "function")

# ================================================================
# 4.  neighbor_graph - 邻居关系验证
# ================================================================
print("\n═══ 4. neighbor_graph ═══")

# 4a. DerivedClass → both → 应有 BaseClass (inherits) 和它的成员
neighbors = run_mcp("codeloom_neighbor_graph", {
    "symbol": "DerivedClass", "repo": REPO, "branch": BRANCH,
    "direction": "both"
})
check("neighbor_graph 返回 forward 字段", "forward" in neighbors, True)
forward = neighbors.get("forward", {})
inherits_edges = forward.get("inherits", [])
check("DerivedClass forward inherits 含 BaseClass", "BaseClass" in inherits_edges, True)

# 4b. BaseClass → reverse → 应看到 DerivedClass (子类via inherits)
neighbors = run_mcp("codeloom_neighbor_graph", {
    "symbol": "BaseClass", "repo": REPO, "branch": BRANCH,
    "direction": "reverse"
})
backward = neighbors.get("backward", {})
check("BaseClass backward 有数据", len(backward) > 0, True)

# ================================================================
# 5.  list_symbols - 符号搜索验证
# ================================================================
print("\n═══ 5. list_symbols ═══")

symbols = run_mcp("codeloom_list_symbols", {
    "pattern": "Derived", "repo": REPO, "branch": BRANCH, "limit": 10
})
raw = symbols.get("_raw_text", "")
check("list_symbols 搜索 Derived 返回内容", "DerivedClass" in raw, True)
check("list_symbols 搜索 Derived 含 DerivedClass::calculate", "DerivedClass::calculate" in raw, True)

symbols = run_mcp("codeloom_list_symbols", {
    "pattern": "Status", "repo": REPO, "branch": BRANCH, "limit": 10
})
raw = symbols.get("_raw_text", "")
check("list_symbols 搜索 Status 返回内容", len(raw) > 0, True)

# ================================================================
# 6.  search (BM25) - 关键词搜索验证
# ================================================================
print("\n═══ 6. search (BM25) ═══")

result = run_mcp("codeloom_search", {
    "query": "DerivedClass", "repo": REPO, "branch": BRANCH, "limit": 5
})
check("搜索 DerivedClass 返回结果", result.get("count", 0) > 0, True)

result = run_mcp("codeloom_search", {
    "query": "clamp", "repo": REPO, "branch": BRANCH, "limit": 5
})
check("搜索 clamp 返回结果", result.get("count", 0) > 0, True)

# ================================================================
# 7.  impact_analysis - 影响分析验证
# ================================================================
print("\n═══ 7. impact_analysis ═══")

impact = run_mcp("codeloom_impact_analysis", {
    "symbol": "BaseClass", "repo": REPO, "branch": BRANCH,
    "direction": "reverse", "radius": 3
})
check("impact_analysis 返回 affected 列表", "affected" in impact, True)
affected = impact.get("affected", [])
check("impact_analysis 影响 BaseClass 的事物", len(affected) > 0, True)

# ================================================================
# 8.  path_analysis - 路径分析验证
# ================================================================
print("\n═══ 8. path_analysis ═══")

path = run_mcp("codeloom_path_analysis", {
    "source": "BaseClass", "target": "DerivedClass",
    "repo": REPO, "branch": BRANCH, "mode": "shortest"
})
check("path_analysis 返回结果", has_key(path, "paths") or has_key(path, "path"), True)

# ================================================================
# 9.  schema（重复但确认稳定性）
# ================================================================
print("\n═══ 9. schema (重复验证) ═══")
schema2 = run_mcp("codeloom_schema", {"repo": REPO, "branch": BRANCH})
check("schema 两次调用一致", schema == schema2, True)

# ================================================================
# 汇总
# ================================================================
print(f"\n{'═'*50}")
print(f"结果: {passed} 通过, {errors} 失败")
if errors > 0:
    sys.exit(1)
else:
    print("全部通过 ✅")
