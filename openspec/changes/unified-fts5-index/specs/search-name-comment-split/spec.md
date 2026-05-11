# Delta for search-name-comment-split

## MODIFIED Requirements

### Requirement: 符号名与注释分通道搜索
FTS5 搜索 SHALL 对统一表 `fts5_all` 的 name 列和 content 列分别执行列过滤查询，符号名命中通过 name 列查询，注释命中通过 content 列查询。

#### Scenario: 英文关键词命中符号名
- GIVEN `fts5_all` 包含符号 `Snappy_Compress`（name='Snappy_Compress', content='...'）
- WHEN 搜索 "snappy" 时对 name 列执行 `fts5_all MATCH 'name:snappy*'`
- THEN 匹配的符号 SHALL 出现在结果中
- AND BM25 分数与其他来源类型的 name 命中共享同一 IDF 域

#### Scenario: 中文关键词搜英文代码
- GIVEN 索引中函数名英文但注释含中文
- WHEN 用户搜索中文关键词
- THEN content 列 SHALL 匹配注释中的中文内容
- AND 与文档 content 列命中在相同 IDF 尺度下排序

### Requirement: 名命中权重大于注释命中
搜索结果融合 SHALL 对 name 列命中赋予更高权重（0.7），对 content 列命中赋予较低权重（0.3）。权重直接作用于 BM25 归一化分数，不再需要跨表补偿。

#### Scenario: 名注释同时命中时的排序
- GIVEN 符号 A 的 name 列命中关键词，符号 B 的 content 列命中
- WHEN 搜索该关键词
- THEN 符号 A 的最终分数 SHALL 高于符号 B（0.7 × norm > 0.3 × norm）
- AND 同列内不同来源类型（sym/doc/file）的 BM25 分数天然可比
