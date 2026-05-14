# schema-metadata

## Purpose
CodeLoom 元数据工具：暴露节点类型（15种）和边类型（10种）的定义、方向和所有合法组合。

## Requirements

### Requirement: Schema 元数据导出
系统 SHALL 提供 `codeloom_schema` 工具，导出当前 CodeLoom 支持的所有节点类型、边类型和属性说明。

#### Scenario: LLM 查询可用能力
- GIVEN LLM 需要知道有哪些边类型和节点类型可用
- WHEN 调用 `codeloom_schema`
- THEN 返回结构化的元数据：`node_kinds`（名称+描述+示例）、`edge_types`（名称+方向语义+参与节点类型）

#### Scenario: 元数据包含节点类型
- GIVEN CodeLoom 索引支持 function, method, class, struct, enum, enum_value, global, static_var, variable, field, string_literal, macro, template_function, template_class, template_struct
- WHEN 查询 schema
- THEN `node_kinds` 数组中每个条目包含 `name`、`description`、`example`

#### Scenario: 元数据包含边类型
- GIVEN CodeLoom edges 表包含 calls, overrides, inherits, contains, uses, references, returns, param_type, field_type, template_use 等边
- WHEN 查询 schema
- THEN `edge_types` 数组中每个条目包含 `prefix`、`direction`（from→to 的语义）、`source_kinds`、`target_kinds`

### Requirement: Schema 工具无参数
系统 SHALL 使 `codeloom_schema` 无需任何参数即可调用，返回硬编码的稳定元数据。

#### Scenario: 无需 repo 或 branch
- GIVEN LLM 在任何阶段
- WHEN 调用 `codeloom_schema`
- THEN 不需要 repo/branch/name 等任何参数

### Requirement: 移除 codeloom_overview 工具
系统 SHALL 移除 `codeloom_overview` MCP 工具，其统计输出对 LLM 无意义，功能被 `codeloom_schema` 替代。

#### Scenario: overview 工具已移除
- GIVEN CodeLoom MCP 工具列表
- WHEN 查询 tools/list
- THEN 不存在 codeloom_overview，存在 codeloom_schema 提供节点和边类型元数据
