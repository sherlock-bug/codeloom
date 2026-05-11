# Tasks

## 1. 基础设施准备

- [ ] 1.1 logger.rs 代码默认日志级别改为 debug（`LevelFilter::Info` → `LevelFilter::Debug`）
- [ ] 1.2 `cargo check` 验证编译通过

## 2. AST 解析日志（ast.rs，最重）

- [ ] 2.1 模块入口加 `log_debug!`："extracting symbols from <file_path>"
- [ ] 2.2 `extract_node` 中 `is_project_file()` 调用前后加 debug 日志，输出输入路径和返回结果
- [ ] 2.3 `is_external` 判定处加 debug 日志，输出判定原因（decl_file vs cur_file）
- [ ] 2.4 每个符号提取后加 trace（CXXRecordDecl/FunctionDecl/CXXMethodDecl 等）输出 kind + name + decl_file
- [ ] 2.5 `add_symbol` dedup 命中时加 debug 日志（"dedup skip: name=<name> ns=<ns> kind=<kind>"）
- [ ] 2.6 退出前加 `log_info!` 耗时日志："AST extraction done: N symbols, M edges in <time>ms"

## 3. 文件收集日志（tree_sitter.rs）

- [ ] 3.1 `collect_files` 入口加 `log_debug!`："collect_files scanning <root>"
- [ ] 3.2 文件被跳过时（语言不支持/ignore 匹配）加 debug 日志："skip <path>: <reason>"
- [ ] 3.3 退出前加 `log_info!` 耗时日志："collect_files done: N files, M skipped in <time>ms"

## 4. 搜索查询日志（query/search.rs）

- [ ] 4.1 `hybrid_search` 入口加 `log_debug!`："search query=<q> repo=<repo> branch=<branch> limit=<limit> kind=<kind>"
- [ ] 4.2 BM25 命中数和向量维度加载状态加 debug 日志
- [ ] 4.3 退出前加 `log_info!` 耗时日志："search done: N hits in <time>ms"
- [ ] 4.4 `weighted_fuse_single` 中 top-3 分数分布加 debug 日志（仅调试级别）

## 5. 校准日志（calib/mod.rs）

- [ ] 5.1 入口加 `log_debug!`："calibrate repo=<repo> branches=[<list>]"
- [ ] 5.2 每个分支校准完成加 info 日志："calibrated <branch>: score=<score>"
- [ ] 5.3 校准失败加 `log_error!` 含失败原因
- [ ] 5.4 退出前加 `log_info!` 耗时日志

## 6. 存储层日志（storage/）

- [ ] 6.1 `fts.rs: fill_all_fts()` 入口/退出加日志："fts rebuild done: N rows in <time>ms"
- [ ] 6.2 `symbols.rs: insert()` upsert 结果（insert/update/skip）加 debug 日志
- [ ] 6.3 `vector.rs: try_load()` 失败时加 `log_warn!`："vec0 not loaded: <reason>"
- [ ] 6.4 `vector.rs: knn_search()` 入口/退出加 debug 日志
- [ ] 6.5 `schema.rs: run()` 迁移执行结果加 info 日志

## 7. 现有日志模块增强

- [ ] 7.1 `indexer/clang/mod.rs` parse_file 调用前后加耗时日志
- [ ] 7.2 `indexer/smart.rs` 索引全过程入口/退出加耗时日志
- [ ] 7.3 `mcp/mod.rs` 工具调用的超时/异常分支加 warn 日志

## 8. `eprintln!` 统一迁移

- [ ] 8.1 搜索所有 `eprintln!` 调用，诊断警告迁移到 `log_warn!`，错误迁移到 `log_error!`

## 9. 编译验证

- [ ] 9.1 `cargo build` 编译通过
- [ ] 9.2 `cargo test` 全部通过
- [ ] 9.3 运行一次 `codeloom search` 确认日志输出正常
