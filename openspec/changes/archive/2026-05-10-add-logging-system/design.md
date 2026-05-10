# Design: 日志系统

## Context

CodeLoom 当前所有输出通过 `eprintln!` 和 `println!` 裸写到 stderr/stdout，共计 148 处。`log` 和 `env_logger` crate 已在 Cargo.toml 且 `env_logger::init()` 已在 `main.rs` 调用——但没有任何代码使用 `log` 宏。这导致：无时间戳、无日志级别、无文件持久化、无法追溯历史操作。

本设计实现一个自建最小化日志模块，替代 `log` + `env_logger`（移除这两个 crate），不与 CLI 用户界面耦合。CLI 的 `eprintln!` 保持原样用于用户交互，日志独立写入文件。

## Goals / Non-Goals

**Goals:**
- 提供文件日志，含时间戳(ms)、PID、TID、日志级别、模块路径
- 配置文件控制开关和级别，release 和 debug 构建均保留日志代码
- 文件绕接：超大小上限创建新文件，文件名固定为创建时间戳
- 文件数量限制：超上限删除最旧文件
- 关键入口/出口/异常分支打点

**Non-Goals:**
- 不修改 CLI 界面（不增加日志相关命令行参数）
- 不替换已有 `eprintln!`（两者并存，各自独立）
- 不用第三方日志框架（自建，移除 `log` + `env_logger` crate）
- 不改变 stderr 输出行为
- 不做控制台日志输出（只写文件）

## Decisions

### Decision 1: 自建日志模块 vs 使用 log crate

选择自建 `src/logger.rs`，移除 `log` + `env_logger` 依赖。

理由：
- `log` crate 的宏在 release 下仍保留格式字符串于二进制中（已验证）
- 需求简单（文件写入 + 绕接 + 清理），不需要 `log` 的门面模式
- 移除两个 crate 减少依赖链
- 自定义宏 `log_info!` / `log_warn!` / `log_error!` / `log_debug!` 接口清晰，用法与 `log` 宏类似

Alternatives considered:
- `fern`：轻量但仍是第三方依赖，引入额外风险
- `log` + `env_logger`：格式字符串残留问题，比自建复杂
- `tracing`：过重，不适合 CLI 工具

### Decision 2: 运行时配置 vs 编译时门控

选择运行时配置：代码始终编译，由 `config.yaml` 中 `logging.enabled` 控制。

理由：
- 用户要求 release 保留日志方便定位问题
- 配置关闭后开销极小：一个 `AtomicBool` 检查 + 提前返回，纳秒级
- 灵活：不用重新编译就能开关日志

### Decision 3: 日志文件命名

文件名固定为创建时间戳：`codeloom_{YYYYMMDD-HHMMSS}_{ms}.log`。进程启动时创建，绕接时新文件用新时间戳，旧文件不重命名。

### Decision 4: 绕接策略

每次写日志前检查当前文件大小。超 `max_file_size_mb` 时：
1. 关闭当前文件
2. 检查 `logs/` 下文件数是否超过 `max_files`
3. 若超限，按修改时间排序删除最旧文件
4. 创建新日志文件（当前时间戳命名）

### Decision 5: 线程安全

Logger 为全局单例 `OnceLock<Mutex<Logger>>`。`log_info!` 等宏获取锁写入，保证多线程安全。锁粒度仅覆盖单条日志的写操作，不阻塞整体流程。

### Decision 6: 日志格式

```
2026-05-10 15:30:12.345 [12345:67890] INFO  indexer::clang: clang parse start: file=src/main.cpp, tu_index=3
```

各部分：
- 时间戳到毫秒
- `[PID:TID]` — 进程号:线程号
- 日志级别右对齐到 5 字符
- `module::function` — 模块路径（手动传入，不自动捕获）
- 消息体

### Decision 7: 模块路径策略

不自动捕获调用位置（`file!()`/`line!()`），由调用方手动传入简短模块标识（如 `"indexer::clang"`）。理由：
- 日志膨胀控制：`file!()` 带完整路径，如 `/home/user/codeloom/src/indexer/clang/mod.rs`
- 手动传入的标识简短且语义明确
- 打点位置有限（~25 处），维护成本低

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 日志写入失败（磁盘满、权限） | `eprintln!` 输出警告到 stderr，不 panic，继续运行 |
| 绕接检查在高频日志下增加开销 | 检查仅为 `file.len() > max`，一次系统调用，可接受 |
| 全局锁在高并发 MCP 场景下成为瓶颈 | 当前 MCP 为 stdio 单连接，无高并发。若未来需要，可改为 channel + 异步写入 |
| 移除 `log` + `env_logger` 后 `main.rs` 需要修改 | 改动极小：删除 `extern crate env_logger` 和 `env_logger::init()`，替换为 `logger::init(&config)` |

## Migration Plan

1. 新建 `src/logger.rs`
2. 修改 `src/config/mod.rs` 增加 `LoggingConfig` 结构体
3. 修改 `src/main.rs`：用 `logger::init()` 替换 `env_logger::init()`
4. 在关键位置添加日志打点（不删已有 `eprintln!`）
5. `Cargo.toml` 移除 `log` 和 `env_logger`
6. 添加单元测试覆盖绕接和清理逻辑
7. 更新 README 日志功能说明
