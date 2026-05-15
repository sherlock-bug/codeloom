# Delta for template-parsing

## ADDED Requirements

### Requirement: 模板实例保留规则
系统 SHALL 仅保留两类模板实例：①项目自有模板的实例化（如 `SkipList<const char*>`）；②知名 STL 容器的实例化且模板参数中包含项目类型（如 `vector<FileMetaData*>`）。纯内置类型参数的 STL 实例（如 `vector<int>`）不保留。STL 内部 detail 模板（`allocator`、`char_traits`、`is_*`、`remove_*`、`__*` 等）不保留。

#### Scenario: 项目模板实例保留
- GIVEN 项目模板 `SkipList` 被实例化为 `SkipList<const char*>`
- WHEN 索引器处理该 CTS 节点
- THEN 创建符号，`name` 为 `SkipList<const char*>`，`kind` 为 `template_instance`
- AND 建立 `instantiates:` 边指向主模板 `SkipList`

#### Scenario: STL 容器含项目类型保留
- GIVEN `vector<FileMetaData*>`，`FileMetaData` 是项目类型
- WHEN Python filter 检查模板参数
- THEN 保留该 CTS 节点
- AND Rust 层创建符号并建立 `uses_type:` 边指向 `FileMetaData`

#### Scenario: 纯内置类型参数丢弃
- GIVEN `vector<int>`，参数 `int` 是内置类型
- WHEN Python filter 检查模板参数
- THEN 丢弃该 CTS 节点，不创建符号

### Requirement: 模板实例命名区分
系统 SHALL 对模板实例使用带类型参数的完整名称（如 `foo<int, double>`），主模板使用简单名（如 `foo`），确保 `name_to_id` 不冲突。

#### Scenario: 实例命名
- GIVEN 函数模板 `foo` 被实例化为 `foo<int, double>`
- WHEN 索引器创建符号
- THEN 实例符号名 SHALL 为 `foo<int, double>`
- AND 主模板符号名 SHALL 为 `foo`

### Requirement: 知名 STL 容器/工具清单
系统 SHALL 将以下 STL 模板视为知名容器/工具，当其实例化且参数含项目类型时保留：`vector`、`map`、`set`、`deque`、`pair`、`unique_ptr`、`shared_ptr`、`string`、`basic_string`、`function`、`unordered_map`、`unordered_set`。

#### Scenario: 知名容器保留
- GIVEN `map<string, Version*>`，`Version` 是项目类型
- WHEN `_has_project_type_arg()` 检查参数
- THEN 参数 `Version*` 是项目类型 → 返回 `true`
- AND 该 CTS 节点被保留
