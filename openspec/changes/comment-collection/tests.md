# tests: comment-collection

## 测试预备

所有测试基于 leveldb 真实仓库（133 文件），测试前执行 `clean + index`。

## 单元测试

### UT-1: 上方注释提取

**输入**: 以下源码片段
```cpp
/// Opens a database.
/// Returns OK on success.
Status Open(const std::string& name, DB** dbptr);
```
**期望**: `collect_comments_for_symbol("test.cc", 4, 4)` 返回 `"Opens a database.\nReturns OK on success."`

### UT-2: 体内注释提取

**输入**: 函数体含 `// Step 1: validate` 和 `/* Range check */`
**期望**: 函数符号的 doc_comment 包含 `"Step 1: validate\nRange check"`

### UT-3: 行内注释

**输入**: `int max_level = 7;  // Maximum number of levels`
**期望**: 变量 `max_level` 的 doc_comment 为 `"Maximum number of levels"`

### UT-4: 无注释

**输入**: 函数声明无任何注释
**期望**: doc_comment 为空字符串 `""`

### UT-5: 文件头注释

**输入**: 以 License 注释块开头的源文件
**期望**: `collect_file_header()` 返回 License 文本

### UT-6: 前缀剥离

**输入**: `/// Open the file`
**期望**: 输出 `"Open the file"`（无 `///` 前缀）

### UT-7: 多行 `/** */` 注释

**输入**:
```
/**
 * Performs compaction.
 * May block.
 */
```
**期望**: 输出 `"Performs compaction.\nMay block."`

## 端到端测试（CLI）

### E2E-CLI-C01: 关键词搜索命中注释

| 字段 | 内容 |
|------|------|
| **名称** | BM25 搜索命中注释中的词语 |
| **前置条件** | leveldb 已索引（comment-collection 版本） |
| **步骤** | `cl search "Copyright 2011" --repo leveldb --branch master --limit 5` |
| **预期结果** | 返回 5 条结果，至少包含一个 file node（如 `include/leveldb/cache.h`），其 snippet 含 `"Copyright (c) 2011 The LevelDB Authors"` |
| **验证方法** | 搜索结果的 snippet 不为空；snippet 包含 "Copyright" |

### E2E-CLI-C02: 体内注释搜索

| 字段 | 内容 |
|------|------|
| **名称** | 搜索函数体内的注释 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl search "Thread safety" --repo leveldb --branch master --limit 5` |
| **预期结果** | 返回结果中包含含有 "Thread safety" 注释的函数符号 |
| **验证方法** | 至少一条结果的 snippet 包含 "Thread safety" |

### E2E-CLI-C03: 文件头注释搜索

| 字段 | 内容 |
|------|------|
| **名称** | 搜索文件头许可证注释 |
| **前置条件** | leveldb 已索引 |
| **步骤** | `cl search "Redistribution" --repo leveldb --branch master --limit 3` |
| **预期结果** | 返回 file node 结果，snippet 包含 BSD License 文本 |
| **验证方法** | 结果的 `hit_type` 为 "file"，snippet 包含 "Redistribution" |

## 端到端测试（MCP）

### E2E-MCP-C01: MCP 搜索命中注释

| 字段 | 内容 |
|------|------|
| **编号** | E2E-MCP-C01 |
| **名称** | `codeloom_search` BM25 命中注释内容 |
| **前置条件** | leveldb@master 已索引 |
| **JSON-RPC 请求** | `echo '{"jsonrpc":"2.0","id":100,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"Thread safety","repo":"leveldb","branch":"master","limit":5}}}' \| /mnt/d/RagMcpHermes/codeloom/target/debug/codeloom mcp 2>/dev/null` |
| **预期结果** | JSON 响应，`count` > 0，至少一条结果的 `snippet` 字段非空 |
| **验证方法** | 解析 JSON：验证 `count` > 0；验证存在 `snippet` 非空的结果 |
