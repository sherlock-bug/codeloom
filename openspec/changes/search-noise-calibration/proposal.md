## Why

向量搜索结果中分数 0.31-0.34 区间的噪声会在无真实命中时浮到 Top 1，与弱但正确的结果（跨语言语义命中如"快照"→Snapshot 0.317）完全重叠。用户无法区分"相关但弱"和"完全不相关"。需要一个不依赖硬编码阈值、对模型/语言/仓自适应的噪声判定机制。

## What Changes

- 新增内置标定语料库（`~/.codeloom/calib/`，6 文件，中英文、长短混合、覆盖代码/注释/doc 三种节点类型）
- 新增 `codeloom check` 噪声标定流程：索引语料 → 6 条噪声探针搜索 → 计算噪声基线 → 存 `config.db:noise_profile`
- `codeloom index` 首次执行时自动标定（如无噪声基线）
- `hybrid_search` 读取噪声基线，分数低于 `ceiling` 的结果**直接过滤不返回**
- CLI/MCP 搜索结果中不再出现噪声条目

## Capabilities

### New Capabilities
- `noise-calibration`: 内置语料库 + 探针系统，在 `codeloom check` 时标定噪声基线（mean + 2.5σ），存储为全局配置供搜索使用。`codeloom index` 无基线时自动触发。
- `search-noise-filter`: `hybrid_search` 搜索结果中分数低于噪声基线的结果**直接过滤**，CLI/MCP 不再返回噪声条目。

## Impact

- 新增 `src/calib/` 模块（语料嵌入、标定逻辑）
- 修改 `src/cli/mod.rs`（check 命令增加标定步骤；index 命令增加自动触发）
- 修改 `src/query/search.rs`（`hybrid_search` 读取噪声基线并标注）
- 新增 `~/.codeloom/calib/` 目录（安装时写入）
- 新增 `config.db:noise_profile` 表
- 不影响已有索引、搜索排名或 API 接口（仅添加新字段）
