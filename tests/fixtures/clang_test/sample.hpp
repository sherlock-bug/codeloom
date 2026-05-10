#pragma once
#include <string>
#include <vector>

namespace myapp {
namespace core {

/// A simple configuration class
class Config {
public:
    Config() = default;
    virtual ~Config() = default;
    
    /// Load configuration from file
    virtual bool load(const std::string& path);
    int get_timeout() const { return timeout_; }
    
protected:
    int timeout_ = 30;
};

/// Extended configuration with retry support
class RetryConfig : public Config {
public:
    bool load(const std::string& path) override;
    void set_retries(int n) { retries_ = n; }
    
private:
    int retries_ = 3;
};

} // namespace core

// Type aliases
typedef unsigned int uint32;
using StringVec = std::vector<std::string>;

// Enums
enum class Status { OK, ERROR, TIMEOUT };
enum LogLevel { DEBUG = 0, INFO = 1, WARN = 2, ERROR = 3 };

// Global state
extern int global_request_count;
static int local_counter = 0;

/// Process a request with timeout
int process_request(const std::string& url, int timeout_ms);

} // namespace myapp
