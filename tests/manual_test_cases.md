# CodeLoom 手工测试用例

> 基于 D:\code 下的真实 C++ 仓库，测试语义搜索 + 分支过滤。
> 环境：WSL，codeloom v0.3.1，模型 bge-small-zh (512d)，sqlite-vec ANN。

---

## 准备：索引所有测试仓

```bash
# leveldb — 133 文件，C++，单分支
codeloom index D:/code/leveldb --repo leveldb --branch master

# json v2 (单头文件版本，旧 API)
codeloom index D:/code/json-v2/src --repo json --branch v2

# json v3 (多文件版本，新 API)  
codeloom index D:/code/json-v3/include/nlohmann --repo json --branch v3

# spdlog — 日志库
codeloom index D:/code/spdlog/include/spdlog --repo spdlog --branch master

# flatbuffers — 序列化库
codeloom index D:/code/flatbuffers/include/flatbuffers --repo flatbuffers --branch master
```

验证索引状态：
```bash
codeloom status --repo leveldb
codeloom status --repo json
codeloom status --repo spdlog
codeloom status --repo flatbuffers
```

预期每个仓输出 `Symbols: XX | Edges: XX | Docs: XX`，数字不为 0。

---

## 测试一：语义搜索 — 自然语言找代码

> 以下全部通过 MCP 调用，格式：`echo '{...}' | codeloom mcp`

### 1.1 功能性搜索

```bash
# "键值对写入" → 应该找到 leveldb 的 Put/Write 相关函数
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"写入键值对数据","repo":"leveldb","branch":"master","limit":5}}}' | codeloom mcp

# 预期：命中 DB::Put、WriteBatch::Put 等符号
```

```bash
# "JSON 序列化" → 应该找到 json 的 dump/to_string 函数
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"将对象序列化为 JSON 字符串","repo":"json","branch":"v3","limit":5}}}' | codeloom mcp

# 预期：命中 json::dump()、to_string() 等符号
```

```bash
# "日志格式化输出"
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"格式化日志并输出到文件","repo":"spdlog","branch":"master","limit":5}}}' | codeloom mcp

# 预期：命中 logger::log()、formatter 等符号
```

### 1.2 数据结构搜索

```bash
# "跳表/有序数据结构" → leveldb 的 SkipList
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"跳表 有序数据结构 并发安全","repo":"leveldb","branch":"master","limit":5}}}' | codeloom mcp

# 预期：命中 SkipList、Arena 等
```

```bash
# "键值比较器" → leveldb 的 Comparator
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"字节序比较 键排序","repo":"leveldb","branch":"master","limit":5}}}' | codeloom mcp

# 预期：命中 Comparator、BytewiseComparator
```

### 1.3 跨语言语义（中文→英文代码）

```bash
# 中文描述 → 英文函数名匹配
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"合并写入操作批量提交","repo":"leveldb","branch":"master","limit":5}}}' | codeloom mcp

# 预期：WriteBatch 相关函数（bge-small-zh 支持中英跨语言）
```

```bash
# "迭代器遍历"
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"遍历所有键值对 迭代器","repo":"leveldb","branch":"master","limit":5}}}' | codeloom mcp

# 预期：Iterator::SeekToFirst、Iterator::Next 等
```

---

## 测试二：分支过滤

> json 仓有两个分支 — v2（旧 API，单头文件）和 v3（新 API，多文件）
> 验证：同一仓、不同分支、不同符号结果。

### 2.1 分支隔离

```bash
# 查 json v2 分支
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_overview","arguments":{"repo":"json","branch":"v2"}}}' | codeloom mcp

# 查 json v3 分支
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_overview","arguments":{"repo":"json","branch":"v3"}}}' | codeloom mcp
```

**预期**：v2 和 v3 返回的符号数量不同（v3 更多，因为有 45 个文件 vs 1 个）。

### 2.2 分支语义搜索

```bash
# 在 v2 分支搜索 "JSON 指针访问"
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"JSON 指针路径访问","repo":"json","branch":"v2","limit":5}}}' | codeloom mcp

# v2 里的 json_pointer 实现
```

