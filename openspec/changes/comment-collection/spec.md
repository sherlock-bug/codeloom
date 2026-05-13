# spec: comment-collection

## Purpose

为 CodeLoom 索引的 C++ 符号和文件收集源码中的注释文本（文档注释、行内注释、函数体内注释），存入 `nodes.content` 字段（FTS5 content），使注释可被 BM25 关键词搜索命中，并在搜索结果中展示为 snippet。注释不参与向量化（FTS5 足够覆盖中文注释搜索需求）。

## Requirements

### Requirement: 注释收集范围

系统 SHALL 为每个 C++ 符号收集以下三类注释，按上下文关联到对应符号：

1. **上方文档注释** — 符号声明/定义上方的连续注释块（`///`、`/** */`、`//`）
2. **函数体内注释** — 符号为函数/方法时，其函数体内的所有单行注释（`//`）和多行注释（`/* */`），包含嵌套作用域中的注释
3. **行内注释** — 符号所在行右侧的尾随注释（`// comment`）

#### Scenario: 上方文档注释收集

- GIVEN 头文件 `db.h` 中函数声明上方有 `/// Open a database.\n/// Returns OK on success.`
- WHEN 解析该符号
- THEN `doc_comment` SHALL 包含 `"Open a database. Returns OK on success."`
- AND prefix `///` 和 `\n` 前的缩进空格 SHALL 被清理

#### Scenario: 函数体内注释收集

- GIVEN 函数 `Compaction::Apply` 实现体包含 `// Step 1: validate inputs` 和 `/* Range check */`
- WHEN 解析该函数
- THEN `doc_comment` SHALL 包含 `"Step 1: validate inputs\nRange check"`
- AND 体内注释 SHALL 按在源码中的出现顺序排列

#### Scenario: 行内注释收集

- GIVEN 代码行 `int max_level = 7;  // Maximum number of levels`
- WHEN 该行的符号（如变量 `max_level`）被解析
- THEN 符号的 `doc_comment` SHALL 包含 `"Maximum number of levels"`

#### Scenario: 仅有上方注释

- GIVEN 函数声明上方有 `///` 注释，但函数体内无注释
- WHEN 解析该符号
- THEN `doc_comment` SHALL 仅包含上方注释文本
- AND 不得包含空行分割的无关注释块

#### Scenario: 仅体内有注释

- GIVEN 函数无上方注释，但实现体内部有注释
- WHEN 解析该符号
- THEN `doc_comment` SHALL 包含函数体内的注释

#### Scenario: 无任何注释

- GIVEN 函数/变量既无上方注释、又无体内注释、也无行内注释
- WHEN 解析该符号
- THEN `doc_comment` SHALL 为空字符串

### Requirement: 文件头注释收集

系统 SHALL 为每个索引的文件节点收集文件头部（第一个非注释非空白行之前）的连续注释块，作为文件的文档注释。文件头注释包含版权声明、文件说明、许可证信息等。

#### Scenario: 文件头 License 注释

- GIVEN leveldb 源文件以 BSD License 注释块开头（`// Copyright (c) 2011 The LevelDB Authors...`）
- WHEN 文件被索引为 file node
- THEN 该文件节点的 `doc_comment` SHALL 包含 License 注释文本

#### Scenario: 文件头后跟代码

- GIVEN 文件开头注释块 `// header.h — 配置定义` 后紧接 `#pragma once`
- WHEN 文件被索引
- THEN `doc_comment` SHALL 包含 `"header.h — 配置定义"`
- AND `#pragma once` 不进入注释

#### Scenario: 无文件头注释

- GIVEN 文件第一行就是代码（无注释）
- WHEN 文件被索引
- THEN 文件节点的 `doc_comment` SHALL 为空

### Requirement: 注释格式归一化

系统 SHALL 对收集的注释行去除 `///`、`//`、`/**`、`*/`、`*` 等前缀字符，清理首尾空白，行间以 `\n` 连接。

#### Scenario: `///` 前缀剥离

- GIVEN 注释 `/// Open the file`
- WHEN 提取注释文本
- THEN 输出 SHALL 为 `"Open the file"`

#### Scenario: 多行 `/** */` 注释

- GIVEN 注释块：
  ```
  /**
   * Performs compaction.
   * May block for I/O.
   */
  ```
- WHEN 提取注释文本
- THEN 输出 SHALL 为 `"Performs compaction.\nMay block for I/O."`

#### Scenario: 单行 `/* */` 注释

- GIVEN 注释 `/* immediate flush */`
- WHEN 提取
- THEN 输出 SHALL 为 `"immediate flush"`

### Requirement: 注释写入 DB

系统 SHALL 将收集到的 doc_comment 通过 Symbol 结构的 `doc_comment` 字段存入 `nodes.content`（格式为 `kind doc_comment`），利用已有的 FTS5 索引和搜索通道。

#### Scenario: 注释入库

- GIVEN 符号 kind="method", doc_comment="Opens a database connection"
- WHEN 执行 `fill_all_fts`
- THEN FTS5 索引中该符号的 content SHALL 为 `"method Opens a database connection"`

#### Scenario: 搜索命中注释

- GIVEN 符号 `ProcessWrite` 的 doc_comment 为 "处理写入请求并压缩"
- WHEN 用户搜索"压缩"（FTS5 content channel）
- THEN `ProcessWrite` SHALL 出现在搜索结果中

#### Scenario: snippet 展示注释

- GIVEN 符号出现在搜索结果中且 doc_comment 非空
- WHEN 展示结果
- THEN CLI 显示 `└─ 处理写入请求并压缩`
- AND MCP 返回的 snippet 字段为该注释文本

### Requirement: 注释不参与向量化

系统 SHALL 不移除已有的 comment 向量通道，而是维持现状——向量搜索只使用符号名通道（`symbol_name_vec`）。注释搜索仅通过 FTS5 content 通道进行，不产生 KNN 查询。

理由：注释文本简短（通常 ≤200 字），中英文混合，FTS5 BM25 足以匹配；向量化增加索引时间和存储开销，性价比低。

#### Scenario: 注释向量通道不变

- GIVEN 索引完成
- WHEN 检查向量表
- THEN 数据库中 SHALL **不**存在 `symbol_comment_vec_*` 表（即维持现状：0 comment vectors）

### Requirement: 向后兼容

系统 SHALL 兼容现有数据库：升级后首次 `clean + reindex` 之前，旧数据中的符号 doc_comment 为空，搜索行为与升级前一致。
