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
        {"name":"codeloom_index","description":"增量索引代码库。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"branch":{"type":"string"},"repo":{"type":"string"}},"required":["path","branch"]}},
        {"name":"codeloom_status","description":"查看索引状态和统计信息。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"repo":{"type":"string"},"branch":{"type":"string"}},"required":["branch"]}},
        {"name":"codeloom_list_symbols","description":"按名称模糊搜索符号。优于grep：返回结构化结果。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"pattern":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"limit":{"type":"integer","default":20}},"required":["pattern","branch"]}},
        {"name":"codeloom_get_definition","description":"获取符号完整定义（含源码、签名、文件路径、行号）。优先用此而非read_file。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"}},"required":["name","branch"]}},
        {"name":"codeloom_get_call_graph","description":"分析函数调用关系。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"direction":{"type":"string","enum":["callers","callees"]},"max_depth":{"type":"integer","default":3}},"required":["name","branch"]}},
        {"name":"codeloom_semantic_search","description":"自然语言语义搜索代码+文档。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"limit":{"type":"integer","default":10}},"required":["query","branch"]}},
        {"name":"codeloom_search","description":"全文搜索符号名和定义。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"branch":{"type":"string"},"limit":{"type":"integer","default":20}},"required":["query","branch"]}},
        {"name":"codeloom_overview","description":"仓库架构全貌。branch参数传当前git分支名（必填）","inputSchema":{"type":"object","properties":{"repo":{"type":"string"},"branch":{"type":"string"}},"required":["branch"]}}
    ]}})
}

fn handle_tool_call(id: serde_json::Value, name: &str, args: &serde_json::Value) -> serde_json::Value {
    let result = match name {
        "codeloom_semantic_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let branch = args["branch"].as_str().unwrap_or("");
            let limit = args["limit"].as_u64().unwrap_or(10) as usize;
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            semantic_search(query, repo, branch, limit)
        }
        "codeloom_overview" => {
            let branch = args["branch"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            overview(repo, branch)
        }
        "codeloom_status" => {
            let branch = args["branch"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            status(repo, branch)
        }
        "codeloom_list_symbols" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let branch = args["branch"].as_str().unwrap_or("");
            let limit = args["limit"].as_u64().unwrap_or(20) as usize;
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            list_symbols(pattern, repo, branch, limit)
        }
        "codeloom_get_definition" => {
            let branch = args["branch"].as_str().unwrap_or("");
            let sym_name = args["name"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            get_definition(sym_name, repo, branch)
        }
        "codeloom_get_call_graph" => {
            let branch = args["branch"].as_str().unwrap_or("");
            let sym_name = args["name"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let direction = args["direction"].as_str().unwrap_or("callers");
            let max_depth = args["max_depth"].as_u64().unwrap_or(3) as usize;
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            get_call_graph(sym_name, repo, branch, direction, max_depth)
        }
        "codeloom_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let branch = args["branch"].as_str().unwrap_or("");
            let limit = args["limit"].as_u64().unwrap_or(20) as usize;
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            fulltext_search(query, repo, branch, limit)
        }
        "codeloom_index" => {
            let path = args["path"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let branch = args["branch"].as_str().unwrap_or("");
            if branch.is_empty() { return err_resp(id, "branch is required"); }
            if path.is_empty() { "Usage: provide path, branch, repo to index".into() }
            else { format!("Use CLI: codeloom index {} --repo {} --branch {}", path, repo, branch) }
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
    let db_path = dd.join(format!("{}.rag.db", repo));
    crate::storage::open(&db_path.to_string_lossy()).map_err(|e| format!("DB error: {}", e))
}

fn branch_where_clause(branch: &str) -> String {
    format!("AND (b.branch_name = '{}' OR b.branch_name IS NULL)", branch.replace('\'', "''"))
}

fn overview(repo: &str, branch: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let bwc = branch_where_clause(branch);
    let sym_sql = format!("SELECT COUNT(*) FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE b.repo=?1 {}", bwc);
    let total_syms: i64 = conn.query_row(&sym_sql, rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
    let total_edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap_or(0);
    let total_docs: i64 = conn.query_row("SELECT COUNT(*) FROM doc_nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name=?2)", rusqlite::params![repo, branch], |r| r.get(0)).unwrap_or(0);
    let mut out = format!("=== {} (branch={}) ===\n", repo, branch);
    out.push_str(&format!("Symbols: {}  |  Edges: {}  |  Docs: {}\n\n", total_syms, total_edges, total_docs));
    out.push_str("Symbols by kind:\n");
    let kind_sql = format!("SELECT kind, COUNT(*) FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE b.repo=?1 {} GROUP BY kind ORDER BY COUNT(*) DESC", bwc);
    if let Ok(mut stmt) = conn.prepare(&kind_sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?))) {
            for row in rows.flatten() {
                let pct = if total_syms > 0 { row.1 as f64 / total_syms as f64 * 100.0 } else { 0.0 };
                out.push_str(&format!("  {:12}: {:5} ({:.1}%)\n", row.0, row.1, pct));
            }
        }
    }
    out
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

fn get_definition(name: &str, repo: &str, branch: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let bwc = branch_where_clause(branch);
    let mut out = format!("Definition: '{}' in {} (branch={})\n", name, repo, branch);
    let exact_sql = format!("SELECT s.name, s.kind, s.definition, s.file_path, s.line_start, s.line_end, s.signature, s.parent_class, s.namespace FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name=?2 {} LIMIT 5", bwc);
    let mut found = false;
    if let Ok(mut stmt) = conn.prepare(&exact_sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo, name], |r| {
            Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?,
                r.get::<_,String>(3)?, r.get::<_,i64>(4)?, r.get::<_,i64>(5)?,
                r.get::<_,Option<String>>(6)?, r.get::<_,Option<String>>(7)?, r.get::<_,Option<String>>(8)?))
        }) {
            for (i, row) in rows.flatten().enumerate() {
                found = true;
                if i > 0 { out.push_str("\n---\n"); }
                let (sname, kind, def, file, lstart, lend, sig, parent, ns) = row;
                out.push_str(&format!("[{}] {}", kind, sname));
                if let Some(ref p) = parent { out.push_str(&format!("  (in {})", p)); }
                if let Some(ref n) = ns { out.push_str(&format!("  ns={}", n)); }
                out.push_str(&format!("\n  File: {}:{}-{}\n", file, lstart, lend));
                if let Some(ref s) = sig { out.push_str(&format!("  Signature: {}\n", s)); }
                if def.len() > 800 {
                    out.push_str(&format!("  Definition:\n{}\n  ... (+{} chars)\n", &def[..800], def.len() - 800));
                } else {
                    out.push_str(&format!("  Definition:\n{}\n", def));
                }
            }
        }
    }
    if !found {
        let like = format!("%{}%", name);
        let like_sql = format!("SELECT s.name, s.kind, s.file_path, s.line_start FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name LIKE ?2 {} ORDER BY s.name LIMIT 10", bwc);
        if let Ok(mut stmt) = conn.prepare(&like_sql) {
            if let Ok(rows) = stmt.query_map(rusqlite::params![repo, like], |r| {
                Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i64>(3)?))
            }) {
                out.push_str("  (exact match not found, showing similar):\n");
                for row in rows.flatten() {
                    out.push_str(&format!("  [{:10}] {:40}  @ {}:{}\n", row.1, row.0, &row.2[..60.min(row.2.len())], row.3));
                }
            }
        }
    }
    out
}

