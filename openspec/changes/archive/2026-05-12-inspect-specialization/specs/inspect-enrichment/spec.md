# inspect-enrichment

## ADDED Requirements

### Requirement: 按节点类型输出专有信息

inspect SHALL 根据节点的 `kind` 和 `node_type` 输出不同类型的架构信息，而非统一的通用 edges 列表。

#### Scenario: 类节点

- GIVEN 用户 inspect 一个 class 或 struct 符号
- WHEN 输出结果
- THEN SHALL 包含 `bases`（基类名列表）、`members`（成员字段）、`methods`（成员方法）和 `template_args`（模板参数，如有）

#### Scenario: 枚举节点

- GIVEN 用户 inspect 一个 enum 符号
- WHEN 输出结果
- THEN SHALL 包含 `values`（枚举值列表）

#### Scenario: 文件节点

- GIVEN 用户 inspect 一个 file 节点
- WHEN 输出结果
- THEN SHALL 包含 `sections`（顶层 section 列表）

#### Scenario: 章节节点

- GIVEN 用户 inspect 一个 section 节点
- WHEN 输出结果
- THEN SHALL 包含 `parent_section`、`child_sections`、`child_chunks`、`prev_section`、`next_section`

#### Scenario: 文档块节点

- GIVEN 用户 inspect 一个 chunk 节点
- WHEN 输出结果
- THEN SHALL 包含 `parent_section`、`prev_chunk`、`next_chunk`

#### Scenario: 通用回退

- GIVEN 节点类型不属于上述任何一种（function/method/namespace/global 等）
- WHEN 输出结果
- THEN SHALL 保持当前通用 edges 分组列表输出
