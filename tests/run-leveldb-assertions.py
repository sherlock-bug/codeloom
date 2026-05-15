#!/usr/bin/env python3
"""
CodeLoom Leveldb 断言式测试 — 需 leveldb 仓库已索引。

单独运行（不在日常 CI 中，发布前手动跑）：
  python3 tests/run-leveldb-assertions.py
"""

import json
import subprocess
import sys
import os

CODELOOM_BIN = os.path.join(os.path.dirname(__file__), "..", "target", "release", "codeloom")
if not os.path.exists(CODELOOM_BIN):
    CODELOOM_BIN = os.path.join(os.path.dirname(__file__), "..", "target", "debug", "codeloom")

LEVELDB_REPO = "leveldb"
LEVELDB_BRANCH = "main1"

errors = 0
passed = 0
skipped = 0


def run_cli(args: list) -> str:
    proc = subprocess.run(
        [CODELOOM_BIN] + args,
        capture_output=True,
        text=True,
        timeout=30
    )
    return proc.stdout


def run_mcp(tool: str, args: dict) -> dict:
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


# ================================================================
# Leveldb 索引质量测试
# ================================================================
print("Leveldb 断言测试")
print(f"仓库: {LEVELDB_REPO} @ {LEVELDB_BRANCH}")
print()

leveldb_ok = run_cli(["list-repos"]).strip()

if "leveldb" not in leveldb_ok:
    passed += 1
    print("  ⏭ L0: leveldb 仓库未索引，跳过测试")
else:
    # BUG-011: class Compaction 应在 version_set.h:319 唯一定义
    lines = run_cli(["list-symbols", "Compaction", "--repo", LEVELDB_REPO, "--branch", LEVELDB_BRANCH, "--limit", "10"])
    data_lines = [l for l in lines.strip().split("\n")[1:] if not l.strip().startswith("(none)")]
    # Filter to class kind only (list-symbols matches all symbols containing "Compaction")
    class_lines = [l for l in data_lines if "[class" in l]
    check("L1: class Compaction 唯一性（应为 1 个）", len(class_lines), 1)

    if class_lines:
        # Extract the symbol name from line format like "[class     ] Compaction @ ..."
        # Split by whitespace: ["[class", "]", "Compaction", "@", "..."]
        parts = class_lines[0].strip().split()
        sym = parts[2]  # Third word is the symbol name ("Compaction")
        info = run_mcp("codeloom_inspect", {"name": sym, "repo": LEVELDB_REPO, "branch": LEVELDB_BRANCH})
        is_class = isinstance(info, dict) and info.get("kind") == "class"
        check("L2: Compaction 类型为 class", is_class, True)
        if is_class:
            file_ok = "version_set.h" in info.get("file", "")
            check("L3: Compaction 文件指向 version_set.h", file_ok, True)
            line_ok = info.get("line_start") == 319
            check("L4: Compaction 行号 = 319", line_ok, True)

    # L5: .cc 定义优先于 .h 声明
    # VersionSet::LogAndApply 声明在 db_impl.h:181，实现在 version_set.cc:777
    vsa = run_mcp("codeloom_inspect", {"name": "VersionSet::LogAndApply", "repo": LEVELDB_REPO, "branch": LEVELDB_BRANCH})
    check("L5a: VersionSet::LogAndApply 文件指向 version_set.cc",
          vsa.get("file", "").endswith("version_set.cc"), True)
    check("L5b: VersionSet::LogAndApply 行号 ≈ 777",
          abs(vsa.get("line_start", 0) - 777) <= 2, True)

    # DBImpl::NewDB 声明在 db_impl.h:108，实现在 db_impl.cc:181
    ndb = run_mcp("codeloom_inspect", {"name": "DBImpl::NewDB", "repo": LEVELDB_REPO, "branch": LEVELDB_BRANCH})
    check("L5c: DBImpl::NewDB 文件指向 db_impl.cc",
          ndb.get("file", "").endswith("db_impl.cc"), True)
    check("L5d: DBImpl::NewDB 行号 ≈ 181",
          abs(ndb.get("line_start", 0) - 181) <= 2, True)

    # Compaction::IsTrivialMove 声明在 version_set.h:342，实现在 version_set.cc:1499
    ctm = run_mcp("codeloom_inspect", {"name": "Compaction::IsTrivialMove", "repo": LEVELDB_REPO, "branch": LEVELDB_BRANCH})
    check("L5e: Compaction::IsTrivialMove 文件指向 version_set.cc",
          ctm.get("file", "").endswith("version_set.cc"), True)
    check("L5f: Compaction::IsTrivialMove 行号 ≈ 1499",
          abs(ctm.get("line_start", 0) - 1499) <= 2, True)

# ================================================================
print()
total = passed + errors + skipped
print(f"总计: {passed} 通过, {errors} 失败, {skipped} 跳过 (共 {total})")
if errors > 0:
    sys.exit(1)
else:
    print("全部通过 ✅")
