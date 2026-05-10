# Proposal: 字符串字面量去重

## Why

`extract_string_literal()` 使用 `content_hash: String::new()`（空 hash），导致 UNIQUE 约束 `(content_hash, file_path, name, repo)` 失效。同一文件中多次出现的相同字面量（如 `"OK"`、`"ERROR"`）重复建节点，浪费存储。

## What Changes

`content_hash: String::new()` → `content_hash: dedup::hash_content(name)`，用字面量内容做 hash。空字面量已在 `if name.is_empty() { return; }` 跳过。

## Capabilities

### Modified Capabilities

- `code-indexing`: 字符串字面量符号的 content_hash 从空字符串改为内容哈希，触发 UNIQUE 约束自动去重

## Impact

- `src/indexer/queries/cpp.rs:957` — 一行改动
