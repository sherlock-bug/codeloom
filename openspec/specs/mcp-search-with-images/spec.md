# mcp-search-with-images

## Purpose
CodeLoom mcp-search-with-images 功能域。本规范描述此功能的需求和行为。

## Purpose
CodeLoom MCP 图片感知搜索功能域——codeloom_search 增强返回 snippet/has_images/image_count/node_type/file_format 字段。

## Requirements

### Requirement: 搜索结果附带图片
系统 SHALL 在 `codeloom_search` 返回 doc 类型结果时，从 SQLite BLOB 读取关联图片并 base64 编码返回。

#### Scenario: 搜索命中文档节点
- GIVEN 搜索 "用户登录流程" 命中了一个 doc 类型节点，该节点在 doc_images 表中有图片 BLOB
- WHEN 返回搜索结果
- THEN 该结果的 JSON SHALL 包含 `images` 数组，其中每项含 `alt_text`、`base64_data`、`width`、`height`

#### Scenario: 搜索命中代码节点
- GIVEN 搜索 "login" 命中了一个 code 类型节点
- WHEN 返回搜索结果
- THEN 该结果的 JSON SHALL 包含 `images: []` 空数组

#### Scenario: 图片数量限制
- GIVEN 一个 doc 节点关联了 10 张图片
- WHEN 搜索返回该节点
- THEN images 数组 SHALL 最多返回前 3 张图片，其余通过 codeloom_get_doc 获取

#### Scenario: 过大的图片跳过
- GIVEN 一张 BLOB 中 WebP 字节超过 200KB 的图片
- WHEN 搜索返回该节点
- THEN 该图片 SHALL 不出现在 images 数组中，标记为 skipped（reason: "size_exceeded"）

#### Scenario: 远程部署可用
- GIVEN CodeLoom MCP 服务部署在远程服务器上
- WHEN LLM 客户端调用 codeloom_search
- THEN 返回的 base64 图片数据 SHALL 可被 LLM 直接解码使用，无需访问服务器文件系统
- AND 所有数据（文本+图片）均来自同一个 SQLite .db 文件
