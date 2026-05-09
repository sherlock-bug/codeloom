# Tasks

## 1. 模板声明标记

- [x] 1.1 修改 `template_declaration` match arm：将 body 中的 class 标记为 `template_class`，function 标记为 `template_function`
- [x] 1.2 提取模板参数列表，为每个参数建 `template_param:NAME[KIND]` 边

## 2. 自定义模板使用

- [x] 2.1 实现 `extract_container_edges()` — 统一检测 std 容器和自定义模板使用
- [x] 2.2 实现双模式 `resolve_or_max()` 辅助函数
- [x] 2.3 对匹配的自定义模板建 `template_use:NAME<ARGS>` 边

## 3. std 容器类间关系

- [x] 3.1 实现 `std_container_edge()` 映射表（18 种容器，4 类关系）
- [x] 3.2 在 `extract_field` 中检测 `std::container<T>` 并建对应边
- [x] 3.3 map 容器 key/value 分别解析（修复原本整体 `resolve_or_max("int,User")` 查不到的问题）

## 4. 测试与验证

- [x] 4.1 `cargo test` 全量通过（75/75）
- [x] 4.2 验证模板类搜索：tmptest → `template_class` kind 正确出现
- [x] 4.3 验证模板参数边：`template_param:T[type]` `template_param:N[non_type]`
- [x] 4.4 验证容器聚合边：cont_test → `aggregate:User` `owns:User` `map_value:User` 全部正确解析
- [x] 4.5 验证模板使用边：`template_use:MyVector<User>` → 双模式 to 正确

## 5. 边解析 bug 修复

- [x] 5.1 `smart.rs`: `resolve_target` 忽略提取器的 `usize::MAX` → 检查 `tgt_idx == usize::MAX` 时直接 to=0
- [x] 5.2 `smart.rs`: `LIKE '%X%'` 对单字符类型名误匹配一切 → 改为 `LIKE '%::X'` 后缀匹配
- [x] 5.3 `cpp.rs`: map 容器的 key/value 共用同一个 `resolve_or_max("int,User")` → 分别提取类型名分别解析

## 6. 文档

- [x] 6.1 符号类型表和边类型表已纳入 SDD spec（无需 README 更新——无集中类型表）
