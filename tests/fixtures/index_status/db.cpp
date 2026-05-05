#include "db.h"
#include <iostream>

bool PostgresDB::connect(const std::string& host, int port) {
    host_ = host;
    std::cout << "Connecting to " << host << ":" << port << std::endl;
    return true;
}

void PostgresDB::disconnect() {
    std::cout << "Disconnecting" << std::endl;
}

int PostgresDB::execute(const std::string& sql) {
    return sql.length();
}

bool MySQLDB::connect(const std::string& host, int port) {
    return true;
}

void MySQLDB::disconnect() {}

int MySQLDB::execute(const std::string& sql) {
    return 0;
}
