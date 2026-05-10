# Delta for pdf-parser

## ADDED Requirements

### Requirement: PDF 文本提取
系统 SHALL 支持解析 PDF 文件，提取文本内容存储为文档节点。

#### Scenario: 解析纯文本 PDF
- GIVEN 一个包含结构化文本的 PDF 文件（< 10MB）
- WHEN 调用 index_docs 进行文档索引
- THEN 系统 SHALL 提取文本内容，存储为单个 doc_node，file_format='pdf'

#### Scenario: 超大 PDF 文件
- GIVEN 一个超过 10MB 的 PDF 文件
- WHEN 尝试索引该文件
- THEN 系统 SHALL 跳过该文件并输出警告信息

#### Scenario: 扫描版 PDF
- GIVEN 一个纯图片扫描的 PDF（无可提取文本）
- WHEN 解析该文件
- THEN 系统 SHALL 静默跳过，不产生空的 doc_node