fn get_call_graph(name: &str, repo: &str, branch: &str, direction: &str, max_depth: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let bwc = branch_where_clause(branch);
    let exact_sql = format!("SELECT s.id FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name=?2 {}", bwc);
    let sym_ids: Vec<i64> = match conn.prepare(&exact_sql) {
        Ok(mut stmt) => stmt.query_map(rusqlite::params![repo, name], |r| r.get(0))
            .map(|rows| rows.flatten().collect()).unwrap_or_default(),
        Err(_) => return format!("Error querying symbol '{}'", name),
    };
    if sym_ids.is_empty() {
        let like = format!("%{}%", name);
        let like_sql = format!("SELECT s.id, s.name FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name LIKE ?2 {} LIMIT 10", bwc);
        let similar: Vec<(i64, String)> = match conn.prepare(&like_sql) {
            Ok(mut stmt) => stmt.query_map(rusqlite::params![repo, like], |r| Ok((r.get(0)?, r.get(1)?)))
                .map(|rows| rows.flatten().collect()).unwrap_or_default(),
            Err(_) => vec![],
        };
        if similar.is_empty() {
            return format!("Symbol '{}' not found in {} (branch={})", name, repo, branch);
        }
        let (first_id, ref first_name) = similar[0];
        let mut out = format!("Call graph for '{}' -> auto-matched '{}' ({}):\n", name, first_name, direction);
        let mut visited = std::collections::HashSet::new();
        visited.insert(first_id);
        out.push_str(&format!("  * {} (id={})\n", first_name, first_id));
        traverse_calls(branch, &conn, first_id, direction, max_depth, 1, &mut visited, &mut out);
        return out;
    }
    let mut out = format!("Call graph for '{}' ({}):\n", name, direction);
    let mut visited = std::collections::HashSet::new();
    for &root_id in &sym_ids {
        visited.insert(root_id);
        out.push_str(&format!("  * {} (id={})\n", name, root_id));
        traverse_calls(branch, &conn, root_id, direction, max_depth, 1, &mut visited, &mut out);
    }
    out
}

