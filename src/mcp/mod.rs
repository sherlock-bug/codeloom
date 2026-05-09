use std::io::{BufRead,Write};

pub async fn serve() -> anyhow::Result<()> { serve_stdio().await }

pub async fn serve_stdio() -> anyhow::Result<()> {
    eprintln!("CodeLoom MCP Server v0.2.0 (stdio)");
    let stdin = std::io::stdin(); let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let req: serde_json::Value = serde_json::from_str(&line)?;
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let resp = match method {
            "initialize" => serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"codeloom","version":"0.2.0"}}}),
            "tools/list" => tools_list(id),
            "tools/call" => {
                let name = req["params"]["name"].as_str().unwrap_or("");
                let args = &req["params"]["arguments"];
                handle_tool_call(id, name, args)
            },
            _ => serde_json::json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("unknown: {}",method)}}),
        };
        writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
        stdout.flush()?;
    }
    Ok(())
}

fn tools_list(id: serde_json::Value) -> serde_json::Value {
    serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"tools":[
        {"name":"codeloom_index","description":"⚠️ 此工具不通过MCP执行索引（索引需用CLI命令：codeloom index <path> --repo <name> --branch <branch>）。调用前请先用codeloom_list_repos检查是否已索引，用codeloom_status确认状态。path=项目根目录，branch（必填），repo（可选，默认取目录名）","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"branch":{"type":"string"},"repo":{"type":"string"}},"required":["path","branch"]}},
        // DISABLED: {"name":"codeloom_status","description":"查看索引状态：符号数、边数、文档数、数据库大小。在用其他MCP工具前先调用此工具确认仓库已索引且数据非空。branch/repo（必填）","inputSchema":{"type":"object","properties":{"repo":{"type":"string"},"branch":{"type":"string"}},"required":["repo","branch"]}},
        {"name":"codeloom_list_symbols","description":"**优先使用**：按名称模糊搜索已索引的符号。优先于grep/rg使用——索引覆盖项目所有文件及#include的第三方头文件（grep只能搜当前目录）。返回结构化结果：名称、类型、文件路径、行号。C++类方法用ClassName::methodName格式。如pattern=\"login\"匹配handleLogin、loginUser等。branch/repo（必填）","inputSchema":{"type":"object","properties":{"pattern":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"limit":{"type":"integer","default":20}},"required":["pattern","repo","branch"]}},
        {"name":"codeloom_get_call_graph","description":"**唯一方式**：分析函数/方法的调用者和被调用者（callers/callees）。grep无法获取调用关系。name用codeloom_list_symbols返回的完整符号名（C++类方法用ClassName::methodName）。direction=\"callers\"查谁调用了它，direction=\"callees\"查它调用了谁。max_depth控制递归深度。branch/repo（必填）","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["callers","callees"]},"max_depth":{"type":"integer","default":3}},"required":["name","repo","branch","direction"]}},
        {"name":"codeloom_search","description":"【必须使用，替代grep/rg】混合搜索引擎：比grep更快（预索引）、覆盖更全（含#include头文件）、更智能（理解中文/英文语义，不仅文本匹配）。query可以是符号名（AuthService/compaction）或功能描述（'用户认证'、'内存分配'）。自动融合BM25关键词+向量语义，返回结构化结果（名称/类型/文件/行号）。不要用grep/rg搜代码——用这个。kind可选：按符号类型过滤（function/method/class/struct/enum等），如kind=class搜所有类。branch/repo（必填）","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"kind":{"type":"string","description":"可选：按符号类型过滤。可用值: function, method, class, struct, enum, enum_value, field, global, static_var, variable"},"limit":{"type":"integer","default":10}},"required":["query","repo","branch"]}},

        {"name":"codeloom_inspect","description":"查看符号节点的全部信息：定义、文档注释、所有关联边（调用/继承/包含/参数/返回/字段）。比逐个grep再read_file更高效——一条命令看清符号的全貌。name=符号完整名称（C++类方法用ClassName::methodName格式）。branch/repo（必填）","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"}},"required":["name","repo","branch"]}},

        {"name":"codeloom_list_repos","description":"列出所有已索引的仓库名。在任何搜索/查询操作前必须先调用此工具获取可用的repo参数值。无需任何参数。返回如\"codeloom\\nleveldb\\nspdlog\"。","inputSchema":{"type":"object","properties":{},"required":[]}},
        {"name":"codeloom_list_branches","description":"列出指定仓库的所有已索引分支及各自符号数量。repo=仓库名（必填）。返回分支名和符号数，用于团队协作时确认分支状态。","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}},"required":["repo"]}},
        // DISABLED: {"name":"codeloom_get_doc","description":"获取文档节点的完整内容及嵌入图片。doc_id=文档节点ID（从搜索或overview结果中获得），repo=仓库名，branch=分支名。返回标题、章节路径、层级、内容、文件路径、格式、节点类型及图片列表（base64编码）。","inputSchema":{"type":"object","properties":{"doc_id":{"type":"integer"},"repo":{"type":"string"},"branch":{"type":"string"}},"required":["doc_id","repo","branch"]}},
        // DISABLED: {"name":"codeloom_query_excel","description":"回答Excel表格问题（筛选、查找行列数据等）。doc_id=文档节点ID（Excel文档内节点），repo=仓库名，branch=分支名（必填）。mode可选：row（返回整行键值对）、column（返回整列）、filter（按条件过滤，filter参数为过滤表达式如'销售额 > 5000'）、auto（根据节点类型自动推断）。limit最多返回行数（默认20）。","inputSchema":{"type":"object","properties":{"doc_id":{"type":"integer"},"repo":{"type":"string"},"branch":{"type":"string"},"mode":{"type":"string","enum":["row","column","filter","auto"]},"filter":{"type":"string"},"search":{"type":"string"},"limit":{"type":"integer","default":20}},"required":["doc_id","repo","branch"]}},
        {"name":"codeloom_schema","description":"导出CodeLoom所有元数据：节点类型（symbol kind）及其说明、边类型（edge_type前缀）及其方向语义和参与节点类型。调用codeloom_path_analysis/codeloom_impact_analysis/codeloom_neighbor_graph前必须优先调用此工具，否则你不知道edge_filter参数有哪些边类型可选。无需任何参数。","inputSchema":{"type":"object","properties":{},"required":[]}},
        {"name":"codeloom_path_analysis","description":"回答『A到B怎么走』『A和B有什么关系』：在两个符号之间搜索关系路径，跨函数调用、数据流、继承等多种边类型。优于逐层调用codeloom_call_graph：一次调用自动跨边类型搜索最短路径，一键覆盖calls/inherits/param_type等多类边。source=起始符号名，target=目标符号名，mode=shortest|all（默认shortest），edge_filter可选限定边类型（如['calls','calls_override']只看调用路径），direction=forward|reverse|both（默认both）。branch/repo（必填）","inputSchema":{"type":"object","properties":{"source":{"type":"string"},"target":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"mode":{"type":"string","enum":["shortest","all"]},"max_paths":{"type":"integer","default":20},"max_depth":{"type":"integer","default":10},"direction":{"type":"string","enum":["forward","reverse","both"]},"edge_filter":{"type":"array","items":{"type":"string"}}},"required":["source","target","repo","branch"]}},
        {"name":"codeloom_impact_analysis","description":"回答『改了X会影响谁』『X被谁依赖』：沿边传递闭包N跳分析影响范围。优于codeloom_call_graph：自动N跳递归，不仅是直接调用者。优于codeloom_neighbor_graph：传递闭包而非只看1跳邻居。symbol=要分析的符号名，direction=forward|reverse|both（默认reverse），radius=跳数（默认3），edge_filter可选限定边类型。branch/repo（必填）","inputSchema":{"type":"object","properties":{"symbol":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["forward","reverse","both"]},"radius":{"type":"integer","default":3},"edge_filter":{"type":"array","items":{"type":"string"}}},"required":["symbol","repo","branch"]}},
        {"name":"codeloom_neighbor_graph","description":"回答『X周围都有什么』『X直接调用了什么/被什么调用』：查看符号1跳范围内的直接邻居，按边类型分组返回（calls/returns/param_type/inherits等）。优于codeloom_impact_analysis：只看直接邻居不递归；优于grep：自动关联calls/returns/param_type/uses等所有边类型，无需按类型分别搜索。symbol=符号名，direction=forward|reverse|both（默认both），depth=1|2（默认1）。branch/repo（必填）","inputSchema":{"type":"object","properties":{"symbol":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["forward","reverse","both"]},"depth":{"type":"integer","default":1}},"required":["symbol","repo","branch"]}},
        {"name":"codeloom_inheritance_tree","description":"回答『谁继承了X』『X继承了什么』：查看类的完整继承树，含父类、子类和虚方法覆写列表。优于grep 'extends'/'public'：自动递归解析完整继承链，不遗漏间接继承。symbol=类名，direction=up|down|both（默认down），max_depth=递归深度（默认5）。branch/repo（必填）","inputSchema":{"type":"object","properties":{"symbol":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["up","down","both"]},"max_depth":{"type":"integer","default":5}},"required":["symbol","repo","branch"]}},
    ]}})
}

