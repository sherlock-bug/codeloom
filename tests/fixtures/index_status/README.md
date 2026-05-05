# Database Module

## Connection
The Database class provides an abstraction for connecting to databases.
Call connect() to establish a connection with host and port.

## Query Execution
Use execute() to run SQL queries. Returns the number of affected rows.

## Supported Backends
- PostgreSQL via PostgresDB
- MySQL via MySQLDB
