# Proposal: neighbor-graph-skip-members

## Intent
当前 `neighbor_graph` 的 `depth` 可配 1 或 2，但 `depth=2` 与 `impact_analysis(radius=2)` 功能重叠，LLM 容易选错。且查一个类时，它的成员方法/字段不应算"邻居"——跳过后延长一级到成员引用的外部符号，对 LLM 更有用。

## Scope
In scope:
- 锁定 `depth` 为 1，移除 depth=2 选项
- 当目标符号为 class/struct 时：跳过其直接成员（kind=field/method）的 `contains:` 边，将这些成员的外部引用（calls/uses 等）暴露为当前符号的邻居
- 当目标符号为其他类型时：逻辑不变

Out of scope:
- 不改 impact_analysis（它保持 depth=N 的传递闭包能力，与 neighbor_graph 划清界限）
- 不加新参数

## Approach
在 `neighbor_map` 函数中，检测当前符号的 `kind`。如果是 class/struct，先获取其成员的 id 列表，再从 edges 中获取这些成员指向的外部符号。返回结果按边类型分组，与当前逻辑一致。
