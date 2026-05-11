# Delta for header-file-path

## ADDED Requirements

### Requirement: 头文件符号文件路径正确
系统 SHALL 为头文件中声明的符号提供正确的文件路径，而非回退到翻译单元文件路径。

#### Scenario: 头文件符号搜索跳转
- GIVEN 头文件 `config.h` 中定义了类 `Config`
- AND `config.cpp` 中 `#include "config.h"`
- WHEN 用户搜索 `Config` 符号并点击结果
- THEN 跳转目标 SHALL 指向 `config.h` 的正确行号
- AND 不应指向 `config.cpp`

#### Scenario: 头文件符号数量正确
- GIVEN fixture `tests/fixtures/clang_test/` 包含 `sample.hpp` 和 `sample.cpp`
- WHEN 索引该 fixture
- THEN `sample.hpp` 中的 21 个符号 SHALL 全部正确指向 `tests/fixtures/clang_test/sample.hpp`
- AND `sample.cpp` 中只有 2 个真正定义在翻译单元中的符号指向 `sample.cpp`

### Requirement: 外部符号判定正确
系统 SHALL 使用当前文件上下文判定符号是否为外部（系统/非项目）符号，而非使用翻译单元文件路径。

#### Scenario: 系统头文件外部判定
- GIVEN 翻译单元 `project/app.cpp` 包含了 `<string>` 和 `"config.h"`
- WHEN 解析器处理 `<string>` 中的符号
- THEN `is_external` SHALL 为 true（系统头文件文件路径不在项目中）
- AND `config.h` 中的符号 `is_external` SHALL 为 false（项目头文件在项目根下）

### Requirement: 已有测试全部通过
本变更 SHALL 不破坏任何现有测试。

#### Scenario: 回归验证
- GIVEN 完成代码修改
- WHEN 运行 `cargo test` 和 `cargo test --test integration -- --ignored`
- THEN 所有 73 个测试 SHALL 通过（63 单元 + 10 集成）
