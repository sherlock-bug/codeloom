# Delta for schema-metadata

## ADDED Requirements

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
- GIVEN CodeLoom edges 表包含 calls, calls_override, inherits, contains, uses, references, returns, param_type, field_type, template_use 等边
- WHEN 查询 schema
- THEN `edge_types` 数组中每个条目包含 `prefix`、`direction`（from→to 的语义）、`source_kinds`、`target_kinds`

### Requirement: Schema 工具无参数
系统 SHALL 使 `codeloom_schema` 无需任何参数即可调用，返回硬编码的稳定元数据。

#### Scenario: 无需 repo 或 branch
- GIVEN LLM 在任何阶段
- WHEN 调用 `codeloom_schema`
- THEN 不需要 repo/branch/name 等任何参数
