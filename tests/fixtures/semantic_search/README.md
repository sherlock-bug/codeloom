# Auth Module

## User Authentication
The AuthService handles user login. Use authenticate() to verify credentials.
After successful login, use authorize() to check access permissions.

## Session Management
Call revokeAllSessions() to force logout all users. Useful during security incidents.

## Cache
CacheManager provides in-memory key-value storage for session tokens.
