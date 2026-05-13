# comment-indexing

## Purpose
注释索引功能：收集 C++ 源码中的行内注释和体内注释，存入 FTS5 索引供搜索。支持中文/英文注释的混合搜索（BM25 + 向量语义融合）。注释在符号和文件节点中展示为 snippet。

⚠️ **实施状态**：✅ 已实现 (2026-05-12) — 见 `openspec/changes/comment-collection/`

## Requirements

### Requirement: 代码注释收集与存储
系统 SHALL 在 C++ 源码解析阶段收集符号关联的注释文本，并存入 symbols 表的 doc_comment 字段。注释来源包括：符号上方的文档注释（`///`、`/** */`）、符号右侧的行内注释（`//`），以及函数/类实现体内的注释块。

#### Scenario: 上方文档注释收集
- GIVEN leveldb 源码中函数 `DoCompactionWork` 上方有 `/// Performs compaction work` 注释
- WHEN tree-sitter 解析该函数
- THEN doc_comment SHALL 包含 "Performs compaction work"

#### Scenario: 行内注释收集
- GIVEN 代码行 `int max = 100; // maximum connections`
- WHEN 该行属于某个符号的作用域
- THEN 该符号的 doc_comment SHALL 包含 "maximum connections"

#### Scenario: 实现体内注释收集
- GIVEN 函数体内有 `// Step 1: validate input` 和 `/* Step 2: transform data */`
- WHEN 解析该函数
- THEN 这些注释 SHALL 被收集并存入 doc_comment

### Requirement: 声明与实现注释合并
系统 SHALL 在同名符号的声明（.h 文件）和实现（.cc 文件）分别收集注释后，在写入 symbols 表去重时将两边的 doc_comment 合并。合并规则：声明注释在前，实现注释在后，以 `\n` 分隔，去重时通过 ON CONFLICT DO UPDATE 追加。

### Requirement: 注释参与搜索
系统 SHALL 将 doc_comment 内容加入 FTS5 符号索引，使注释可通过关键词搜索命中。

#### Scenario: 注释关键词搜索
- GIVEN 函数名 `Process` 不含 "compaction"，但其注释包含 "Handles compaction"
- WHEN 用户通过 FTS5 搜索 "compaction"
- THEN `Process` SHALL 出现在搜索结果中

### Requirement: 搜索结果包含注释
系统 SHALL 在返回搜索结果时，若符号有 doc_comment，则将注释内容作为 snippet 一并返回。

#### Scenario: 搜索结果含注释 snippet
- GIVEN 符号 `DoCompactionWork` 的 doc_comment 为 "Performs compaction work across all levels in the background"
- WHEN 用户搜索 "compaction" 命中该符号
- THEN CLI 结果显示 `└─ Performs compaction work across all levels in the background`
- AND MCP 返回的 snippet 字段为该完整注释
