# Design: filter-forward-decls

## Context

当前 `ast.rs` 的 `extract_node` 函数在 `CXXRecordDecl` / `ClassTemplateDecl` 处理分支（~line 220）中，对所有匹配该 kind 的节点一视同仁地生成符号。Clang 的 AST dump JSON 对前向声明和定义的区别是：定义节点有 `completeDefinition: true` 字段，前向声明没有该字段。

## Goals / Non-Goals

Goals:
- 前向声明 `class Foo;` 不再产生 class 符号
- 真正的类定义 `class Foo { ... }` 照常产生符号
- 现有单元测试通过

Non-Goals:
- 不修改 `upsert_symbol` 的去重逻辑（`file_path` 仍然是 dedup key 的一部分）
- 不处理 Clang 19 下 `loc.file` 缺失的问题

## Decisions

### Decision: 守卫条件而非 upsert 层过滤

在 AST 处理层直接跳过前向声明，而不是依赖 upsert 层的 dedup。原因：
- AST 层更早过滤，减少不必要的 SQL 写入
- 语义清晰：前向声明本身不定义新实体
- 符合用户直觉："class 只会有一个声明"

### Decision: 用 `completeDefinition` 而非 `isDefinition`

Clang JSON dump 使用 `completeDefinition` 字段标识类定义。`isDefinition` 在 JSON 中不存在。实测确认：
- 定义节点：`completeDefinition = True`，有 `definitionData` 子字段
- 前向声明：无 `completeDefinition` 字段

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 跨 TU 的同类定义因 file_path 不同仍去重失败 | 该问题属于 upsert_symbol 的 dedup key 设计，不在此 change 范围内 |
| 某些 Clang 版本可能对 partial/incomplete types 输出不同 | 守卫条件用 `unwrap_or(false)`，保守安全 |
