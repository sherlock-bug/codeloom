# Tasks

## 1. 删除 doc_code_links 表

- [x] 1.1 删除 `storage/schema.rs` 中 `doc_code_links` 的 CREATE TABLE 语句
- [x] 1.2 `cargo build` 确认编译通过

## 2. 删除 link_docs_to_symbols 和相关函数

- [x] 2.1 删除 `embedding/mod.rs` 中 `link_docs_to_symbols()` 函数
- [x] 2.2 删除 `embedding/mod.rs` 中 `symbol_text_for_embedding()` 函数
- [x] 2.3 `cargo build` 确认编译通过

## 3. 删除 CLI index 中的调用

- [x] 3.1 删除 `cli/mod.rs` 中 index 命令的 `link_docs_to_symbols` 调用（含 embedder 创建和 println）
- [x] 3.2 `cargo build` 确认编译通过

## 4. 删除 MCP status/overview 中的 links 统计

- [x] 4.1 删除 `mcp/mod.rs` 中 `status()` 的 doc_code_links 查询和输出
- [x] 4.2 删除 `mcp/mod.rs` 中 `overview()` 的 doc_code_links 相关输出（如有）
- [x] 4.3 `cargo build` 确认编译通过

## 5. 删除 spec 文件

- [x] 5.1 删除 `openspec/specs/code-doc-linking/` 整个目录

## 6. 验证

- [x] 6.1 `cargo build --release` 零错误
- [x] 6.2 `openspec validate remove-doc-code-linking --json` 通过
- [x] 6.3 `openspec validate --specs --json` 全部通过
