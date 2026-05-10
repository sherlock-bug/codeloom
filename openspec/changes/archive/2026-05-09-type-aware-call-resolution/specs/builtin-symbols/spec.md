# Delta for builtin-symbols

## ADDED Requirements

### Requirement: 预置 C++ 标准库内置符号
系统 SHALL 在首次索引仓库前，向 `symbols` 表插入约 70 个 C++ 标准库合成符号节点（容器方法和算法函数），repo 字段为 `__builtin__`，作为调用图解析的 target 节点。

#### Scenario: 容器方法解析有 target
- GIVEN 已索引 leveldb 仓库，索引前已插入内置符号
- WHEN 查询 `db->Put()` 的调用图
- THEN `calls:leveldb::DB::Put` 的内部调用若含 `calls:std::string::append`，该边应指向内置符号节点而非 unresolved (to=0)

#### Scenario: 算法函数解析有 target
- GIVEN 已索引项目仓库
- WHEN 解析到 `std::sort(vec.begin(), vec.end())`
- THEN 调用边 `calls:std::sort` 的 target_id SHALL 指向内置符号节点

### Requirement: 内置符号不参与搜索
内置符号 SHALL 不参与 FTS5 全文搜索和向量化语义搜索，仅作为图中节点存在。

#### Scenario: FTS5 搜索不返回内置符号
- GIVEN 数据库中有 `__builtin__` repo 的符号和 leveldb repo 的符号
- WHEN 用户搜索 "push_back"
- THEN 搜索结果 SHALL 仅包含 leveldb 仓库中实际出现的符号，不包含内置符号

#### Scenario: 向量索引跳过的内置符号
- GIVEN 索引 leveldb 仓库时触发了 `index_vectors()`
- THEN `symbol_name_vec_leveldb` 和 `symbol_comment_vec_leveldb` 向量表 SHALL 不包含 `__builtin__` 符号的向量

### Requirement: 内置符号列表涵盖常用容器和算法
内置符号集 SHALL 至少包含以下 C++ 标准库方法：std::vector、std::map、std::unordered_map、std::set、std::string、std::unique_ptr、std::shared_ptr 的常用方法，以及 `<algorithm>` 中的 std::find、std::sort、std::copy 等常用自由函数模板。

#### Scenario: 容器方法覆盖
- GIVEN 代码中使用 `std::vector<int> v; v.push_back(1); v.size(); v.empty();`
- THEN 所有上述调用的 target_id SHALL 指向对应的内置符号节点

## REMOVED Requirements
- 无