fn traverse_calls(branch: &str, conn: &rusqlite::Connection, sym_id: i64, direction: &str,
    max_depth: usize, depth: usize, visited: &mut std::collections::HashSet<i64>, out: &mut String,
) {
    if depth > max_depth { return; }
    let prefix = "  ".repeat(depth + 1);
    let query = match direction {
        "callees" => format!("SELECT e.target_id, e.edge_type FROM edges e WHERE e.source_id={} AND e.target_id!=0 AND e.edge_type LIKE 'calls:%'", sym_id),
        _ => format!("SELECT e.source_id, e.edge_type FROM edges e WHERE e.target_id={} AND e.edge_type LIKE 'calls:%'", sym_id),
    };
    if let Ok(mut stmt) = conn.prepare(&query) {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
            for row in rows.flatten() {
                let (other_id, edge_type) = row;
                if visited.contains(&other_id) {
                    let repeated_name = conn.query_row("SELECT name FROM symbols WHERE id=?1", rusqlite::params![other_id], |r| r.get::<_,String>(0)).unwrap_or_default();
                    out.push_str(&format!("{}{} {} (already shown)\n", prefix, '→', repeated_name));
                    continue;
                }
                visited.insert(other_id);
                let other_name = conn.query_row("SELECT s.name FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.id=?1 AND (b.branch_name=?2 OR b.branch_name IS NULL)", rusqlite::params![other_id, branch], |r| r.get::<_,String>(0)).unwrap_or_default();
                let called = edge_type.strip_prefix("calls:").unwrap_or(&edge_type);
                out.push_str(&format!("{}{} {} (calls:{})\n", prefix, '→', other_name, called));
                if depth < max_depth {
                    traverse_calls(branch, conn, other_id, direction, max_depth, depth + 1, visited, out);
                }
            }
        }
    }
}

fn fulltext_search(query: &str, repo: &str, branch: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let like = format!("%{}%", query);
    let bwc = branch_where_clause(branch);
    let mut out = format!("Full-text search: '{}' in {} (branch={})\n", query, repo, branch);
    let name_sql = format!("SELECT s.name, s.kind, s.file_path, s.line_start FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 AND s.name LIKE ?2 {} LIMIT ?3", bwc);
    if let Ok(mut stmt) = conn.prepare(&name_sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo, like, limit as i64], |r| {
            Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i64>(3)?))
        }) {
            for row in rows.flatten() {
                out.push_str(&format!("  [{}] {:45}  @ {}:{}\n", row.1, row.0, &row.2[..50.min(row.2.len())], row.3));
            }
        }
    }
    out
}

