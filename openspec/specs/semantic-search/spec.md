# semantic-search

## Purpose
CodeLoom semantic-search 功能域。本规范描述此功能的需求和行为。

## Purpose
使用本地 bge-small-zh ONNX embedding 模型实现自然语言向量搜索，混合返回代码符号和文档段落的查询结果。

## Requirements

### Requirement: 向量化语义搜索
系统 SHALL 支持对代码符号和文档段落进行自然语言语义搜索，使用本地 ONNX embedding 模型生成向量，通过 sqlite-vec 扩展进行相似度检索。

#### Scenario: 自然语言查询代码符号（candle 模式）
- GIVEN 项目已索引且模型已就绪
- WHEN 用户调用 codeloom_semantic_search(query="用户登录相关功能", branch="main", top_k=10)
- THEN 使用 candle 生成查询向量并在符号向量中搜索
- AND 返回 top-10 最相关的代码符号
- AND 响应时间小于 500ms

#### Scenario: 模型不可用时自动降级
- GIVEN candle 模型文件不存在
- WHEN 用户执行语义搜索
- THEN 系统使用 TextEmbedder (Jaccard) 完成搜索
- AND 结果中标明 "(Jaccard fallback)"

#### Scenario: 自然语言查询文档
- GIVEN 文档已索引且文档向量已生成
- WHEN 用户搜索"轧差的业务规则"
- THEN 返回相关文档段落（doc_nodes）
- AND 同时返回段落关联的代码符号（通过 doc_code_links）

### Requirement: 本地 Embedding 模型
系统 SHALL 内置 bge-small-zh ONNX 模型（约 96MB），用于生成文本 embedding 向量（384 维 float32），无需调用外部 API。

#### Scenario: 索引时自动生成向量
- GIVEN 索引完成后
- WHEN 系统处理符号和文档
- THEN 自动对每个符号签名和文档段落调用 embedding 模型
- AND 向量存入 sqlite-vec 虚拟表
- AND 索引总耗时因 embedding 增加不超过 30%

### Requirement: 混合搜索结果
系统 SHALL 支持在一次搜索中同时返回代码符号和文档段落，按相似度混合排名。

#### Scenario: 混合搜索
- GIVEN 代码和文档均已索引并向量化
- WHEN 搜索"结算金额计算"
- THEN 返回结果包含：结算相关文档段落（如 docs/业务/结算规则.md §2）和代码符号（如 SettlementEngine::calculate）
- AND 每个结果标注类型（doc 或 code）
- AND 文档结果附带关联的代码符号，代码结果附带关联的文档段落
