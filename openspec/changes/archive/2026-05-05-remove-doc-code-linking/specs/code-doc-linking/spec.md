# Delta for code-doc-linking

## REMOVED Requirements

### Requirement: Embedding 驱动的自动关联
**Reason**: `doc_code_links` 表虽在索引时预计算相似度，但所有 MCP 查询工具从未使用。LLM 本身具备理解文档和代码关系的能力，只需在语义搜索中同时返回匹配的 doc 和 code 即可。
**Migration**: 删除 `doc_code_links` 表（schema migration），删除 `link_docs_to_symbols()` 函数，删除 `symbol_text_for_embedding()` 函数。语义搜索保持不变。

### Requirement: 关联强度可查询
**Reason**: 依赖 doc_code_links 表，随主功能移除。
**Migration**: 无。

### Requirement: 双向链路
**Reason**: 依赖 doc_code_links 表，随主功能移除。
**Migration**: 无。

### Requirement: 支持手动补充关联
**Reason**: 无 doc_code_links 表后不需要此能力。
**Migration**: 无。
