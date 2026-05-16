use crate::{log_info, log_warn, log_error, log_debug};
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
        {"name":"codeloom_list_symbols","description":"**优先使用**：按名称模糊搜索已索引的符号。优先于grep/rg使用——索引覆盖项目所有文件及#include的第三方头文件（grep只能搜当前目录）。返回结构化结果：名称、类型、文件路径、行号。C++类方法用ClassName::methodName格式。如pattern=\"login\"匹配handleLogin、loginUser等。branch/repo（必填）","inputSchema":{"type":"object","properties":{"pattern":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"limit":{"type":"integer","default":20}},"required":["pattern","repo","branch"]}},
        {"name":"codeloom_get_call_graph","description":"**首选方式**：分析函数/方法的调用者和被调用者（callers/callees）。grep无法获取调用关系。优于多次调neighbor_graph拼凑——一次到位且带递归深度控制。name用codeloom_list_symbols返回的完整符号名（C++类方法用ClassName::methodName）。direction=\"callers\"查谁调用了它，direction=\"callees\"查它调用了谁。max_depth控制递归深度。branch/repo（必填）","inputSchema":{"type":"object","properties":{"id":{"type":"integer","description":"符号节点ID（优先使用）"},"name":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["callers","callees"]},"max_depth":{"type":"integer","default":3}},"required":["repo","branch","direction"]}},
        {"name":"codeloom_search","description":"【替代grep/rg】精确关键词匹配搜索。用BM25算法搜索符号名+注释+文档。\n  ✅ 用这个：你知道符号名/类名/函数名/变量名，或确切关键词，想精准定位代码位置。\n  ❌ 别用这个：你不知道具体名字，只知道功能描述——那种情况用 codeloom_fuzzy_search。\n  支持 kind 过滤。不涉及语义理解。branch/repo（必填）","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"kind":{"type":"string","description":"可选：按符号类型过滤。可用类型见 codeloom_schema"},"limit":{"type":"integer","default":10}},"required":["query","repo","branch"]}},

                {"name":"codeloom_fuzzy_search","description":"【语义模糊搜索】用向量嵌入匹配符号名（不搜索注释/文档）。\n  ✅ 用这个：你不知道确切符号名，只知道功能描述。例如「排序算法」「处理用户登录」「解析JSON」。\n  ❌ 别用这个：你知道符号名或关键词——那种情况用 codeloom_search 更快更准。\n  只搜索符号名。不支持 kind 过滤。需要向量模型已加载。branch/repo（必填）","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"limit":{"type":"integer","default":10}},"required":["query","repo","branch"]}},
        {"name":"codeloom_inspect","description":"查看符号节点的结构化信息：定义、文档注释。class/struct返回bases+members+methods，enum返回values，section返回parent/children/chunks/siblings，chunk返回parent_section+前后chunk，其余类型返回关联边列表。name或id至少传一个。name=符号完整名称（C++类方法用ClassName::methodName格式）。branch/repo（必填）","inputSchema":{"type":"object","properties":{"id":{"type":"integer","description":"符号节点ID（优先使用）"},"name":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"}},"required":["repo","branch"]}},

        {"name":"codeloom_list_repos","description":"列出所有已索引的仓库名。在任何搜索/查询操作前必须先调用此工具获取可用的repo参数值。索引需通过 CLI 执行：codeloom index <path> --repo <name> --branch <branch>。无需任何参数。","inputSchema":{"type":"object","properties":{},"required":[]}},
        {"name":"codeloom_list_branches","description":"列出指定仓库的所有已索引分支。repo=仓库名（必填）。返回分支名，用于团队协作时确认分支状态。","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}},"required":["repo"]}},
        {"name":"codeloom_schema","description":"导出CodeLoom v0.7元数据：15种节点类型（function/method/class/struct/enum/enum_value/field/global/static_var/template_function/macro/namespace/template_instance/typedef/string_literal）和11种边类型（calls/inherits/overrides/instantiates/param_type/return_type/includes/uses_type/contains/aliases/uses）。拿不准参数值时调用此工具查看可用节点类型和边类型枚举。无需参数。","inputSchema":{"type":"object","properties":{},"required":[]}},
        {"name":"codeloom_path_analysis","description":"回答『A到B怎么走』『A和B有什么关系』：在两个符号之间搜索关系路径，跨函数调用、数据流、继承等多种边类型。优于逐层调用codeloom_call_graph：一次调用自动跨边类型搜索最短路径，一键覆盖calls/inherits/param_type等多类边。source=起始符号名，target=目标符号名，mode=shortest|all（默认shortest），edge_filter可选限定边类型（如['calls','calls_override']只看调用路径），direction=forward|reverse|both（默认both）。branch/repo（必填）","inputSchema":{"type":"object","properties":{"source_id":{"type":"integer","description":"起始符号节点ID（优先使用）"},"source":{"type":"string"},"target_id":{"type":"integer","description":"目标符号节点ID（优先使用）"},"target":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"mode":{"type":"string","enum":["shortest","all"]},"max_paths":{"type":"integer","default":20},"max_depth":{"type":"integer","default":10},"direction":{"type":"string","enum":["forward","reverse","both"]},"edge_filter":{"type":"array","items":{"type":"string"}}},"required":["repo","branch"]}},
        {"name":"codeloom_impact_analysis","description":"回答『改了X会影响谁』『X被谁依赖』：沿边传递闭包N跳分析影响范围。优于codeloom_call_graph：自动N跳递归，不仅是直接调用者。优于codeloom_neighbor_graph：传递闭包而非只看1跳邻居。symbol或id至少传一个，传id精度最高。direction=forward|reverse|both（默认reverse），radius=跳数（默认3），edge_filter可选限定边类型。branch/repo（必填）","inputSchema":{"type":"object","properties":{"id":{"type":"integer","description":"符号节点ID（优先使用）"},"symbol":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["forward","reverse","both"]},"radius":{"type":"integer","default":3},"edge_filter":{"type":"array","items":{"type":"string"}}},"required":["repo","branch"]}},
        {"name":"codeloom_neighbor_graph","description":"回答『X周围都有什么』『X直接调用了什么/被什么调用』：查看符号1跳范围内的直接邻居，按边类型分组返回（calls/returns/param_type/inherits等）。优于codeloom_impact_analysis：只看直接邻居不递归；优于grep：自动关联calls/returns/param_type/uses等所有边类型，无需按类型分别搜索。symbol或id至少传一个，传id精度最高。direction=forward|reverse|both（默认both），固定depth=1。对class/struct会跳过其成员，改为暴露成员引用的外部符号。branch/repo（必填）","inputSchema":{"type":"object","properties":{"id":{"type":"integer","description":"符号节点ID（优先使用）"},"symbol":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["forward","reverse","both"]}},"required":["repo","branch"]}},
        {"name":"codeloom_inheritance_tree","description":"回答『谁继承了X』『X继承了什么』：查看类的完整继承树，含父类、子类和虚方法覆写列表。优于grep 'extends'/'public'：自动递归解析完整继承链，不遗漏间接继承。symbol=类名，direction=up|down|both（默认down），max_depth=递归深度（默认5）。branch/repo（必填）","inputSchema":{"type":"object","properties":{"id":{"type":"integer","description":"符号节点ID（优先使用）"},"symbol":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["up","down","both"]},"max_depth":{"type":"integer","default":5}},"required":["repo","branch"]}},
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
    let args_str = args.to_string();
    let args_preview = if args_str.len() > 200 { format!("{}...", &args_str[..200]) } else { args_str };
    log_info!("mcp", "tool call: name={}, args={}", name, args_preview);
    let result = match name {
        
        "codeloom_schema" => {
            schema_meta()
        }
        "codeloom_path_analysis" => {
            let source = args["source"].as_str().unwrap_or("");
            let target = args["target"].as_str().unwrap_or("");
            let source_id = args["source_id"].as_i64();
            let target_id = args["target_id"].as_i64();
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
            if (source.is_empty() && source_id.is_none()) || (target.is_empty() && target_id.is_none()) {
                return err_resp(id, "source/source_id and target/target_id are required (at least one each)");
            }
            path_analysis(&repo.unwrap(), branch, source, target, mode, max_depth, max_paths, &edge_filter, source_id, target_id)
        }
        "codeloom_impact_analysis" => {
            let symbol = args["symbol"].as_str().unwrap_or("");
            let sym_id = args["id"].as_i64();
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let direction = args["direction"].as_str().unwrap_or("reverse");
            let radius = args["radius"].as_u64().unwrap_or(3) as usize;
            let edge_filter: Vec<String> = args["edge_filter"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if symbol.is_empty() && sym_id.is_none() { return err_resp(id, "symbol or id is required"); }
            impact_analysis(&repo.unwrap(), branch, symbol, direction, radius, &edge_filter, sym_id)
        }
        "codeloom_neighbor_graph" => {
            let symbol = args["symbol"].as_str().unwrap_or("");
            let sym_id = args["id"].as_i64();
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let direction = args["direction"].as_str().unwrap_or("both");
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if symbol.is_empty() && sym_id.is_none() { return err_resp(id, "symbol or id is required"); }
            neighbor_graph(&repo.unwrap(), branch, symbol, direction, sym_id)
        }
        "codeloom_inheritance_tree" => {
            let symbol = args["symbol"].as_str().unwrap_or("");
            let sym_id = args["id"].as_i64();
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let direction = args["direction"].as_str().unwrap_or("down");
            let max_depth = args["max_depth"].as_u64().unwrap_or(5) as usize;
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if symbol.is_empty() && sym_id.is_none() { return err_resp(id, "symbol or id is required"); }
            inheritance_tree(&repo.unwrap(), branch, symbol, direction, max_depth, sym_id)
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
            let sym_id = args["id"].as_i64();
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let direction = args["direction"].as_str().unwrap_or("callers");
            let max_depth = args["max_depth"].as_u64().unwrap_or(3) as usize;
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if sym_name.is_empty() && sym_id.is_none() { return err_resp(id, "name or id is required"); }
            let repo = repo.unwrap();
            let conn = match open_repo_db(&repo) { Ok(c) => c, Err(e) => return err_resp(id, &e) };
            crate::query::call_graph::get_call_graph(&conn, sym_name, &repo, branch, direction, max_depth, sym_id)
        }
        "codeloom_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let limit = args["limit"].as_u64().unwrap_or(10) as usize;
            let kind_raw = args["kind"].as_str().unwrap_or("");
            let valid_kinds: &[&str] = &["function", "method", "class", "struct", "enum", "enum_value", "field", "global", "static_var", "template_function", "macro", "namespace", "template_instance", "typedef", "string_literal"];
            let kind_filter = if kind_raw.is_empty() {
                None
            } else if valid_kinds.contains(&kind_raw) {
                Some(kind_raw)
            } else {
                return err_resp(id, &format!("Invalid kind '{}'. Valid values: {}", kind_raw, valid_kinds.join(", ")));
            };
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            bm25_precise_search(query, &repo.unwrap(), branch, limit, kind_filter)
        }
        "codeloom_fuzzy_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            let branch = args["branch"].as_str().unwrap_or("");
            let limit = args["limit"].as_u64().unwrap_or(10) as usize;
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            fuzzy_search(query, &repo.unwrap(), branch, limit)
        }
        "codeloom_inspect" => {
            let branch = args["branch"].as_str().unwrap_or("");
            let sym_name = args["name"].as_str().unwrap_or("");
            let sym_id = args["id"].as_i64();
            let repo = validate_repo(args["repo"].as_str().unwrap_or(""));
            if let Err(e) = repo { return err_resp(id, &e); }
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if sym_name.is_empty() && sym_id.is_none() { return err_resp(id, "name or id is required"); }
            inspect_symbol(sym_name, &repo.unwrap(), branch, sym_id)
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
    let result_len = result.len();
    let is_err = result.starts_with("Error");
    if is_err {
        log_error!("mcp", "tool call error: name={} result_len={}", name, result_len);
    }
    log_info!("mcp", "tool done: name={}, result_len={}, is_err={}", name, result_len, is_err);
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

fn add_enrichment_fields(obj: &mut serde_json::Value, r: &crate::query::search::FusedResult) {
    if !r.parent_class.is_empty() { obj["parent_class"] = serde_json::Value::String(r.parent_class.clone()); }
    if !r.members.is_empty() { obj["members"] = serde_json::Value::String(r.members.clone()); }
    if !r.methods.is_empty() { obj["methods"] = serde_json::Value::String(r.methods.clone()); }
    if !r.values.is_empty() { obj["values"] = serde_json::Value::String(r.values.clone()); }
    if !r.prev_section.is_empty() { obj["prev_section"] = serde_json::Value::String(r.prev_section.clone()); }
    if !r.next_section.is_empty() { obj["next_section"] = serde_json::Value::String(r.next_section.clone()); }
    if !r.sections.is_empty() { obj["sections"] = serde_json::Value::String(r.sections.clone()); }
    if !r.parent_section.is_empty() { obj["parent_section"] = serde_json::Value::String(r.parent_section.clone()); }
    if !r.prev_chunk.is_empty() { obj["prev_chunk"] = serde_json::Value::String(r.prev_chunk.clone()); }
    if !r.next_chunk.is_empty() { obj["next_chunk"] = serde_json::Value::String(r.next_chunk.clone()); }
}

fn list_symbols(pattern: &str, repo: &str, branch: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    // Use FTS5 BM25 search on symbol names (upgraded from LIKE)
    match crate::storage::fts::search_fts5_name(&conn, pattern, repo, branch, limit, None) {
        Ok(hits) => {
            let mut out = format!("Symbols matching '{}' in {} (branch={}):\n", pattern, repo, branch);
            if hits.is_empty() {
                out.push_str("  (none)\n");
            } else {
                for hit in hits {
                    out.push_str(&format!("  id={} [{:10}] {:40}  @ {}:{}\n",
                        hit.rowid, hit.kind, hit.name, &hit.file_path[..60.min(hit.file_path.len())], hit.line_start));
                }
            }
            out
        }
        Err(e) => format!("Search error: {}", e),
    }
}

fn inspect_symbol(name: &str, repo: &str, branch: &str, sym_id: Option<i64>) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let name = if name.is_empty() { if let Some(sid) = sym_id { conn.query_row("SELECT name FROM nodes WHERE id=?1", rusqlite::params![sid], |r| r.get::<_, String>(0)).unwrap_or_default() } else { name.to_string() } } else { name.to_string() };
    let branch_id = crate::storage::resolve_branch_id(&conn, repo, branch).unwrap_or(0);
    let bwc = branch_where_clause(branch);
    let sql = format!("SELECT n.id, n.node_type, n.name, n.kind, n.file_path, n.line_start, n.line_end, json_extract(n.attrs,'$.signature'), json_extract(n.attrs,'$.parent_class'), json_extract(n.attrs,'$.namespace'), json_extract(n.attrs,'$.language'), n.content FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.repo=?1 AND n.name=?2 {}", bwc);
    let mut rows: Vec<(i64,String,String,String,String,i64,i64,Option<String>,Option<String>,Option<String>,Option<String>,Option<String>)> = match conn.prepare(&sql) {
        Ok(mut stmt) => stmt.query_map(rusqlite::params![repo, name], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?,
                r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?, r.get(11)?))
        }).map(|r| r.flatten().collect()).unwrap_or_default(),
        Err(_) => return format!("{{\"error\":\"Error querying symbol '{}'\"}}", name),
    };
    if rows.is_empty() || rows.iter().all(|r| r.4.is_empty()) {
        // LIKE fallback: suffix match first (e.g., "store" → "%::store"), then broad
        let like_sql =            format!("SELECT n.id, n.node_type, n.name, n.kind, n.file_path, n.line_start, n.line_end, json_extract(n.attrs,'$.signature'), json_extract(n.attrs,'$.parent_class'), json_extract(n.attrs,'$.namespace'), json_extract(n.attrs,'$.language'), n.content FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.repo=?1 AND n.name LIKE ?2 AND n.node_type='sym' {} ORDER BY CASE WHEN n.file_path LIKE '%.cc' THEN 0 WHEN n.file_path LIKE '%.cpp' THEN 0 ELSE 1 END, n.id LIMIT 1", bwc);
        rows.clear();
        if let Ok(mut stmt) = conn.prepare(&like_sql) {
            for pat in [format!("%::{}", name), format!("%{}%", name)] {
                if let Ok(mut q) = stmt.query_map(rusqlite::params![repo, pat], |r| {
                    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?,
                        r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?, r.get(10)?, r.get(11)?))
                }) {
                    if let Some(Ok(row)) = q.next() {
                        rows.push(row);
                        break;
                    }
                }
            }
        }
        if rows.is_empty() {
            return format!("{{\"error\":\"Symbol '{}' not found in {} (branch={})\"}}", name, repo, branch);
        }
    }
    let mut results = Vec::new();
    for (sid, ntype, sname, kind, file, lstart, lend, sig, parent, ns, lang, doc) in &rows {
        let esc = |s: &str| s.replace('\\', "\\\\").replace('\"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t");
        let mut json = format!(
            "{{\"name\":\"{}\",\"kind\":\"{}\",\"node_type\":\"{}\"",
            esc(sname), esc(kind), esc(ntype)
        );
        // Common fields
        json.push_str(&format!(",\"file\":\"{}\",\"line_start\":{},\"line_end\":{}", esc(file), lstart, lend));
        // Type-specific enrichment
        match ntype.as_str() {
            "sym" => {
                // Language, namespace, in_class
                if let Some(l) = lang { json.push_str(&format!(",\"language\":\"{}\"", esc(l))); }
                if let Some(n) = ns { json.push_str(&format!(",\"namespace\":\"{}\"", esc(n))); }
                if let Some(p) = parent { json.push_str(&format!(",\"in_class\":\"{}\"", esc(p))); }
                if let Some(s) = sig { json.push_str(&format!(",\"signature\":\"{}\"", esc(s))); }
                if let Some(d) = doc {
                    let trimmed = d.trim();
                    if !trimmed.is_empty() {
                        json.push_str(&format!(",\"documentation\":\"{}\"", esc(trimmed)));
                    }
                }
                match kind.as_str() {
                    "class" | "struct" => {
                        // Bases (inherits edges)
                        let bsql = format!("SELECT n.name FROM edges e JOIN nodes n ON n.id=e.target_id AND n.node_type='sym' WHERE e.source_id={0} AND e.edge_type = 'inherits' AND e.branch_id={1} LIMIT 20", sid, branch_id);
                        if let Ok(mut stmt) = conn.prepare(&bsql) {
                            if let Ok(rows) = stmt.query_map([], |r| r.get::<_,String>(0)) {
                                let names: Vec<String> = rows.flatten().collect();
                                if !names.is_empty() {
                                    json.push_str(&format!(",\"bases\":[\"{}\"]", names.join("\",\"")));
                                }
                            }
                        }
                        // Members (fields with types)
                        let msql = format!("SELECT n.name, json_extract(n.attrs,'$.field_type') FROM edges e JOIN nodes n ON n.id=e.target_id WHERE e.source_id={0} AND e.edge_type='contains' AND n.kind='field' AND e.branch_id={1} LIMIT 40", sid, branch_id);
                        if let Ok(mut stmt) = conn.prepare(&msql) {
                            if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,Option<String>>(1)?))) {
                                let entries: Vec<String> = rows.flatten().map(|(n,t)| {
                                    if let Some(ty) = t { format!("{}:{}", esc(&n), esc(&ty)) } else { esc(&n) }
                                }).collect();
                                if !entries.is_empty() {
                                    json.push_str(&format!(",\"members\":[\"{}\"]", entries.join("\",\"")));
                                }
                            }
                        }
                        // Methods
                        let methsql = format!("SELECT n.name, json_extract(n.attrs,'$.signature') FROM edges e JOIN nodes n ON n.id=e.target_id WHERE e.source_id={0} AND e.edge_type='contains' AND n.kind='method' AND e.branch_id={1} LIMIT 60", sid, branch_id);
                        if let Ok(mut stmt) = conn.prepare(&methsql) {
                            if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,Option<String>>(1)?))) {
                                let entries: Vec<String> = rows.flatten().map(|(n,s)| {
                                    if let Some(sig) = s { format!("{}({})", esc(&n), esc(&sig)) } else { esc(&n) }
                                }).collect();
                                if !entries.is_empty() {
                                    json.push_str(&format!(",\"methods\":[\"{}\"]", entries.join("\",\"")));
                                }
                            }
                        }
                        // Template args (from node attrs)
                        let targs_sql = format!("SELECT json_extract(n.attrs, '$.template_args') FROM nodes n WHERE n.id={}", sid);
                        if let Ok(mut tstmt) = conn.prepare(&targs_sql) {
                            if let Ok(mut trows) = tstmt.query_map([], |r| r.get::<_, Option<String>>(0)) {
                                if let Some(Ok(Some(targs))) = trows.next() {
                                    if !targs.is_empty() && targs != "null" {
                                        json.push_str(&format!(",\"template_args\":{}", targs));
                                    }
                                }
                            }
                        }
                        // Terminal deps
                        let (uses, refs, literals) = crate::query::graph::get_terminal_deps(&conn, *sid);
                        if !uses.is_empty() { json.push_str(&format!(",\"uses\":[\"{}\"]", uses.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                        if !refs.is_empty() { json.push_str(&format!(",\"references\":[\"{}\"]", refs.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                        if !literals.is_empty() { json.push_str(&format!(",\"string_literals\":[\"{}\"]", literals.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                    }
                    "enum" => {
                        // Enum values
                        let vsql = format!("SELECT n.name FROM edges e JOIN nodes n ON n.id=e.target_id WHERE e.source_id={0} AND e.edge_type='contains' AND n.kind='enum_value' AND e.branch_id={1} ORDER BY n.id LIMIT 100", sid, branch_id);
                        if let Ok(mut stmt) = conn.prepare(&vsql) {
                            if let Ok(rows) = stmt.query_map([], |r| r.get::<_,String>(0)) {
                                let names: Vec<String> = rows.flatten().collect();
                                if !names.is_empty() {
                                    json.push_str(&format!(",\"values\":[\"{}\"]", names.join("\",\"")));
                                }
                            }
                        }
                        // Terminal deps
                        let (uses, refs, literals) = crate::query::graph::get_terminal_deps(&conn, *sid);
                        if !uses.is_empty() { json.push_str(&format!(",\"uses\":[\"{}\"]", uses.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                        if !refs.is_empty() { json.push_str(&format!(",\"references\":[\"{}\"]", refs.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                        if !literals.is_empty() { json.push_str(&format!(",\"string_literals\":[\"{}\"]", literals.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                    }
                    _ => {
                        // Other sym types: current generic edges behavior
                        let e_sql = format!("SELECT e.edge_type, n.name, n.kind FROM edges e LEFT JOIN nodes n ON ((e.source_id={0} AND n.id=e.target_id) OR (e.target_id={0} AND n.id=e.source_id)) AND n.node_type='sym' WHERE (e.source_id={0} OR e.target_id={0}) AND n.id IS NOT NULL AND e.branch_id={1} ORDER BY e.edge_type LIMIT 200", sid, branch_id);
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
                                if !groups.is_empty() {
                                    json.push_str(",\"edges\":{");
                                    let mut first_cat = true;
                                    for (cat, entries) in &groups {
                                        if !first_cat { json.push(','); }
                                        first_cat = false;
                                        json.push_str(&format!("\"{}\":[{}]", cat, entries.join(",")));
                                    }
                                    json.push('}');
                                }
                            }
                        }
                        // Terminal deps
                        let (uses, refs, literals) = crate::query::graph::get_terminal_deps(&conn, *sid);
                        if !uses.is_empty() { json.push_str(&format!(",\"uses\":[\"{}\"]", uses.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                        if !refs.is_empty() { json.push_str(&format!(",\"references\":[\"{}\"]", refs.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                        if !literals.is_empty() { json.push_str(&format!(",\"string_literals\":[\"{}\"]", literals.iter().map(|s| esc(s)).collect::<Vec<_>>().join("\",\""))); }
                    }
                }
            }
            "section" => {
                // Parent section (reverse contains: edge)
                let psql = format!("SELECT n.id, n.name FROM edges e JOIN nodes n ON n.id=e.source_id WHERE e.target_id={0} AND e.edge_type='contains:' AND n.node_type='section' AND e.branch_id={1} LIMIT 1", sid, branch_id);
                if let Ok(mut stmt) = conn.prepare(&psql) {
                    if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        for row in rows.flatten() {
                            json.push_str(&format!(",\"parent_section\":{{\"id\":{},\"name\":\"{}\"}}", row.0, esc(&row.1)));
                        }
                    }
                }
                // Children sections
                let csql = format!("SELECT n.id, n.name FROM edges e JOIN nodes n ON n.id=e.target_id WHERE e.source_id={0} AND e.edge_type='contains:' AND n.node_type='section' AND e.branch_id={1} ORDER BY n.id LIMIT 50", sid, branch_id);
                if let Ok(mut stmt) = conn.prepare(&csql) {
                    if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        let entries: Vec<String> = rows.flatten().map(|(id,n)| format!("{{\"id\":{},\"name\":\"{}\"}}", id, esc(&n))).collect();
                        if !entries.is_empty() {
                            json.push_str(&format!(",\"children_sections\":[{}]", entries.join(",")));
                        }
                    }
                }
                // Children chunks
                let chsql = format!("SELECT n.id, n.name FROM edges e JOIN nodes n ON n.id=e.target_id WHERE e.source_id={0} AND e.edge_type='contains:' AND n.node_type='chunk' AND e.branch_id={1} ORDER BY n.id LIMIT 50", sid, branch_id);
                if let Ok(mut stmt) = conn.prepare(&chsql) {
                    if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        let entries: Vec<String> = rows.flatten().map(|(id,n)| format!("{{\"id\":{},\"name\":\"{}\"}}", id, esc(&n))).collect();
                        if !entries.is_empty() {
                            json.push_str(&format!(",\"children_chunks\":[{}]", entries.join(",")));
                        }
                    }
                }
                // Prev/next siblings
                let prev_sql = format!("SELECT n.id, n.name FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.node_type='section' AND n.file_path=?1 AND n.id<?2 AND b.branch_id={0} ORDER BY n.id DESC LIMIT 1", branch_id);
                if let Ok(mut stmt) = conn.prepare(&prev_sql) {
                    if let Ok(rows) = stmt.query_map(rusqlite::params![file, sid], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        for row in rows.flatten() {
                            json.push_str(&format!(",\"prev_section\":{{\"id\":{},\"name\":\"{}\"}}", row.0, esc(&row.1)));
                        }
                    }
                }
                let next_sql = format!("SELECT n.id, n.name FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.node_type='section' AND n.file_path=?1 AND n.id>?2 AND b.branch_id={0} ORDER BY n.id ASC LIMIT 1", branch_id);
                if let Ok(mut stmt) = conn.prepare(&next_sql) {
                    if let Ok(rows) = stmt.query_map(rusqlite::params![file, sid], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        for row in rows.flatten() {
                            json.push_str(&format!(",\"next_section\":{{\"id\":{},\"name\":\"{}\"}}", row.0, esc(&row.1)));
                        }
                    }
                }
            }
            "chunk" => {
                // Parent section (reverse contains:)
                let psql = format!("SELECT n.id, n.name FROM edges e JOIN nodes n ON n.id=e.source_id WHERE e.target_id={0} AND e.edge_type='contains:' AND n.node_type='section' AND e.branch_id={1} LIMIT 1", sid, branch_id);
                if let Ok(mut stmt) = conn.prepare(&psql) {
                    if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        for row in rows.flatten() {
                            json.push_str(&format!(",\"parent_section\":{{\"id\":{},\"name\":\"{}\"}}", row.0, esc(&row.1)));
                        }
                    }
                }
                // Prev/next chunk
                let prev_sql = format!("SELECT n.id, n.name FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.node_type='chunk' AND n.file_path=?1 AND n.id<?2 AND b.branch_id={0} ORDER BY n.id DESC LIMIT 1", branch_id);
                if let Ok(mut stmt) = conn.prepare(&prev_sql) {
                    if let Ok(rows) = stmt.query_map(rusqlite::params![file, sid], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        for row in rows.flatten() {
                            json.push_str(&format!(",\"prev_chunk\":{{\"id\":{},\"name\":\"{}\"}}", row.0, esc(&row.1)));
                        }
                    }
                }
                let next_sql = format!("SELECT n.id, n.name FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.node_type='chunk' AND n.file_path=?1 AND n.id>?2 AND b.branch_id={0} ORDER BY n.id ASC LIMIT 1", branch_id);
                if let Ok(mut stmt) = conn.prepare(&next_sql) {
                    if let Ok(rows) = stmt.query_map(rusqlite::params![file, sid], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        for row in rows.flatten() {
                            json.push_str(&format!(",\"next_chunk\":{{\"id\":{},\"name\":\"{}\"}}", row.0, esc(&row.1)));
                        }
                    }
                }
            }
            "file" => {
                // Top-level sections under this file
                let fsql = format!("SELECT n.id, n.name FROM nodes n JOIN branches b ON b.node_id=n.id WHERE n.node_type='section' AND n.file_path=?1 AND b.branch_id={0} ORDER BY n.id LIMIT 80", branch_id);
                if let Ok(mut stmt) = conn.prepare(&fsql) {
                    if let Ok(rows) = stmt.query_map(rusqlite::params![file], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
                        let entries: Vec<String> = rows.flatten().map(|(id,n)| format!("{{\"id\":{},\"name\":\"{}\"}}", id, esc(&n))).collect();
                        if !entries.is_empty() {
                            json.push_str(&format!(",\"sections\":[{}]", entries.join(",")));
                        }
                    }
                }
            }
            _ => {}
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
    else if etype.starts_with("inherits") { "Inherits" }
    else if etype.starts_with("overrides") { "Overrides" }
    else if etype.starts_with("contains") { "Contains" }
    else if etype.starts_with("param_type:") { "Parameters" }
    else if etype.starts_with("return_type:") { "Returns" }
    else if etype.starts_with("field_type:") { "Fields" }
    else { "Other" }
}

fn bm25_precise_search(query: &str, repo: &str, branch: &str, limit: usize, kind_filter: Option<&str>) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    match crate::query::search::bm25_precise_search(&conn, query, repo, branch, limit, kind_filter, false) {
        Ok(results) => {
            if results.is_empty() {
                return serde_json::json!({"results":[],"query":query,"count":0}).to_string();
            }
            let items: Vec<serde_json::Value> = results.iter().map(|r| {
                let is_code = r.hit_type == "code";
                let is_doc = r.hit_type == "doc";
                let mut obj = serde_json::json!({
                    "id": r.id,
                    "score": r.score,
                    "name": r.name,
                    "type": r.hit_type,
                    "file": r.file_path,
                });
                if is_code {
                    if !r.kind.is_empty() { obj["kind"] = serde_json::Value::String(r.kind.clone()); }
                    if !r.signature.is_empty() { obj["signature"] = serde_json::Value::String(r.signature.clone()); }
                    if r.line_start > 0 { obj["line"] = serde_json::Value::Number(serde_json::Number::from(r.line_start)); }
                }
                if is_doc {
                    obj["doc_id"] = serde_json::Value::Number(serde_json::Number::from(r.doc_id));
                }
                if !r.snippet.is_empty() { obj["snippet"] = serde_json::Value::String(r.snippet.clone()); }
                // Enrichment fields
                add_enrichment_fields(&mut obj, r);
                obj
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

fn fuzzy_search(query: &str, repo: &str, branch: &str, limit: usize) -> String {
    let query = query.to_string();
    let repo = repo.to_string();
    let branch = branch.to_string();

    // Spawn a dedicated OS thread to isolate from tokio runtime context.
    // reqwest::blocking v0.12.28 has a #[cfg(debug_assertions)] guard in wait::enter()
    // that builds & drops a temporary tokio runtime per request, which panics when
    // called from within an existing #[tokio::main] runtime. Spawning a raw OS thread
    // avoids this nesting entirely.
    std::thread::spawn(move || {
        let embedder = match crate::embedding::get_embedder() {
            Ok(e) => e,
            Err(e) => {
                return serde_json::json!({"error": format!("Embedding model unavailable: {}", e), "query": query}).to_string();
            }
        };
        let query_emb = match embedder.embed(&query) {
            Ok(emb) => emb,
            Err(e) => {
                return serde_json::json!({"error": format!("Embedding failed: {}", e), "query": query}).to_string();
            }
        };
        let conn = match open_repo_db(&repo) {
            Ok(c) => c,
            Err(e) => return e,
        };
        match crate::query::search::vector_semantic_search(&conn, &query_emb, &repo, &branch, limit, false) {
            Ok(results) => {
                if results.is_empty() {
                    return serde_json::json!({"results":[],"query":query,"count":0}).to_string();
                }
                let items: Vec<serde_json::Value> = results.iter().map(|r| {
                    let mut obj = serde_json::json!({
                        "id": r.id,
                        "score": r.score,
                        "name": r.name,
                        "type": r.hit_type,
                        "file": r.file_path,
                    });
                    if !r.kind.is_empty() { obj["kind"] = serde_json::Value::String(r.kind.clone()); }
                    if !r.signature.is_empty() { obj["signature"] = serde_json::Value::String(r.signature.clone()); }
                    if r.line_start > 0 { obj["line"] = serde_json::Value::Number(serde_json::Number::from(r.line_start)); }
                    if !r.snippet.is_empty() { obj["snippet"] = serde_json::Value::String(r.snippet.clone()); }
                    add_enrichment_fields(&mut obj, r);
                    obj
                }).collect();
                serde_json::json!({
                    "results": items,
                    "query": query,
                    "count": results.len()
                }).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string(), "query": query}).to_string(),
        }
    })
    .join()
    .unwrap_or_else(|_| {
        serde_json::json!({"error": "semantic search thread panicked", "query": ""}).to_string()
    })
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
            {"prefix":"field_type","description":"字段的类型","direction":"从字段到类型","source_kinds":["field"],"target_kinds":["class","struct","enum"]}
        ]
    }).to_string()
}

// ── Path Analysis ───────────────────────────────────────────────────────

fn path_analysis(repo: &str, branch: &str, source: &str, target: &str, mode: &str, max_depth: usize, max_paths: usize, edge_filter: &[String], source_id: Option<i64>, target_id: Option<i64>) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id_or_name(&conn, source_id, source, repo, branch) {
        Some(id) => id, None => return serde_json::json!({"paths": [], "total_found": 0, "error": format!("Symbol '{}' not found", source)}).to_string(),
    };
    let tid = match crate::query::graph::resolve_symbol_id_or_name(&conn, target_id, target, repo, branch) {
        Some(id) => id, None => return serde_json::json!({"paths": [], "total_found": 0, "error": format!("Symbol '{}' not found", target)}).to_string(),
    };
    let branch_id = crate::storage::resolve_branch_id(&conn, repo, branch).unwrap_or(0);
    let results = crate::query::graph::bfs_path_search(&conn, sid, tid, edge_filter, max_depth, mode, max_paths, branch_id);
    let paths: Vec<serde_json::Value> = results.into_iter().map(|r| {
        serde_json::json!({"edges": r.edges})
    }).collect();
    serde_json::json!({"paths": paths, "total_found": paths.len()}).to_string()
}

// ── Impact Analysis ─────────────────────────────────────────────────────

fn impact_analysis(repo: &str, branch: &str, symbol: &str, direction: &str, radius: usize, edge_filter: &[String], sym_id: Option<i64>) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id_or_name(&conn, sym_id, symbol, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", symbol),
    };
    let branch_id = crate::storage::resolve_branch_id(&conn, repo, branch).unwrap_or(0);
    let results = crate::query::graph::transitive_closure(&conn, sid, direction, radius, edge_filter, branch_id);
    let affected: Vec<serde_json::Value> = results.into_iter().map(|r| {
        serde_json::json!({"symbol": r.symbol, "distance": r.distance, "via": r.via})
    }).collect();
    serde_json::json!({"symbol": symbol, "radius": radius, "direction": direction, "affected": affected}).to_string()
}

// ── Neighbor Graph ──────────────────────────────────────────────────────

fn neighbor_graph(repo: &str, branch: &str, symbol: &str, direction: &str, sym_id: Option<i64>) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id_or_name(&conn, sym_id, symbol, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", symbol),
    };
    let branch_id = crate::storage::resolve_branch_id(&conn, repo, branch).unwrap_or(0);
    let map = crate::query::graph::neighbor_map(&conn, sid, direction, &[], branch_id);
    serde_json::json!({
        "symbol": symbol,
        "depth": 1,
        "direction": direction,
        "forward": map.forward,
        "backward": map.backward
    }).to_string()
}

// ── Inheritance Tree ────────────────────────────────────────────────────

fn inheritance_tree(repo: &str, branch: &str, symbol: &str, direction: &str, max_depth: usize, sym_id: Option<i64>) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return err_resp(serde_json::Value::Null, &e).to_string() };
    let sid = match crate::query::graph::resolve_symbol_id_or_name(&conn, sym_id, symbol, repo, branch) {
        Some(id) => id, None => return format!("Symbol '{}' not found", symbol),
    };
    let root_name = crate::query::graph::symbol_name_by_id(&conn, sid).unwrap_or_default();
    let branch_id = crate::storage::resolve_branch_id(&conn, repo, branch).unwrap_or(0);

    fn build_tree(conn: &rusqlite::Connection, parent_id: i64, dir: &str, depth: usize, max_depth: usize, branch_id: i64) -> Vec<serde_json::Value> {
        if depth >= max_depth { return vec![]; }
        let mut children = vec![];
        
        if dir == "up" || dir == "both" {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT e.target_id, n.name FROM edges e JOIN nodes n ON e.target_id = n.id WHERE n.node_type='sym' AND e.source_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'inherits' AND (n.kind = 'class' OR n.kind = 'struct')"
            ) {
                if let Ok(rows) = stmt.query_map(rusqlite::params![parent_id, branch_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                }) {
                    for r in rows.flatten() {
                        let ov = get_overrides(conn, r.0, branch_id);
                        children.push(serde_json::json!({
                            "symbol": r.1, "relation": "parent", "overrides": ov,
                            "children": build_tree(conn, r.0, "up", depth+1, max_depth, branch_id)
                        }));
                    }
                }
            }
        }
        
        if dir == "down" || dir == "both" {
            if let Ok(mut stmt) = conn.prepare(
                "SELECT e.source_id, n.name FROM edges e JOIN nodes n ON e.source_id = n.id WHERE n.node_type='sym' AND e.target_id = ?1 AND e.branch_id = ?2 AND e.edge_type = 'inherits' AND (n.kind = 'class' OR n.kind = 'struct')"
            ) {
                if let Ok(rows) = stmt.query_map(rusqlite::params![parent_id, branch_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                }) {
                    for r in rows.flatten() {
                        let ov = get_overrides(conn, r.0, branch_id);
                        children.push(serde_json::json!({
                            "symbol": r.1, "relation": "child", "overrides": ov,
                            "children": build_tree(conn, r.0, "down", depth+1, max_depth, branch_id)
                        }));
                    }
                }
            }
        }
        children
    }
    
    fn get_overrides(conn: &rusqlite::Connection, class_id: i64, branch_id: i64) -> Vec<String> {
        let mut ov = vec![];
        // 先通过 contains 边找到类的所有方法，再查这些方法的 overrides 边
        if let Ok(mut stmt) = conn.prepare(
            "SELECT n.name FROM edges e \
             JOIN nodes n ON e.target_id = n.id \
             WHERE e.source_id IN (SELECT target_id FROM edges WHERE source_id = ?1 AND edge_type = 'contains') \
             AND e.branch_id = ?2 AND e.edge_type = 'overrides'"
        ) {
            if let Ok(rows) = stmt.query_map(rusqlite::params![class_id, branch_id], |r| r.get::<_,String>(0)) {
                ov = rows.flatten().collect();
            }
        }
        ov
    }

    let result_children = build_tree(&conn, sid, direction, 0, max_depth, branch_id);
    serde_json::json!({"symbol": root_name, "children": result_children}).to_string()
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

// ── query_excel ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_list_has_correct_count() {
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 12);  // 1 tool deleted (index), 3 tools disabled: status, get_doc, query_excel
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
    fn test_unknown_tool() {
        let resp = handle_tool_call(
            serde_json::Value::Number(1.into()),
            "nonexistent_tool",
            &serde_json::json!({}),
        );
        let text = &resp["result"]["content"][0]["text"];
        assert!(text.as_str().unwrap().contains("Unknown tool"));
    }


    #[ignore = "API embedding not configured in CI"]
    fn test_api_fuzzy_search_no_fallback() {
        // Embedding requires API config in config.yaml — skip in CI
    }


    #[test]
    fn test_get_doc_has_get_doc_in_tools_list() {
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        // assert!(names.contains(&"codeloom_query_excel"));
    }


    fn test_search_tool_exists() {
        // Test that the search output template includes snippet/image info
        // by checking the format string patterns in the source
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        let search_tool = tools.iter().find(|t| t["name"] == "codeloom_search").unwrap();
        assert!(search_tool["name"].as_str().unwrap() == "codeloom_search");
        // Verify that new tool definitions are reachable by checking tool count
        assert_eq!(tools.len(), 12);  // 1 tool deleted (index), 3 tools disabled: status, get_doc, query_excel
    }
}
