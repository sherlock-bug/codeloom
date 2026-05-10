# Delta for mcp-get-doc

## ADDED Requirements

### Requirement: codeloom_get_doc MCP 工具
系统 SHALL 提供一个 MCP 工具 `codeloom_get_doc`，根据 doc_id 返回文档节点的完整内容和关联图片（从 SQLite BLOB 读取并 base64 编码）。

#### Scenario: 获取文档节点及其图片
- GIVEN 数据库中 doc_id=42 的文档节点有 2 张关联图片（image_data BLOB）
- WHEN LLM 调用 codeloom_get_doc(doc_id=42)
- THEN 返回结果 SHALL 包含标题、章节路径、级别、完整文本内容、图片列表（每项含 alt_text、base64_data、width、height、section_context）

#### Scenario: 无图片的文档节点
- GIVEN doc_id=5 的文档节点没有关联图片
- WHEN LLM 调用 codeloom_get_doc(doc_id=5)
- THEN 返回结果 SHALL 包含 images: [] 空数组

#### Scenario: 不存在的 doc_id
- GIVEN doc_id=99999 在数据库中不存在
- WHEN LLM 调用 codeloom_get_doc(doc_id=99999)
- THEN 返回结果 SHALL 为错误提示 "Document not found"

#### Scenario: BLOB→base64 编码
- GIVEN doc_images 表中有一条 image_data BLOB（WebP 字节）
- WHEN 构建 MCP 响应
- THEN 系统 SHALL 从 SQLite 读取 BLOB，base64 编码为字符串，嵌入 JSON 响应
