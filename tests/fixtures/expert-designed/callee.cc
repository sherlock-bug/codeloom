// Callee — defines cross_tu::Calculator methods.
// These ARE extracted as CXXMethodDecl with bodies.
// This file is in the same fixture as caller.cc, so both are indexed.

#include "cross_tu_calls.h"

namespace cross_tu {

int Calculator::add(int a, int b) {
    return a + b;
}

int Calculator::multiply(int a, int b) {
    return a * b;
}

}  // namespace cross_tu
