# Design: 改善 MCP 工具用户体验

## Context

当前 CodeLoom 的 MCP 工具用户体验存在两个核心缺陷：

**缺陷 1：工具发现与选择**
- MCP 8 个工具的 description 采用"功能说明书"风格（如"全文精确搜索符号名和定义内容"），缺少行为引导
- LLM 读 description 后无法感知 MCP 相对于 grep 的优势（索引覆盖 #include 头文件、结构化返回、调用关系分析）
- 没有工具引导 LLM 先调用 `codeloom_overview` 或 `codeloom_list_repos` 来感知仓库状态

**缺陷 2：repo 参数静默 fallback**
- 所有 8 个工具中 repo 参数标记为 optional，后端 `unwrap_or("default")` 静默 fallback
- OpenCode 不传 repo → `default.rag.db` → 空结果 → LLM 误判"未索引" → 触发重复索引
- 缺少 `list_repos` / `list_branches` 工具让 LLM 和用户查询当前已索引的仓库和分支

## Goals / Non-Goals

**Goals:**
- 重写 8 个 MCP 工具 description 为行为引导风格，明确标注"优先于 grep/rg 使用"
- 新增 `codeloom_list_repos` MCP 工具 + `codeloom list-repos` CLI 命令
- 新增 `codeloom_list_branches` MCP 工具 + `codeloom list-branches` CLI 命令
- 当 repo 参数为空时返回错误并列出可用仓库，不再静默 fallback
- 修改 `codeloom_index` 的 description 和 MCP 行为，标注"不通过 MCP 执行索引"
- 添加 `.opencode/instructions.md` 文件引导 LLM 优先使用 MCP 工具

**Non-Goals:**
- 不让 `codeloom_index` 在 MCP 层执行实际索引（太重，应保持 CLI 独占）
- 不新增 `codeloom_quick_search` 等额外搜索工具（描述重写已足够，避免功能臃肿）
- 不修改索引逻辑或数据库 schema

## Decisions

### Decision 1: 描述重写策略 — "行为引导"风格

选择：在 description 开头加 `**首选工具**` / `**优先使用**` 等强调标记，列出相对于 grep 的优势

替代方案：用系统提示词在 OpenCode 层面强制优先级 → 弃用，因为不同客户端（Claude Code、Continue）系统提示词机制不同，MCP description 是通用入口。

### Decision 2: list-repos / list-branches 实现方式

选择：查询 `~/.codeloom/` 下的 `*.rag.db` 文件，从文件名提取 repo 名；对 branches，查询 SQLite 的 `branches` 表 DISTINCT branch_name

替代方案：维护一个 repos registry yaml → 弃用，引入额外状态文件增加了同步负担，文件名即 repo 名是自描述的。

### Decision 3: repo 参数空值处理

选择：当 `repo` 参数为空字符串或 `"default"` 时，返回错误 + 调用 `list_repos()` 列出可用仓库

替代方案：自动从当前目录推断 repo 名 → 弃用，MCP 进程不一定在项目目录下运行（远程部署场景），无法可靠推断。

### Decision 4: `codeloom_index` MCP 行为

选择：返回 "索引请用 CLI: codeloom index <path> --repo <name> --branch <branch>" + 提示先用 `codeloom_list_repos` 检查是否已索引

替代方案：在 MCP 层执行 index → 弃用，索引操作耗时长、需要写锁，会阻塞其他 MCP 请求，干扰团队共享。

### Decision 5: CLI 命令设计

选择：
- `codeloom list-repos` — 列出 `~/.codeloom/*.rag.db` → 输出简洁列表
- `codeloom list-branches <repo>` — 查询 `<repo>.rag.db` 的 branches 表 → 输出分支列表

CLI 和 MCP 共享同一套查询逻辑（提取为 `src/query/` 下的公共函数）。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 描述重写后 LLM 仍然 prefer grep | 通过 `.opencode/instructions.md` 做双保险；观察线上行为收集反馈 |
| repo 报错可能导致 LLM 困惑 | 错误信息中附带 `codeloom_list_repos` 的输出，降低困惑度 |
| `codeloom list-branches` 依赖 DB 存在 | 前置检查 .rag.db 是否存在，不存在时友好提示 |
| 新工具增加 token 消耗 | 每个工具 description ~200 字符，8 个工具总计 ~1.6K tokens，可忽略 |
