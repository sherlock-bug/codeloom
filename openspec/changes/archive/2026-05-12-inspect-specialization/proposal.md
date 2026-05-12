# Proposal: inspect-specialization

## Intent
当前 `inspect` 对所有节点类型返回统一的通用 edges 列表，LLM 拿到一个类/枚举/文件/文档时还要额外调 `neighbor_graph` 才能看到成员。需要让 `inspect` 根据节点类型智能返回 LLM 最关心的信息。

## Scope
In scope:
- **class/struct**：返回基类（inherits: 边）、成员变量（contains: + kind=field）、方法列表（contains: + kind=method）、模板参数（attrs.template_args）
- **enum**：返回枚举值列表（contains: + kind=enum_value）
- **file**：返回包含的顶层 section/code 符号列表（contains: 边，只返回 id+name，不展开详情）
- **section**：返回父 section（反向 contains:）、子 section（contains:）+ 子 chunk（contains:）+ 前后兄弟 section（同 file_path 按 id 排序）
- **chunk**：返回父 section id+name（反向 contains:）、前/后 chunk id（同父节点按 idx 排序）
- 其余节点类型（function/method/field/static_var/namespace 等）保持当前通用 edges 输出

Out of scope:
- 不改搜索结果格式（那是 search-specialization）
- 不改索引层
- 不加新工具

## Approach
在 `inspect_symbol()` 函数中，根据 `node_type` + `kind` 分支查询。class/enum/file/section/chunk 各一个分支，其余走 fallback（当前通用 edges）。