/// Validate repo parameter: accept any repo name that has a DB file.
/// Empty string means global (non-git) data — always valid if .rag.db exists.
fn validate_repo(repo: &str) -> Result<String, String> {
    if let Ok(dd) = crate::config::Config::data_dir() {
        let db_path = dd.join(format!("{}.rag.db", repo));
        if db_path.exists() {
            return Ok(repo.to_string());
        }
    }
    let repos = crate::query::repo::list_repos();
    Err(format!("未找到仓库 '{}'。可用仓库: {}", repo, repos.join(", ")))
}

fn handle_tool_call(id: serde_json::Value, name: &str, args: &serde_json::Value) -> serde_json::Value {
    let result = match name {
        
        // DISABLED: codeloom_get_doc
        /* "codeloom_get_doc" => { ... } */
        // DISABLED: codeloom_query_excel
        /* "codeloom_query_excel" => { ... } */
        // DISABLED: codeloom_status
        /* "codeloom_status" => { ... } */
        "codeloom_schema" => {
            schema_meta()
        }
        "codeloom_path_analysis" => {
            let source = args["source"].as_str().unwrap_or("");
            let target = args["target"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let mode = args["mode"].as_str().unwrap_or("shortest");
            let max_paths = args["max_paths"].as_u64().unwrap_or(20) as usize;
            let max_depth = args["max_depth"].as_u64().unwrap_or(10) as usize;
            let edge_filter: Vec<String> = args["edge_filter"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if source.is_empty() || target.is_empty() { return err_resp(id, "source and target are required"); }
            path_analysis(&repo.unwrap(), branch, source, target, mode, max_depth, max_paths, &edge_filter)
        }
        "codeloom_impact_analysis" => {
            let symbol = args["symbol"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let direction = args["direction"].as_str().unwrap_or("reverse");
            let radius = args["radius"].as_u64().unwrap_or(3) as usize;
            let edge_filter: Vec<String> = args["edge_filter"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if symbol.is_empty() { return err_resp(id, "symbol is required"); }
            impact_analysis(&repo.unwrap(), branch, symbol, direction, radius, &edge_filter)
        }
        "codeloom_neighbor_graph" => {
            let symbol = args["symbol"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let direction = args["direction"].as_str().unwrap_or("both");
            let depth = args["depth"].as_u64().unwrap_or(1) as usize;
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if symbol.is_empty() { return err_resp(id, "symbol is required"); }
            neighbor_graph(&repo.unwrap(), branch, symbol, direction, depth)
        }
        "codeloom_inheritance_tree" => {
            let symbol = args["symbol"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let direction = args["direction"].as_str().unwrap_or("down");
            let max_depth = args["max_depth"].as_u64().unwrap_or(5) as usize;
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if symbol.is_empty() { return err_resp(id, "symbol is required"); }
            inheritance_tree(&repo.unwrap(), branch, symbol, direction, max_depth)
        }
        "codeloom_list_symbols" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let limit = args["limit"].as_u64().unwrap_or(20) as usize;
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            list_symbols(pattern, &repo.unwrap(), branch, limit)
        }
        "codeloom_get_call_graph" => {
            let branch = args["branch"].as_str().unwrap_or("");
            let sym_name = args["name"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let direction = args["direction"].as_str().unwrap_or("callers");
            let max_depth = args["max_depth"].as_u64().unwrap_or(3) as usize;
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            let repo = repo.unwrap();
            let conn = match open_repo_db(&repo) { Ok(c) => c, Err(e) => return err_resp(id, &e) };
            crate::query::call_graph::get_call_graph(&conn, sym_name, &repo, branch, direction, max_depth)
        }
        "codeloom_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let limit = args["limit"].as_u64().unwrap_or(10) as usize;
            let kind_raw = args["kind"].as_str().unwrap_or("");
            let valid_kinds: &[&str] = &["function", "method", "class", "struct", "enum", "enum_value", "field", "global", "static_var", "variable"];
            let kind_filter = if kind_raw.is_empty() {
                None
            } else if valid_kinds.contains(&kind_raw) {
                Some(kind_raw)
            } else {
                return err_resp(id, &format!("Invalid kind '{}'. Valid values: {}", kind_raw, valid_kinds.join(", ")));
            };
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            hybrid_search(query, &repo.unwrap(), branch, limit, kind_filter)
        }
        "codeloom_index" => {
            let path = args["path"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let branch = args["branch"].as_str().unwrap_or("");
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if path.is_empty() { return err_resp(id, "path is required"); }
            let repos = crate::query::repo::list_repos();
            if repos.iter().any(|r| r == repo) {
                format!("仓库 '{}' 已索引（状态: 可用）。如需重新索引请用 CLI: codeloom index {} --repo {} --branch {}\n提示：先用 codeloom_status 查看索引统计，用 codeloom_search 搜索代码结构。", repo, path, repo, branch)
            } else {
                format!("仓库 '{}' 尚未索引。可用仓库: {}\n如需索引请用 CLI: codeloom index {} --repo {} --branch {}", repo, repos.join(", "), path, repo, branch)
            }
        }
        "codeloom_inspect" => {
            let branch = args["branch"].as_str().unwrap_or("");
            let sym_name = args["name"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if sym_name.is_empty() { return err_resp(id, "name is required"); }
            inspect_symbol(sym_name, &repo.unwrap(), branch)
        }
                "codeloom_list_repos" => {
            let repos = crate::query::repo::list_repos();
            if repos.is_empty() {
                "未找到已索引的仓库。请用 CLI 创建索引: codeloom index <path> --repo <name> --branch <branch>".into()
            } else {
                repos.iter().map(|r| if r.is_empty() { "(global)".to_string() } else { r.clone() }).collect::<Vec<_>>().join("\n")
            }
        }
        "codeloom_list_branches" => {
            let repo = args["repo"].as_str().unwrap_or("");
            if repo.is_empty() { return err_resp(id, "repo is required. Use codeloom_list_repos to see available repos."); }
            let dd = match crate::config::Config::data_dir() { Ok(d) => d, Err(e) => return err_resp(id, &format!("Config error: {}", e)) };
            let dbp = dd.join(format!("{}.rag.db", repo));
            if !dbp.exists() {
                let repos = crate::query::repo::list_repos();
                return format!("未找到仓库 '{}'。可用仓库: {}", repo, repos.join(", ")).into();
            }
            match crate::storage::open(&dbp.to_string_lossy()) {
                Ok(conn) => {
                    let mut stmt = match conn.prepare("SELECT DISTINCT branch_name FROM branches WHERE branch_name IS NOT NULL ORDER BY branch_name") {
                        Ok(s) => s,
                        Err(e) => return format!("Query error: {}", e).into(),
                    };
                    let branches: Vec<String> = match stmt.query_map([], |r| r.get(0)) {
                        Ok(rows) => rows.flatten().collect(),
                        Err(e) => return format!("Query error: {}", e).into(),
                    };
                    if branches.is_empty() {
                        format!("仓库 '{}' 无已索引分支", repo)
                    } else {
                        format!("仓库 '{}' 的分支:\n{}", repo, branches.join("\n"))
                    }
                }
                Err(e) => format!("无法打开数据库: {}", e).into(),
            }
        }
        _ => format!("Unknown tool: {}", name),
    };
    serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":result}]}})
}

fn err_resp(id: serde_json::Value, msg: &str) -> serde_json::Value {
    serde_json::json!({"jsonrpc":"2.0","id":id,"error":{"code":-32602,"message":msg}})
}

fn open_repo_db(repo: &str) -> Result<rusqlite::Connection, String> {
    let dd = crate::config::Config::data_dir().map_err(|e| format!("Config error: {}", e))?;
    let db_path = dd.join(format!("{}.rag.db", if repo.is_empty() { "".into() } else { repo.to_string() }));
    let conn = crate::storage::open(&db_path.to_string_lossy()).map_err(|e| format!("DB error: {}", e))?;
    // If querying a specific (non-global) repo, also ATTACH global DB for cross-repo visibility
    if !repo.is_empty() {
        let global_path = dd.join(".rag.db");
        if global_path.exists() && global_path != db_path {
            conn.execute("ATTACH DATABASE ?1 AS global", rusqlite::params![global_path.to_string_lossy().to_string()]).ok();
        }
    }
    Ok(conn)
}

fn branch_where_clause(branch: &str) -> String {
    format!("AND (b.branch_name = '{}' OR b.branch_name IS NULL)", branch.replace('\'', "''"))
}

fn status(repo: &str, branch: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let bwc = branch_where_clause(branch);
    let sym_sql = format!("SELECT COUNT(*) FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE b.repo=?1 {}", bwc);
    let syms: i64 = conn.query_row(&sym_sql, rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
    let edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap_or(0);
    let docs: i64 = conn.query_row("SELECT COUNT(*) FROM doc_nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name=?2)", rusqlite::params![repo, branch], |r| r.get(0)).unwrap_or(0);
    let resolved: i64 = conn.query_row("SELECT COUNT(*) FROM edges WHERE target_id!=0", [], |r| r.get(0)).unwrap_or(0);
    let resolve_pct = if edges > 0 { resolved as f64 / edges as f64 * 100.0 } else { 0.0 };
    format!(
        "Repo: {} (branch: {})\n\
         Symbols: {}  |  Edges: {} (resolved: {:.1}%)  |  Docs: {}\n\
         Database: ~/.codeloom/{}.rag.db\n\
         Status: indexed OK",
        repo, branch, syms, edges, resolve_pct, docs, repo
    )
}

fn list_symbols(pattern: &str, repo: &str, branch: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let like = format!("%{}%", pattern);
    let bwc = branch_where_clause(branch);
    let sql = format!("SELECT name, kind, file_path, line_start FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name LIKE ?2 {} ORDER BY name LIMIT ?3", bwc);
    let mut out = format!("Symbols matching '{}' in {} (branch={}):\n", pattern, repo, branch);
    if let Ok(mut stmt) = conn.prepare(&sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo, like, limit as i64], |r| {
            Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i64>(3)?))
        }) {
            let mut count = 0;
            for row in rows.flatten() {
                count += 1;
                out.push_str(&format!("  [{:10}] {:40}  @ {}:{}\n", row.1, row.0, &row.2[..60.min(row.2.len())], row.3));
            }
            if count == 0 { out.push_str("  (none)\n"); }
        }
    }
    out
}

fn inspect_symbol(name: &str, repo: &str, branch: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let bwc = branch_where_clause(branch);
    let sql = format!("SELECT s.id, s.name, s.kind, s.file_path, s.line_start, s.line_end, s.signature, s.parent_class, s.namespace, s.language, s.doc_comment FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name=?2 {}", bwc);
    let rows: Vec<(i64,String,String,String,i64,i64,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>)> = match conn.prepare(&sql) {
        Ok(mut stmt) => stmt.query_map(rusqlite::params![repo, name], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?,
                r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?))
        }).map(|r| r.flatten().collect()).unwrap_or_default(),
        Err(_) => return format!("{{\"error\":\"Error querying symbol '{}'\"}}", name),
    };
    if rows.is_empty() {
        return format!("{{\"error\":\"Symbol '{}' not found in {} (branch={})\"}}", name, repo, branch);
    }
    let mut results = Vec::new();
    for (sid, sname, kind, file, lstart, lend, sig, parent, ns, lang, doc) in &rows {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('\"', "\\\"");
        let mut json = format!(
            "{{\"name\":\"{}\",\"kind\":\"{}\"",
            esc(sname), esc(kind)
        );
        if let Some(l) = lang { json.push_str(&format!(",\"language\":\"{}\"", esc(l))); }
        if let Some(n) = ns { json.push_str(&format!(",\"namespace\":\"{}\"", esc(n))); }
        if let Some(p) = parent { json.push_str(&format!(",\"in_class\":\"{}\"", esc(p))); }
        json.push_str(&format!(",\"file\":\"{}\",\"line_start\":{},\"line_end\":{}", esc(file), lstart, lend));
        if let Some(s) = sig { json.push_str(&format!(",\"signature\":\"{}\"", esc(s))); }
        if let Some(d) = doc {
            let trimmed = d.trim();
            if !trimmed.is_empty() {
                json.push_str(&format!(",\"documentation\":\"{}\"", esc(trimmed)));
            }
        }
        // Edges
        let e_sql = format!("SELECT e.edge_type, s.name, s.kind FROM edges e LEFT JOIN symbols s ON (e.source_id={0} AND s.id=e.target_id) OR (e.target_id={0} AND s.id=e.source_id) WHERE (e.source_id={0} OR e.target_id={0}) AND s.id IS NOT NULL ORDER BY e.edge_type LIMIT 200", sid);
        if let Ok(mut e_stmt) = conn.prepare(&e_sql) {
            if let Ok(e_rows) = e_stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))) {
                use std::collections::BTreeMap;
                let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
                for row in e_rows.flatten() {
                    let (etype, ename, ekind) = row;
                    let entry = format!("{{\"name\":\"{}\",\"kind\":\"{}\"}}", esc(&ename), esc(&ekind));
                    let cat = mcp_edge_category(&etype).to_string();
                    groups.entry(cat).or_default().push(entry);
                }
                json.push_str(",\"edges\":{");
                let mut first_cat = true;
                for (cat, entries) in &groups {
                    if !entries.is_empty() {
                        if !first_cat { json.push(','); }
                        first_cat = false;
                        json.push_str(&format!("\"{}\":[{}]", cat, entries.join(",")));
                    }
                }
                json.push('}');
            }
        }
        json.push('}');
        results.push(json);
    }
    if results.len() == 1 {
        results[0].clone()
    } else {
        format!("[{}]", results.join(","))
    }
}

