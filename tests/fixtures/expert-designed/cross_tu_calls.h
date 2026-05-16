#ifndef CROSS_TU_CALLS_H
#define CROSS_TU_CALLS_H

// Class with out-of-line method declarations.
// Methods are defined in callee.cc, NOT inline in this header.
// When caller.cc includes this header, Clang does NOT emit
// CXXMethodDecl for these methods — only MemberExpr call sites.
// This reproduces the bug where cross-TU call edges are lost.

namespace cross_tu {

class Calculator {
public:
    int add(int a, int b);
    int multiply(int a, int b);
    int compute_result();
};

}  // namespace cross_tu
#endif
