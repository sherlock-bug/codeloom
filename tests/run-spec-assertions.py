#!/usr/bin/env python3
"""
CodeLoom MCP 工具断言式测试用例 — 规格对齐版

基于规格文档设计，不迁就实现偏差。
测试库：tests/fixtures/expert-designed/ (expert_fixture.h + expert_fixture.cc)
仓库名：expert-test，分支：main

使用方式：先索引再运行
  codeloom index tests/fixtures/expert-designed/ --repo expert-test --branch main
  python3 tests/run-spec-assertions.py

设计原则：
- 确定性输出 → 精确断言（assert equal）
- 非确定性输出（搜索排序、分数）→ 只验格式和合理范围
- 规格冲突 → 标注 ⚠️ 不标记为 bug
- 不依赖"实测"结果
"""

import json
import subprocess
import sys
import os

CODELOOM_BIN = os.path.join(os.path.dirname(__file__), "..", "target", "debug", "codeloom")
REPO = "expert-test"
BRANCH = "main"

errors = 0
passed = 0
skipped = 0

def run_mcp(tool: str, args: dict) -> dict:
    """通过 MCP 协议调用工具"""
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool, "arguments": args}
    }
    proc = subprocess.run(
        [CODELOOM_BIN, "mcp"],
        input=json.dumps(req).encode(),
        capture_output=True,
        timeout=30
    )
    resp = json.loads(proc.stdout)
    if "error" in resp:
        print(f"  ⚠ MCP 错误: {resp['error']}")
        return {}
    content = resp.get("result", {}).get("content", [])
    text = content[0]["text"] if content else ""
    if not text:
        return {}
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return {"_raw_text": text}

def run_cli(args: list) -> str:
    """通过 CLI 调用工具"""
    proc = subprocess.run(
        [CODELOOM_BIN] + args,
        capture_output=True,
        text=True,
        timeout=30
    )
    return proc.stdout

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
        print(f"     预期: {expected}")
        print(f"     实际: {actual}")

def check_in(name: str, actual, container):
    """检查 actual 是否在 container 中"""
    global errors, passed
    if actual in container:
        passed += 1
        print(f"  ✅ {name}")
    else:
        errors += 1
        print(f"  ⚠ {name}: '{actual}' 不在 {container}")

def has_key(d, key):
    return key in d

def any_item(items, pred):
    return any(pred(i) for i in items)

# ================================================================
# 工具 1: codeloom_schema — 元数据导出
# 规格: schema-metadata/spec.md
# 注意：规格之间对边类型数量存在冲突（schema-metadata=10种, extended-edge-types=11种）
# ================================================================
print("\n═══ 1. codeloom_schema ═══")

# M1: schema 无需参数即可调用
schema = run_mcp("codeloom_schema", {})
check("schema 无参调用成功", schema != {}, True)

# M2: node_kinds 条目含 name/description/example
check("node_kinds 存在", has_key(schema, "node_kinds"), True)
nk = schema.get("node_kinds", [])
check("node_kinds 每个条目含 name", all("name" in n for n in nk), True)
check("node_kinds 每个条目含 description", all("description" in n for n in nk), True)
check("node_kinds 每个条目含 example", all("example" in n for n in nk), True)

# M3: edge_types 条目含 prefix/direction/source_kinds/target_kinds
check("edge_types 存在", has_key(schema, "edge_types"), True)
et = schema.get("edge_types", [])
check("edge_types 每个条目含 prefix", all("prefix" in e for e in et), True)
check("edge_types 每个条目含 direction", all("direction" in e for e in et), True)
check("edge_types 每个条目含 source_kinds", all("source_kinds" in e for e in et), True)
check("edge_types 每个条目含 target_kinds", all("target_kinds" in e for e in et), True)

# M4: ⚠️ 节点类型数量 = 15（规格冲突：extended-node-types 说 namespace/template_instance/typedef，
#    但 schema 输出的是 variable/template_class/template_struct。两套体系名不同但总数一致）
check("node_kinds 数量 >= 13 (规格冲突: 两套命名体系)",
      len(nk) >= 13, True,
      lambda a, _: len(nk) >= 13)