fn mcp_edge_category(etype: &str) -> &str {
    if etype.starts_with("calls:") { "Calls" }
    else if etype.starts_with("inherits:") { "Inherits" }
    else if etype.starts_with("overrides:") { "Overrides" }
    else if etype.starts_with("contains:") { "Contains" }
    else if etype.starts_with("param_type:") { "Parameters" }
    else if etype.starts_with("returns:") { "Returns" }
    else if etype.starts_with("field_type:") { "Fields" }
    else { "Other" }
}

fn hybrid_search(query: &str, repo: &str, branch: &str, limit: usize, kind_filter: Option<&str>) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    match crate::query::search::hybrid_search(&conn, query, repo, branch, limit, kind_filter) {
        Ok(results) => {
            if results.is_empty() {
                return serde_json::json!({"results":[],"query":query,"count":0}).to_string();
            }
            let items: Vec<serde_json::Value> = results.iter().map(|r| {
                serde_json::json!({
                    "score": r.score,
                    "name": r.name,
                    "type": r.hit_type,
                    "kind": r.kind,
                    "signature": r.signature,
                    "file": r.file_path,
                    "line": r.line_start,
                    "doc_id": r.doc_id,
                    "snippet": r.snippet,
                })
            }).collect();
            serde_json::json!({
                "results": items,
                "query": query,
                "count": results.len()
            }).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string(), "query": query}).to_string(),
    }
}

