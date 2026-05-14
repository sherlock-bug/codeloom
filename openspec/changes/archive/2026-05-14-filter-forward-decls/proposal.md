# Proposal: filter-forward-decls

## Intent

Clang 解析器在处理 CXXRecordDecl（class/struct）时，不区分前向声明和真正定义，导致前向声明也被生成 class 符号节点入库。同一个类因在多个头文件中有前向声明而产生多个重复节点。

## Scope

In scope:
- 在 ast.rs 的 CXXRecordDecl/StructDecl/ClassTemplateDecl 处理分支中，检查 `completeDefinition` 字段，跳过前向声明
- 保持隐式节点（`isImplicit`）的现有行为不变
- 添加对应的单元测试覆盖前向声明过滤逻辑

Out of scope:
- 非 CXXRecordDecl 的其他节点类型（FunctionDecl 等已用 has_body 区分声明和定义）
- Clang 19 下 `loc.file` 缺失导致 class 落到 .cc 路径的问题（需另案处理）

## Approach

在 `ast.rs` 中 `CXXRecordDecl` / `ClassTemplateDecl` 的处理分支头部增加守卫条件：

```rust
if !node.get("completeDefinition").and_then(|v| v.as_bool()).unwrap_or(false) {
    return; // Skip forward declarations — only process real definitions
}
```

同时需要在 `upsert_symbol` 中确认：已有的 declaration→definition 合并逻辑（decl→def upgrade）仍然适用于跨文件的 class 定义合并。

## Capabilities

### New Capabilities

- `filter-forward-class-decls`: Clang 索引器 SHALL 跳过没有 `completeDefinition` 字段的 CXXRecordDecl 和 ClassTemplateDecl 节点
