# Delta for mcp-full-attribute-json

## ADDED Requirements

### Requirement: MCP 搜索返回完整 JSON 属性
系统 SHALL 在 MCP 搜索工具返回的 JSON 中包含节点的所有属性，并按节点类型静态裁剪不可能存在的字段。

#### Scenario: 符号节点返回完整属性
- GIVEN 搜索命中一个函数符号
- WHEN MCP 返回该结果
- THEN JSON SHALL 包含 name、kind、file_path、line_start、line_end、signature、doc_comment、namespace、parent_class、language
- AND 不包含 title、section_path、content 等文档专属字段

#### Scenario: 文档节点返回完整属性
- GIVEN 搜索命中一个文档节点
- WHEN MCP 返回该结果
- THEN JSON SHALL 包含 title、content、file_path、file_format、section_path、level、node_type
- AND 不包含 kind、signature、namespace 等符号专属字段

#### Scenario: 文件节点返回完整属性
- GIVEN 搜索命中一个文件节点
- WHEN MCP 返回该结果
- THEN JSON SHALL 包含 file_path、summary、repo
- AND 不包含 kind、signature、doc_comment 等符号专属字段

### Requirement: 静态字段裁剪规则
系统 SHALL 根据 hit_type 字段（code/doc/file）静态决定输出的属性集，不依赖运行时字段是否为空。

#### Scenario: 符号节点跳过文档字段
- GIVEN hit_type="code"
- WHEN 构建 JSON 输出
- THEN title、section_path、content、level、parent_id、file_format 字段 SHALL 被省略
