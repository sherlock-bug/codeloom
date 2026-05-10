# Design: vec0-static-inline-dedup

## Context
CodeLoom v0.3.9 的 vec0 向量扩展通过运行时 `sqlite3_load_extension(db, "vec0.so")` 加载。这在 musl 静态编译时无法工作，且部署时需额外分发 .so 文件。同时，符号向量化是索引后的独立全量扫描，文档索引缺乏增量感知。

## Goals / Non-Goals
- ✅ vec0 静态编译：源码链接进二进制，零外部依赖
- ✅ 内联向量化：符号解析时嵌入，消除独立扫描步骤
- ✅ 文档去重：content_hash 跳过无变化文档
- ❌ 不改变 vec0 的 SQL 查询接口
- ❌ 不改变 embedding 模型（继续用 candle bge-small-zh）

## System Architecture

```
编译时:
  vendor/sqlite-vec/sqlite-vec.c ──┐
  vendor/vec0_static.c ────────────┤──→ cargo build ──→ codeloom (static, no .so)
  vendor/sqlite3.h ───────────────┘

运行时:
  codeloom index
    ├── smart_index (解析阶段)
    │   └── 内联嵌入: 符号完成 → 立即 embed → 存入向量
    ├── build.rs (编译时下载模型到 models/)
    └── vector.rs
        └── vec0_static_init() ──→ 直接调用 C 函数，零外部依赖
```

## Database Design

### doc_nodes 表变更
```sql
-- 新增 UNIQUE 约束
CREATE UNIQUE INDEX IF NOT EXISTS idx_doc_nodes_path 
  ON doc_nodes(repo, branch_name, doc_path);

-- 新增列
ALTER TABLE doc_nodes ADD COLUMN content_hash TEXT;
```

### content_hash 计算公式
```rust
let hash = format!("{}:{}:{}", title, section_path, content);
let hash = blake3::hash(hash.as_bytes()).to_hex().to_string();
```

## Decisions

### Decision 1: sqlite-vec 源码静态链接而非编译为 .a
选择将 sqlite-vec 源码直接加入 `vendor/` 目录而非编译为独立的 .a 静态库，因为：
- cc crate 可以直接编译 C 文件到 Rust 二进制
- 无需额外 Makefile 或 CMake 构建步骤
- 版本精确锁定在 vendor 目录中

### Decision 2: 内联向量化位置选在 smart_index 解析阶段
选在 `smart_index` 的符号解析完成时立刻嵌入，而非保留独立 `index_vectors` 步骤，因为：
- 每个符号只用处理一次，避免全量二次扫描
- 进度条实时反馈更好
- 增量索引时自动只有新符号被向量化

### Decision 3: content_hash 用 blake3 而非 SHA-256
blake3 比 SHA-256 快 10x，对文档去重场景足够了。碰撞概率极低（256-bit）。

### Decision 4: FTS5 rowid 用 `ORDER BY rowid` 替代 `'delete'` 语法
`'delete'` 在 SQLite FTS5 中有已知的 rowid 重排问题。直接 `ORDER BY rowid` 更安全可靠。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| vendor 目录膨胀（309 文件、10718 行 C 源码） | `.gitignore` 忽略，编译产物不入库 |
| blake3 哈希冲突 | 256-bit 碰撞概率 < 2^-256，可接受 |
| 内联向量化增加索引延迟 | 每个符号的嵌入计算（~2ms）已在向量化阶段存在，仅平移非新增 |

## Migration Plan
1. 存量数据库通过 `migrate()` 检测 content_hash 列
2. 首次 index_doc_vectors 时 NULL hash 全量计算并回填
3. 后续增量索引自动跳过不变文档
4. 无需数据库格式升级，向后兼容
