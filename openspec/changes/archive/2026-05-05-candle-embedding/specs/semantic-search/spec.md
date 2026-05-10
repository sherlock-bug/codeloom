# Delta for semantic-search

## MODIFIED Requirements

### Requirement: 向量化语义搜索
系统 SHALL 支持对代码符号和文档段落进行自然语言语义搜索，使用 candle + bge-small-zh 生成 384 维向量，通过余弦相似度进行检索。模型不可用时 SHALL 自动降级为 Jaccard token overlap 搜索。

#### Scenario: 自然语言查询代码符号（candle 模式）
- GIVEN 项目已索引且模型已就绪
- WHEN 用户调用 codeloom_semantic_search(query="用户登录相关功能", branch="main", top_k=10)
- THEN 使用 candle 生成查询向量并在符号向量中搜索
- AND 返回 top-10 最相关的代码符号
- AND 每个结果包含符号名、定义摘要、相似度分数
- AND 响应时间小于 500ms

#### Scenario: 自然语言查询文档（candle 模式）
- GIVEN 文档已索引且模型已就绪
- WHEN 用户搜索"轧差的业务规则"
- THEN 返回相关文档段落（doc_nodes）
- AND 同时返回段落关联的代码符号（通过 doc_code_links）

#### Scenario: 模型不可用时自动降级
- GIVEN candle 模型文件不存在
- WHEN 用户执行语义搜索
- THEN 系统使用 TextEmbedder (Jaccard) 完成搜索
- AND 结果中标明 "(Jaccard fallback)"

### Requirement: 本地 Embedding 模型
系统 SHALL 使用 candle 加载 bge-small-zh 模型（safetensors 格式，384 维），纯本地 CPU 推理，零外部 API 依赖。

#### Scenario: 无外部网络依赖
- GIVEN 系统离线运行且模型已下载
- WHEN 执行语义搜索
- THEN 完全通过本地 candle 推理完成
- AND 不产生任何外部 API 调用

## REMOVED Requirements

### Requirement: sqlite-vec 向量存储
**Reason**: candle 方案在应用层计算余弦相似度，逐个比对向量。符号数 < 10 万时 CPU 遍历 < 500ms，不需要 sqlite-vec 扩展。
**Migration**: 移除 sqlite-vec 依赖，移除 `symbol_vec`/`doc_vec` 虚拟表相关代码（如果存在）。
