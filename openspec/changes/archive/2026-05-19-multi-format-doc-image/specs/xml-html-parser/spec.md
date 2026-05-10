# Delta for xml-html-parser

## ADDED Requirements

### Requirement: XML 解析
系统 SHALL 支持解析 .xml 文件，按元素层次提取结构化文本内容。

#### Scenario: 解析嵌套 XML
- GIVEN 一个包含多层嵌套元素的 XML 文件
- WHEN 调用 index_docs 进行文档索引
- THEN 系统 SHALL 将每个顶层元素作为 doc_node section，子元素文本递归拼接为 content

#### Scenario: 解析 HTML 文件
- GIVEN 一个 HTML 文档
- WHEN 调用 index_docs 进行文档索引
- THEN 系统 SHALL 提取 body 内的可见文本，img 标签作为图片引用存入 doc_images

#### Scenario: XML 元素属性保留
- GIVEN 一个 XML 元素带有 name/id 等属性
- WHEN 创建 doc_node section
- THEN section_path SHALL 使用元素的层级路径（如 `/config/database/connection`）
