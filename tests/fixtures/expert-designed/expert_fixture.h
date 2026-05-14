#ifndef EXPERT_FIXTURE_H
#define EXPERT_FIXTURE_H

// Macro definition
#define MAX_BUFFER 256

// Enum with enum values
enum LogLevel {
    LOG_DEBUG,
    LOG_INFO,
    LOG_WARN,
    LOG_ERROR
};

// Struct with fields
struct BaseConfig {
    int         max_retries;
    LogLevel    default_level;
    bool        enable_timestamp;
};

// Root class (no base classes)
class Logger {
public:
    virtual void log(LogLevel level, const char* msg);
    virtual ~Logger() = default;

    static int instance_count;
};

// Single inheritance + virtual override (override keyword)
class FileLogger : public Logger {
public:
    void log(LogLevel level, const char* msg) override;
};

// Single inheritance + virtual override
class ConsoleLogger : public Logger {
public:
    void log(LogLevel level, const char* msg) override;
};

// Multiple inheritance (diamond), leaf class
class HybridLogger : public FileLogger, public ConsoleLogger {
public:
    void log(LogLevel level, const char* msg) override;
};

// Typedef alias
typedef HybridLogger AdvancedLogger;

// Using alias
using SimpleLogger = ConsoleLogger;

// Template class
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

// Template struct
template<typename T>
struct Pair {
    T first;
    T second;
};

// Global variables
extern Logger*     g_default_logger;
extern const char* g_app_name;
extern int         g_max_msg_len;

// Template function
template<typename T>
T max_of(T a, T b);

// Regular functions
int         initialize_logging(Logger* logger);
void        report(LogLevel level, const char* msg);
void        cleanup_logging();
const char* get_status_message(int code);

#endif