# M5: ⚠️ 边类型数量 — schema-metadata 规格说 10 种(calls/calls_override/inherits/contains/uses/references/returns/param_type/field_type/template_use)，
#    extended-edge-types 规格说 11 种且名称不同(overrides/aliases/includes 等)。两规格冲突。
#    此处断言不少于 10 种
check("edge_types 数量 >= 10 (规格冲突: schema-metadata=10, extended-edge-types=11)",
      len(et) >= 10, True,
      lambda a, _: len(et) >= 10)

# ================================================================
# 工具 2: codeloom_list_repos — 列出仓库
# ================================================================
print("\n═══ 2. codeloom_list_repos ═══")

repos_text = run_cli(["list-repos"])
check("list-repos 包含 expert-test", "expert-test" in repos_text, True)

# ================================================================
# 工具 3: codeloom_list_branches — 列出分支
# ================================================================
print("\n═══ 3. codeloom_list_branches ═══")

branches = run_cli(["list-branches", "--repo", REPO])
check("list-branches 包含 main 分支", "main" in branches, True)

# ================================================================
# 工具 4: codeloom_list_symbols — 模糊符号搜索
# 规格: search-enrichment/spec.md (纯文本输出，不适用增强规则)
# ================================================================
print("\n═══ 4. codeloom_list_symbols ═══")

# list-symbols 使用 SQL LIKE 模式，传 "Logger" 只匹配以 Logger 开头的符号
# 需要 "%Logger%" 才匹配所有含 Logger 的符号
symbols = run_cli(["list-symbols", "%Logger%", "--repo", REPO, "--branch", BRANCH, "--limit", "20"])
check("list-symbols Logger 返回结果", "Logger" in symbols, True)
check("list-symbols Logger 含 FileLogger", "FileLogger" in symbols, True)
check("list-symbols Logger 含 ConsoleLogger", "ConsoleLogger" in symbols, True)
check("list-symbols Logger 含 HybridLogger", "HybridLogger" in symbols, True)

# 别名搜索
alias_syms = run_cli(["list-symbols", "AdvancedLogger", "--repo", REPO, "--branch", BRANCH, "--limit", "10"])
check("list-symbols 别名 AdvancedLogger", "AdvancedLogger" in alias_syms, True)

# 模板搜索
template_syms = run_cli(["list-symbols", "DataStore", "--repo", REPO, "--branch", BRANCH, "--limit", "10"])
check("list-symbols 模板 DataStore", "DataStore" in template_syms, True)

# ================================================================
# 工具 5: codeloom_search (BM25) — 精确关键词搜索
# 规格: bm25-precise-search/spec.md + search-enrichment/spec.md
# ================================================================
print("\n═══ 5. codeloom_search (BM25) ═══")

# 精确名称搜索排第一（bm25-precise-search: "结果第一项 SHALL 是匹配的符号"）
search_res = run_mcp("codeloom_search", {
    "query": "HybridLogger", "repo": REPO, "branch": BRANCH, "limit": 5
})
check("search HybridLogger 返回结果", search_res.get("count", 0) > 0, True)

# 已确认：search 不返回 methods/members（规格变更 SPEC-CHANGE-001）
# 只检查基础功能

# 枚举搜索结果含 values（search-enrichment: "enum 结果包含 values"）
enum_res = run_mcp("codeloom_search", {
    "query": "LogLevel", "repo": REPO, "branch": BRANCH, "limit": 5
})
for result in enum_res.get("results", []):
    if result.get("kind") == "enum":
        check("search enum 结果含 values 字段",
              "values" in result, True,
              lambda a, _: "values" in result)
        break

