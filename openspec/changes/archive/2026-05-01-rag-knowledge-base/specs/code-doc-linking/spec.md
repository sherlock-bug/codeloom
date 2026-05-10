# Delta for Code-Doc Linking

## ADDED Requirements

### Requirement: Embedding 驱动的自动关联
系统 SHALL 在索引完成后，自动计算文档段落向量与代码符号向量之间的余弦相似度，对相似度超过阈值（默认 0.75）的配对自动建立 doc_code_links 关联边。

#### Scenario: 文档描述业务术语，代码命名不同
- GIVEN 文档 docs/术语/轧差.md 包含"轧差是指多方交易中相互抵消计算净额的过程"
- AND 代码中存在 NettingService::calculate 和 TransactionManager::reverse
- WHEN 索引完成并计算相似度
- THEN 自动建立轧差.md 段落与 NettingService::calculate 的关联（相似度 0.87）
- AND 自动建立轧差.md 段落与 TransactionManager::reverse 的关联（相似度 0.76）
- AND source 字段设为 'embedding'

#### Scenario: 低相似度不建关联
- GIVEN 文档段落与代码符号的余弦相似度 < 0.6
- WHEN 相似度计算完成
- THEN 不建立 doc_code_link
- AND 日志记录跳过的配对数量

### Requirement: 关联强度可查询
系统 SHALL 记录自动关联的相似度分数和来源类型，供查询时使用。

#### Scenario: 查询文档关联的代码
- GIVEN 文档"轧差.md"已自动关联多个代码符号
- WHEN 查询该文档的关联代码
- THEN 返回按相似度降序排列的代码符号列表
- AND 显示每个符号的相似度 (0.6-1.0) 和来源标记（embedding 或 manual）

### Requirement: 双向链路
系统 SHALL 确保文档到代码的关联是双向的，查询代码符号时也能看到关联的文档段落。

#### Scenario: 查询代码符号看到关联文档
- GIVEN SettlementEngine::calculate 已自动关联"结算规则.md §2"
- WHEN 调用 rag_get_definition("SettlementEngine::calculate")
- THEN 返回代码定义，同时附带关联文档列表
- AND 文档条目包含标题、路径、段落摘要、关联强度

### Requirement: 支持手动补充关联
系统 SHALL 同时支持手动建立的显式文档-代码关联（source='manual'），手动关联的 link_strength 始终为 1.0，且优先于自动关联。

#### Scenario: 归档后手动补联
- GIVEN 自动关联未能发现某术语与代码的对应关系
- WHEN 用户在文档中显式引用代码符号（如 SettlementEngine::calculate）
- THEN 索引时识别为手动引用并建立 source='manual' 的关联
- AND 手动关联在查询结果中优先于自动关联展示
