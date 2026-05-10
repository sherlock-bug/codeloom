# Proposal: optimize-vector-storage

## Intent

当前向量存储占 DB 总量 88.9%（flatbuffers: 96/108 MB），其中符号名向量占 66.6%。通过 INT8 量化和选择性向量化，将名向量存储压缩 6.8×（72 MB → ~14 MB），并移除低价值的中文文档/注释向量。

## Scope

In scope:
- 符号名向量从 FLOAT32 切换为 INT8（4× 压缩）
- 移除 doc_vec 和 symbol_comment_vec（中文文字 FTS5 够用）
- 符号向量化筛选：排除 is_external=1、kind=template_instance、kind=namespace
- 搜索通道适配：doc/comment 通道移除，KNN 查询向量同步量化

Out of scope:
- file_vec 保持 FLOAT32 不变
- 存量 DB 迁移（需 re-index）
- 搜索质量 A/B 测试

## Approach

1. vec0 建表语法 `FLOAT[{dim}]` → `INT8[{dim}]`
2. 新增 `quantize_f32_to_i8()` 函数，嵌入向量按 `round(v * 127)` 量化
3. 删除 `index_vectors` 中的 comment 和 doc 相关逻辑
4. `hybrid_search` 移除 comment/doc 通道
5. `index_vectors` 中 SQL 查询加 WHERE 过滤条件

## Capabilities

### New Capabilities
- `int8-vector-storage`: 符号名向量以 INT8 格式存储，4× 压缩
- `selective-vectorization`: 只对特定 kind 的非外部符号生成向量

### Modified Capabilities
- `fts5-fulltext-index`: 文档搜索仅通过 FTS5，不再有向量通道
- `semantic-search`: 移除 comment 和 doc 语义通道，只保留 name 和 file
