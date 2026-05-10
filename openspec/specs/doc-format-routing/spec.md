# doc-format-routing

## Purpose
CodeLoom doc-format-routing 功能域。本规范描述此功能的需求和行为。

## Purpose
CodeLoom 文档格式路由功能域——根据文件扩展名自动分发到对应解析器，支持 md/rst/xlsx/xls/docx/pdf/xml/html 格式的统一索引入口。

## Requirements

### Requirement: 文档格式路由分发
系统 SHALL 在 `index_docs` 中按文件扩展名将文档分发到对应的解析器。

#### Scenario: 路由已知格式
- GIVEN 遍历文档目录时遇到 .xlsx、.docx、.pdf、.xml、.html 等文件
- WHEN 调用 index_docs
- THEN 系统 SHALL 根据扩展名调用对应的解析器（calamine/zip+xml/pdf-extract/quick-xml）

#### Scenario: 兼容已有 md/rst
- GIVEN 遍历文档目录时遇到 .md 和 .rst 文件
- WHEN 调用 index_docs
- THEN 系统 SHALL 仍然调用 index_markdown，行为与变更前完全一致

#### Scenario: 未知格式跳过
- GIVEN 遍历文档目录时遇到 .doc（旧Word）或 .odt 格式文件
- WHEN 调用 index_docs
- THEN 系统 SHALL 跳过该文件并输出一行警告

#### Scenario: 所有解析器输出统一 IR
- GIVEN 任意格式的文档被解析完成
- WHEN 输出解析结果
- THEN 结果 SHALL 为统一的 `DocSection` 结构（title, content, level, images），由调用方统一写入 doc_nodes 和 doc_images
