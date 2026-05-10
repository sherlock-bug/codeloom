     1|# Tasks
     2|
     3|## 1. auto 跨函数类型追踪
     4|
     5|- [x] 1.1 扩展 `try_resolve_auto()` — 检测右侧 `call_expression`，提取被调函数名
     6|- [x] 1.2 在同文件 `symbols` 中查被调函数的 `returns` 边，提取返回类型
     7|- [x] 1.3 Fast path：查同文件 `edges` 中 `returns:X` → 返回 `X`
     8|- [x] 1.4 `auto` 类型存入 `types` 表后，后续成员调用正常解析
     9|- [x] 1.5 未查到返回类型时 fallback `"auto"`
    10|
    11|## 2. 虚函数展开
    12|
    13|- [x] 2.1 在 `extract_calls_with_types()` 中：成员调用解析到基类方法后，查符号表的 `overrides` 边
    14|- [x] 2.2 对每个 override 生成 `calls_override:DerivedClass::method` 边
    15|- [x] 2.3 `call_graph.rs` 的 `traverse_calls` 改为匹配 `calls:%` OR `calls_override:%`
    16|- [x] 2.4 `resolve_target` 中 override 边也走正常解析逻辑
    17|
    18|## 3. 重载参数计数消歧
    19|
    20|- [x] 3.1 `extract_calls_with_types()` 中解析 `call_expression` 的 `arguments` 子节点
    21|- [x] 3.2 计数 `argument_list` 中的子节点（跳过逗号等非参数节点）
    22|- [x] 3.3 边标签加 `(N)` 后缀：`calls:func(2)`、`calls:ClassName::method(1)`
    23|- [x] 3.4 `resolve_target` 支持 `func(N)` 格式匹配
    24|- [x] 3.5 自由函数和成员调用都覆盖
    25|
    26|## 4. 测试
    27|
    28|- [x] 4.1 单元测试：`auto x = getFoo()` 通过 returns 边解析类型
    29|- [x] 4.2 单元测试：`auto x = UnknownFunc()` fallback `"auto"`
    30|- [x] 4.3 集成测试：基类虚函数调用 → 展开所有 override
    31|- [x] 4.4 单元测试：`func(a, b)` → `calls:func(2)`
    32|- [x] 4.5 回归测试：`cargo test` 全过
    33|
    34|## 5. 部署验证
    35|
    36|- [x] 5.1 编译 `cargo build --release`
    37|- [x] 5.2 leveldb 清库重索引验证 auto 和虚函数展开效果
    38|- [x] 5.3 更新 README.md
    39|