# Delta for compile-commands-discovery

## ADDED Requirements

### Requirement: 自动搜索 compile_commands.json
系统 SHALL 在索引 C/C++ 项目时自动搜索 `compile_commands.json` 文件，搜索路径按优先级依次为：用户显式指定路径 → 项目根目录 → build/ → out/。

#### Scenario: 在 build 目录下找到
- GIVEN 项目根目录为 `/home/user/project`，存在 `build/compile_commands.json`
- WHEN 启动索引，用户未指定 `--compile-commands` 参数
- THEN 系统自动加载 `build/compile_commands.json` 作为编译参数来源

#### Scenario: 在项目根目录找到
- GIVEN 项目根目录为 `/home/user/project`，存在 `project/compile_commands.json`，但 build/ 和 out/ 下都不存在
- WHEN 启动索引
- THEN 系统加载 `project/compile_commands.json`

#### Scenario: 用户显式指定路径
- GIVEN `compile_commands.json` 位于非标准路径 `/tmp/cmds.json`
- WHEN 用户通过 `--compile-commands /tmp/cmds.json` 启动索引
- THEN 系统直接使用 `/tmp/cmds.json`，不执行默认搜索

#### Scenario: 未找到 compile_commands.json
- GIVEN 项目不存在任何 `compile_commands.json`
- WHEN 启动索引
- THEN 系统以降级模式运行：每个源文件单独执行 `clang -fsyntax-only -- <file>`，不使用额外编译参数

### Requirement: 解析编译命令中的参数
系统 SHALL 从 `compile_commands.json` 的每个条目中提取 `directory`（工作目录）、`command` 或 `arguments`（编译参数），并在调用 clang 子进程时使用这些参数。

#### Scenario: 使用 command 字段
- GIVEN `compile_commands.json` 中包含 `{"directory": "/build", "command": "clang++ -I/include -std=c++17 -c src/main.cpp"}`
- WHEN 系统解析该条目
- THEN 从 command 中提取 `-I/include -std=c++17`，在 `/build` 目录下执行 `clang -fsyntax-only -ast-dump=json -I/include -std=c++17 -- src/main.cpp`

#### Scenario: 使用 arguments 字段
- GIVEN `compile_commands.json` 中包含 `{"directory": "/build", "arguments": ["clang++", "-I/include", "-std=c++17", "-c", "src/main.cpp"]}`
- WHEN 系统解析该条目
- THEN 从 arguments 中提取编译参数，排除编译器名和 `-c` 标志，生成 `clang -fsyntax-only -ast-dump=json -I/include -std=c++17 -- src/main.cpp`

### Requirement: 翻译单元合并优化
系统 SHALL 识别 `compile_commands.json` 中同一编译命令覆盖多个源文件的情况（如通配符或构建系统生成的批量命令），按翻译单元组织 clang 调用以最小化子进程 spawn 次数。

#### Scenario: 多个源文件归属同一编译命令
- GIVEN `compile_commands.json` 中 `src/a.cpp` 和 `src/b.cpp` 具有相同编译参数
- WHEN 索引执行
- THEN 两个文件分别独立调用 clang（每个翻译单元一次），复用相同的 `-I` 和 `-D` 参数
