# doc-indexing

## Purpose
CodeLoom doc-indexing 功能域。本规范描述此功能的需求和行为。

## Purpose
解析 Markdown、reST 和纯文本文档，提取标题层级结构、内部链接和代码块，构建文档知识图谱并与代码符号交叉链接。

## Requirements

### Requirement: 解析 Markdown 文档结构
系统 SHALL 解析 Markdown 文档的标题层级、内部链接和代码块，提取为结构化知识节点。文档节点 SHALL 默认不绑定分支（branch_name=NULL），使其对所有分支可见。如果索引时传入了 branch 参数，文档节点 SHALL 记录该分支信息。

#### Scenario: 提取文档标题层级（不绑定分支）
- GIVEN 一个包含多级标题的 Markdown 文档
- WHEN 执行 `codeloom index .`（未指定 --branch）
- THEN 每个标题创建为独立 doc_node
- AND doc_node 的 branch_name 为 NULL

#### Scenario: 代码块关联到代码符号
- GIVEN 文档代码块引用函数名 `UserService::authenticate`
- AND 该函数已在代码索引中存在
- WHEN 执行文档索引
- THEN 创建 doc_code_links 链接文档节点和代码符号
- AND link_type 设为 "documents"
- AND 文档节点 branch_name 为 NULL，但代码符号的 branch 过滤不影响文档节点的可见性

#### Scenario: 文档在所有分支可见
- GIVEN doc_nodes 表中文档 D 的 branch_name 为 NULL
- WHEN 在任意分支查询文档
- THEN 文档 D 出现在查询结果中

### Requirement: 代码块关联到代码符号
系统 SHALL 识别文档中的代码块引用，并将其与已索引的代码符号关联。

#### Scenario: 文档描述 API 函数
- GIVEN 文档代码块引用函数名 `UserService::authenticate`
- AND 该函数已在代码索引中存在
- WHEN 执行文档索引
- THEN 创建 doc_code_links 链接文档节点和代码符号
- AND link_type 设为 "documents"

### Requirement: 支持纯文本文档
系统 SHALL 索引纯文本文档，将整个文件作为单个文档节点存储。纯文本文档节点 SHALL 默认 branch_name=NULL。

#### Scenario: 索引纯文本文件（不绑定分支）
- GIVEN 一个 README 纯文本文件
- WHEN 执行文档索引（未指定 --branch）
- THEN 创建单个 doc_node，branch_name 为 NULL
- AND title 为文件名
- AND content 为全文内容
- AND 该文档在所有分支的查询中均可见

### Requirement: 文档索引可选
系统 SHALL 将文档索引标记为可选功能，缺失文档解析依赖时不影响代码索引。

#### Scenario: 无 Python 环境执行文档索引
- GIVEN 系统未安装文档解析所需的 Python 依赖
- WHEN 用户执行文档索引命令
- THEN 系统给出明确提示 "文档索引需要 Python 依赖，请先安装"
- AND 代码索引功能不受影响
