// test enum values, global vars, static vars
enum Status { kOk, kNotFound, kCorruption };

int global_counter = 0;
static int file_static = 42;

class AuthService {
public:
    Status authenticate(const char* user);
    static int instance_count;
private:
    int retry_max;
};
