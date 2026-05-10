# int8-vector-storage Specification

## Purpose
TBD - created by archiving change optimize-vector-storage. Update Purpose after archive.
## Requirements
### Requirement: INT8 向量存储
符号名向量表 SHALL 使用 vec0 INT8 元素类型而非 FLOAT32，将每维度存储从 4 字节压缩至 1 字节。

#### Scenario: 创建 INT8 向量表
- GIVEN 一个新仓库需要索引
- WHEN 执行 codeloom index
- THEN 符号名向量表 SHALL 以 INT8[{dim}] 创建
- AND 文件向量表 SHALL 保持 FLOAT[{dim}] 不变

#### Scenario: 旧表兼容
- GIVEN 仓库已有 FLOAT32 符号名向量表
- WHEN 执行 codeloom index
- THEN 索引器 SHALL drop 旧表并重建为 INT8 格式

### Requirement: Float32 到 INT8 量化
嵌入向量插入 vec0 前 SHALL 经过 int8 量化：`v_int8 = round(v_f32 * 127)`，结果限制在 [-128, 127]。

#### Scenario: 批量量化
- GIVEN API 返回的 1024 维 float32 嵌入
- WHEN 准备插入 symbol_name_vec 表
- THEN 每个维度 SHALL 被量化到 int8 范围
- AND JSON 格式 SHALL 为整数数组 `[12, -45, ...]`

### Requirement: KNN 查询向量量化
混合搜索的查询嵌入 SHALL 使用与索引相同的 int8 量化，确保查询向量与存储向量类型一致。

#### Scenario: 语义搜索查询
- GIVEN 用户输入搜索文本
- WHEN 执行混合搜索中的向量通道
- THEN 查询嵌入 SHALL 被量化为 int8 后传入 KNN 查询

