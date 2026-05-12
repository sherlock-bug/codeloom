# Tasks

## 1. 修复 codeloom_index 死链

- [x] 1.1 `codeloom_index` 描述中 "用codeloom_status确认状态" → "用codeloom_list_repos检查是否已索引"

## 2. 修正 codeloom_schema 描述语气

- [x] 2.1 `codeloom_schema` 描述中 "调用图类工具前须先调用此工具" → "拿不准参数值时调用此工具查看可用节点类型和边类型枚举"

## 3. 提升 codeloom_get_call_graph 竞争区分度

- [x] 3.1 `codeloom_get_call_graph` 描述中 "**唯一方式**" → "**首选方式**"
- [x] 3.2 末尾加 "优于多次调neighbor_graph拼凑——一次到位且带递归深度控制"

## 4. 精简 codeloom_list_repos 输出格式噪音

- [x] 4.1 末尾输出示例句删除，以"无需任何参数。"结尾

## 5. 编译验证

- [x] 5.1 `cargo build --release` 编译通过（clean+rebuild 验证）
- [x] 5.2 确认 `tools_list()` 返回的新描述正确（4项均通过）

## 6. 更新 README.md

- [x] 6.1 无变更（仅改描述文本，不影响用户文档）
