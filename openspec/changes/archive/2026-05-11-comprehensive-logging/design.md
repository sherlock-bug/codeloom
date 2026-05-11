# Design: comprehensive-logging

## Context

CodeLoom 的日志基础设施已就位（`src/logger.rs`），但使用率极低：全库 21 处 `log_*!` 调用，其中 8 个核心模块零日志。现有日志模式不统一——有些地方用 `eprintln!`，有些空操作，有些用 `log_*!`。

目标模块及现状：

| 模块 | 行数 | 当前 log_* 数 | 缺失类型 |
|------|------|-------------|---------|
| indexer/clang/ast.rs | 788 | 0 | AST 决策点 |
| indexer/tree_sitter.rs | 66 | 0 | 文件收集 |
| query/search.rs | 614 | 0 | 搜索查询 |
| calib/mod.rs | ~200 | 0 | 校准 |
| storage/fts.rs | 285 | 0 | FTS 构建/查询 |
| storage/symbols.rs | 268 | 0 | 符号 upsert |
| storage/vector.rs | ~200 | 0 | 向量加载/查询 |
| storage/schema.rs | ~100 | 0 | 迁移执行 |
| indexer/clang/mod.rs | 450 | 1 | TU 处理全过程 |
| indexer/smart.rs | 300 | 2 | 索引编排 |
| mcp/mod.rs | 900 | 2 | 工具调用 |

## Goals / Non-Goals

**Goals:**
- 8 个零日志模块的关键路径追加结构化日志
- 每个模块的入口/退出加耗时标记
- 异常分支、catch-all、skip/ignore 决策点打日志
- `eprintln!` 统一迁移到 `log_error!`/`log_warn!`
- 默认日志级别切换为 debug（已完成 config 层）

**Non-Goals:**
- 不改造 logger 基础设施本身（旋转、级别过滤等已够用）
- 不在高频热路径加日志（如搜索结果的逐条打印）
- 不创造新的日志级别或过滤机制
- 不改动现有 `println!` 用户可见输出

## Decisions

### Decision: 三层日志粒度
日志级别按信息类型分层：
- `log_info!` — 模块入口/退出 + 耗时（如 "AST parsing done: 12 symbols, 8 edges in 1.2s"）
- `log_warn!` — 非致命异常路径（如 "parse failed: file.cc: <error>"）
- `log_error!` — 致命错误（如 "API call failed after 3 retries"）
- `log_debug!` — 关键决策点（如 "is_project_file=/header.h → false, root=/project/./."）

### Decision: 耗时标记格式
统一的耗时输出格式：`"<action> done: <summary> in <duration>ms"`
示例：`"indexer::clang" "parse done: 45 symbols, 12 edges in 234ms"`

使用 `std::time::Instant::now()` 在入口记录 start，退出前计算 elapsed。

### Decision: `eprintln!` 迁移路径
`eprintln!` 用于诊断警告的，迁移到 `log_warn!`；用于错误的迁移到 `log_error!`。用户可见的 `println!`（如 search 结果输出、help 信息）保持不动。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|---------|
| 日志淹没 stdout 输出 | log_* 写入文件，不入 stdout；println! 走 stdout，互不干扰 |
| Debug 级别 I/O 开销 | BufWriter + 级别过滤：低于配置级别立即 return，高频路径零成本 |
| 日志内容随代码变化过时 | 不文档化具体日志字符串，只保证关键决策点有日志，未来重构成新逻辑时自然更新 |

## Migration Plan

1. config/mod.rs 默认日志级别改为 debug（已做）
2. logger.rs 代码默认改为 debug（待做）
3. 8 个零日志模块逐个追加：
   - ast.rs（最重，~15 个决策点 + 耗时）
   - tree_sitter.rs（文件收集入口 + 跳过原因）
   - query/search.rs（查询参数 + 命中数）
   - calib/mod.rs（校准进度 + 失败原因）
   - storage/fts.rs（FTS 构建完成 + 文档数）
   - storage/symbols.rs（符号 upsert 结果）
   - storage/vector.rs（向量加载 + KNN 查询）
   - storage/schema.rs（迁移执行结果）
4. 现有日志模块增强（mod.rs/mcp/smart.rs 补耗时标记）
5. `eprintln!` 统一迁移
