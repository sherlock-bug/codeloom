# Proposal: fix-abstract-interface-cross-tu

## Intent
修复抽象接口/纯虚类的指针跨 TU 调用边缺失问题（G18）。根因是 `ast.rs` 的 MemberExpr 类名提取器从 `ImplicitCastExpr.type.qualType` 解析类名时，未去除 `const` 前缀修饰，导致目标名被误拼为 `const AbstractWorker::DoOp` 而非 `AbstractWorker::DoOp`，与 DB 节点名不匹配。

## Scope
In scope:
- `ast.rs` 的 `extract_call_targets` 中 MemberExpr 处理：从 `qualType` 提取类名前去除 `const` 等 cv-qualifier
- 验证：G18d/G18e 断言通过（197 → 199 全绿）
- 回归验证：G17（namespace 跨 TU 边）不受影响

Out of scope:
- 其他场景的 const 处理（如 `const Client* ptr` 参数是否也会触发 — 待确认但同方案覆盖）
- filter.py 的修改（不必要 — filter 输出已完整）
- 非 C++ 语言

## Approach
1. **修复类名提取**：在 `ast.rs:1251` 的 `qualType` 解析逻辑中，增加 `trim_start_matches("const ")` 和 `trim_start_matches("volatile ")`，确保提取的类名不包含 cv-qualifier。
2. **验证**：跑 full assertions（G18d/e 预期通过，G17 回归验证）

## Capabilities

### New Capabilities
- `abstract-interface-cross-tu-calls`: 通过抽象接口/纯虚类指针的跨 TU 调用（如 `env_->GetChildren()` → `Env::GetChildren`）产生正确的 `calls:` 边