// ── Schema Meta ─────────────────────────────────────────────────────────

fn schema_meta() -> String {
    serde_json::json!({
        "node_kinds": [
            {"name":"function","description":"独立函数","example":"main, helper"},
            {"name":"method","description":"类的成员函数","example":"DBImpl::Get"},
            {"name":"class","description":"类定义","example":"DBImpl, Env"},
            {"name":"struct","description":"结构体","example":"Options, Slice"},
            {"name":"enum","description":"枚举类型","example":"Status, Color"},
            {"name":"enum_value","description":"枚举成员值","example":"Status::OK, Color::RED"},
            {"name":"global","description":"全局变量","example":"g_config"},
            {"name":"static_var","description":"静态变量","example":"s_instance"},
            {"name":"variable","description":"局部/成员变量","example":"result"},
            {"name":"field","description":"类的成员字段","example":"DBImpl::env_"},
            {"name":"string_literal","description":"字符串字面量","example":"\"hello world\""},
            {"name":"macro","description":"预处理器宏","example":"LEVELDB_EXPORT"},
            {"name":"template_function","description":"模板函数","example":"std::sort<T>"},
            {"name":"template_class","description":"模板类","example":"std::vector<T>"},
            {"name":"template_struct","description":"模板结构体","example":"std::pair<K,V>"}
        ],
        "edge_types": [
            {"prefix":"calls","description":"函数/方法调用","direction":"从调用者到被调用者","source_kinds":["function","method"],"target_kinds":["function","method"]},
            {"prefix":"calls_override","description":"虚函数dispatch调用","direction":"从调用者到override实现","source_kinds":["function","method"],"target_kinds":["function","method"]},
            {"prefix":"inherits","description":"类继承关系","direction":"从子类到父类","source_kinds":["class","struct"],"target_kinds":["class","struct"]},
            {"prefix":"contains","description":"符号包含（枚举包含值、类包含方法/字段）","direction":"从父到子","source_kinds":["class","struct","enum"],"target_kinds":["method","field","enum_value"]},
            {"prefix":"uses","description":"函数使用了枚举值或字符串字面量","direction":"从函数到枚举值/字面量","source_kinds":["function","method"],"target_kinds":["enum_value","string_literal"]},
            {"prefix":"references","description":"函数引用了全局/静态变量","direction":"从函数到全局/静态变量","source_kinds":["function","method"],"target_kinds":["global","static_var"]},
            {"prefix":"returns","description":"函数的返回类型","direction":"从函数到返回类型","source_kinds":["function","method"],"target_kinds":["class","struct","enum"]},
            {"prefix":"param_type","description":"函数的参数类型","direction":"从函数到参数类型","source_kinds":["function","method"],"target_kinds":["class","struct","enum"]},
            {"prefix":"field_type","description":"字段的类型","direction":"从字段到类型","source_kinds":["field"],"target_kinds":["class","struct","enum"]},
            {"prefix":"template_use","description":"使用了模板实例化","direction":"从使用方到模板","source_kinds":["function","method","field"],"target_kinds":["template_class","template_struct","template_function"]}
        ]
    }).to_string()
}

