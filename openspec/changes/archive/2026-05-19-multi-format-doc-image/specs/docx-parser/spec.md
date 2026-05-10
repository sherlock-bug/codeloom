# Delta for docx-parser

## ADDED Requirements

### Requirement: Word 文档解析
系统 SHALL 支持解析 .docx 格式的 Word 文档，提取段落文本和嵌入图片。

#### Scenario: 解析 docx 段落文本
- GIVEN 一个包含多个段落的 .docx 文件
- WHEN 调用 index_docs 进行文档索引
- THEN 所有段落文本 SHALL 按顺序拼接为 doc_node 的 content 字段

#### Scenario: 提取 docx 嵌入图片
- GIVEN 一个 .docx 文件包含嵌入在段落间的图片
- WHEN 解析该文件
- THEN 系统 SHALL 提取嵌入图片，压缩为 WebP BLOB，记录在 doc_images 表中

#### Scenario: 按标题分段
- GIVEN 一个 .docx 文件使用 Word 内置标题样式（Heading 1/2/3）
- WHEN 解析该文件
- THEN 系统 SHALL 按标题级别拆分为多个 doc_node section

### Requirement: 大段落自动拆分
系统 SHALL 对超过阈值的段落自动按自然段边界拆分，避免单个 doc_node 过大。

#### Scenario: 超长段落拆分
- GIVEN 一个 docx section 的内容超过 2000 字符
- WHEN 索引该 section
- THEN 系统 SHALL 在自然段边界（空行/换段）处拆分，每段不超过 2000 字符，section_path 附加序号（如 "第一章-1"、"第一章-2"）

#### Scenario: 小段落不拆分
- GIVEN 一个 docx section 的内容只有 500 字符
- WHEN 索引该 section
- THEN 系统 SHALL 保持为一个 doc_node，不拆分

#### Scenario: 无标题文档按段落自动分段
- GIVEN 一个 docx 文件没有任何标题样式
- WHEN 索引该文件
- THEN 系统 SHALL 每 10 个自然段或每 2000 字符自动创建 section，section_path 为 "第1部分"、"第2部分"...
