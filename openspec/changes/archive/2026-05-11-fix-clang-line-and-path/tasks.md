## 1. Python filter 子节点路径传播修复

- [x] 1.1 修改 `~/.codeloom/scripts/clang_filter.py` 中 `filter_node()`，子节点 `loc.file` 传播用 `abs_node_file` 替代 `node_file`
- [x] 1.2 用真实 C++ 文件跑一遍 pipeline 验证输出 JSON 中所有子节点都有绝对路径

## 2. Rust 侧 offset→line 换算

- [x] 2.1 在 `ast.rs` 中实现 `LineIndex` 结构体：接受源文件路径，扫描建 `Vec<u32>` 换行符偏移表，提供 `offset_to_line(offset) -> u32` 方法（binary_search）
- [x] 2.2 实现缓存机制：`OnceLock<HashMap<String, LineIndex>>`，每个源文件只扫一次
- [x] 2.3 修改 `get_range()`：当 `range.begin.line` 不存在（或为 0）时，使用 `range.begin.offset` 通过 `LineIndex` 换算行号；`range.end` 同理，不存在则回退到 `line_start`
- [x] 2.4 `cargo check` 验证编译通过

## 3. Rust 侧 is_project_file 防御性修复

- [x] 3.1 修改 `is_project_file()`：在 `startswith(project_root)` 前判断路径是否绝对，相对则拼上 `project_root` 再比较
- [x] 3.2 `cargo check` 验证编译通过

## 4. 端到端集成测试

- [x] 4.1 创建 `tests/fixtures/line_and_path/` 目录，编写：
  - `include/my_class.h` — 定义类 `MyClass`，含方法 `int calc(int x)`
  - `main.cpp` — `#include "my_class.h"`，实现 `MyClass::calc`，含函数 `int helper()` 调用 `calc`
- [x] 4.2 创建集成测试 `tests/integration.rs` 中新增 `test_clang_line_and_path`：
  - 索引 fixture 目录
  - 查 DB：验证 `MyClass` 符号 `line_start != 0`、`is_external=false`、`file_path` 指向 `.h`
  - 查 DB：验证有 `calc` 方法的参数或调用边
  - 查 DB：验证 `helper` 函数 `line_start != 0`
- [x] 4.3 `cargo test --test integration test_clang_line_and_path` 通过

## 5. 验证

- [x] 5.1 `cargo test` 全部通过（已有测试不受影响）
- [x] 5.2 （可选）用 leveldb 实际索引验证头文件符号行号非零
