# Delta for vec0-static-compilation

## ADDED Requirements

### Requirement: 静态编译 sqlite-vec
系统 SHALL 将 sqlite-vec C 源码编译进 CodeLoom 二进制，通过静态链接 `vendor/vec0_static.c` 和 `vendor/sqlite-vec/sqlite-vec.c` 实现 vec0 功能，无需运行时加载外部动态库。

#### Scenario: musl 静态编译
- GIVEN 目标平台为 Linux x86_64-musl
- WHEN 执行 `cargo build --release --target x86_64-unknown-linux-musl`
- THEN 生成的二进制 SHALL 不依赖任何 `.so` 动态库，且 vec0 向量功能正常工作

#### Scenario: 移除 load_extension 调用
- GIVEN CodeLoom 初始化 SQLite 连接
- WHEN 启用 vec0 扩展时
- THEN 系统 SHALL 调用 `vec0_static_init(db)` 而非 `load_extension("vec0.so")`
