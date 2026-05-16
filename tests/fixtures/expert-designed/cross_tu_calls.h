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

// A class inside the namespace, called via pointer — reproduces the
// leveldb pattern where env_ is Env* in namespace leveldb.
// ptr->handle() with qualType="cross_tu::Handler *" should produce
// target_name = "cross_tu::Handler::handle", but DB stores
// name = "Handler::handle" (namespace in attrs, not in name).
class Handler {
public:
    int handle(int value);
};

}  // namespace cross_tu

// A class used via pointer calls — reproduces the pattern of
// env_->GetChildren() in leveldb where the MemberExpr's type.qualType
// includes the namespace (e.g. "leveldb::Env *"), producing
// target_name = "cross_tu::Client::process".
class Client {
public:
    int process(int value);
};

#endif
