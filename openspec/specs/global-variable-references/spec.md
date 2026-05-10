# global-variable-references Specification

## Purpose
TBD - created by archiving change symbol-usage-edges. Update Purpose after archive.
## Requirements
### Requirement: 全局变量引用边提取
系统 SHALL 在索引函数体时检测对全局变量和静态变量的引用，并创建 `references:var_name` 边关联函数与变量符号。

#### Scenario: 函数读取全局变量
- GIVEN 全局变量 `g_config`（kind=global）存在，函数体包含 `g_config.reload()`
- WHEN 索引该文件
- THEN 创建 `references:g_config` 边，source 为该函数符号，target 为 `g_config` 符号

#### Scenario: 函数写入静态变量
- GIVEN 静态变量 `s_counter`（kind=static_var）在函数体内声明，另一函数体包含 `s_counter++`
- WHEN 索引该文件
- THEN 创建 `references:s_counter` 边关联到引用函数

#### Scenario: 本地变量不创建引用边
- GIVEN 函数体声明了局部变量 `int result = 0;` 并后续引用 `result`
- WHEN 索引该文件
- THEN 不创建 references 边（`result` 不是 global/static_var 符号）

#### Scenario: 函数参数不创建引用边
- GIVEN 函数 `void foo(int x)` 在函数体中引用 `x`
- WHEN 索引该文件
- THEN 不创建 references 边（`x` 是参数而非全局变量）

### Requirement: 全局变量引用边可搜索
系统 SHALL 支持通过边类型 `references:var_name` 查询哪些函数引用了指定全局或静态变量。

#### Scenario: 搜索引用全局变量的函数
- GIVEN 已索引的 repo 中有 3 个函数引用了 `g_debug_mode`
- WHEN 查询 target 为 `g_debug_mode` 符号且 edge_type 前缀为 `references:g_debug_mode`
- THEN 返回 3 个函数符号

