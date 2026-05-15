#ifndef EXPERT_FIXTURE_H
#define EXPERT_FIXTURE_H

// Macro definition
#define MAX_BUFFER 256

#include <vector>
#include <memory>
#include <exception>

// ============================================================
// Project type for STL template instantiation testing
// ============================================================
struct Record {
    int id;
    char name[64];
};

// ============================================================
// Forward declarations (tests: 前向声明不创建符号)
// ============================================================
class ConfigLoader;

// ============================================================
// Enum
// ============================================================
enum LogLevel {
    LOG_DEBUG,
    LOG_INFO,
    LOG_WARN,
    LOG_ERROR
};

// ============================================================
// Struct with fields (uses_type from field type)
// ============================================================
struct BaseConfig {
    int         max_retries;
    LogLevel    default_level;
    bool        enable_timestamp;
};

// ============================================================
// Class hierarchy (inheritance + override)
// ============================================================
class Logger {
public:
    virtual void log(LogLevel level, const char* msg);
    virtual ~Logger() = default;
    static int instance_count;
};

class FileLogger : public Logger {
public:
    void log(LogLevel level, const char* msg) override;
};

class ConsoleLogger : public Logger {
public:
    void log(LogLevel level, const char* msg) override;
};

class HybridLogger : public FileLogger, public ConsoleLogger {
public:
    void log(LogLevel level, const char* msg) override;
};

// ============================================================
// Custom exceptions — inherits from external base std::exception
// (tests: 外部基类存根, overrides 边跨系统边界)
// ============================================================
struct CustomError : public std::exception {
    const char* what() const noexcept override;
    int error_code;
};

// ============================================================
// Struct with STL field type
// (tests: uses_type 边的模板名规范化 — unique_ptr → unique_ptr)
// ============================================================
struct AppContext {
    std::unique_ptr<Record> current;
    int flags;
};

// ============================================================
// Aliases
// ============================================================
typedef HybridLogger AdvancedLogger;
using SimpleLogger = ConsoleLogger;

// ============================================================
// Templates
// ============================================================
template<typename T>
class DataStore {
public:
    void store(T value);
    T    retrieve() const;
    void clear();
private:
    T    data;
    int  count;
};

template<typename T>
struct Pair {
    T first;
    T second;
};

template<typename T>
T max_of(T a, T b);

// ============================================================
// Global variables
// ============================================================
extern Logger*     g_default_logger;
extern const char* g_app_name;
extern int         g_max_msg_len;

// ============================================================
// Regular functions
// ============================================================
int         initialize_logging(Logger* logger);
void        report(LogLevel level, const char* msg);
void        cleanup_logging();
const char* get_status_message(int code);

// Subdirectory header type usage (tests includedFrom)
int         use_config();

// External base type usage (tests external stub + overrides)
const char* get_error_message(const CustomError& err);

// Pure builtin STL instantiation (tests builtin-only filtering)
int         use_plain_int_vector();

// STL return type (tests return_type normalization)
std::unique_ptr<Record> make_record(int id);

// STL template instance with project type (tests bridge symbols)
void demo_stl_with_project_types();

// C library function calls (tests system symbol filtering)
void demo_c_library_calls();

#endif