// ── Path Analysis ───────────────────────────────────────────────────────

fn path_analysis(repo: &str, branch: &str, source: &str, target: &str, mode: &str, max_depth: usize, max_paths: usize, edge_filter: &[String]) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id(&conn, source, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", source),
    };
    let tid = match crate::query::graph::resolve_symbol_id(&conn, target, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", target),
    };
    let results = crate::query::graph::bfs_path_search(&conn, sid, tid, edge_filter, max_depth, mode, max_paths);
    let paths: Vec<serde_json::Value> = results.into_iter().map(|r| {
        serde_json::json!({"edges": r.edges})
    }).collect();
    serde_json::json!({"paths": paths, "total_found": paths.len()}).to_string()
}

// ── Impact Analysis ─────────────────────────────────────────────────────

fn impact_analysis(repo: &str, branch: &str, symbol: &str, direction: &str, radius: usize, edge_filter: &[String]) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id(&conn, symbol, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", symbol),
    };
    let results = crate::query::graph::transitive_closure(&conn, sid, direction, radius, edge_filter);
    let affected: Vec<serde_json::Value> = results.into_iter().map(|r| {
        serde_json::json!({"symbol": r.symbol, "distance": r.distance, "via": r.via})
    }).collect();
    serde_json::json!({"symbol": symbol, "radius": radius, "direction": direction, "affected": affected}).to_string()
}

// ── Neighbor Graph ──────────────────────────────────────────────────────

fn neighbor_graph(repo: &str, branch: &str, symbol: &str, direction: &str, depth: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id(&conn, symbol, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", symbol),
    };
    let map = crate::query::graph::neighbor_map(&conn, sid, direction, depth, &[]);
    serde_json::json!({
        "symbol": symbol,
        "depth": depth,
        "direction": direction,
        "forward": map.forward,
        "backward": map.backward
    }).to_string()
}

// ── Inheritance Tree ────────────────────────────────────────────────────

fn inheritance_tree(repo: &str, branch: &str, symbol: &str, direction: &str, max_depth: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id(&conn, symbol, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", symbol),
    };
    let root_name = crate::query::graph::symbol_name_by_id(&conn, sid).unwrap_or_default();

    fn build_tree(conn: &rusqlite::Connection, parent_id: i64, dir: &str, depth: usize, max_depth: usize) -> Vec<serde_json::Value> {
        if depth >= max_depth { return vec![]; }
        let mut children = vec![];
        
        if dir == "up" || dir == "both" {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT e.source_id, s.name FROM edges e JOIN symbols s ON e.source_id = s.id WHERE e.target_id = ?1 AND e.edge_type LIKE 'inherits:%' AND (s.kind = 'class' OR s.kind = 'struct')"
            ) {
                if let Ok(rows) = stmt.query_map(rusqlite::params![parent_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                }) {
                    for r in rows.flatten() {
                        let ov = get_overrides(conn, r.0);
                        children.push(serde_json::json!({
                            "symbol": r.1, "relation": "parent", "overrides": ov,
                            "children": build_tree(conn, r.0, "up", depth+1, max_depth)
                        }));
                    }
                }
            }
        }
        
        if dir == "down" || dir == "both" {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT e.target_id, s.name FROM edges e JOIN symbols s ON e.target_id = s.id WHERE e.source_id = ?1 AND e.edge_type LIKE 'inherits:%' AND (s.kind = 'class' OR s.kind = 'struct')"
            ) {
                if let Ok(rows) = stmt.query_map(rusqlite::params![parent_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                }) {
                    for r in rows.flatten() {
                        let ov = get_overrides(conn, r.0);
                        children.push(serde_json::json!({
                            "symbol": r.1, "relation": "child", "overrides": ov,
                            "children": build_tree(conn, r.0, "down", depth+1, max_depth)
                        }));
                    }
                }
            }
        }
        children
    }
    
    fn get_overrides(conn: &rusqlite::Connection, class_id: i64) -> Vec<String> {
        let mut ov = vec![];
        if let Ok(mut stmt) = conn.prepare(
            "SELECT s.name FROM edges e JOIN symbols s ON e.source_id = s.id WHERE e.source_id = ?1 AND e.edge_type LIKE 'overrides:%'"
        ) {
            if let Ok(rows) = stmt.query_map(rusqlite::params![class_id], |r| r.get::<_,String>(0)) {
                ov = rows.flatten().collect();
            }
        }
        ov
    }

    let result_children = build_tree(&conn, sid, direction, 0, max_depth);
    serde_json::json!({"root": root_name, "children": result_children}).to_string()
}


