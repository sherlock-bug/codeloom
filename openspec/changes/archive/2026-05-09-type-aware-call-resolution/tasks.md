     1|# Tasks
     2|
     3|## 1. 内置符号
     4|
     5|- [x] 1.1 在 `src/storage/symbols.rs` 新增 `insert_builtin_symbols()` — 插入约 70 个 std 合成符号节点
     6|- [x] 1.2 覆盖容器方法：std::vector/map/unordered_map/set/string/unique_ptr/shared_ptr 的常用方法（~40 个）
     7|- [x] 1.3 覆盖自由模板函数：std::find/sort/copy/move/swap 等 `<algorithm>/<utility>` 函数（~30 个）
     8|- [x] 1.4 在 `smart_index` 中首次索引前调用 `insert_builtin_symbols()`
     9|- [x] 1.5 验证：`__builtin__` 符号不进入 FTS5/向量索引（查询 `fts5_sym` 和 `symbol_*_vec_*` 表）
    10|
    11|## 2. 成员调用类型感知解析
    12|
    13|- [x] 2.1 在 `cpp.rs` 新增 `scan_local_declarations()` — 扫描函数体的变量声明，建 `HashMap<String, String>`
    14|- [x] 2.2 处理 `declaration` 节点：提取 `type` 字段 + `declarator` 的变量名
    15|- [x] 2.3 处理 `parameter_declaration`：从函数签名提取参数名及类型
    16|- [x] 2.4 简单 `auto` 推断：`auto x = new MyClass()` → 识别为 MyClass
    17|- [x] 2.5 改造 `extract_calls()` → `extract_calls_with_types()`：检测 `field_expression`，查表，生成限定边
    18|- [x] 2.6 `this->method()` 特殊处理：用当前 `parent_class` 直接限定
    19|- [x] 2.7 未查到类型时降级：生成 `calls:method`（保持现有行为，不退化）
    20|- [x] 2.8 自由函数调用：`function` 非 `field_expression` → 行为不变
    21|
    22|## 3. call_graph.rs 模块化
    23|
    24|- [x] 3.1 将 `mcp/mod.rs` 的 `get_call_graph()` 函数体移至 `src/query/call_graph.rs`
    25|- [x] 3.2 将 `traverse_calls()` 一同搬移，改为私有函数
    26|- [x] 3.3 `mcp/mod.rs` 中的 `codeloom_get_call_graph` handler 改为调用 `crate::query::call_graph::get_call_graph()`
    27|- [x] 3.4 删除 `mcp/mod.rs` 中原有 `get_call_graph` 和 `traverse_calls` 定义
    28|- [x] 3.5 验证：`cargo build` 成功，MCP 工具 `codeloom_get_call_graph` 输出格式不变
    29|
    30|## 4. 测试
    31|
    32|- [x] 4.1 单元测试：`scan_local_declarations()` 正确解析 `MyClass* p`、`std::vector<int> v`、`auto x = new Foo()`
    33|- [x] 4.2 单元测试：成员调用 `obj->method()` 生成 `calls:MyClass::method`
    34|- [x] 4.3 单元测试：`this->method()` 生成 `calls:ParentClass::method`
    35|- [x] 4.4 单元测试：自由函数调用不受影响
    36|- [x] 4.5 单元测试：未查到类型降级到纯名称
    37|- [x] 4.6 集成测试：用 fixture 文件（含成员调用、自由函数、this 调用），索引后验证调用图
    38|- [x] 4.7 回归测试：`cargo test` 全过
    39|
    40|## 5. 部署验证
    41|
    42|- [x] 5.1 编译 `cargo build --release`，零警告
    43|- [x] 5.2 leveldb 清库重索引：`codeloom index leveldb`
    44|- [x] 5.3 MCP 调用图验证：`codeloom_get_call_graph("DB::Put", "leveldb", "main", "callees", 2)` 输出正确
    45|- [x] 5.4 确认内置符号不污染搜索：搜 "push_back" 只返回 leveldb 实际符号
    46|- [x] 5.5 更新 README.md
    47|