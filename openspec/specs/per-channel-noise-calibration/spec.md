# per-channel-noise-calibration Specification

## Purpose
TBD - created by archiving change search-tool-separation. Update Purpose after archive.
## Requirements
### Requirement: 分通道噪音标定存储
系统 SHALL 在 noise_profile 表中区分 BM25 和向量两种通道的噪音基线，各自独立存储和查询。

#### Scenario: 存储 BM25 噪音基线
- GIVEN 执行 `codeloom calibrate` 命令
- WHEN 标定 BM25 通道
- THEN noise_profile 表 SHALL 包含一条 channel='bm25' 的记录
- AND 记录 SHALL 包含 top1_mean、top1_std、samples、model、calibrated_at

#### Scenario: 存储向量噪音基线
- GIVEN 执行 `codeloom calibrate` 命令
- WHEN 标定向量通道
- THEN noise_profile 表 SHALL 包含一条 channel='vector' 的记录

#### Scenario: 兼容旧数据迁移
- GIVEN noise_profile 表存在旧记录（无 channel 列）
- WHEN 首次启动
- THEN 旧记录 SHALL 被标记为 channel='hybrid' 保留兼容
- AND 旧记录不被新搜索工具使用

### Requirement: BM25 通道独立标定
系统 SHALL 使用纯 BM25 搜索（不涉及向量）对噪声探针进行标定，记录 top1 分数的均值和标准差。

#### Scenario: BM25 标定过程
- GIVEN 至少有一个已索引仓库
- WHEN 执行 BM25 通道标定
- THEN 系统 SHALL 用 5 条噪声探针分别调用 bm25_precise_search
- AND 取每次的 top1 score 计算均值和标准差
- AND 标定结果 SHALL 存入 noise_profile(channel='bm25')

### Requirement: 向量通道独立标定
系统 SHALL 使用纯向量搜索（不涉及 BM25）对噪声探针进行标定，记录 cosine 相似度的均值和标准差。

#### Scenario: 向量标定过程
- GIVEN 至少有一个已索引仓库且向量扩展已加载
- WHEN 执行向量通道标定
- THEN 系统 SHALL 用 5 条噪声探针分别调用 vector_semantic_search
- AND 取每次的 top1 score 计算均值和标准差
- AND 标定结果 SHALL 存入 noise_profile(channel='vector')

