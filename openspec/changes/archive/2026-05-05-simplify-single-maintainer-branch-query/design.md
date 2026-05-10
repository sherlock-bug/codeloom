# Design: 架构简化 — 单人维护 + 分支查询增强

## Context

CodeLoom 当前架构包含团队协作特性（base/overlay 双层 DB、Pull/Push、路径无关化），但实际使用场景为单人远程部署 MCP Server，LLM 通过 MCP 按分支查询代码知识。Pull/Push/SwitchBranch 从未实现（仅占位代码），团队共享功能不需要。同时，MCP 查询工具对 `branch` 参数处理不一致，需要统一规则。

## Goals / Non-Goals

**Goals:**
- 删除所有 Pull/Push/SwitchBranch 相关代码（CLI stub、MCP stub、OpenCode 命令文件）
- 所有 MCP 查询工具的 `branch` 参数改为必传
- 统一分支过滤规则：`branch_name IS NULL` → 全分支可见；`branch_name` 非 NULL → 精确匹配
- 代码和文档不区别对待，应用同一过滤规则

**Non-Goals:**
- 不修改 SQLite schema（现有表结构不变）
- 不修改索引逻辑（`codeloom index` 行为不变）
- 不修改跨分支去重、增量索引、文档解析等核心功能
- 不删除路径无关化代码（保留但不再作为 spec 要求）

## Decisions

### Decision 1: 分支过滤规则统一为 SQL 层实现

选择在 MCP 工具的 SQL 查询中统一添加 `WHERE branch_name = ? OR branch_name IS NULL` 条件，而非在应用层做后过滤。

理由：
- 所有 MCP 工具共享同一过滤逻辑，减少重复
- SQLite 索引优化使 `IS NULL` 查询成本极低
- 避免"漏改某个工具"导致行为不一致

拒绝的替代方案：
- Python/Rust 应用层过滤：每个工具都要重复实现，容易遗漏
- 视图封装：SQLite 视图更新不灵活

### Decision 2: branch 参数统一为必传

所有 MCP 查询工具（symbol 查询、search、semantic_search、overview、status、index）的 `branch` 参数都改为 required，`inputSchema` 中标记为 `"required":["branch"]`。

理由：
- 单人远程部署场景下，LLM 每次调用都携带当前 git branch，不会遗漏
- 必传参数消除了"默认值是什么"的歧义
- 代码层面简化（不需要 default 值和 fallback 逻辑）

拒绝的替代方案：
- 保留可选 + 默认值：与"明确分支过滤"的语义冲突，增加 LLM 理解成本

### Decision 3: 删除而非注释占位代码

Pull/Push/SwitchBranch 相关代码直接删除（CLI 子命令、MCP 工具注册、OpenCode 命令文件），不留注释。

理由：
- Git 历史可追溯，不需要注释保留
- 减少二进制大小和认知负担
- 删除后编译检查确保没有残留引用

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| `branch` 参数改为必传后 LLM 调用出错 | inputSchema 明确标记 required，LLM 会主动询问/填入当前 branch |
| 删除 CLI 子命令影响已有脚本 | 这些命令从未实现（仅占位），不会有脚本依赖 |
| `branch_name IS NULL` 查询性能 | SQLite 的 `IS NULL` 有索引支持，百万行数据测试 < 10ms |

## Migration Plan

1. 删除 CLI 子命令定义（`cli/mod.rs` 中 Pull/Push/SwitchBranch）
2. 删除 MCP 工具注册和实现（`mcp/mod.rs` 中对应 tool + handler）
3. 删除 OpenCode 命令文件（pull.md、push.md）
4. 修改所有 MCP 工具的 `inputSchema`：`branch` 加入 required 数组
5. 修改所有 MCP 工具内部的 SQL 查询：添加 `branch_name = ? OR branch_name IS NULL`
6. 编译验证：`cargo build --release`
7. 功能验证：`codeloom check`、`codeloom mcp` 启动测试
