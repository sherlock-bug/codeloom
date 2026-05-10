# Delta for logging-system

## ADDED Requirements

### Requirement: 日志系统初始化
系统 SHALL 在进程启动时根据配置文件初始化日志系统，若配置中 `logging.enabled` 为 true 则创建日志文件并开始记录，若为 false 则所有日志宏编译为空操作。

#### Scenario: 配置启用时初始化
- GIVEN `config.yaml` 中 `logging.enabled: true`
- WHEN 进程启动
- THEN 系统 SHALL 在 `~/.codeloom/logs/` 目录下创建日志文件，文件名格式为 `codeloom_{YYYYMMDD-HHMMSS}_{毫秒}.log`
- AND 后续 `log_info!` 等宏调用 SHALL 写入该文件

#### Scenario: 配置禁用时跳过
- GIVEN `config.yaml` 中 `logging.enabled: false`
- WHEN 进程启动
- THEN 系统 SHALL 不创建日志文件
- AND 所有 `log_info!` 等宏调用 SHALL 为空操作，不产生任何 I/O

#### Scenario: 日志目录自动创建
- GIVEN `~/.codeloom/logs/` 目录不存在
- WHEN 日志系统初始化且 enabled 为 true
- THEN 系统 SHALL 自动创建该目录

### Requirement: 日志级别过滤
系统 SHALL 支持四个日志级别（ERROR > WARN > INFO > DEBUG），仅记录大于等于配置级别的日志。

#### Scenario: INFO 级别过滤
- GIVEN 配置 `logging.level: "info"`
- WHEN 代码调用 `log_debug!("详细调试信息")`
- THEN 该条日志 SHALL 不被记录
- AND 代码调用 `log_info!("操作完成")`
- THEN 该条日志 SHALL 被记录

#### Scenario: DEBUG 级别输出全部
- GIVEN 配置 `logging.level: "debug"`
- WHEN 代码调用任意级别的日志宏
- THEN 所有日志 SHALL 被记录

### Requirement: 日志格式
每条日志 SHALL 包含时间戳到毫秒、进程号、线程号、日志级别、模块标识和消息内容。

#### Scenario: 标准日志行格式
- GIVEN 日志系统已初始化且 enabled 为 true
- WHEN 代码调用 `log_info!("indexer::clang", "parse start: file={}", path)`
- THEN 输出行 SHALL 格式为 `2026-05-10 15:30:12.345 [12345:67890] INFO  indexer::clang: parse start: file=src/main.cpp`
- AND 级别标签 SHALL 右对齐到 5 字符宽度（如 ` ERROR`、`  WARN`、`  INFO`、` DEBUG`）

### Requirement: 文件绕接
系统 SHALL 在单日志文件大小超过配置的 `max_file_size_mb` 时创建新日志文件，旧文件名保持不变。

#### Scenario: 文件超限绕接
- GIVEN 当前日志文件 `codeloom_20260510-150000_000.log` 大小为 50MB
- AND 配置 `max_file_size_mb: 50`
- WHEN 下一条日志写入
- THEN 系统 SHALL 关闭当前文件
- AND SHALL 创建新文件 `codeloom_20260510-153000_000.log`（以当前时间戳命名）
- AND 旧文件保持原名不重命名
- AND 后续日志 SHALL 写入新文件

### Requirement: 文件数量限制
系统 SHALL 在 `logs/` 目录下日志文件数超过配置的 `max_files` 时，按修改时间升序删除最旧的文件直至数量符合限制。

#### Scenario: 文件数超限清理
- GIVEN `logs/` 目录下有 11 个日志文件
- AND 配置 `max_files: 10`
- WHEN 绕接触发后创建新文件
- THEN 系统 SHALL 删除修改时间最早的那个文件
- AND 目录下 SHALL 保留 10 个文件

#### Scenario: 启动时清理
- GIVEN `logs/` 目录下有 15 个日志文件
- AND 配置 `max_files: 10`
- WHEN 日志系统初始化
- THEN 系统 SHALL 删除修改时间最早的 5 个文件
- AND 目录下 SHALL 保留 10 个文件

### Requirement: 配置文件定义
系统 SHALL 在 `config.yaml` 中支持 `logging` 配置节，包含 enabled、level、max_file_size_mb、max_files 四个字段，所有字段均有默认值。

#### Scenario: 完整日志配置
- GIVEN `config.yaml` 包含：
  ```yaml
  logging:
    enabled: true
    level: "info"
    max_file_size_mb: 50
    max_files: 10
  ```
- WHEN 系统加载配置
- THEN 日志系统 SHALL 按此配置运行

#### Scenario: 省略日志配置时的默认值
- GIVEN `config.yaml` 中无 `logging` 节
- WHEN 系统加载配置
- THEN 日志系统 SHALL 使用默认值：enabled=true, level="info", max_file_size_mb=50, max_files=10

### Requirement: 关键入口出口打点
系统 SHALL 在 CLI 命令、MCP 工具调用、Clang 解析、embedding 和 FTS5 索引的关键入口和出口记录 INFO 级别日志，在异常分支记录 WARN 或 ERROR 级别日志。

#### Scenario: CLI index 命令打点
- GIVEN 日志启用
- WHEN 执行 `codeloom index --repo myrepo --branch master`
- THEN 入口 SHALL 记录：repo、branch、文件数
- AND 出口 SHALL 记录：耗时、符号总数、边总数

#### Scenario: MCP 工具调用打点
- GIVEN 日志启用
- WHEN MCP 收到 `tools/call` 请求（如 `codeloom_search`）
- THEN 入口 SHALL 记录：工具名、关键参数（截断至 200 字符）
- AND 出口 SHALL 记录：结果数、耗时

#### Scenario: Clang 解析异常打点
- GIVEN Clang 解析某个翻译单元失败
- WHEN 错误被捕获
- THEN 系统 SHALL 记录 WARN 级别日志：文件路径、错误信息
- AND SHALL 继续处理后续文件不中断

#### Scenario: Embedding 批量失败打点
- GIVEN embedding API 调用返回错误
- WHEN 重试仍未成功
- THEN 系统 SHALL 记录 WARN 级别日志：批次大小、错误信息、重试次数

### Requirement: 日志宏空操作安全
日志宏在日志未初始化或禁用时 SHALL 不 panic、不写入、不产生任何副作用。

#### Scenario: 未初始化时调用
- GIVEN 日志系统尚未初始化（如 `logger::init()` 未调用）
- WHEN 代码调用 `log_info!("test", "msg")`
- THEN 该调用 SHALL 安全返回，不 panic
- AND SHALL 不产生任何文件 I/O

#### Scenario: 配置禁用后调用
- GIVEN 日志系统已初始化但 `logging.enabled: false`
- WHEN 代码调用 `log_error!("mcp", "fatal error")`
- THEN 该调用 SHALL 安全返回，不写入文件

### Requirement: 移除 log 和 env_logger 依赖
系统 SHALL 从 Cargo.toml 移除 `log` 和 `env_logger` crate，并从 `src/main.rs` 移除 `env_logger::init()` 调用，替换为 `logger::init(&config)`。

#### Scenario: 依赖清理
- GIVEN 日志模块已实现
- WHEN 编译项目
- THEN `log` 和 `env_logger` SHALL 不在 Cargo.toml 依赖列表中
- AND `main.rs` SHALL 调用 `logger::init(&config)` 而非 `env_logger::init()`
- AND `cargo build` SHALL 成功
