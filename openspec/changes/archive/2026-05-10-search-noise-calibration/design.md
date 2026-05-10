## Context

搜索验证发现噪声分数聚类在 0.31-0.34 区间，无真实命中时会浮到 Top 1（如"快照隔离"→GUARDED_BY）。需要全局噪声基线来过滤。基线取决于嵌入模型和融合算法，与代码仓无关。

当前 `codeloom check` 做 5 项检查（data dir、vec0、FTS5、embedding、repo list）。本次在 embedding 检查后插入标定逻辑。当前 `hybrid_search` 无过滤能力。

## Goals / Non-Goals

**Goals:**
- 内置小型语料库（6 文件），用真实 index+search 流程标定噪声基线
- `codeloom check` 强制重新标定；`codeloom index` 无基线时自动触发
- `hybrid_search` 过滤低于噪声阈值的噪声结果
- 健康探针验证搜索管线正常

**Non-Goals:**
- 不修改融合权重或搜索排名逻辑
- 不修改已有索引格式
- 不依赖任何特定嵌入模型（换模型后 check 重新标定即可）

## Decisions

### Decision 1: 语料库文件方式
内嵌为 const 字符串，首次 `check`/`index` 写入 `~/.codeloom/calib/`。之后永久保留。

**理由**: 比二进制内嵌更透明可检查；比网络下载更可靠。文件极小（6 个共 < 2KB）。

**Alternatives considered**: 
- 纯 const 字符串 + 内存索引：跨进程不生效（index 在独立进程），且无法用真实 index flow 测试
- 网络下载：离线环境不可用

### Decision 2: 标定使用真实索引流程
`codeloom check` 内部调用 `crate::indexer` 索引 calib 目录到临时 `_calib` DB，然后调用 `hybrid_search` 跑探针。用完删除临时 DB。

**理由**: 确保权重、融合逻辑与生产完全一致。不重新实现简化版搜索。

**Alternatives considered**:
- 在内存中模拟索引+搜索：无法保证与生产一致性
- 直接调用向量 API 测距离：缺少 FTS5 通道和融合逻辑

### Decision 3: 全局配置存 config.db
新建 `~/.codeloom/config.db`（与已有 repo DB 分开），单表 `noise_profile`。

**理由**: 噪声基线是全局属性（非 per-repo）。SQLite 比 JSON 文件更适合结构化配置，与项目技术栈一致。

**Alternatives considered**:
- `~/.codeloom/noise_profile.json`：简单但类型不安全，需额外解析
- 存在每个 repo DB 中：冗余，且换模型需更新所有 DB

### Decision 4: 阈值计算 mean + 2.5σ
5 条探针 × 20 结果 = 100 个噪声样本，计算均值 μ 和标准差 σ，ceiling = μ + 2.5σ。

**理由**: 2.5σ 覆盖 ~99% 噪声分布，比 2σ (~95%) 更保守。探针数量在精度和速度间取平衡（< 10s 完成）。

**Alternatives considered**:
- mean + 2σ：可能保留 5% 噪声
- 绝对值阈值 0.35：换模型失效
- 中位数 + MAD：对小样本不够稳定

### Decision 5: index 自动触发标定
`codeloom index` 完成后检查 `config.db`。无 `noise_profile` 则自动标定（不报错、不阻塞）。

**理由**: 不要求用户先跑 `check`。首次使用即自动建立基线。

**Alternatives considered**:
- 要求用户先跑 `check`：增加上手步骤
- 不自动标定：搜索无过滤能力，体验差

### Decision 6: 搜索直接过滤而非标注
`hybrid_search` 融合完成后，在返回前删除分数 < ceiling 的结果。

**理由**: 用户明确要求不再显示噪声结果。无 `noise_profile` 时跳过过滤（不阻塞搜索）。

**Alternatives considered**:
- 标注 `[LOW]`：保留但标记——用户否定了
- 过滤后补到 limit 条：可能引入更多噪声——不做

### Decision 7: 语料内容域选择
纯生活领域（天气、花园、烘焙、海洋、阅读、城市漫步），零编程语义。

**理由**: 探针为随机/日常用语，确保向量距离仅反映噪声分布，不会因领域重叠产生伪命中。

## Project Structure

```
src/
├── calib/                    # 新增模块
│   └── mod.rs               # corpus 嵌入 + calibrate() + health_check()
├── cli/
│   └── mod.rs               # 修改: check 加标定步骤; index 加自动触发
├── query/
│   └── search.rs            # 修改: hybrid_search 加过滤
└── ...

~/.codeloom/
├── calib/                    # 语料文件（安装/首次运行时写入）
│   ├── weather.h
│   ├── garden.cpp
│   ├── recipe.md
│   ├── 阅读时光.md
│   ├── ocean_sim.h
│   └── 城市漫步.md
└── config.db                 # 全局配置（新增）
    └── noise_profile 表
```

## Database Design

```sql
-- config.db (新建，全局配置数据库)
CREATE TABLE IF NOT EXISTS noise_profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    noise_mean REAL NOT NULL,
    noise_std REAL NOT NULL,
    noise_ceiling REAL NOT NULL,
    samples INTEGER NOT NULL,
    model TEXT NOT NULL,          -- 嵌入模型名（如 bge-small-zh）
    calibrated_at TEXT NOT NULL   -- ISO 8601
);
```

只允许单行 (id=1)，更新时用 INSERT OR REPLACE。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 探针语料过小导致 σ 不准 | 5 条探针 × 20 结果 = 100 样本，足够中层分布估计 |
| 过滤太激进删掉弱但正确的结果 | 2.5σ 只覆盖极端噪声；弱正确结果（如跨语言 0.31）应在 ceiling 之上 |
| 换嵌入模型后 noise_profile 过期 | `codeloom check` 强制重新标定；用户知道换模型需跑 check |
| 暂无 repo 时 index 自动标定失败 | 跳过标定，搜索无过滤（向后兼容）；用户 index 完后自动标定生效 |
