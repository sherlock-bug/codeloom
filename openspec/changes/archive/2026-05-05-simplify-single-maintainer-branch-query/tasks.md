# Tasks

## 1. 删除 Pull/Push/SwitchBranch CLI

- [x] 1.1 删除 `cli/mod.rs` 中 Pull、Push、SwitchBranch 子命令定义
- [x] 1.2 删除 cli/mod.rs 中对应的 match 分支
- [x] 1.3 `cargo build --release` 确认编译通过

## 2. 删除 MCP Pull/Push/SwitchBranch 工具

- [x] 2.1 删除 `mcp/mod.rs` 中 `tools_list()` 里的 codeloom_pull、codeloom_push、codeloom_switch_branch
- [x] 2.2 删除 `mcp/mod.rs` 中 `handle_tool_call()` 对应的三个 match 分支
- [x] 2.3 `cargo build --release` 确认编译通过

## 3. 修改 MCP 工具：branch 参数改为必传

- [x] 3.1 修改 `tools_list()` 中所有 8 个工具的 `inputSchema`：`branch` 加入 `required` 数组
- [x] 3.2 移除所有工具中 `branch` 参数的默认值逻辑
- [x] 3.3 如果 `branch` 为空字符串，返回明确错误 "branch is required"
- [x] 3.4 `codeloom_index` 工具的 MCP handler 改为参数校验后返回错误而非占位提示

## 4. 统一分支过滤规则（SQL 层）

- [x] 4.1 修改 `overview()`：symbols 统计 SQL 加 `AND (b.branch_name=?1 OR b.branch_name IS NULL)`
- [x] 4.2 修改 `status()`：symbols 统计 SQL 同上
- [x] 4.3 修改 `list_symbols()`：查询 SQL 加 branch 过滤
- [x] 4.4 修改 `get_definition()`：查询 SQL 加 branch 过滤
- [x] 4.5 修改 `get_call_graph()`：查询 SQL 加 branch 过滤
- [x] 4.6 修改 `fulltext_search()`：查询 SQL 加 branch 过滤
- [x] 4.7 修改 `semantic_search()`：doc_nodes 查询移除 branch 过滤（NULL 全可见），symbols 查询加 branch 过滤
- [x] 4.8 修改 `traverse_calls()` 辅助函数中的 name lookup SQL 加 branch 过滤

## 5. 删除 OpenCode 命令文件

- [x] 5.1 删除 `opencode/commands/codeloom-pull.md`
- [x] 5.2 删除 `opencode/commands/codeloom-push.md`

## 6. 验证

- [x] 6.1 `cargo build --release` 零错误（1 个预存 warning）
- [x] 6.2 `openspec validate simplify-single-maintainer-branch-query --json` 通过
- [x] 6.3 功能测试：MCP tools/list 返回 8 个工具，全部 branch=required，缺 branch 返回错误
