# Proposal: comprehensive-logging

## Why

CodeLoom 的日志系统基础设施（`src/logger.rs`）已完备——文件轮转、级别过滤、时间戳+线程ID——但全库 30K+ 行代码中仅 21 处使用了 `log_*!` 宏。8 个核心模块零日志，导致调试严重依赖直觉而非数据。以最近排查的"Clang 索引器头文件符号缺失"bug 为例，AST 解析过程中符号被标记为 `is_external`、路径匹配失败等关键决策点没有任何日志输出，只能手动 dump Clang AST 来逆向推断。

本次变更的目标是：将日志覆盖率扩展到每一个关键执行路径，使得未来所有调试都可以从日志中直接定位问题。

## What Changes

- **引 `log_*!` 到 8 个零日志模块**：ast.rs（AST 决策点）、tree_sitter.rs（文件收集）、query/search.rs（搜索查询）、calib/mod.rs（校准过程）、storage/（FTS、symbols、vector、schema 全部存储层）
- **出口加耗时标记**：每个模块的关键入口/退出点加耗时日志，可量化索引/搜索/校准的性能
- **常分支打日志**：异常路径、catch-all 分支、skip/ignore 决策点必须输出日志
- **规 `eprintln!` → `log_error!`**：诊断输出统一走日志通道，`println!` 仅限用户可见输出
- **认日志级别改为 debug**：默认 `info` → `debug`，新会话即可看到更详细的运行日志
- **AST 路径判定日志**：`is_project_file()` 的匹配结果、`is_external` 判定原因等关键决策点输出 debug 日志

## Capabilities

### New Capabilities
- `logging-system`: 日志系统的全面覆盖 —— 8 个模块新增结构化日志、耗时标记、异常路径日志、统一错误输出通道

### Modified Capabilities

## Impact

- 代码量：约 150-200 行新增日志调用（纯追加，不影响现有逻辑）
- 性能：log_* 在日志级别 < 配置级别时是空操作（`level > self.level` 时直接 return），高频路径正常。Debug 级别下 BufWriter 的 flush 有少量 I/O，但可控
- 测试：无影响，日志宏在 LOGGER 未初始化时是空操作，测试不会新增日志文件
- 配置：默认日志级别从 `info` 改为 `debug`（已部分实施），用户可通过 config.yaml 控制
