# Delta for excel-query

## ADDED Requirements

### Requirement: codeloom_query_excel MCP 工具
系统 SHALL 提供 `codeloom_query_excel` MCP 工具，基于节点模型查询 Excel 数据。LLM 可从搜索命中的任意节点（cell/row/header_cell/sheet）进入查询。

#### Scenario: 从 cell 节点查询整行
- GIVEN LLM 搜索命中 cell section_path="销售数据/Row5/销售额"
- WHEN 调用 codeloom_query_excel(doc_id=cell_id, mode="row")
- THEN 返回 Row5 的所有 cell: ["城市: 上海", "销售额: 2000", "日期: 2024-02"]

#### Scenario: 从 row 节点展开
- GIVEN LLM 搜索命中 row 节点 section_path="销售数据/Row5"
- WHEN 调用 codeloom_query_excel(doc_id=row_id, mode="row")
- THEN 同 Scenario 1，返回该行所有 cell

#### Scenario: 从 header_cell 查询该列所有值
- GIVEN LLM 搜索命中 header_cell section_path="销售数据/_header/销售额"
- WHEN 调用 codeloom_query_excel(doc_id=header_id, mode="column", limit=50)
- THEN 返回 "销售额" 列的前 50 个值: [1000, 2000, 1500, ...]

#### Scenario: 从 sheet 节点按列过滤
- GIVEN LLM 搜索命中 sheet 节点 "销售数据"
- WHEN 调用 codeloom_query_excel(doc_id=sheet_id, filter="销售额 > 1500", limit=20)
- THEN 返回表头行 + 所有销售额>1500 的数据行

#### Scenario: 模糊搜索
- GIVEN LLM 不确定数据在哪列
- WHEN 调用 codeloom_query_excel(doc_id=sheet_id, search="张三")
- THEN 返回表头行 + 所有任意列含 "张三" 的数据行

### Requirement: 参数设计
`codeloom_query_excel` SHALL 支持 mode="row"|"column"|"filter" 三种查询模式。

#### Scenario: mode 自动推断
- GIVEN LLM 未指定 mode 参数
- WHEN 调用 codeloom_query_excel
- THEN 系统 SHALL 根据 doc_id 对应节点的 node_type 自动推断: cell→row, row→row, header_cell→column, sheet→filter
