# Proposal: 删除文档-代码自动关联

## Why

`doc_code_links` 表在每次索引后预计算文档段落与代码符号的相似度（9936 条边 / leveldb），但所有 MCP 查询工具从未使用这些数据：`get_definition` 不返回关联文档，`semantic_search` 不返回关联代码。LLM 本身具备理解文档和代码关联的能力——只需在语义搜索结果中同时返回匹配的 doc 段落和 code 符号即可。

删掉预计算逻辑可以：减少索引耗时（省略 O(N×M) 的逐对相似度计算）、减少 DB 体积（9936 行）、简化代码。

## What Changes

- **删除** `doc_code_links` 表（schema migration）
- **删除** `link_docs_to_symbols()` 函数（embedding/mod.rs ~90 行）
- **删除** `symbol_text_for_embedding()` 函数（不再需要）
- **删除** CLI `index` 命令中对 `link_docs_to_symbols` 的调用
- **删除** MCP `status()` 和 `overview()` 中 doc_code_links 统计
- **保留** doc_nodes 的向量化和 `semantic_search` 中返回文档（文档搜索本身不变）

## Capabilities

### Modified Capabilities

- `code-doc-linking`: 整个 spec 移除
- `doc-indexing`: 移除与代码符号建立关联的 Requirement，保留纯文档解析和索引
- `mcp-server`: 移除 `status`/`overview` 中 Doc-Code links 统计
- `cli-mode`: 移除 `index` 命令中的 `link_docs_to_symbols` 调用

## Impact

- 索引提速：省去逐对相似度计算（leveldb 级别省 120000 次比较）
- DB 缩小：去掉 `doc_code_links` 表
- 语义搜索不变：文档照样可搜，LLM 自己理解关联
- **BREAKING**: `doc_code_links` 表删除，旧 DB 会多一个空表（不影响查询，但 schema 变干净了）
