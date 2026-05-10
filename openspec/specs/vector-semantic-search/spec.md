# vector-semantic-search Specification

## Purpose
TBD - created by archiving change search-tool-separation. Update Purpose after archive.
## Requirements
### Requirement: 语义向量搜索 MCP 工具
系统 SHALL 提供独立的语义向量搜索工具，使用 vec0 INT8 KNN 搜索，只搜索符号名称通道（symbol_name_vec），不搜索文档、注释或文件。

#### Scenario: 语义搜索符号
- GIVEN leveldb 仓库已索引且向量表已加载
- WHEN 调用语义搜索工具 query="数据库写操作"
- THEN 返回的符号 SHALL 包含 Put、Write 等与"数据库写操作"语义相关的函数
- AND 不返回文档节点或文件节点

#### Scenario: 向量未加载时返回错误
- GIVEN 嵌入模型未加载或 vec0 扩展不可用
- WHEN 调用语义搜索工具
- THEN 返回错误信息说明向量搜索不可用

#### Scenario: 空结果
- GIVEN 向量搜索已可用
- WHEN 调用语义搜索工具 query="xyxxy_random_noise"
- THEN 返回结果经噪音过滤后 SHALL 为空

### Requirement: 语义搜索噪音过滤
系统 SHALL 使用向量专用噪音基线过滤语义搜索结果中的低相似度条目。

#### Scenario: 向量噪音基线过滤
- GIVEN 向量噪音基线已标定（top1_mean=0.35, top1_std=0.05）
- WHEN 某搜索结果 cosine_sim=0.32
- THEN 该结果 SHALL 被过滤（z-score = (0.32-0.35)/0.05 = -0.6 < 1.0）

