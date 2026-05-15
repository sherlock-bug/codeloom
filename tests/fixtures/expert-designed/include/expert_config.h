#ifndef EXPERT_CONFIG_H
#define EXPERT_CONFIG_H

// Struct defined in a project subdirectory header (include/).
// This tests:
// 1. includedFrom fallback — when Clang AST has no loc.file for this node,
//    includedFrom.file points to the project .cc file → retained as project symbol.
// 2. Subdirectory header class retention — project headers outside the
//    fixture root directory are still considered project symbols.
struct Config {
    int version;
    bool debug_mode;
};

#endif
