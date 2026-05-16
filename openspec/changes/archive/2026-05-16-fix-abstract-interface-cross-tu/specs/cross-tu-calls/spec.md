# Delta for cross-tu-calls

## ADDED Requirements

### Requirement: 抽象接口指针跨 TU 调用边
系统 SHALL 为通过抽象接口/纯虚类指针发起的跨 TU 调用生成 `calls:` 边。如 `env_->GetChildren(...)` → `Env::GetChildren`，`w->DoOp(v)` → `AbstractWorker::DoOp`。

#### Scenario: 纯虚类指针调用
- GIVEN 类 `AbstractWorker` 声明纯虚方法 `DoOp(int)`
- AND 函数 `call_abstract_worker(AbstractWorker* w, int v)` 调用 `w->DoOp(v)`
- AND `AbstractWorker` 的定义在另一 TU 中
- WHEN 索引后查询 `call_abstract_worker` 的调用边
- THEN 调用图中 SHALL 包含 `call_abstract_worker` → `AbstractWorker::DoOp` 边
