#include "expert_fixture.h"
#include "include/expert_config.h"
#include <cstring>
#include <vector>
#include <cstdio>

// ============================================================
// Static member definition
// ============================================================
int Logger::instance_count = 0;

// ============================================================
// Global variable definitions
// ============================================================
Logger*     g_default_logger = nullptr;
const char* g_app_name       = "ExpertFixture";
int         g_max_msg_len    = MAX_BUFFER;

// ============================================================
// Virtual method implementations
// ============================================================

void Logger::log(LogLevel level, const char* msg) {
    ++instance_count;
}

void FileLogger::log(LogLevel level, const char* msg) {
    const char* header = "[FILE] ";
}

void ConsoleLogger::log(LogLevel level, const char* msg) {
    const char* header = "[CONSOLE] ";
}

void HybridLogger::log(LogLevel level, const char* msg) {
    FileLogger::log(level, msg);
    ConsoleLogger::log(level, msg);
}

// ============================================================
// CustomError implementation (external base)
// ============================================================
const char* CustomError::what() const noexcept {
    return "custom error";
}

// ============================================================
// Template method definitions
// ============================================================

template<typename T>
void DataStore<T>::store(T value) {
    data = value;
    ++count;
}

template<typename T>
T DataStore<T>::retrieve() const {
    return data;
}

template<typename T>
void DataStore<T>::clear() {
    count = 0;
}

// ============================================================
// Template function definition
// ============================================================

template<typename T>
T max_of(T a, T b) {
    return (a > b) ? a : b;
}

// ============================================================
// Regular function implementations
// ============================================================

int initialize_logging(Logger* logger) {
    if (!logger) {
        return -1;
    }
    g_default_logger = logger;
    logger->log(LOG_INFO, "Logging system initialized");
    return 0;
}

void report(LogLevel level, const char* msg) {
    if (g_default_logger) {
        g_default_logger->log(level, msg);
    }
}

void cleanup_logging() {
    if (g_default_logger) {
        g_default_logger->log(LOG_INFO, "Logging system shut down");
        g_default_logger = nullptr;
    }
}

const char* get_status_message(int code) {
    switch (code) {
        case 0:  return "OK";
        case -1: return "ERROR";
        default: return "UNKNOWN";
    }
}

// Uses Config from subdirectory header (tests includedFrom retention)
int use_config() {
    Config cfg;
    cfg.version = 1;
    cfg.debug_mode = true;
    return cfg.version;
}

// Uses CustomError derived from external base (tests external stub)
const char* get_error_message(const CustomError& err) {
    return err.what();
}

// Uses std::vector<int> — pure builtin type params (should be filtered)
int use_plain_int_vector() {
    std::vector<int> v;
    v.push_back(42);
    return (int)v.size();
}

// Returns std::unique_ptr<Record> (tests return_type normalization)
std::unique_ptr<Record> make_record(int id) {
    auto r = std::make_unique<Record>();
    r->id = id;
    return r;
}

// ============================================================
// Cover remaining scenarios
// ============================================================

void demo_strings_and_enums() {
    report(LOG_DEBUG, "This is a debug message");
    report(LOG_WARN,  "Warning: something happened");
    report(LOG_ERROR, "Error: critical failure");

    const char* fmt = "Value: %d";
    g_app_name = "NewAppName";
}

int use_templates() {
    int a = 5, b = 10;
    int result = max_of(a, b);

    DataStore<int> store;
    store.store(42);

    Pair<int> p;
    p.first = 1;

    return result;
}

void use_aliases() {
    AdvancedLogger al;
    SimpleLogger   sl;
    al.log(LOG_INFO, "test");
    sl.log(LOG_INFO, "test");
}

BaseConfig get_default_config() {
    BaseConfig cfg;
    cfg.max_retries     = 3;
    cfg.default_level   = LOG_INFO;
    cfg.enable_timestamp = true;
    return cfg;
}

// ============================================================
// Explicit template instantiations
// ============================================================

template class DataStore<int>;
template class DataStore<double>;
template struct Pair<long>;

// ============================================================
// System symbol filter scenarios
// ============================================================

void demo_c_library_calls() {
    Record r;
    r.id = 42;
    std::memcpy(r.name, "demo", 5);
    std::printf("id=%d\n", r.id);
    char buf[64];
    std::strcpy(buf, "hello");
}

// ============================================================
// Bridge symbol scenario
// ============================================================

int demo_stl_with_project_types() {
    std::vector<Record*> records;
    Record* r = nullptr;
    records.push_back(r);
    return (int)records.size();
}
