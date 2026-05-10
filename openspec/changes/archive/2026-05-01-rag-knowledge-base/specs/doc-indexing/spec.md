# Delta for Document Indexing

## ADDED Requirements

### Requirement: 解析 Markdown 文档结构
系统 SHALL 解析 Markdown 文档的标题层级、内部链接和代码块，提取为结构化知识节点。

#### Scenario: 提取文档标题层级
- GIVEN 一个包含多级标题的 Markdown 文档
- WHEN 执行文档索引
- THEN 每个标题创建为独立 doc_node
- AND 节点包含标题文本、层级（1-6）、section_path（标题路径如 "API > 认证"）
- AND 子标题通过层级关系关联

#### Scenario: 文档内链接建立边
- GIVEN Markdown 文档包含 [链接文字](#section-name) 内部链接
- WHEN 执行文档索引
- THEN 创建文档节点之间的链接边

### Requirement: 代码块关联到代码符号
系统 SHALL 识别文档中的代码块引用，并将其与已索引的代码符号关联。

#### Scenario: 文档描述 API 函数
- GIVEN 文档代码块引用函数名 `UserService::authenticate`
- AND 该函数已在代码索引中存在
- WHEN 执行文档索引
- THEN 创建 doc_code_links 链接文档节点和代码符号
- AND link_type 设为 "documents"

### Requirement: 支持纯文本文档
系统 SHALL 索引纯文本文档，将整个文件作为单个文档节点存储。

#### Scenario: 索引 README 纯文本
- GIVEN 一个 README 纯文本文件
- WHEN 执行文档索引
- THEN 创建单个 doc_node，content 为全文内容
- AND title 为文件名

### Requirement: 文档索引可选
系统 SHALL 将文档索引标记为可选功能，缺失文档解析依赖时不影响代码索引。

#### Scenario: 无 Python 环境执行文档索引
- GIVEN 系统未安装文档解析所需的 Python 依赖
- WHEN 用户执行文档索引命令
- THEN 系统给出明确提示 "文档索引需要 Python 依赖，请先安装"
- AND 代码索引功能不受影响
