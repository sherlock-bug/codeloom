# Proposal: extract-access-specifier

## Intent

Clang 解析器当前不提取类成员的访问级别（public/private/protected）。`symbols.access` 列始终为空，导致下游无法按访问级别过滤（如"只向量化 public 方法"）。

## Scope

In scope:
- CXXMethodDecl 和 FieldDecl 节点提取正确的 access 值
- 识别 AccessSpecDecl 节点并传播到后续成员
- 正确使用 struct（默认 public）和 class（默认 private）的隐式访问级别
- FieldDecl 的 access 也需要写回 symbol

Out of scope:
- 枚举、typedef 等不适用 access 的节点
- 修改 DB schema（access 列已存在）
- uses_type 边的 access 传播

## Approach

在 `CXXRecordDecl` 的 inner 遍历循环中新增 `current_access` 状态跟踪：
1. 根据 `tagUsed` 设初始值：struct="public"，class="private"
2. 遇到 `AccessSpecDecl` 节点时更新 `current_access`
3. 将 `current_access` 传递给成员节点的 extract_node 和 FieldDecl 处理

当前 `extract_node` 没有 access 参数。最小改动方案：在 `AstVisitor` 上新增 `current_access: String` 字段，成员遍历前初始化、遍历中更新、子节点处理中读取。无需改 `extract_node` 签名。

## Capabilities

### New Capabilities
- `extract-access-specifier`: Clang 解析器提取 C++ 类成员的访问级别（public/private/protected），写入 symbols.access 列。

## Impact

- 修改文件：`src/indexer/clang/ast.rs`（约 15 行改动）
- 不影响现有测试（access 列已有，只是之前为空）
- 重新索引后存量数据不自动更新，需 re-index