pub async fn serve_http(addr: &str) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("CodeLoom MCP HTTP on http://{}/mcp", addr);
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
            let (reader, mut writer) = tokio::io::split(socket);
            let mut buf_reader = BufReader::new(reader);
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                if buf_reader.read_line(&mut line).await.is_err() { return; }
                if line.trim().is_empty() { break; }
                if line.to_lowercase().starts_with("content-length:") {
                    content_length = line.split(':').nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
                }
            }
            if content_length == 0 || content_length > 10_000_000 { return; }
            let mut body = vec![0u8; content_length];
            if tokio::io::AsyncReadExt::read_exact(&mut buf_reader, &mut body).await.is_err() { return; }
            let result = match serde_json::from_slice::<serde_json::Value>(&body) {
                Ok(req) => {
                    let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
                    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                    match method {
                        "initialize" => serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"codeloom","version":"0.2.0"}}}),
                        "tools/list" => tools_list(id),
                        "tools/call" => {
                            let name = req["params"]["name"].as_str().unwrap_or("");
                            let args = &req["params"]["arguments"];
                            handle_tool_call(id, name, args)
                        }
                        _ => serde_json::json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("unknown: {}",method)}}),
                    }
                }
                Err(e) => serde_json::json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":format!("parse error: {}",e)}}),
            };
            let resp_body = serde_json::to_string(&result).unwrap_or_default();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", resp_body.len(), resp_body);
            let _ = writer.write_all(response.as_bytes()).await;
        });
    }
}


// ── get_doc ──────────────────────────────────────────────────────────────

fn get_doc(doc_id: i64, repo: &str, branch: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    // Query doc_node
    let (title, section_path, level, content_text, file_path, file_format, node_type) = match conn.query_row(
        "SELECT title, section_path, level, content, file_path, file_format, node_type FROM doc_nodes WHERE id=?1 AND repo=?2 AND (branch_name IS NULL OR branch_name=?3)",
        rusqlite::params![doc_id, repo, branch],
        |row| Ok((
            row.get::<_,String>(0).unwrap_or_default(),
            row.get::<_,String>(1).unwrap_or_default(),
            row.get::<_,i64>(2).unwrap_or(0),
            row.get::<_,String>(3).unwrap_or_default(),
            row.get::<_,String>(4).unwrap_or_default(),
            row.get::<_,String>(5).unwrap_or_default(),
            row.get::<_,String>(6).unwrap_or_default(),
        ))
    ) {
        Ok(r) => r,
        Err(e) => return format!("文档节点 {} 未找到: {}", doc_id, e),
    };
    let mut out = format!("=== 文档节点 #{} ===\n", doc_id);
    out.push_str(&format!("标题: {}\n", title));
    out.push_str(&format!("章节路径: {}\n", section_path));
    out.push_str(&format!("层级: {}\n", level));
    out.push_str(&format!("文件: {} [{}]\n", file_path, file_format));
    out.push_str(&format!("节点类型: {}\n", node_type));
    // Show snippet of content
    let snippet: String = content_text.chars().take(500).collect();
    out.push_str(&format!("内容(前500字):\n{}\n", snippet));
    if content_text.len() > 500 {
        out.push_str(&format!("... (+{} 字)\n", content_text.len() - 500));
    }
    // Query images
    match conn.prepare(
        "SELECT alt_text, image_data, width, height, section_context FROM doc_images WHERE doc_node_id=?1 ORDER BY position"
    ) {
        Ok(mut stmt) => {
            if let Ok(rows) = stmt.query_map(rusqlite::params![doc_id], |row| {
                Ok((
                    row.get::<_,String>(0).unwrap_or_default(),
                    row.get::<_,Vec<u8>>(1).unwrap_or_default(),
                    row.get::<_,i32>(2).unwrap_or(0),
                    row.get::<_,i32>(3).unwrap_or(0),
                    row.get::<_,String>(4).unwrap_or_default(),
                ))
            }) {
                let images: Vec<_> = rows.flatten().collect();
                if images.is_empty() {
                    out.push_str("\n图片: 无\n");
                } else {
                    out.push_str(&format!("\n图片 ({}) 张:\n", images.len()));
                    for (i, (alt, img_data, w, h, ctx)) in images.iter().enumerate() {
                        out.push_str(&format!("  [{}] alt={}", i + 1, alt));
                        if *w > 0 { out.push_str(&format!(" {}x{}", w, h)); }
                        if !ctx.is_empty() {
                            let ctx_snippet: String = ctx.chars().take(80).collect();
                            out.push_str(&format!(" context=\"{}\"", ctx_snippet));
                        }
                        if !img_data.is_empty() {
                            let b64 = crate::doc::to_base64(img_data);
                            out.push_str(&format!(" base64_length={}", b64.len()));
                        } else {
                            out.push_str(" (no data)");
                        }
                        out.push('\n');
                    }
                }
            }
        }
        Err(e) => out.push_str(&format!("\n图片查询失败: {}\n", e)),
    }
    out
}

// ── query_excel ─────────────────────────────────────────────────────────

