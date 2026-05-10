# fts5-enhanced-index

## Purpose
CodeLoom fts5-enhanced-index 功能域。本规范描述此功能的需求和行为。

## Purpose
FTS5 索引架构优化：去 definition 列（数据库缩小、向量更聚焦），HTML 文件跳过索引，智能批次向量化（双限制条数+字符数），智谱 embedding-2 API 集成替代本地 candle。

## Requirements

### Requirement: 去 definition 列的搜索架构
系统 SHALL 从 FTS5 符号索引和向量嵌入中移除 definition 列，FTS5 仅索引 name + file_path + kind + signature，向量嵌入仅使用 name + kind。文档搜索保留正文全文嵌入。

#### Scenario: FTS5 不含 definition
- GIVEN leveldb 源码中函数 body 包含 "sstable" 一词但函数名不含
- WHEN 用户通过 FTS5 搜索 "sstable"
- THEN 该函数 SHALL NOT 出现在 FTS5 结果中（无法搜索函数体内容）

#### Scenario: 向量搜索跨语言语义匹配
- GIVEN 用户用中文搜索 "压缩"
- WHEN 向量搜索执行
- THEN 系统 SHALL 返回 Compaction/Compression 相关符号和文档，跨语言语义匹配生效

### Requirement: kind 过滤支持
系统 SHALL 在 FTS5 搜索中支持按符号类型（kind）过滤，只返回指定类型的符号。

#### Scenario: 按 function 过滤
- GIVEN 用户搜索 "my" 且指定 kind="function"
- WHEN FTS5 执行搜索
- THEN 结果中 SHALL 仅包含 kind="function" 的符号，排除 enum_value 等其他类型

### Requirement: HTML 文件跳过索引
系统 SHALL 在文档索引阶段跳过所有 `*.html` 和 `*.htm` 文件，不将其内容存入 doc_nodes 表。

#### Scenario: benchmark.html 不索引
- GIVEN leveldb/doc/benchmark.html 存在
- WHEN 执行 codeloom index
- THEN benchmark.html SHALL 不出现在 doc_nodes 中

### Requirement: 智能批次向量化
系统 SHALL 在向量化时使用智能分批策略：同时限制单批条数（≤batch_size，默认 64）和字符数（≤max_chars_per_batch，默认 90000），两者任一先达到上限即发送请求。两个参数均可通过 config.yaml 配置。

#### Scenario: 短文本合并批次
- GIVEN 符号名很短导致累计字符数 < 90000
- WHEN 累计条数达到 64
- THEN 系统 SHALL 立即发送批次，不等字符数达到上限

#### Scenario: 长文本提前触发
- GIVEN 某文档正文极长导致累计字符数在条数不到 64 时已超 90000
- WHEN 累计字符数达到 90000
- THEN 系统 SHALL 立即发送批次，不等条数达到上限

### Requirement: 智谱 embedding API 集成
系统 SHALL 使用智谱 embedding-2 模型（1024 维）通过 OpenAI 兼容 API 接口进行文本向量化，替代本地 candle 嵌入。API 地址和密钥通过 config.yaml 的 EmbeddingConfig 配置。

#### Scenario: 批量嵌入
- GIVEN 需要向量化 64 条文本
- WHEN 调用 ApiEmbedder.batch_embed()
- THEN 系统 SHALL 发送单次 HTTP POST 请求，返回 64 个 1024 维向量
