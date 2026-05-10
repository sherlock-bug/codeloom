# Delta for inline-vectorization

## ADDED Requirements

### Requirement: 符号解析时内联嵌入
系统 SHALL 在 `smart_index` 解析每个符号时直接计算并嵌入文本向量，而非在索引完成后进行独立的全量向量化扫描。

#### Scenario: 新文件索引
- GIVEN 一个包含 10 个函数的新 .rs 文件
- WHEN 执行 `codeloom index`
- THEN 每个符号在解析阶段即获得嵌入向量，`index_vectors` 步骤不再需要单独扫描这些符号

#### Scenario: 进度条显示向量化进度
- GIVEN 正在执行 `codeloom index` 且有符号需要向量化
- WHEN 进度条更新时
- THEN SHALL 显示符号向量化实时百分比（如 "embedding symbols: 45/100 (45.0%)"）

### Requirement: 全文搜索符号保留内联向量
系统 SHALL 确保通过 FTS5 全文搜索返回的符号同样包含内联嵌入的向量，支持后续语义搜索融合。

#### Scenario: FTS5 搜索结果带向量
- GIVEN 已有索引包含内联向量的符号
- WHEN 执行 `codeloom search "auth token"`
- THEN 返回的搜索结果 SHALL 包含符号的嵌入向量，可用于 RRF 融合排序
