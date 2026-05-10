# Delta for path-analysis

## ADDED Requirements

### Requirement: 跨类型 BFS 路径搜索
系统 SHALL 提供跨所有边类型和节点类型的双向 BFS 路径搜索能力，支持最短路径和全路径两种模式。

#### Scenario: 函数到函数的调用路径
- GIVEN 函数 `main` 间接调用 `handle_error`，经过 `dispatch` 和 `validate`
- WHEN 调用 `codeloom_path_analysis` 查询 `source=main target=handle_error mode=shortest`
- THEN 返回最短路径 `main → calls → dispatch → calls → validate → calls → handle_error`

#### Scenario: 全路径模式返回多条路径
- GIVEN 存在多条从 `A` 到 `D` 的路径
- WHEN 调用 `codeloom_path_analysis` 查询 `source=A target=D mode=all max_paths=10`
- THEN 返回所有找到的路径，条数不超过 max_paths

#### Scenario: 环检测丢弃重复节点路径
- GIVEN 路径 `A → B → C → A → ...` 中出现重复节点 A
- WHEN BFS 扩展时检测到下一个节点已在当前路径前缀中
- THEN 丢弃该路径，不产生无限循环

#### Scenario: 无路径时返回空
- GIVEN 两个符号之间不存在任何路径
- WHEN 调用路径分析
- THEN 返回 `{"paths": [], "total_found": 0}`

### Requirement: 输入参数定义
系统 SHALL 接受必填参数 `source`、`target`、`repo`、`branch`；可选参数 `mode`（默认 "shortest"）、`max_paths`（默认 20）、`max_depth`（默认 10）、`direction`（默认 "both"）、`edge_filter`（默认 null，即所有边类型）。

#### Scenario: 使用 edge_filter 限定边类型
- GIVEN 用户想查从 A 到 B 的纯调用路径
- WHEN 调用路径分析 `source=A target=B edge_filter=["calls", "calls_override"]`
- THEN BFS 只沿 calls 和 calls_override 边遍历，不穿类或其他关系

#### Scenario: 不指定 edge_filter 时走全边
- GIVEN 用户不指定 edge_filter
- WHEN 调用路径分析
- THEN 走所有可用边类型

#### Scenario: 默认参数
- GIVEN 用户只指定 source 和 target
- WHEN 工具执行
- THEN 使用 mode=shortest, direction=both, max_paths=20, max_depth=10