```bash
# 在 v3 分支搜索同样的内容
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"JSON 指针路径访问","repo":"json","branch":"v3","limit":5}}}' | codeloom mcp

# v3 里的 json_pointer 实现（可能分布在多个文件中）
```

**预期**：相同查询，不同分支返回不同排名/不同符号（v3 有更细粒度的文件拆分）。

### 2.3 分支必须传参

```bash
# 缺 branch → 预期报错
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_overview","arguments":{"repo":"leveldb"}}}' | codeloom mcp

# 预期输出：branch is required
```

```bash
# 缺 branch 的语义搜索 → 预期报错
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"写数据库","repo":"leveldb"}}}' | codeloom mcp

# 预期输出：branch is required
```

### 2.4 分支术语表

```bash
# 给 v2 分支设置别名
codeloom branch set-alias old-api v2 --repo json --desc "v2.x 旧单头文件 API"

# 给 v3 分支设置别名
codeloom branch set-alias new-api v3 --repo json --desc "v3.x 新多文件 API"

# 查看所有别名
codeloom branch list-aliases --repo json
```

**预期**：输出 old-api → v2、new-api → v3。

---

## 测试三：符号定义查询

### 3.1 精确名称查找

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_get_definition","arguments":{"symbol":"DB::Put","repo":"leveldb","branch":"master"}}}' | codeloom mcp

# 预期：返回 DB::Put 的完整函数签名和源码
```

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_get_definition","arguments":{"symbol":"basic_json","repo":"json","branch":"v3"}}}' | codeloom mcp

# 预期：返回 basic_json 类定义
```

### 3.2 调用图

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_get_call_graph","arguments":{"symbol":"DB::Get","repo":"leveldb","branch":"master","direction":"callees","depth":2}}}' | codeloom mcp

# 预期：展示 DB::Get 调用了什么（如 internal_get、Seek 等）
```

### 3.3 跨仓对比

```bash
# leveldb 查 "写入"
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"写入数据","repo":"leveldb","branch":"master","limit":3}}}' | codeloom mcp

# spdlog 查 "写入"（不同语义域）
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"写入数据","repo":"spdlog","branch":"master","limit":3}}}' | codeloom mcp
```

**预期**：leveldb 返回 DB/Put/Write，spdlog 返回 logger/sink/flush — 同一查询在不同仓应返回不同域的结果。

---

## 测试四：全文搜索 + 边类型

### 4.1 全文搜符号名

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_search","arguments":{"query":"comparator","repo":"leveldb","branch":"master","limit":5}}}' | codeloom mcp

# 预期：命中 BytewiseComparator、InternalKeyComparator
```

### 4.2 边类型验证（仅 v0.3.1+）

先用 codeloom 索引 edge_types 测试夹具：
```bash
codeloom index D:/code/.../codeloom/tests/fixtures/edge_types --repo et --branch master
```

然后验证 enum_value、global、static_var：
```bash
# 看所有符号种类
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_overview","arguments":{"repo":"et","branch":"master"}}}' | codeloom mcp

# 语义搜索 "错误码" → 应命中 enum_value 类型的 Status::kOk
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"错误码 状态值","repo":"et","branch":"master","limit":3}}}' | codeloom mcp
```

---

## 测试五：性能基准（可选）

```bash
# 语义搜索延迟
time echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_semantic_search","arguments":{"query":"压缩数据块","repo":"leveldb","branch":"master","limit":10}}}' | codeloom mcp

# 预期：< 100ms（vec0 ANN）
```

---

## 问题记录模板

每项测试完成后记录：

| 用例 | 查询 | 预期 | 实际 | 通过 | 备注 |
|------|------|------|------|------|------|
| 1.1 | "写入键值对数据" | Put/Write 相关 | | | |
| 2.1 | overview v2 vs v3 | 符号数不同 | | | |
| 3.1 | get_definition DB::Put | 返回函数体 | | | |

---

## 清理

```bash
# 删除测试索引
rm ~/.codeloom/leveldb.rag.db ~/.codeloom/json.rag.db ~/.codeloom/spdlog.rag.db ~/.codeloom/flatbuffers.rag.db ~/.codeloom/et.rag.db
```
