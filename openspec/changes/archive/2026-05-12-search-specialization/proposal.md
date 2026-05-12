# Proposal: search-specialization

## Intent
当前搜索工具返回统一的 `name + kind + file_path + line_start + signature + snippet`。同一个搜索结果，类有基类信息、枚举有值列表、文档有章节路径——但 LLM 看不到这些，得多次调 inspect 才能判断是不是它要的。

## Scope
In scope:
- 三个搜索工具（search / semantic_search / list_symbols）每个结果附带 `id`
- 根据结果节点类型附加额外字段：
  - **class/struct**：基类列表 + 成员数 + 方法数
  - **enum**：枚举值数
  - **function/method**：签名（已有）+ 所属类名（parent_class）
  - **section**：章节路径（section_path）+ 层级 + 子节点数
  - **chunk**：所属 section 名称
  - 其他：不变
- 额外字段控制在 1-2 个，不膨胀结果体积

Out of scope:
- 不改 inspect（那是 inspect-specialization）
- 不改索引层
- 不改搜索算法的打分逻辑

## Approach
查询结果拼接阶段，判断命中节点的 `kind` 和 `node_type`，额外 JOIN 一次 edges 表获取计数信息（成员数/方法数/子节点数等），或从 `attrs` JSON 提取。
