# Design: fix-abstract-interface-cross-tu

## Context
抽象接口指针跨 TU 调用边缺失。根因已定位到 `ast.rs:1251` 的 `extract_call_targets`：从 `ImplicitCastExpr.type.qualType` 提取类名前未去除 `const` 前缀，导致 `const AbstractWorker::DoOp` 而非 `AbstractWorker::DoOp`。

## Goals / Non-Goals
Goals:
- G18d/G18e 断言通过（抽象接口指针跨 TU 调用边补齐）
- leveldb `env_->GetChildren()` 场景修复

Non-Goals:
- filter.py 修改（无必要）
- G17 namespace 回归（不涉及）

## Decisions

### Decision: `qualType` 前缀去除 cv-qualifier
在 line 1251 增加 `trim_start_matches("const ").trim_start_matches("volatile ").trim_start_matches("constexpr ")`。理由：
- `const` 不影响类身份标识，提取时不应保留
- 最小改动（一行修改），无副作用
- 与原逻辑的 `trim_end_matches(" *")` 风格一致

## Risks / Trade-offs
| 风险 | 缓解措施 |
|------|---------|
| `const int*` 等非类类型被误匹配 | `trim_start_matches` 只在 `qualType` 以 `const ` 开头时生效，最终提取名仍需要 `!is_empty() && != "<bound member function type>"` 守卫 |
| 漏掉其他 cv-qualifier 如 `_Atomic` | 当前无此场景，遇到后留 issue 补充 |
