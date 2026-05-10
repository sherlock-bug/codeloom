# Design: 搜索工具拆分与噪音标定独立化

## Context

当前 CodeLoom 只有一个 `codeloom_search` 工具，内部混合 BM25 FTS5 和 vec0 向量搜索，通过 `weighted_fuse` 融合成单一分数。存在三个问题：

1. **LLM 无法区分通道**：融合分是黑盒，LLM 不能判断结果是来自关键词精确命中还是语义近似
2. **噪音标定绑定融合分**：`calibrate()` 跑的是 `hybrid_search()`，拆开后 BM25 和向量各有自己的分数尺度，用同一基线会误杀
3. **注释通道未使用**：`bm25_comment` 获取了但从未参与融合（search.rs:188 只有注释无代码）

目标：拆分搜索 → 独立标定 → 标准化输出。

## Goals / Non-Goals

**Goals:**
- 精确搜索（BM25 FTS5）和语义搜索（vec0 向量）拆为两个独立 MCP 工具
- 统一所有节点类型的权重逻辑：名称通道 ×0.7，内容通道 ×0.3
- BM25 和向量各自独立噪音标定，互不干扰
- MCP 返回完整 JSON 属性，按节点类型静态裁剪空字段
- CLI 输出精简三列

**Non-Goals:**
- 不改动 doc chunking 或向量化存储格式
- 不换 embeddings 模型
- 不新增 MCP 工具参数（如切换模式）

## Decisions

### Decision 1: 精确搜索入口函数 `bm25_precise_search()`

选择新增独立函数而非给 `hybrid_search` 加参数，因为：
- 两个搜索的内部查询路径完全不同（BM25 需查 fts5_sym/doc/files 三表，向量只查 vec0 一表）
- 返回值不同（精确搜索可能返回 doc/file 节点，语义搜索只返回符号）
- 代码更清晰，删除旧融合逻辑无顾虑

### Decision 2: 统一权重模型

所有节点类型（符号/文档/文件）统一两个通道：

| 通道 | 权重 | 符号搜索列 | 文档搜索列 | 文件搜索列 |
|------|------|-----------|-----------|-----------|
| 名称 | ×0.7 | name, signature, kind | title | file_path |
| 内容 | ×0.3 | doc_comment | section_path, content | summary |

两组结果按 `(name, file_path)` 去重取 max score，再按 score 降序排列。

**备选方案（不采用）**：按节点类型分别设权重（doc ×0.3, file ×0.2）。缺点：权重配置复杂，对 LLM 选工具无帮助，对搜索质量无实质提升。

### Decision 3: 噪音标定分通道存储

`noise_profile` 表新增 `channel TEXT` 列（`bm25` / `vector` / `hybrid`）：

```sql
CREATE TABLE noise_profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    channel TEXT NOT NULL DEFAULT 'hybrid',  -- 新增列
    top1_mean REAL NOT NULL,
    top1_std REAL NOT NULL,
    samples INTEGER NOT NULL,
    model TEXT NOT NULL,
    calibrated_at TEXT NOT NULL
);
```

- `calibrate_bm25()`：5 条探针 → `bm25_precise_search()` → 记录 top1
- `calibrate_vector()`：5 条探针 → `vector_semantic_search()` → 记录 top1
- 旧数据 `channel='hybrid'` 保留兼容，但不被新搜索使用
- 主键改为 `(id, channel)` 或每条记录独立 id

**备选方案**：分表存储。缺点：需维护两张表，反而增加复杂度。

### Decision 4: CLI 搜索输出格式

CLI 不返回全量 JSON，只输出三列：

```
[类型          ] 名称                                       @ 注释/内容摘要
[function       ] CompactMemTable                            @ 压缩MemTable并写入
[class          ] DBImpl                                    @ 数据库实现类
[doc            ] 架构设计                                  @ 系统分为三层...
```

字段宽度：类型 16 字符左对齐，名称 40 字符左对齐，注释截断 120 字符。

### Decision 5: 保留 `codeloom_list_symbols` 但升级内部实现

当前 `codeloom_list_symbols` 使用 `LIKE '%pattern%'` 简单匹配，无排序。升级为 FTS5 `search_symbols_name()`，获得 BM25 排序能力。工具名称和 description 不变，仅内部实现升级。

### Decision 6: fts5_sym schema 兼容性

当前 fts5_sym schema 是 `name, file_path, signature, kind, doc_comment`（schema.rs:191）。精确搜索需要同时搜 name 和 doc_comment，但 FTS5 不支持单次查询对不同列设不同权重。因此仍用两次查询（名称通道 + 内容通道），在应用层加权合并。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 噪音标定迁移时旧 config.db 可能损坏 | 加 `ALTER TABLE` 迁移，失败时自动重建 noise_profile 表 |
| doc 节点 title 可能为空（未拆分出标题） | 回退用 content 前 60 字符作为 title 参与名称通道搜索 |
| 向量搜索 INT8 量化后精度下降 | 沿用现有 INT8 量化参数，标定后验证噪音过滤阈值合理性 |
| 旧集成测试依赖 `codeloom_semantic_search` 不存在 | 更新测试：精确搜索测试走 `codeloom_search`，语义搜索测试走新 `codeloom_semantic_search` |

## Migration Plan

1. 新增 `bm25_precise_search()` 和 `vector_semantic_search()` 函数
2. 新增 `calibrate_bm25()` 和 `calibrate_vector()` 函数
3. 修改 `noise_profile` schema（在线迁移）
4. MCP 注册新增 `codeloom_semantic_search` 工具
5. 修改 `codeloom_search` 路由到 `bm25_precise_search()`
6. 修改 CLI 搜索输出格式
7. `cargo test` 确保旧测试兼容
8. 运行 `codeloom calibrate` 重新标定两个通道
9. 集成测试验证搜索可用性
