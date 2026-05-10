# Proposal: 日志系统

## Intent
为 CodeLoom 提供可配置的文件日志系统，记录关键操作入口/出口、异常分支和重要参数，包含时间戳、PID、TID，支持文件绕接和数量限制。替代当前 148 处裸 `eprintln!`，让 `eprintln!` 回归 CLI 用户界面职责，日志记录独立到文件。

## Scope

In scope:
- 新建 `src/logger.rs` 日志模块，实现文件写入、绕接、清理
- 自定义日志宏（`log_info!` 等），release 和 debug 均编译保留代码，运行时由配置控制开关
- 配置文件 `config.yaml` 新增 `logging` 节：enabled、level、max_file_size_mb、max_files
- 在关键入口/出口/异常分支打点（~25 处）：CLI 命令、MCP 工具、Clang 解析、embedding、FTS5 索引
- 日志格式：`时间戳(ms) [PID] [TID] [LEVEL] [module] message`
- 文件绕接：单文件超 max_file_size_mb 时创建新文件，文件名固定为创建时的时间戳，不重命名
- 文件数量限制：`logs/` 目录下文件数超 max_files 时删除最旧的文件

Out of scope:
- CLI 界面不增加任何日志相关参数（不暴露日志配置给命令行）
- 不替换已有 `eprintln!`（CLI 用户输出和文件日志分开，各自独立）
- 不引入第三方日志框架（不用 `fern`/`log4rs`/`tracing`，自建够用）
- 不修改 stderr 输出行为

## Approach
纯自建最小化日志模块。自定义宏 `log_info!`/`log_warn!`/`log_error!`/`log_debug!`，在 release 和 debug 下均可用，由配置文件控制开关。日志写入 `~/.codeloom/logs/` 目录，文件以创建时间戳命名。绕接时新文件用当前时间戳命名，旧文件保持原名。定期检查文件数量，超限删最旧。

## Capabilities

### New Capabilities
- `logging-system`: 文件日志系统，支持配置化开关、级别过滤、文件绕接和数量限制
