# Proposal: 改善 MCP 工具用户体验

## Why

当前 CodeLoom 面临两个关键可用性问题：
1. **LLM 倾向于 grep 而非调用 MCP** — 工具描述缺少"优先使用我"的引导，导致 LLM 绕过索引直接搜索文件，丢失了索引覆盖的 #include 头文件符号和跨文件调用关系。
2. **repo 参数自动 fallback 到 "default"** — OpenCode 不传 repo 参数时，所有查询默认查到空库 `default.rag.db`，LLM 误判仓库未索引，触发重复索引。团队维护者缺少 CLI 命令来查看已有仓库和分支。

## What Changes

- **MCP 工具描述重写** — 8 个工具的 description 从"功能说明"改为"行为引导"，强调优先于 grep/rg 使用，明确优势（覆盖 #include 头文件、结构化返回、调用关系分析）
- **新增 `codeloom_list_repos` MCP 工具** — 列出所有已索引的仓库名，让 LLM 在打开项目后先确认 repo 参数值
- **新增 `codeloom_list_branches` MCP 工具** — 列出指定仓库下所有已索引的分支名，支持团队协作场景下的分支查询
- **新增 `codeloom list-repos` CLI 命令** — 终端直接查询已索引仓库
- **新增 `codeloom list-branches` CLI 命令** — 终端查询仓库下已索引分支
- **repo 参数缺失时返回错误** — 不再静默 fallback 到 "default"，改为返回友好错误并列出可用仓库
- **`codeloom_index` 描述诚实化** — 明确标注此工具不执行索引（需 CLI），防止 LLM 误调用
- **添加 OpenCode 使用指引文件** — `.opencode/instructions.md`，引导 LLM 优先使用 MCP 工具

## Capabilities

### New Capabilities
- `mcp-tool-descriptions`: MCP 工具描述重写为行为引导风格，引导 LLM 优先选择 MCP 而非 grep
- `list-repos`: 列出已索引仓库的 MCP 工具和 CLI 命令
- `list-branches`: 列出仓库下已索引分支的 MCP 工具和 CLI 命令
- `repo-required-validation`: repo 参数缺失时返回错误而非静默 fallback

### Modified Capabilities
- `mcp-server`: 修改 `codeloom_index` 工具行为和描述，明确标注 MCP 层不执行实际索引
- `opencode-integration`: 新增 `.opencode/instructions.md` 使用指引文件

## Impact

- `src/mcp/mod.rs` — 工具描述重写 + 新增 2 个工具 + repo 校验逻辑
- `src/cli/mod.rs` — 新增 `list-repos`、`list-branches` 子命令
- `.opencode/instructions.md` — 新增使用指引文件
- 无 API 破坏性变更，向后兼容
