# Delta for search-name-comment-split

## ADDED Requirements

### Requirement: 符号名与注释分通道搜索
FTS5 搜索 SHALL 将符号名（含 signature）和注释（doc_comment）从单一 `fts5_sym MATCH` 查询拆分为两个独立查询，分别返回名命中和注释命中结果。

#### Scenario: 英文关键词命中符号名
- GIVEN 索引中有函数 `Snappy_Compress` 且注释含 "compression"
- WHEN 用户搜索 "snappy"
- THEN 名命中的 `Snappy_Compress` 分数 SHALL 高于仅注释命中 "compression" 的其他符号

#### Scenario: 中文关键词搜英文代码
- GIVEN 索引中函数 `MaxGrandParentOverlapBytes` 注释含 "compaction" 的中文描述
- WHEN 用户搜索 "压缩"
- THEN 向量名搜索 SHALL 匹配语义相近的英文符号名，而非仅匹配注释中的中文

### Requirement: 名命中权重大于注释命中
搜索结果融合 SHALL 对名命中赋予更高的权重（默认 0.7），对注释命中赋予较低权重（默认 0.3）。

#### Scenario: 名注释同时命中时的排序
- GIVEN 符号 A 名直接命中关键词，符号 B 仅在注释中命中
- WHEN 两符号均有 BM25 和向量结果
- THEN 符号 A 的最终融合分数 SHALL 高于符号 B

### Requirement: 向量索引按名和注释分别构建
向量搜索 SHALL 将符号名单独向量化（不含注释），注释单独向量化，分别存入独立 vec0 虚拟表。

#### Scenario: 名向量与注释向量分表存储
- GIVEN leveldb 项目索引完成
- THEN 数据库中 SHALL 存在 `symbol_name_vec_{repo}` 和 `symbol_comment_vec_{repo}` 两张独立向量表

### Requirement: 去掉子串命中加分
搜索结果融合 SHALL 不再对查询词为结果名称子串的情况给予加分。

#### Scenario: 子串查询无额外加分
- GIVEN 查询词 "port" 是结果名 "port_stdcxx.h" 的子串
- WHEN 执行搜索
- THEN 该结果的分数 SHALL 仅由 BM25 和向量相似度决定，不因子串关系额外加分

## REMOVED Requirements
- 无