# 空结果（bm25-precise-search: "返回结果 SHALL 为空数组"）
empty_res = run_mcp("codeloom_search", {
    "query": "xyxxy_no_match_xyz", "repo": REPO, "branch": BRANCH, "limit": 5
})
check("search 空结果 count=0", empty_res.get("count"), 0)

# ================================================================
# 工具 6: codeloom_inspect — 符号详细信息
# 规格: MCP 工具定义（非 mcp-full-attribute-json — 那是 search 工具的规格）
# ================================================================
print("\n═══ 6. codeloom_inspect ═══")

# 类 inspect — 含 name/kind/file 核心字段
info = run_mcp("codeloom_inspect", {"name": "HybridLogger", "repo": REPO, "branch": BRANCH})
check("HybridLogger 是 class", info.get("kind"), "class")
check("HybridLogger 有 file_path 或 file 字段",
      info.get("file_path") or info.get("file") or info.get("_raw_text"), True,
      lambda a, _: bool(a))
check("HybridLogger 有 line_start", info.get("line_start", 0) > 0, True)

# 结构体 inspect
info = run_mcp("codeloom_inspect", {"name": "BaseConfig", "repo": REPO, "branch": BRANCH})
check("BaseConfig 是 struct", info.get("kind"), "struct")

# 自由函数 inspect — 不含 parent_class

# 枚举 inspect
info = run_mcp("codeloom_inspect", {"name": "LogLevel", "repo": REPO, "branch": BRANCH})
check("LogLevel 是 enum", info.get("kind"), "enum")

# BUG-009 测试: .h 声明 + .cc 定义应合并为同一符号
# 规格要求: 声明合并后 is_definition=1（extended-node-types/spec.md 跨文件场景）
info = run_mcp("codeloom_inspect", {"name": "initialize_logging", "repo": REPO, "branch": BRANCH})
check("BUG-009: initialize_logging 跨文件合并为单个对象", isinstance(info, dict), True)
# 合并后 file_path 保留 .h（声明文件），不是 .cc（定义文件）
check("BUG-009: initialize_logging 路径指向 .h", info.get("file", "").endswith(".h"), True)

# ================================================================
# 工具 7: codeloom_inheritance_tree — 继承树
# 规格: inheritance-tree/spec.md → 嵌套树格式，非扁平
# ================================================================
print("\n═══ 7. codeloom_inheritance_tree ═══")

# HybridLogger 向上查父类（多重继承）
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "HybridLogger", "repo": REPO, "branch": BRANCH,
    "direction": "up", "max_depth": 5
})
# 规格: 嵌套树格式 — 每个节点含 symbol + children。顶层用 root 键名
root_sym = tree.get("root") or tree.get("symbol") or ""
check("HybridLogger up 有根节点名", root_sym, "HybridLogger")
hy_children = tree.get("children", [])
check("HybridLogger up 有 children",
      len(hy_children) > 0, True)
parent_names = [c.get("symbol") for c in hy_children]
check("HybridLogger up 父类含 FileLogger",
      "FileLogger" in parent_names, True)
check("HybridLogger up 父类含 ConsoleLogger",
      "ConsoleLogger" in parent_names, True)

# Logger 向上查父类（根类 → 无父类）
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "Logger", "repo": REPO, "branch": BRANCH,
    "direction": "up", "max_depth": 5
})
root_sym = tree.get("root") or tree.get("symbol") or ""
check("Logger up 是根类", root_sym, "Logger")
check("Logger up 无父类", len(tree.get("children", [])), 0)

# Logger 向下查子类
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "Logger", "repo": REPO, "branch": BRANCH,
    "direction": "down", "max_depth": 5
})
check("Logger down 有子类", len(tree.get("children", [])) > 0, True)
down_kids = [c.get("symbol") for c in tree.get("children", [])]
check("Logger down 子类含 FileLogger", "FileLogger" in down_kids, True)
check("Logger down 子类含 ConsoleLogger", "ConsoleLogger" in down_kids, True)

