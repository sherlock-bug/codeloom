// Database abstraction layer
#pragma once
#include <string>

class Database {
public:
    virtual bool connect(const std::string& host, int port) = 0;
    virtual void disconnect() = 0;
    virtual int execute(const std::string& sql) = 0;
};

class PostgresDB : public Database {
public:
    bool connect(const std::string& host, int port) override;
    void disconnect() override;
    int execute(const std::string& sql) override;
private:
    int socket_fd;
    std::string host_;
};

class MySQLDB : public Database {
public:
    bool connect(const std::string& host, int port) override;
    void disconnect() override;
    int execute(const std::string& sql) override;
};
