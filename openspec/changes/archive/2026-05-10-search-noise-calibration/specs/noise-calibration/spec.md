## ADDED Requirements

### Requirement: 内置语料库
系统 SHALL 内置 6 个小型语料文件（中英文、长短混合、覆盖代码/注释/doc 节点），内容为纯生活领域（天气、花园、烘焙、海洋、阅读、城市漫步），零编程语义。

#### Scenario: 语料文件写入磁盘
- WHEN 语料目录 `~/.codeloom/calib/` 不存在
- THEN 系统 SHALL 从内置 const 字符串写入 6 个文件（weather.h, garden.cpp, recipe.md, 阅读时光.md, ocean_sim.h, 城市漫步.md）

#### Scenario: 语料文件已存在
- WHEN 语料目录已存在且包含全部 6 文件
- THEN 系统 SHALL 跳过写入，保留已有文件

### Requirement: 噪声探针
系统 SHALL 内置 6 条噪声探针（中英文各 3 条，长短混合），每条探针的文本语义与语料库内容无任何重叠。

#### Scenario: 探针不命中语料
- WHEN 使用任意噪声探针搜索标定语料库
- THEN FTS5 名搜索和注释搜索 SHALL 均返回 0 结果

### Requirement: 健康探针
系统 SHALL 内置 2 条健康探针，用于验证搜索管线在已索引仓库上正常工作。

#### Scenario: 健康探针命中
- WHEN 使用健康探针搜索已索引仓库
- THEN 搜索 SHALL 返回非空结果

### Requirement: 噪声标定算法
系统 SHALL 对每条噪声探针执行 Top 20 混合搜索，收集所有分数，计算均值 μ 和标准差 σ，噪声天花板 ceiling = μ + 2.5σ。

#### Scenario: 标定计算
- GIVEN 6 条噪声探针各返回 20 个结果
- WHEN 收集 120 个分数样本
- THEN ceiling SHALL 大于 mean 且小于 mean + 4σ

### Requirement: 真实搜索算法
标定 SHALL 使用与生产环境完全一致的真实索引流程（index 语料目录 → hybrid_search 跑探针），确保权重和融合逻辑一致。

#### Scenario: 使用生产算法
- WHEN 执行噪声标定
- THEN 系统 SHALL 调用 `crate::indexer` 索引语料目录到临时数据库，调用 `crate::query::search::hybrid_search` 执行探针搜索

### Requirement: 标定结果存储
标定结果 SHALL 存储到 `~/.codeloom/config.db` 的 `noise_profile` 表（noise_mean, noise_std, noise_ceiling, samples, model, calibrated_at）。

#### Scenario: 首次标定写入
- WHEN 完成噪声标定且 config.db 无 noise_profile 记录
- THEN 系统 SHALL INSERT 新行

#### Scenario: 重复标定覆盖
- WHEN 完成噪声标定且 noise_profile 已有记录
- THEN 系统 SHALL UPDATE 覆盖旧值

### Requirement: check 命令强制标定
`codeloom check` 命令 SHALL 强制执行噪声标定，覆盖已有 noise_profile。同时执行 2 条健康探针对所有已索引仓库的验证。

#### Scenario: check 触发标定
- WHEN 用户执行 `codeloom check`
- THEN 系统 SHALL 无条件执行噪声标定并更新 noise_profile 表
- AND 对每个已索引仓库执行健康探针验证并报告结果

### Requirement: index 命令自动首次标定
`codeloom index` 命令完成后 SHALL 检查 config.db，如无 noise_profile 则自动触发噪声标定（不报错、不阻塞索引流程）。

#### Scenario: 首次 index 自动标定
- GIVEN config.db 无 noise_profile 记录
- WHEN 完成索引操作
- THEN 系统 SHALL 自动执行噪声标定并写入 noise_profile

#### Scenario: 已有基线跳过
- GIVEN config.db 已有 noise_profile 记录
- WHEN 完成索引操作
- THEN 系统 SHALL 跳过噪声标定

### Requirement: 标定失败容错
噪声标定失败（embedding 不可用、FTS5 异常等）SHALL 不阻塞 `check` 或 `index` 命令，仅打印警告。

#### Scenario: embedding 不可用
- WHEN embedding API 不可用导致标定失败
- THEN 系统 SHALL 打印 `[WARN] Noise calibration failed: <原因>`
- AND `check`/`index` SHALL 继续正常完成
