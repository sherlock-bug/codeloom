# search-noise-filter

## Purpose
CodeLoom 搜索结果噪声过滤——基于噪声标定的天花板分数，在混合搜索返回结果前自动过滤低于阈值的噪声条目，确保 CLI 和 MCP 搜索输出不包含无意义结果。

## Requirements

### Requirement: 噪声过滤
`hybrid_search` 函数 SHALL 在四通道融合完成后、返回结果前，过滤所有分数低于 `noise_ceiling` 的结果。

#### Scenario: 过滤噪声结果
- GIVEN noise_ceiling = 0.34，搜索结果包含 [0.472, 0.319, 0.315, 0.303]
- WHEN `hybrid_search` 返回前执行过滤
- THEN 返回结果 SHALL 仅包含 [0.472]（>= 0.34）

#### Scenario: 全部低于阈值
- GIVEN noise_ceiling = 0.34，所有搜索结果分数 < 0.34
- WHEN 执行过滤
- THEN 返回空结果集 SHALL NOT 报错

### Requirement: 过滤时机
过滤 SHALL 在 `weighted_fuse` 融合、去重、排序之后，`truncate(limit)` 之前执行。

#### Scenario: 融合后过滤
- GIVEN 名通道和注释通道已加法合并
- WHEN 过滤执行
- THEN 每个结果的 score SHALL 是名+注释两通道的最终合计分数

### Requirement: 过滤容错
如 `config.db` 无 `noise_profile` 记录，搜索 SHALL 跳过过滤（不报错、不阻塞搜索）。

#### Scenario: 无 profile 时不过滤
- WHEN `hybrid_search` 加载 noise_profile 失败
- THEN 搜索 SHALL 正常返回结果，不执行任何过滤

### Requirement: CLI 搜索输出
CLI `search` 命令 SHALL 使用 `hybrid_search` 的过滤结果，噪声条目不出现。

#### Scenario: CLI 搜索过滤后
- GIVEN noise_ceiling 已标定
- WHEN 执行 `codeloom search "快照隔离"`
- THEN 输出 SHALL 不包含分数 < noise_ceiling 的条目

### Requirement: MCP 搜索工具
MCP `codeloom_search` 工具 SHALL 使用 `hybrid_search` 的过滤结果，返回的 JSON 中不包含噪声条目。

#### Scenario: MCP 搜索过滤后
- GIVEN noise_ceiling 已标定
- WHEN MCP 调用 `codeloom_search({query: "快照隔离"})`
- THEN 返回 JSON 中 SHALL 不包含分数 < noise_ceiling 的条目