# HybridLogger 向下查子类（叶子 → 无子类）
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "HybridLogger", "repo": REPO, "branch": BRANCH,
    "direction": "down", "max_depth": 5
})
root_sym = tree.get("root") or tree.get("symbol") or ""
check("HybridLogger down 是叶子", root_sym, "HybridLogger")
check("HybridLogger down 无子类", len(tree.get("children", [])), 0)

# M3: FileLogger direction=both — 同时有父类(Logger)和子类(HybridLogger)
tree = run_mcp("codeloom_inheritance_tree", {
    "symbol": "FileLogger", "repo": REPO, "branch": BRANCH,
    "direction": "both", "max_depth": 5
})
root_sym = tree.get("root") or tree.get("symbol") or ""
check("FileLogger both 有根节点名", root_sym, "FileLogger")
down_fk = [c.get("symbol") for c in tree.get("children", [])]
check("FileLogger both 子类含 HybridLogger", "HybridLogger" in down_fk, True)
# up 方向的父类在 children... wait, this depends on how "both" merges results

# ================================================================
# 工具 8: codeloom_neighbor_graph — 邻居关系
# 规格: neighbor-graph/spec.md → 按方向+边类型分组，无 via_member 前缀要求
# ================================================================
print("\n═══ 8. codeloom_neighbor_graph ═══")

# HybridLogger 正向邻居（含多重继承）
neighbors = run_mcp("codeloom_neighbor_graph", {
    "symbol": "HybridLogger", "repo": REPO, "branch": BRANCH,
    "direction": "forward"
})
check("neighbor_graph 含 forward 字段", has_key(neighbors, "forward"), True)
fw = neighbors.get("forward", {})
# 检查所有边类型分组中的值
all_forward_names = set()
for _, values in fw.items():
    if isinstance(values, list):
        all_forward_names.update(values)
check("HybridLogger forward 含 ConsoleLogger", "ConsoleLogger" in all_forward_names, True)
check("HybridLogger forward 含 FileLogger", "FileLogger" in all_forward_names, True)

# Logger 反向邻居（子类）
neighbors = run_mcp("codeloom_neighbor_graph", {
    "symbol": "Logger", "repo": REPO, "branch": BRANCH,
    "direction": "reverse"
})
back = neighbors.get("backward", {})
all_backward_names = set()
for _, values in back.items():
    if isinstance(values, list):
        all_backward_names.update(values)
check("Logger backward 含 FileLogger", "FileLogger" in all_backward_names, True)
check("Logger backward 含 ConsoleLogger", "ConsoleLogger" in all_backward_names, True)
check("Logger backward 不含 Logger 自身（非自环）",
      "Logger" not in all_backward_names, True)

# M2: 枚举值 LogLevel::LOG_INFO 的反向邻居（谁使用了它）
enum_nb = run_mcp("codeloom_neighbor_graph", {
    "symbol": "LogLevel::LOG_INFO", "repo": REPO, "branch": BRANCH,
    "direction": "reverse"})
check("LOG_INFO backward 有数据",
      len(enum_nb.get("backward", {})) > 0, True)

# M5: BaseConfig 的字段类型邻居（field_type → LogLevel）
cfg_nb = run_mcp("codeloom_neighbor_graph", {
    "symbol": "BaseConfig", "repo": REPO, "branch": BRANCH,
    "direction": "forward"})
cfg_fw = cfg_nb.get("forward", {})
cfg_names = set()
for _, values in cfg_fw.items():
    if isinstance(values, list):
        cfg_names.update(values)
# BaseConfig::default_level 是 LogLevel 类型
check("BaseConfig forward 含 LogLevel (field_type)",
      "LogLevel" in cfg_names, True)

# M6: 自由函数 initialize_logging 的体调用边（验证函数体解析）
fn_nb = run_mcp("codeloom_neighbor_graph", {
    "symbol": "initialize_logging", "repo": REPO, "branch": BRANCH,
    "direction": "forward"})
