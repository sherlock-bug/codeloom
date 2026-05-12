# search-enrichment

## ADDED Requirements

### Requirement: 搜索结果按类型增强

搜索工具 SHALL 根据结果节点的类型附加额外字段，帮助 LLM 直接看到节点内容而不需要额外 inspect。所有列表字段使用逗号分隔字符串，不超过 15 项。

#### Scenario: 类结果展示

- GIVEN BM25/向量搜索返回一个 class 或 struct 符号结果
- WHEN 格式化输出
- THEN 结果 JSON SHALL 包含 `members`（成员字段名，逗号分隔）和 `methods`（方法名，逗号分隔）

#### Scenario: 枚举结果展示

- GIVEN 搜索返回一个 enum 符号结果
- WHEN 格式化输出
- THEN 结果 JSON SHALL 包含 `values`（枚举值名，逗号分隔）

#### Scenario: 函数/方法结果展示

- GIVEN 搜索返回 function 或 method 符号结果
- WHEN 格式化输出
- THEN 结果 JSON SHALL 包含 `parent_class`（所属类名，若为成员方法）

#### Scenario: 章节结果展示

- GIVEN 搜索返回一个 section 节点结果
- WHEN 格式化输出
- THEN 结果 JSON SHALL 包含 `prev_section`（前一章节标题）和 `next_section`（后一章节标题）

#### Scenario: 文档块结果展示

- GIVEN 搜索返回一个 chunk 节点结果
- WHEN 格式化输出
- THEN 结果 JSON SHALL 包含 `parent_section`（所属章节名）、`prev_chunk`（前一块标题）、`next_chunk`（后一块标题）

#### Scenario: 文件结果展示

- GIVEN 搜索返回一个 file 节点结果
- WHEN 格式化输出
- THEN 结果 JSON SHALL 包含 `sections`（顶层 section 标题列表，逗号分隔）

#### Scenario: 空字段省略

- GIVEN 某个类型没有对应的 enrichment 数据
- WHEN 格式化输出
- THEN 对应字段 SHALL 在 JSON 中省略，不输出空值

### Requirement: 三种搜索工具一致增强

BM25 搜索、向量语义搜索两个工具 SHALL 对搜索结果应用相同的类型增强规则。`list_symbols` 是纯文本输出，不适用。

#### Scenario: 跨工具一致性

- GIVEN 同一类符号通过 BM25 搜索和向量搜索返回
- WHEN 两个结果的 JSON 输出
- THEN 增强字段名称和行为 SHALL 完全一致
