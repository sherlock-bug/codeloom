# xlsx-parser

## Purpose
CodeLoom xlsx-parser 功能域。本规范描述此功能的需求和行为。

## Purpose
CodeLoom Excel 四层节点解析功能域——sheet/header_cell/row/cell 模型，智能表头检测，关键关系建边其余靠 section_path 隐式导航。

## Requirements

### Requirement: Excel 四层节点模型
系统 SHALL 将 Excel 文件解析为四层 doc_nodes 节点：sheet、header_cell、row、cell，通过 node_type 字段区分。

#### Scenario: Sheet 节点
- GIVEN 一个 .xlsx 文件包含 Sheet "销售数据"
- WHEN 解析该文件
- THEN 系统 SHALL 创建 doc_node: title="销售数据", section_path="销售数据", level=1, node_type="sheet", file_format="xlsx"

#### Scenario: 表头单元格节点
- GIVEN Sheet "销售数据" 的表头行包含 "城市","销售额","日期"
- WHEN 解析表头行
- THEN 系统 SHALL 为每个列创建 doc_node: title="城市", section_path="销售数据/_header/城市", level=2, node_type="header_cell", content="城市"

#### Scenario: 行节点
- GIVEN Sheet "销售数据" 有 500 行数据
- WHEN 解析数据行
- THEN 系统 SHALL 为每行创建 doc_node: title="Row1", section_path="销售数据/Row1", level=3, node_type="row"

#### Scenario: 普通单元格节点
- GIVEN Row1 的第 2 列值为 "1000"
- WHEN 解析该单元格
- THEN 系统 SHALL 创建 doc_node: title="销售额", section_path="销售数据/Row1/销售额", level=4, node_type="cell", content="1000"

### Requirement: 智能表头检测
系统 SHALL 自动检测表头行，跳过前导空行，以文本占比最高的行作为表头。

#### Scenario: 文本表头识别
- GIVEN Sheet 前 3 行为空，第 4 行是 "姓名 | 年龄 | 部门"
- WHEN 检测表头
- THEN 系统 SHALL 跳过空行，将第 4 行识别为表头

#### Scenario: 无表头降级
- GIVEN Sheet 所有行为纯数值
- WHEN 检测表头
- THEN 系统 SHALL 使用 "列A, 列B, 列C..." 作为默认列名

### Requirement: 图关系（edges）
系统 SHALL 在图（edges 表）中建立 Excel 节点间的关键关系。

#### Scenario: sheet→header_cell 边
- GIVEN Sheet 节点和 3 个 header_cell 节点
- WHEN 建立图关系
- THEN 系统 SHALL 创建 3 条 edge: source=sheet.id, target=header_cell.id, edge_type="has_column"

#### Scenario: sheet→row 边
- GIVEN Sheet 节点和 500 个 row 节点
- WHEN 建立图关系
- THEN 系统 SHALL 创建 500 条 edge: source=sheet.id, target=row.id, edge_type="has_row"

#### Scenario: row↔cell 不建边
- GIVEN Row1 节点下的 3 个 cell 节点
- WHEN 建立图关系
- THEN 系统 SHALL NOT 创建 row→cell 的 edges 边，该关系通过 section_path 前缀隐式表达

#### Scenario: header_cell→cell 不建边
- GIVEN "销售额" header_cell 对应的所有 cell 值
- WHEN 建立图关系
- THEN 系统 SHALL NOT 创建 header_cell→cell 的 edges 边，该关系通过 section_path 中的列名隐式表达

### Requirement: 搜索命中与导航
系统 SHALL 通过 FTS5 + vec0 搜索命中任意 Excel 节点，并在结果中返回 node_type 和 section_path 供 LLM 导航。

#### Scenario: 搜索命中 cell
- GIVEN LLM 搜索 "1000"
- WHEN FTS5 命中 city="1000" 的 cell 节点
- THEN 返回结果 SHALL 包含 node_type="cell", section_path="销售数据/Row1/销售额", snippet="1000"

#### Scenario: 搜索命中 header_cell
- GIVEN LLM 搜索 "销售额"
- WHEN FTS5 命中 header_cell 节点
- THEN 返回结果 SHALL 包含 node_type="header_cell", section_path="销售数据/_header/销售额"

#### Scenario: LLM 通过 section_path 导航
- GIVEN 搜索返回 cell 的 section_path="销售数据/Row1/销售额"
- WHEN LLM 需要该行所有列的值
- THEN LLM SHALL 通过 section_path 前缀 "销售数据/Row1/" 查询同行的所有 cell