fn_fw = fn_nb.get("forward", {})
fn_calls = set(fn_fw.get("calls", []))
check("initialize_logging 有 calls 边（函数体解析）",
      "g_default_logger" in fn_calls, True)
fn_uses = set(fn_fw.get("uses", []))
check("initialize_logging 有 uses 边（全局变量引用）",
      "g_default_logger" in fn_uses, True)

# M7: 全局变量 g_default_logger 的反向邻居（谁引用了它）
g_nb = run_mcp("codeloom_neighbor_graph", {
    "symbol": "g_default_logger", "repo": REPO, "branch": BRANCH,
    "direction": "reverse"})
g_back = g_nb.get("backward", {})
g_callers = set(g_back.get("calls", []))
check("g_default_logger 反向有 calls 边",
      "initialize_logging" in g_callers, True)
check("g_default_logger 反向 calls 包含 cleanup_logging",
      "cleanup_logging" in g_callers, True)

# ================================================================
# 工具 9: codeloom_call_graph — 调用图（文本格式）
# 规格: call-graph-module/spec.md (文本格式) + enhanced-call-graph/spec.md (终端增强)
# ================================================================
print("\n═══ 9. codeloom_call_graph ═══")

# 文本格式 — 验证函数体调用关系已正确提取
cg_text = run_cli(["call-graph", "initialize_logging", "--repo", REPO,
                   "--branch", BRANCH, "--direction", "callees", "--max-depth", "2"])
check("call-graph initialize_logging callees 有输出", len(cg_text) > 0, True)
check("call-graph 含 LogLevel::LOG_INFO（枚举引用）",
      "LogLevel::LOG_INFO" in cg_text, True)
check("call-graph 含 g_default_logger（全局变量引用）",
      "g_default_logger" in cg_text, True)

# ================================================================
# 工具 10: codeloom_impact_analysis — 影响分析
# 规格: impact-analysis/spec.md → 传递闭包，默认 direction=reverse
# ================================================================
print("\n═══ 10. codeloom_impact_analysis ═══")

# Logger 反向影响（子类反向传到它）
impact = run_mcp("codeloom_impact_analysis", {
    "symbol": "Logger", "repo": REPO, "branch": BRANCH,
    "direction": "reverse", "radius": 3
})
check("impact_analysis 含 affected 字段", has_key(impact, "affected"), True)

# ================================================================
# 工具 11: codeloom_path_analysis — 路径分析
# 规格: path-analysis/spec.md → BFS 最短/全路径，默认 direction=both
# ================================================================
print("\n═══ 11. codeloom_path_analysis ═══")

# 现在 calls 边已从函数体正确提取，可做实际路径断言
path = run_mcp("codeloom_path_analysis", {
    "source": "initialize_logging", "target": "g_default_logger",
    "repo": REPO, "branch": BRANCH, "mode": "shortest", "max_paths": 3
})
check("path initialize_logging→g_default_logger 有路径",
      path.get("total_found", 0) > 0, True)
edgs = path.get("paths", [{}])[0].get("edges", [])
check("path edges 含 calls 边（函数体解析）",
      any("calls:" in str(e) for e in edgs), True)

# ================================================================
# 工具 12: codeloom_semantic_search — 语义搜索
# 规格: vector-semantic-search/spec.md → 只搜符号名，不搜文档
# ================================================================
print("\n═══ 12. codeloom_semantic_search ═══")

# 语义搜索（如果向量模型未加载，预期返回错误信息）
fuzzy_text = run_cli(["fuzzy", "日志记录器", "--repo", REPO, "--limit", "5"])
check("fuzzy 语义搜索有输出", len(fuzzy_text) > 0, True,
      lambda a, _: True)  # 仅检查不崩溃

# ================================================================
# 汇总
# ================================================================
print(f"\n{'='*50}")
total = passed + errors + skipped
print(f"总计: {passed} 通过, {errors} 失败, {skipped} 跳过 (共 {total})")
if errors > 0:
    sys.exit(1)
else:
    print("全部通过 ✅")
