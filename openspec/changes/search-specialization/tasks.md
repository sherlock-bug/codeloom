# Tasks

## 1. FusedResult 增加 enrichment 字段

- [x] 1.1 `src/query/search.rs`: `FusedResult` 结构体增加 `parent_class`、`member_count`、`method_count`、`enum_value_count`、`child_count`、`section_level`、`parent_section` 字段（默认空/0）
- [x] 1.2 `src/query/search.rs`: `bm25_precise_search()` 和 `vector_semantic_search()` 终点增加 enrichment 填充
- [x] 1.4 编译通过

## 2. MCP 输出格式

- [x] 2.1 `src/mcp/mod.rs`: `bm25_precise_search()` 的 JSON 输出中附加 enrichment 字段（仅非空时输出）
- [x] 2.2 `src/mcp/mod.rs`: `semantic_search()` 的 JSON 输出同样附加 enrichment 字段
- [x] 2.3 `src/mcp/mod.rs`: `list_symbols()` 是纯文本输出，不适用

## 3. 编译验证

- [x] 3.1 `cargo build` 编译通过
- [x] 3.2 `cargo test` 全部通过（64 passed, 0 failed）

## 4. 更新 README.md

- [x] 4.1 搜索工具描述和输出示例同步更新（MCP 描述新增 enrichment 说明，README 工具数量一致）
