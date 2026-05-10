# Proposal: 多格式文档解析 + 图片关联支持

## Why
CodeLoom 当前仅支持 `.md` 和 `.rst` 格式文档，但实际项目中大量知识存储在 Excel 表格、Word 文档、PDF 规格说明书、XML 配置文件中。同时文档中的图片（架构图、流程图、截图）是重要的知识载体，当前完全被忽略。需要扩展文档解析管线，让 LLM Agent 能通过 MCP 搜索到这些格式中的内容及关联图片。

## What Changes
- **新增 5 种文档格式解析器**：xlsx、docx、PDF、XML、HTML，统一路由分发
- **Excel 行级索引**：智能检测表头，每行存为独立 doc_node（列名: 值），提供 SQL-like 查询接口
- **DOCX 智能拆分**：按标题分段 + 大段自动拆分（>2000 字符）
- **图片提取与存储**：从所有格式中提取图片引用/嵌入图片，小图原样、大图 WebP 压缩，全存 SQLite BLOB
- **新增 2 个 MCP 工具**：`codeloom_get_doc`（完整内容+base64 图片）、`codeloom_query_excel`（SQL-like 查询）
- **搜索增强**：`codeloom_search` 返回 snippet + has_images 提示

## Capabilities

### New Capabilities
- `xlsx-parser`: Excel 解析，智能表头检测，行级索引（level=5 不参与 FTS5）
- `excel-query`: 新增 MCP 工具，filter/search/order_by 像 SQL 一样查 Excel
- `docx-parser`: Word 解析，标题分段 + 大段自动拆分 + 嵌入图片提取
- `pdf-parser`: PDF 文本提取
- `xml-html-parser`: XML/HTML 解析，按元素层级提取 + img 标签
- `image-extraction`: 从所有格式提取图片，记录文档位置和文字上下文
- `image-compression`: 大图 WebP 压缩（>100KB 或 >800px），小图原样保留
- `mcp-get-doc`: 返回文档节点完整内容 + base64 图片
- `mcp-search-with-images`: search 返回 snippet + has_images，不直接返回 base64
- `doc-format-routing`: 统一格式路由分发

## Impact
- **依赖新增**: `calamine`, `zip`, `quick-xml`, `pdf-extract`, `image`
- **二进制增量**: ~2MB（当前 15MB）
- **DB**: doc_images 表（BLOB 存图），全在 SQLite 一个文件
- **MCP 工具数**: 9 → 11（新增 get_doc + query_excel）
- **向后兼容**: 现有 md/rst 路径完全不变
