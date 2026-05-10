# Tasks

## 1. 创建日志模块

- [x] 1.1 新建 `src/logger.rs`，实现 Logger 结构体和所有公开接口
  - Logger 结构体：file, level, max_size, max_files, enabled
  - `init(config: &LoggingConfig)` — 初始化全局单例
  - `log(level, module, message)` — 格式化并写入
  - `rotate()` — 检查文件大小，必要时绕接
  - `cleanup()` — 删除超出数量限制的最旧文件
- [x] 1.2 实现日志宏：`log_error!`、`log_warn!`、`log_info!`、`log_debug!`
  - 四个宏接受 `(module: &str, format_str, args...)` 参数
  - 未初始化或 disabled 时为空操作不 panic
- [x] 1.3 实现绕接逻辑：每次 write 前检查 `file.len() > max_size`，超限则 rotate
- [x] 1.4 实现文件清理：启动时和绕接后检查 `logs/` 文件数 > max_files，删除最旧文件

## 2. 修改配置模块

- [x] 2.1 在 `src/config/mod.rs` 新增 `LoggingConfig` 结构体，含 `enabled`/`level`/`max_file_size_mb`/`max_files` 字段
  - 所有字段实现 `Default` trait，默认值：enabled=false, level="info", max_file_size_mb=50, max_files=10
- [x] 2.2 在 `Config` 根结构体添加 `logging: Option<LoggingConfig>` 字段

## 3. 修改入口

- [x] 3.1 修改 `src/main.rs`：用 `logger::init(&config.logging)` 替换 `env_logger::init()`
- [x] 3.2 修改 `src/cli/mod.rs` 主流程：索引操作入口/出口添加日志打点

## 4. 清理依赖

- [x] 4.1 从 `Cargo.toml` 移除 `log = "0.4"` 和 `env_logger = "0.11"`
- [x] 4.2 `cargo check` 确认无编译错误

## 5. 关键打点

- [x] 5.1 CLI index 入口/出口：`log_info!("cli", "index start/done: ...")` 含耗时和汇总统计
- [ ] 5.2 CLI clean 入口：待后续补充
- [x] 5.3 MCP 工具调用入口/出口：工具名、关键参数（截断至 200 字符）、结果长度
- [x] 5.4 MCP 异常：日志出口标记 `is_err=true`
- [x] 5.5 Clang 解析入口：`log_debug!("indexer::clang", "parse start: {} translation units")`
- [x] 5.6 Clang 解析失败：保留原有 `eprintln!`，日志由上层错误捕获
- [ ] 5.7 Embedding 批量：待后续补充
- [ ] 5.8 Embedding 失败：待后续补充
- [x] 5.9 FTS5 索引：`log_debug!("indexer", "fts5: {} symbols indexed")` / `log_warn!` 错误
- [ ] 5.10 smart.rs 增量判断：待后续补充

## 6. 测试

- [x] 6.1 现有测试全部通过：`cargo test` 11/11 passed
- [ ] 6.2 单元测试（logger）：待后续补充
- [ ] 6.3 绕接和清理测试：待后续补充
- [ ] 6.4 集成测试：待后续补充

## 7. 文档

- [x] 7.1 更新 README.md：添加日志功能说明（配置方式、日志位置、级别过滤）
- [x] 7.2 更新 AGENTS.md：新增「日志诊断」章节
