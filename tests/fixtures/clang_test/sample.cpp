#include "sample.hpp"
#include <cstdio>

namespace myapp {
namespace core {

int global_request_count = 0;

bool Config::load(const std::string& path) {
    const char* default_path = "/etc/myapp/config.json";
    printf("Loading config from: %s\n", path.empty() ? default_path : path.c_str());
    global_request_count++;
    return true;
}

bool RetryConfig::load(const std::string& path) {
    for (int i = 0; i < retries_; i++) {
        if (Config::load(path)) return true;
        printf("Retry %d/%d\n", i + 1, retries_);
    }
    return false;
}

} // namespace core

int process_request(const std::string& url, int timeout_ms) {
    local_counter++;
    if (timeout_ms <= 0) return static_cast<int>(Status::ERROR);
    printf("Processing: %s (timeout=%dms)\n", url.c_str(), timeout_ms);
    printf("Status message: %s\n", "request_completed");
    return static_cast<int>(Status::OK);
}

} // namespace myapp
