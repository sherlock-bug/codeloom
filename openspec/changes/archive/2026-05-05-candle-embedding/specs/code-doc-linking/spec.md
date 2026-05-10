# Delta for code-doc-linking

## MODIFIED Requirements

### Requirement: Embedding 驱动的自动关联
系统 SHALL 在索引完成后，使用 candle + bge-small-zh 计算文档段落向量与代码符号向量之间的余弦相似度，对相似度超过阈值（默认 0.75）的配对自动建立 doc_code_links 关联边。模型不可用时 SHALL 降级为 Jaccard token overlap 关联。

#### Scenario: 文档描述业务术语，代码命名不同（candle 模式）
- GIVEN 文档 docs/术语/轧差.md 包含"轧差是指多方交易中相互抵消计算净额的过程"
- AND 代码中存在 NettingService::calculate 和 TransactionManager::reverse
- AND candle 模型已就绪
- WHEN 索引完成并计算相似度
- THEN 自动建立轧差.md 段落与 NettingService::calculate 的关联
- AND source 字段设为 'embedding'

#### Scenario: 模型不可用时降级为 Jaccard
- GIVEN candle 模型文件不存在
- WHEN 索引完成后执行文档-代码关联
- THEN 使用 TextEmbedder (Jaccard) 计算关联
- AND source 字段设为 'text'（标记为非语义关联）
