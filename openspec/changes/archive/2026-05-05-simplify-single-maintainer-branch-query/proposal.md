# Proposal: 架构简化 — 单人维护 + 分支查询增强

## Why

CodeLoom 初始设计面向团队协作（双层 DB、路径无关化、Pull/Push 共享），但实际使用场景是**单人远程部署 MCP Server**，LLM 通过 MCP 按分支查询代码知识。Pull/Push/SwitchBranch 从未实现、团队共享用不上，反而增加了 MCP 工具的复杂度。同时，当前 MCP 查询工具对 `branch` 参数处理不一致（部分可选、部分缺省），需要统一为**必传分支 + NULL=全分支可见**的清晰规则。

## What Changes

- **删除** `codeloom pull` / `codeloom push` / `codeloom switch-branch` CLI 子命令（均为占位代码）
- **删除** MCP 工具 `codeloom_pull` / `codeloom_push` / `codeloom_switch_branch`（均为占位实现）
- **删除** OpenCode 命令 `/codeloom:pull` / `/codeloom:push`
- **修改** 所有 MCP 查询工具：`branch` 参数改为必传（LLM 调用时需提供当前分支名）
- **统一分支过滤规则**：索引时 `branch_name IS NULL` 的数据（如文档）对所有分支可见；`branch_name` 有值的按分支精确过滤。代码和文档一视同仁，不区别对待
- **移除** `team-sharing` spec（base+overlay 双层 DB、路径无关化、Pull/Push 团队共享均不在单人场景使用）
- 代码层面：删除 CLI stub、MCP stub、OpenCode 命令文件；mcp/mod.rs 中所有查询 SQL 统一分支过滤逻辑

## Capabilities

### Modified Capabilities

- `mcp-server`: 所有 MCP 查询工具 `branch` 参数必传；统一分支过滤规则（NULL=全可见，非NULL=精确匹配）
- `cli-mode`: 移除 `pull`/`push`/`switch-branch` 子命令
- `branch-management`: 分支过滤规则明确化 — 索引时未指定分支的数据所有分支可见
- `code-indexing`: 查询时按 branch 过滤代码符号
- `doc-indexing`: 文档索引不强制绑定分支，NULL branch 的文档全分支可见
- `team-sharing`: 移除整个 spec（功能不再需要）
- `opencode-commands`: 移除 `/codeloom:pull` / `/codeloom:push` 命令文件

## Impact

- MCP 接口：`branch` 参数从可选变必传，**BREAKING** 现有 LLM 调用需更新
- CLI 接口：移除 3 个子命令，**BREAKING** 已有脚本需调整
- 存储：schema 不变，仅查询逻辑调整
- 部署：单人维护，MCP Server 远程运行，无需考虑团队同步
