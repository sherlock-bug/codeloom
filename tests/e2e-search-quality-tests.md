# CodeLoom 搜索质量端到端测试

> **测试目标**：验证搜索排名质量——不是"有输出就行"，而是"排名是否符合预期"
> **验证方式**：用 LLM（deepseek-v4-pro）作为客观评判者，评估搜索结果的排序合理性
> **测试二进制**：`/home/huangxiaohai/.local/bin/codeloom`
> **测试仓库**：leveldb @ main1

---

## 测试用例

### SQ-01: 注释精确命中应排第一

搜索包含完整注释短语的长查询，验证精确命中注释的符号排在首位。

```bash
codeloom search "key is larger than any previously added key" --repo leveldb --branch main1 --limit 10
```

**预期**：`BlockBuilder::Add` 排第一（其 doc comment 精确包含查询串）
**验证方式**：LLM 评估 top-3 的排序合理性

---

### SQ-02: 名称匹配应优于弱注释匹配

搜索常见英文词，验证函数名直接匹配的排在注释偶尔提到该词的前面。

```bash
codeloom search "insert" --repo leveldb --branch main1 --limit 10
```

**预期**：名为 `Insert`/`insert` 的方法/函数排第一，注释中偶然提到"Insert()"的字段排其后
**验证方式**：LLM 评估名称匹配是否优先

---

### SQ-03: 跨通道符号去重

同一个符号同时命中名称通道和注释通道时，只出现一次，取最好的 BM25 分数。

```bash
codeloom search "SkipList" --repo leveldb --branch main1 --limit 10
```

**预期**：`SkipList` 类及其方法各出现一次，无重复条目
**验证方式**：检查输出无重复 `name@file` 对

---

### SQ-04: BM25 关键词搜索基础功能

短关键词搜索应返回相关符号。

```bash
codeloom search "compaction" --repo leveldb --branch main1 --limit 10
```

**预期**：返回与 compaction（压缩）相关的符号，top-3 包含 `Compaction` 类或 `CompactionState`
**验证方式**：LLM 评估所有结果均与 "compaction" 相关

---

### SQ-05: Kind 过滤正确性

指定 --kind 参数后，结果类型应全部匹配。

```bash
codeloom search "DB" --repo leveldb --branch main1 --kind method --limit 5
```

**预期**：全部结果为 method 类型
**验证方式**：检查每行均以 `[method]` 开头

---

### SQ-06: 语义搜索（向量）— 中文查询

自然语言中文查询应返回语义相关符号，不要求名称包含查询词。

```bash
codeloom semantic "键值对写入操作" --repo leveldb --branch main1 --limit 5
```

**预期**：返回 Put / Write / WriteBatch 等写入操作相关符号
**验证方式**：LLM 评估 top-3 是否与"键值对写入"语义相关

---

### SQ-07: 语义搜索（向量）— 英文查询

```bash
codeloom semantic "file compression and decompression" --repo leveldb --branch main1 --limit 5
```

**预期**：返回压缩相关符号（CompressionType、SnappyCompression 等），而非仅有名称含 "compression" 的符号
**验证方式**：LLM 评估结果是否语义匹配

---

### SQ-08: 噪声过滤 — 弱匹配不返回

搜一个几乎不可能匹配的短词，应返回空或接近空。

```bash
codeloom search "xyzabc" --repo leveldb --branch main1 --limit 5
```

**预期**：返回 `(no results)` 或空列表
**验证方式**：检查输出含 `(no results)` 或 `(none)`

---

### SQ-09: MCP codeloom_search 结果完整性

通过 MCP 协议调用搜索，验证 JSON 结果的字段完整性。

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"DBImpl","repo":"leveldb","branch":"main1"}}}' | /home/huangxiaohai/.local/bin/codeloom mcp 2>/dev/null
```

**预期**：JSON 响应包含 `count`、`results` 数组；每个 result 含 `name`/`kind`/`file`/`line`/`score`/`type`/`parent_class`（若为 method）/`signature` 字段
**验证方式**：解析 JSON 检查字段完整性

---

### SQ-10: MCP codeloom_semantic_search 语义搜索

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"compaction and version management","repo":"leveldb","branch":"main1","limit":5}}}' | /home/huangxiaohai/.local/bin/codeloom mcp 2>/dev/null
```

**预期**：返回语义相关的符号，结果含 `name`/`kind`/`file`/`line`/`score`/`type` 字段
**验证方式**：LLM 评估 top-3 的语义相关性

---

### SQ-11: MCP codeloom_inspect 类检查

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_inspect","arguments":{"name":"Compaction","repo":"leveldb","branch":"main1"}}}' | /home/huangxiaohai/.local/bin/codeloom mcp 2>/dev/null
```

**预期**：JSON 包含 `name`=`Compaction`、`kind`=`class`、`namespace`、`file`、`line_start`、`line_end`、`documentation` 等字段；`edges` 包含 `members`、`methods` 等
**验证方式**：解析 JSON 验证字段完整性

---

### SQ-12: MCP codeloom_get_call_graph 调用图

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_get_call_graph","arguments":{"name":"DBImpl::Put","repo":"leveldb","branch":"main1","direction":"callees","max_depth":2}}}' | /home/huangxiaohai/.local/bin/codeloom mcp 2>/dev/null
```

**预期**：树状文本输出 `DBImpl::Put` 的被调用函数，深度 ≤ 2
**验证方式**：检查输出含 "callees" 和 `DBImpl::Put`

---

## 执行脚本

每次测试运行使用以下 Python 脚本批量执行，含 LLM 评判：

```python
# 批量执行搜索质量测试
# 使用 hermes chat -m deepseek-v4-pro 作为评判者
import subprocess, json

CL = "/home/huangxiaohai/.local/bin/codeloom"
results = []

def run(cmd):
    r = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=60)
    return r.stdout + r.stderr, r.returncode

def llm_judge(test_name, actual_output, criteria):
    """Use pro model to judge search quality"""
    prompt = f"""你是一个搜索质量评审专家。请评审以下搜索测试结果。

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
        f'hermes chat -Q -m deepseek-v4-pro -q {json.dumps(prompt)}',
        shell=True, capture_output=True, text=True, timeout=60
    )
    return r.stdout[:500]

# 执行并评审每个用例...
```

## 测试记录

| 用例 | 状态 | 日期 | 评判摘要 |
|------|------|------|---------|
| SQ-01 | ⬜ | | |
| SQ-02 | ⬜ | | |
| SQ-03 | ⬜ | | |
| SQ-04 | ⬜ | | |
| SQ-05 | ⬜ | | |
| SQ-06 | ⬜ | | |
| SQ-07 | ⬜ | | |
| SQ-08 | ⬜ | | |
| SQ-09 | ⬜ | | |
| SQ-10 | ⬜ | | |
| SQ-11 | ⬜ | | |
| SQ-12 | ⬜ | | |
