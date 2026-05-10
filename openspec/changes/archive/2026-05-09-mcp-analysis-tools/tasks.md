# Tasks

## 1. 删除旧工具
- [x] 1.1 删除 `codeloom_overview` 的 MCP 注册和实现
- [x] 1.2 更新 `codeloom_index` 提示语

## 2. 通用图遍历引擎
- [x] 2.1 `src/query/graph.rs` — BFS + 传递闭包 + 邻接查询 + 终端节点依赖
- [x] 2.2 `resolve_symbol_id` + `symbol_name_by_id`
- [x] 2.3 `get_edges` — direction + edge_filter
- [x] 2.4 `bfs_path_search` — shortest/all + 环检测 + edge_filter
- [x] 2.5 `transitive_closure` — direction + radius + edge_filter
- [x] 2.6 `neighbor_map` — direction + depth + 分组

## 3. Schema 元数据工具
- [x] 3.1 `codeloom_schema` — 硬编码 15 种节点类型 + 10 种边类型
- [x] 3.2 注册到 tools_list + handle_tool_call

## 4. 分析 MCP 工具 + 增强 call_graph
- [x] 4.1 `codeloom_path_analysis` — mode + max_paths + edge_filter + 环检测
- [x] 4.2 `codeloom_impact_analysis` — direction + 闭包
- [x] 4.3 `codeloom_neighbor_graph` — direction + 分组
- [x] 4.4 `codeloom_inheritance_tree` — 递归树 + overrides
- [x] 4.5 `codeloom_get_call_graph` — 描述更新（提及终端节点依赖，实际增强在 graph.rs）

## 5. MCP 工具描述
- [x] 5.1 6 个工具 description（schema + 4 analysis + call_graph updated）
- [x] 5.2 tools_list 更新（15 工具）

## 6. 测试
- [x] 6.1-6.6 cargo test 76 passed (64 unit + 2 ignored + 10 integration)
- [x] 6.7 工具数断言更新（11→15）+ schema 豁免参数检查

## 7. 验证
- [x] 7.1 build release + leveldb 实测验证（工具逻辑正确，4 个边数据 bug 独立记录）
- [x] 7.2 更新 README.md
