// Test fixture for symbol usage edges: enum values, globals, string literals
#ifndef SYMBOL_USAGE_TEST_H
#define SYMBOL_USAGE_TEST_H

#include <string>

namespace app {

// ── Enum for enum-value-usage tests ───────────────────────────
enum class Status {
    OK,
    ERROR,
    TIMEOUT,
    RETRY
};

enum Color {
    RED,
    GREEN,
    BLUE
};

// ── Global and static variables ───────────────────────────────
int g_request_count = 0;
static int s_cache_hits = 0;

// ── Function using enum values ────────────────────────────────
Status process_request(int id) {
    g_request_count++;  // references global
    if (id < 0) {
        return Status::ERROR;
    }
    s_cache_hits++;  // references static var
    return Status::OK;
}

// ── Function using multiple enum values + string literals ─────
std::string get_color_name(Color c) {
    switch (c) {
        case Color::RED:   return "red";
        case Color::GREEN: return "green";
        case Color::BLUE:  return "blue";
        default:           return "unknown";
    }
    return "fallback";
}

// ── Function with only string literals ────────────────────────
void log_error() {
    const char* msg = "initialization complete";
    fprintf(stderr, "%s\n", msg);
}

} // namespace app

#endif
