# Delta for cli-search-output

## ADDED Requirements

### Requirement: CLI 搜索输出三列精简格式
系统 SHALL 在 CLI 搜索命令的输出中仅显示三列：名称、类型、注释（文档节点注释=内容摘要）。

#### Scenario: 符号搜索 CLI 输出
- GIVEN 执行 `codeloom search --query "Compact" --repo leveldb`
- WHEN 输出搜索结果
- THEN 每行 SHALL 格式为 "[类型] 名称 @ 注释"
- AND 类型列 SHALL 左对齐 16 字符
- AND 名称列 SHALL 左对齐 40 字符

#### Scenario: 文档搜索 CLI 输出
- GIVEN 执行搜索命中文档节点
- WHEN 输出搜索结果
- THEN 类型列 SHALL 显示 "doc"
- AND 注释列 SHALL 显示文档内容的摘要（前 120 字符）

#### Scenario: 无结果时 CLI 输出
- GIVEN 搜索无匹配
- WHEN CLI 输出
- THEN 显示 "(no results)"