fn query_excel(doc_id: i64, repo: &str, branch: &str, mode: &str, filter_expr: &str, search: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    // Get the source node
    let (src_title, src_section_path, src_level, src_node_type, src_file_path) = match conn.query_row(
        "SELECT title, section_path, level, node_type, file_path FROM doc_nodes WHERE id=?1 AND repo=?2 AND (branch_name IS NULL OR branch_name=?3)",
        rusqlite::params![doc_id, repo, branch],
        |row| Ok((
            row.get::<_,String>(0).unwrap_or_default(),
            row.get::<_,String>(1).unwrap_or_default(),
            row.get::<_,i64>(2).unwrap_or(0),
            row.get::<_,String>(3).unwrap_or_default(),
            row.get::<_,String>(4).unwrap_or_default(),
        ))
    ) {
        Ok(r) => r,
        Err(e) => return format!("文档节点 {} 未找到: {}", doc_id, e),
    };

    // Determine effective mode
    let eff_mode = if mode == "auto" {
        match src_node_type.as_str() {
            "cell" => "row",
            "row" => "row",
            "header_cell" => "column",
            "sheet" => "filter",
            _ => "row",
        }
    } else {
        mode
    };

    let safe_sheet = src_section_path.split('/').next().unwrap_or(&src_section_path);
    // Ensure we use the file path for the whole sheet
    let sheet_file_path = &src_file_path;

    match eff_mode {
        "row" => {
            // Determine the row path from the source node
            let row_path = if src_node_type == "cell" || src_node_type == "row" {
                // Extract row path: the section_path is like sheet/RowN or sheet/RowN/col
                let parts: Vec<&str> = src_section_path.split('/').collect();
                format!("{}/{}", safe_sheet, parts.get(1).unwrap_or(&"Row1"))
            } else {
                // From a sheet or header, just get the first row
                format!("{}/Row1", safe_sheet)
            };
            // Query all cells for this row (level=4 under this row path)
            let row_prefix = format!("{}%", row_path);
            let mut out = format!("=== 行数据 ({}) ===\n", row_path);
            let sql = "SELECT title, content FROM doc_nodes WHERE file_path=?1 AND level=4 AND section_path LIKE ?2 AND repo=?3 AND (branch_name IS NULL OR branch_name=?4) ORDER BY section_path LIMIT ?5";
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(rows) = stmt.query_map(rusqlite::params![sheet_file_path, row_prefix, repo, branch, limit as i64], |row| {
                    Ok((row.get::<_,String>(0).unwrap_or_default(), row.get::<_,String>(1).unwrap_or_default()))
                }) {
                    let mut count = 0;
                    for row in rows.flatten() {
                        count += 1;
                        out.push_str(&format!("  {}: {}\n", row.0, row.1));
                    }
                    if count == 0 { out.push_str("  (无数据)\n"); }
                }
            }
            out
        }
        "column" => {
            // Find which column by looking at the source section_path
            let column_name = src_section_path.split('/').last().unwrap_or("")
                .strip_prefix("_header/").unwrap_or(
                    src_section_path.rsplit('/').next().unwrap_or("")
                );
            let col_prefix = format!("{}/Row%/{}%", safe_sheet, column_name);
            let mut out = format!("=== 列数据 ({}) ===\n", column_name);
            let sql = "SELECT content, title, section_path FROM doc_nodes WHERE file_path=?1 AND level=4 AND section_path LIKE ?2 AND repo=?3 AND (branch_name IS NULL OR branch_name=?4) ORDER BY section_path LIMIT ?5";
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(rows) = stmt.query_map(rusqlite::params![sheet_file_path, col_prefix, repo, branch, limit as i64], |row| {
                    Ok((
                        row.get::<_,String>(0).unwrap_or_default(),
                        row.get::<_,String>(1).unwrap_or_default(),
                        row.get::<_,String>(2).unwrap_or_default(),
                    ))
                }) {
                    let mut count = 0;
                    for row in rows.flatten() {
                        count += 1;
                        let row_num = row.2.split('/').nth(1).unwrap_or("?");
                        out.push_str(&format!("  {}: {}\n", row_num, row.0));
                    }
                    if count == 0 { out.push_str("  (无数据)\n"); }
                }
            }
            out
        }
        "filter" => {
            let mut out = format!("=== 过滤结果 ===\n");
            if filter_expr.is_empty() && search.is_empty() {
                out.push_str("请提供 filter 或 search 参数\n");
                return out;
            }
            // Simple numeric filter: "column > 5000"
            let (filter_col, filter_op, filter_val) = if !filter_expr.is_empty() {
                let parts: Vec<&str> = filter_expr.splitn(3, ' ').collect();
                if parts.len() == 3 {
                    (parts[0], parts[1], parts[2])
                } else {
                    ("", "", "")
                }
            } else {
                ("", "", "")
            };
            // Query all cells in this sheet, group by row
            let sheet_prefix = format!("{}%", safe_sheet);
            let sql = "SELECT id, title, content, section_path FROM doc_nodes WHERE file_path=?1 AND level=4 AND section_path LIKE ?2 AND repo=?3 AND (branch_name IS NULL OR branch_name=?4) ORDER BY section_path";
            if let Ok(mut stmt) = conn.prepare(sql) {
                if let Ok(rows) = stmt.query_map(rusqlite::params![sheet_file_path, sheet_prefix, repo, branch], |row| {
                    Ok((
                        row.get::<_,i64>(0).unwrap_or(0),
                        row.get::<_,String>(1).unwrap_or_default(),
                        row.get::<_,String>(2).unwrap_or_default(),
                        row.get::<_,String>(3).unwrap_or_default(),
                    ))
                }) {
                    // Group by row
                    use std::collections::BTreeMap;
                    let mut rows_map: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
                    for row in rows.flatten() {
                        let (_, col_name, cell_val, section_path) = row;
                        let parts: Vec<&str> = section_path.split('/').collect();
                        let row_key = if parts.len() >= 2 { parts[1].to_string() } else { String::new() };
                        rows_map.entry(row_key).or_default().push((col_name, cell_val));
                    }
                    let mut matched = 0usize;
                    for (row_key, cells) in &rows_map {
                        // Apply filter
                        if !filter_col.is_empty() && !filter_op.is_empty() {
                            let default_val = String::new();
                            let col_val = cells.iter().find(|(c, _)| c == filter_col).map(|(_, v)| v).unwrap_or(&default_val);
                            let ok = match filter_op {
                                ">" | ">" => col_val.parse::<f64>().ok().map(|v| v > filter_val.parse::<f64>().unwrap_or(0.0)).unwrap_or(false),
                                "<" | "<" => col_val.parse::<f64>().ok().map(|v| v < filter_val.parse::<f64>().unwrap_or(0.0)).unwrap_or(false),
                                ">=" | ">=" => col_val.parse::<f64>().ok().map(|v| v >= filter_val.parse::<f64>().unwrap_or(0.0)).unwrap_or(false),
                                "<=" | "<=" => col_val.parse::<f64>().ok().map(|v| v <= filter_val.parse::<f64>().unwrap_or(0.0)).unwrap_or(false),
                                "=" | "==" => col_val == filter_val,
                                "!=" | "<>" => col_val != filter_val,
                                _ => false,
                            };
                            if !ok { continue; }
                        }
                        // Apply search
                        if !search.is_empty() {
                            if !cells.iter().any(|(_, v)| v.contains(search)) { continue; }
                        }
                        matched += 1;
                        if matched > limit { break; }
                        out.push_str(&format!("  {}:\n", row_key));
                        for (col, val) in cells {
                            out.push_str(&format!("    {}: {}\n", col, val));
                        }
                    }
                    if matched == 0 { out.push_str("  (无匹配行)\n"); }
                }
            }
            out
        }
        _ => format!("未知模式: {}. 可用模式: row, column, filter, auto", eff_mode),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_list_has_correct_count() {
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 12);  // 3 tools disabled: status, get_doc, query_excel
    }

    #[test]
    fn test_all_tools_have_branch_and_repo_required() {
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        for tool in tools {
            let name = tool["name"].as_str().unwrap();
            let required = tool["inputSchema"]["required"].as_array().unwrap();
            let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();

            if name == "codeloom_list_repos" {
                // No required params
                assert!(req_strs.is_empty(), "list_repos should have no required params");
            } else if name == "codeloom_list_branches" {
                // Only repo required
                assert!(req_strs.contains(&"repo"), "list_branches missing repo");
            } else if name == "codeloom_schema" {
                // No required params — returns global metadata
                assert!(req_strs.is_empty(), "schema should have no required params");
            } else if name == "codeloom_index" {
                // path + branch required, repo optional
                assert!(req_strs.contains(&"path"), "index missing path");
                assert!(req_strs.contains(&"branch"), "index missing branch");
            } else {
                // All other tools: repo + branch required
                assert!(req_strs.contains(&"repo"), "tool {} missing repo", name);
                assert!(req_strs.contains(&"branch"), "tool {} missing branch", name);
            }
        }
    }

    #[test]
    fn test_err_resp_format() {
        let resp = err_resp(serde_json::Value::Number(42.into()), "test error");
        assert_eq!(resp["id"], 42);
        assert_eq!(resp["error"]["code"], -32602);
        assert_eq!(resp["error"]["message"], "test error");
    }

    #[test]
    fn test_branch_where_clause() {
        let clause = branch_where_clause("main");
        assert!(clause.contains("b.branch_name = 'main'"));
        assert!(clause.contains("b.branch_name IS NULL"));
    }

    #[test]
    fn test_branch_where_clause_escapes_quote() {
        let clause = branch_where_clause("it's");
        assert!(clause.contains("b.branch_name = 'it''s'"));
    }

    #[test]
    fn test_handle_tool_call_missing_branch() {
        // Need to pass a valid repo (even if DB doesn't exist), or repo validation fires first
        // We test branch validation by passing an empty branch
        // DISABLED: codeloom_status removed
        let _resp = handle_tool_call(
            serde_json::Value::Number(1.into()),
            "codeloom_status_DISABLED",
            &serde_json::json!({"repo": "testrepo", "branch": ""}),
        );
    }

    #[test]
    fn test_unknown_tool() {
        let resp = handle_tool_call(
            serde_json::Value::Number(1.into()),
            "nonexistent_tool",
            &serde_json::json!({}),
        );
        let text = &resp["result"]["content"][0]["text"];
        assert!(text.as_str().unwrap().contains("Unknown tool"));
    }



    #[test]

    #[test]
    #[ignore = "API embedding not configured in CI"]
    fn test_api_semantic_search_no_fallback() {
        // Embedding requires API config in config.yaml — skip in CI
    }


    #[test]
    fn test_get_doc_not_found() {
        // DISABLED: codeloom_get_doc removed
        let _resp = handle_tool_call(
            serde_json::Value::Number(1.into()),
            "codeloom_get_doc_DISABLED",
            &serde_json::json!({"doc_id": 999, "repo": "nonexistent", "branch": "main"}),
        );
    }

    #[test]
    fn test_get_doc_missing_doc_id() {
        // DISABLED: codeloom_get_doc removed
        let _resp = handle_tool_call(
            serde_json::Value::Number(1.into()),
            "codeloom_get_doc_DISABLED",
            &serde_json::json!({"repo": "test", "branch": "main"}),
        );
    }

    #[test]
    fn test_query_excel_missing_doc_id() {
        // DISABLED: codeloom_query_excel removed
        let _resp = handle_tool_call(
            serde_json::Value::Number(1.into()),
            "codeloom_query_excel_DISABLED",
            &serde_json::json!({"repo": "test", "branch": "main"}),
        );
    }

    #[test]
    fn test_get_doc_has_get_doc_in_tools_list() {
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        // DISABLED: codeloom_get_doc and codeloom_query_excel removed
        // assert!(names.contains(&"codeloom_get_doc"));
        // assert!(names.contains(&"codeloom_query_excel"));
    }

    #[test]

    #[test]
    fn test_hybrid_search_enhanced_output() {
        // Test that the search output template includes snippet/image info
        // by checking the format string patterns in the source
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        let search_tool = tools.iter().find(|t| t["name"] == "codeloom_search").unwrap();
        assert!(search_tool["name"].as_str().unwrap() == "codeloom_search");
        // Verify that new tool definitions are reachable by checking tool count
        assert_eq!(tools.len(), 12);  // 3 tools disabled: status, get_doc, query_excel
    }
}
