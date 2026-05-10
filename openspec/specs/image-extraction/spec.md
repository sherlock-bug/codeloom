# image-extraction

## Purpose
CodeLoom image-extraction 功能域。本规范描述此功能的需求和行为。

## Purpose
CodeLoom 图片提取功能域——从 Markdown/HTML/DOCX/XLSX 中提取图片，以 BLOB 存入 SQLite，MCP 返回 base64 内联。

## Requirements

### Requirement: Markdown 图片引用提取
系统 SHALL 从 Markdown 文档中提取 `![alt](url)` 格式的图片引用。

#### Scenario: 提取本地相对路径图片
- GIVEN 一个 Markdown 文件包含 `![架构图](images/arch.png)`
- WHEN 解析该文件
- THEN 系统 SHALL 记录该图片引用，alt_text='架构图'，original_src='images/arch.png'，section_context 为图片前后各 100 字符文本

#### Scenario: 提取远程 URL 图片
- GIVEN 一个 Markdown 文件包含 `![logo](https://example.com/logo.png)`
- WHEN 解析该文件
- THEN 系统 SHALL 记录该图片引用但不下载，image_type='linked'

#### Scenario: 提取 HTML img 标签
- GIVEN 一个 HTML 文件包含 `<img src="photo.jpg" alt="照片">`
- WHEN 解析该文件
- THEN 系统 SHALL 提取 src 和 alt 属性，行为与 Markdown 图片一致

### Requirement: 嵌入图片提取
系统 SHALL 从 xlsx 和 docx 文件中提取内嵌图片。

#### Scenario: 提取 docx 嵌入图片
- GIVEN 一个 .docx 文件的 word/media/ 目录下有 image1.png
- WHEN 解析该 docx 文件
- THEN 系统 SHALL 提取该图片，计算其出现在文档中的大致位置（前后段落 text_context）

#### Scenario: 提取 xlsx 嵌入图片
- GIVEN 一个 .xlsx 文件的 xl/media/ 目录下有 chart.png
- WHEN 解析该 xlsx 文件
- THEN 系统 SHALL 提取该图片并按所在 Sheet 记录位置

### Requirement: 图片位置记录
系统 SHALL 为每张图片记录其在文档中的近似位置信息。

#### Scenario: 记录图片顺序和上下文
- GIVEN 文档中第 3 段和第 4 段之间有一张图片
- WHEN 提取该图片
- THEN position 字段 SHALL 标记图片在文档中的序号，section_context 包含前后文字
