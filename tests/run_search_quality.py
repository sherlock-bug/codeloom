#!/usr/bin/env python3
import subprocess, json, shlex

CL = "/home/huangxiaohai/.local/bin/codeloom"

def run(cmd, timeout=60):
    r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
    return r.stdout + r.stderr, r.returncode

def llm_judge(test_name, actual_output, criteria):
    prompt = f"""你是一个搜索质量评审专家。请评审以下代码搜索测试结果。

测试名称：{test_name}
评审标准：{criteria}

实际输出：
```
{actual_output[:2000]}
```

请回答：
1. PASS 还是 FAIL？
2. 理由（一句话）"""
    
    r = subprocess.run(
        ['hermes', 'chat', '-Q', '-m', 'deepseek-v4-pro', '-q', prompt],
        capture_output=True, text=True, timeout=60
    )
    return r.stdout.strip()[:300]

def mcp_call(request_json):
    payload = json.dumps(request_json)
    quoted = shlex.quote(payload)
    out, _ = run(f"echo {quoted} | {CL} mcp 2>/dev/null")
    return out

# SQ-06: Semantic search Chinese
print("=== SQ-06: 语义搜索（中文） ===")
out, _ = run(f"{CL} semantic \"键值对写入操作\" --repo leveldb --branch main1 --limit 5")
verdict = llm_judge("SQ-06", out, "语义搜索结果应与键值对写入操作相关，包含 Put/Write 等符号")
print(out[:500])
print(f"评判: {verdict}")

# SQ-03: Dedup check
print("\n=== SQ-03: 去重检查 ===")
out, _ = run(f"{CL} search \"SkipList\" --repo leveldb --branch main1 --limit 10")
lines = [l for l in out.split("\n") if l.strip().startswith("  [")]
keys = set()
dup = False
for l in lines:
    parts = l.strip().split()
    if len(parts) >= 2:
        key = parts[1] + "@" + parts[-1] if len(parts) >= 4 else parts[1]
        if key in keys:
            dup = True
        keys.add(key)
print(f"{'✅ 无重复' if not dup else '❌ 有重复'} ({len(lines)} 条结果, {len(keys)} 个唯一)")

# SQ-09 to SQ-12: MCP tests
print("\n=== SQ-09: MCP search ===")
resp = mcp_call({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"DBImpl","repo":"leveldb","branch":"main1"}}})
try:
    parsed = json.loads(resp.strip())
    text = parsed.get("result",{}).get("content",[{}])[0].get("text","")
    print(f"✅ JSON响应正常, text前100字: {text[:100]}")
except Exception as e:
    print(f"❌ {e}")

print("\n=== SQ-10: MCP semantic search ===")
resp = mcp_call({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"compaction and version management","repo":"leveldb","branch":"main1","limit":5}}})
try:
    parsed = json.loads(resp.strip())
    text = parsed.get("result",{}).get("content",[{}])[0].get("text","")
    verdict = llm_judge("SQ-10", text, "语义搜索结果应与 compaction 和 version management 相关")
    print(f"✅ 响应正常")
    print(f"评判: {verdict}")
except Exception as e:
    print(f"❌ {e}")

print("\n=== SQ-11: MCP inspect ===")
resp = mcp_call({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_inspect","arguments":{"name":"Compaction","repo":"leveldb","branch":"main1"}}})
try:
    text = json.loads(resp.strip()).get("result",{}).get("content",[{}])[0].get("text","")
    has_kind = '"kind"' in text or '"class"' in text
    has_name = '"Compaction"' in text
    print(f"{'✅' if has_kind and has_name else '❌'} name=Compaction kind=class")
except Exception as e:
    print(f"❌ {e}")

print("\n=== SQ-12: MCP call graph ===")
resp = mcp_call({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_get_call_graph","arguments":{"name":"DBImpl::Put","repo":"leveldb","branch":"main1","direction":"callees","max_depth":2}}})
try:
    text = json.loads(resp.strip()).get("result",{}).get("content",[{}])[0].get("text","")
    has_callees = "callees" in text
    has_name = "DBImpl::Put" in text
    print(f"{'✅' if has_callees and has_name else '❌'} callees={has_callees} name={has_name}")
except Exception as e:
    print(f"❌ {e}")