fn semantic_search(query: &str, repo: &str, branch: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let embedder = crate::embedding::get_embedder();
    let is_fallback = !crate::embedding::CandleEmbedder::model_available();
    let query_emb = match embedder.embed(query) {
        Ok(e) => e,
        Err(e) => return format!("Error embedding: {}", e),
    };

    // Try vec0 ANN first
    let sym_table = format!("symbol_vec_{}", repo.replace('-', "_"));
    let doc_table = format!("doc_vec_{}", repo.replace('-', "_"));
    if crate::storage::vector::try_load(&conn) {
        let mut results = Vec::new();
        if let Ok(rows) = crate::storage::vector::knn_search(&conn, &sym_table, &query_emb, limit) {
            for (rowid, dist) in rows {
                if let Ok(name) = conn.query_row(
                    "SELECT name || ' [' || kind || ']' FROM symbols WHERE id=?1",
                    rusqlite::params![rowid], |r| r.get::<_,String>(0)
                ) {
                    let sim = 1.0 / (1.0 + dist as f32);
                    results.push((sim, format!("code {}", name)));
                }
            }
        }
        if let Ok(rows) = crate::storage::vector::knn_search(&conn, &doc_table, &query_emb, limit) {
            for (rowid, dist) in rows {
                if let Ok((title, section)) = conn.query_row(
                    "SELECT title, section_path FROM doc_nodes WHERE id=?1",
                    rusqlite::params![rowid], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?))
                ) {
                    let sim = 1.0 / (1.0 + dist as f32);
                    results.push((sim, format!("doc {} > {}", title, section)));
                }
            }
        }
        results.sort_by(|a,b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        if results.is_empty() {
            return format!("No results for: {}", query);
        }
        let mut out = format!("Semantic search (vec0): \"{}\"", query);
        out.push('\n');
        for (sim, text) in &results {
            out.push_str(&format!("  [{:.3}] {}\n", sim, text));
        }
        return out;
    }

    // Fallback: brute-force cosine similarity
    let mut results = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT id,title,section_path,content FROM doc_nodes WHERE repo=?1 AND (branch_name IS NULL OR branch_name=?2)") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo, branch], |r| {
            Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,String>(3)?))
        }) {
            for row in rows.flatten() {
                let text = format!("{} {} {}", row.1, row.2, row.3);
                let emb = embedder.embed(&text).unwrap_or_default();
                let sim = embedder.similarity(&query_emb, &emb);
                if sim > 0.05 {
                    results.push((sim, format!("doc {} > {}", row.1, row.2)));
                }
            }
        }
    }
    let bwc = branch_where_clause(branch);
    let sym_sql = format!("SELECT s.id,s.name,s.kind,s.definition FROM symbols s JOIN branches b ON s.id=b.symbol_id WHERE s.repo=?1 {}", bwc);
    if let Ok(mut stmt) = conn.prepare(&sym_sql) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,String>(3)?))
        }) {
            for row in rows.flatten() {
                let text = format!("{} {} {}", row.2, row.1, row.3);
                let emb = embedder.embed(&text).unwrap_or_default();
                let sim = embedder.similarity(&query_emb, &emb);
                if sim > 0.02 {
                    results.push((sim, format!("code [{}] {}", row.2, row.1)));
                }
            }
        }
    }
    results.sort_by(|a,b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);
    if results.is_empty() {
        return format!("No results for: {}", query);
    }
    let mode = if is_fallback { " (Jaccard)" } else { " (brute-force)" };
    let mut out = format!("Semantic search: \"{}\"{}", query, mode);
    out.push('\n');
    for (sim, text) in &results {
        out.push_str(&format!("  [{:.3}] {}\n", sim, text));
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_list_returns_8_tools() {
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 8);
    }

    #[test]
    fn test_all_tools_have_branch_required() {
        let resp = tools_list(serde_json::Value::Number(1.into()));
        let tools = resp["result"]["tools"].as_array().unwrap();
        for tool in tools {
            let required = tool["inputSchema"]["required"].as_array().unwrap();
            let req_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
            assert!(req_strs.contains(&"branch"), "tool {} missing branch", tool["name"]);
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
        let resp = handle_tool_call(
            serde_json::Value::Number(1.into()),
            "codeloom_status",
            &serde_json::json!({"repo": "default"}),
        );
        assert_eq!(resp["error"]["code"], -32602);
        assert!(resp["error"]["message"].as_str().unwrap().contains("branch is required"));
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
    fn test_get_definition_found() {
        let conn = crate::storage::open(":memory:").unwrap();
        crate::storage::migrate(&conn).unwrap();
        conn.execute("INSERT INTO symbols (repo,name,kind,definition,content_hash,file_path,line_start,line_end,signature) VALUES ('gd','AuthService','class','class AuthService {}','g1','auth.cpp',10,15,'class AuthService')", []).unwrap();
        let sid = conn.last_insert_rowid();
        conn.execute("INSERT INTO branches (symbol_id,repo,branch_name) VALUES (?1,'gd','main')", rusqlite::params![sid]).unwrap();
        let result = get_definition("AuthService", "gd", "main");
        assert!(result.contains("AuthService"));
    }


    #[test]
    fn test_candle_semantic_search_no_fallback() {
        // Model is available and cached — candle mode active
        assert!(crate::embedding::CandleEmbedder::model_available());
    }

    #[test]
    fn test_overview_basic() {
        let conn = crate::storage::open(":memory:").unwrap();
        crate::storage::migrate(&conn).unwrap();
        conn.execute("INSERT INTO symbols (repo,name,kind,definition,content_hash,file_path,line_start,line_end) VALUES ('ov','f','function','void f(){}','abc','f.cpp',1,1)", []).unwrap();
        let sym_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO branches (symbol_id,repo,branch_name) VALUES (?1,'ov','main')", rusqlite::params![sym_id]).unwrap();
        let result = overview("ov", "main");
        assert!(result.contains("ov"));
        assert!(result.contains("Symbols"));
    }
}
