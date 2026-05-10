# Tasks

## 1. MCP 工具描述重写

- [x] 1.1 重写 `codeloom_search` 描述 — 添加「**首选工具**」标记，列出 vs grep 优势，给 query 示例
- [x] 1.2 重写 `codeloom_list_symbols` 描述 — 强调先于 grep 使用，说明 C++ ClassName::methodName 格式
- [x] 1.3 重写 `codeloom_get_definition` 描述 — 强调返回精确区间不浪费 token，标明必须先 list_symbols
- [x] 1.4 重写 `codeloom_get_call_graph` 描述 — 强调必须先 index + list_symbols 再查调用图
- [x] 1.5 重写 `codeloom_semantic_search` 描述 — 给中文 query 示例，说明向量搜索结果格式
- [x] 1.6 重写 `codeloom_overview` 描述 — 标注「打开仓库后第一个调用的工具」
- [x] 1.7 重写 `codeloom_status` 描述 — 标注用于检查索引状态
- [x] 1.8 重写 `codeloom_index` 描述 — 添加「⚠️ 不通过 MCP 执行索引」标注，引导先用 list_repos 检查

## 2. 新增 list-repos (MCP + CLI)

- [x] 2.1 在 `src/query/` 下新增查询函数 `list_repos()` — 扫描 `~/.codeloom/*.rag.db`，解析文件名
- [x] 2.2 在 `src/mcp/mod.rs` 注册 `codeloom_list_repos` 工具，调用 `list_repos()`
- [x] 2.3 在 `src/cli/mod.rs` 添加 `list-repos` 子命令
- [x] 2.4 为 `list_repos()` 添加单元测试

## 3. 新增 list-branches (MCP + CLI)

- [x] 3.1 在 `src/query/` 下新增查询函数 `list_branches(repo)` — 查询 `branches` 表 DISTINCT branch_name
- [x] 3.2 在 `src/mcp/mod.rs` 注册 `codeloom_list_branches` 工具，repo 参数必填
- [x] 3.3 在 `src/cli/mod.rs` 添加 `list-branches <repo>` 子命令
- [x] 3.4 为 `list_branches()` 添加单元测试

## 4. repo 参数缺失校验

- [x] 4.1 修改 `handle_tool_call` 中 7 个工具的 `unwrap_or("default")` → 空值时报错
- [x] 4.2 错误信息调用 `list_repos()` 附加可用仓库列表
- [x] 4.3 保留 `codeloom_index` 和 `codeloom_list_repos` 的 repo 可选行为
- [x] 4.4 添加 repo 校验相关的单元测试

## 5. codeloom_index 行为修正

- [x] 5.1 修改 `codeloom_index` 的 MCP 返回值，加入「先用 list_repos 检查是否已索引」的提示
- [x] 5.2 更新 README 中 `codeloom_index` 工具描述，标明 MCP 不支持执行索引

## 6. OpenCode 使用指引

- [x] 6.1 创建 `.opencode/instructions.md`，包含搜索优先级规则 + MCP 工具速查表
- [x] 6.2 验证 OpenCode 自动加载 guidance 文件

## 7. 测试与文档

- [x] 7.1 运行 `cargo test` 确保全过（含新增测试）
- [x] 7.2 运行 `cargo check` 零警告
- [x] 7.3 更新 `README.md` — 工具数量 8→10，工具表新增 list-repos/list-branches，常见查询模式更新
