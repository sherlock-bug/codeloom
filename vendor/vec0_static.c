/*
 * vec0_static.c — 将 sqlite-vec 静态注册进 SQLite
 *
 * 编译方式：与 sqlite-vec.c 一起用 -DSQLITE_CORE 编译，链接进 codeloom。
 * Rust 侧在程序启动时调用 vec0_static_init() 即可。
 */

#include "sqlite3.h"

/* 声明在 sqlite-vec.c 中定义的入口函数 */
int sqlite3_vec_init(sqlite3 *db, char **pzErrMsg,
                     const struct sqlite3_api_routines *pApi);

/*
 * vec0_static_init — Rust 侧在首次打开连接前调用一次即可。
 * 通过 sqlite3_auto_extension() 注册，之后所有新连接自动初始化 vec0 模块。
 * 
 * 注意：auto_extension 的函数指针签名是 void(*)(void)，
 * SQLite 内部会将其转换回正常的扩展初始化签名。
 */
void vec0_static_init(void) {
    sqlite3_auto_extension((void (*)(void))sqlite3_vec_init);
}
