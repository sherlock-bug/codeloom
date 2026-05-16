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

}  // namespace cross_tu
