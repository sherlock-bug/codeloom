// Caller — uses cross_tu::Calculator methods defined in callee.cc.
// This TU does NOT have CXXMethodDecl for Calculator::add etc.
// Call edges from here to those methods are lost because they're
// not in name_to_id during this TU's processing.

#include "cross_tu_calls.h"

namespace cross_tu {

int compute() {
    Calculator calc;
    return calc.add(40, 2) + calc.multiply(6, 7);
}

int compute_direct() {
    Calculator calc;
    return calc.add(10, 20);
}

// Pointer-based call — reproduces the env_->GetChildren() pattern.
// Here, ptr is Client*, and ptr->process(val) makes Clang produce
// ImplicitCastExpr with qualType="Client *", so the MemberExpr
// handler resolves class_name="Client" and builds
// target_name="Client::process" (no namespace, global scope).
// Uses a passed-in pointer to avoid CXXNewExpr filter issues.
int call_via_pointer(Client* ptr, int val) {
    int result = ptr->process(val);
    return result;
}

// Pointer-based call via cross_tu::Handler — reproduces the exact leveldb
// pattern: namespace-qualified class called via pointer.
// handler is Handler*, so Clang's qualType should be "cross_tu::Handler *"
// leading to target_name="cross_tu::Handler::handle".
// DB stores name="Handler::handle", namespace="cross_tu".
// Uses a passed-in pointer to avoid CXXNewExpr filter issues.
int call_handler(Handler* h, int val) {
    int result = h->handle(val);
    return result;
}

}  // namespace cross_tu

// Abstract interface call — reproduces the env_->GetChildren() pattern.
// ptr is AbstractWorker*, DoOp is pure virtual. The MemberExpr in Clang
// resolves to AbstractWorker::DoOp (the static type).
int call_abstract_worker(AbstractWorker* w, int v) {
    int result = w->DoOp(v);
    return result;
}
