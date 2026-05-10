# Design: 删除文档-代码自动关联

## Context

`doc_code_links` 表在索引后预计算文档段落与代码符号的余弦相似度（leveldb: 1956 符号 × 63 文档 → 9936 条边）。但 MCP 查询工具从未使用这些数据：`get_definition` 不返回关联文档，`semantic_search` 不返回关联代码。删除此表和相关逻辑可减少索引耗时和 DB 体积，语义搜索功能不受影响。

## Goals / Non-Goals

**Goals:**
- 删除 `doc_code_links` 表及相关代码
- 删除 `link_docs_to_symbols()` (~90 行)
- 删除 `symbol_text_for_embedding()` 工具函数
- 删除 MCP status/overview 中的 links 统计

**Non-Goals:**
- 不影响 doc_nodes 的索引和向量化
- 不影响 semantic_search 返回文档段落
- 不修改 schema 中其他表

## Decisions

### Decision 1: 直接删除而非标记 deprecated

`doc_code_links` 从未被实际使用（查询工具不读它），直接删除。

理由：
- 不存在迁移兼容性问题
- 旧 DB 中多一个空表不影响查询

### Decision 2: schema 中移除 `doc_code_links` 表

`CREATE TABLE IF NOT EXISTS` 语法意味着旧 DB 中的表会保留但不再被代码引用。新 DB 不再创建此表。

## Migration Plan

1. 删除 `storage/schema.rs` 中 `doc_code_links` 表定义
2. 删除 `embedding/mod.rs` 中 `link_docs_to_symbols()` 和 `symbol_text_for_embedding()`
3. 删除 `cli/mod.rs` 中 index 命令的 link_docs_to_symbols 调用
4. 删除 `mcp/mod.rs` 中 status/overview 的 doc_code_links 统计
5. 删除 `openspec/specs/code-doc-linking/` spec 目录
6. 编译验证 + spec 验证
