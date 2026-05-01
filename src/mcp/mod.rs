use crate::embedding::Embedder;

pub async fn serve() -> anyhow::Result<()> {
    eprintln!("CodeLoom MCP Server v0.1.0");
    use std::io::{BufRead,Write};
    let stdin = std::io::stdin(); let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let req: serde_json::Value = serde_json::from_str(&line)?;
        let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let resp = match method {
            "initialize" => serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"codeloom","version":"0.1.0"}}}),
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
        {"name":"codeloom_index","description":"增量索引代码库","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"branch":{"type":"string"},"repo":{"type":"string"}}}},
        {"name":"codeloom_status","description":"查看索引状态和统计信息","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}}}},
        {"name":"codeloom_list_symbols","description":"按名称模糊搜索符号。优于grep：自动跳过test/vendor，返回结构化结果（名称+类型+文件+行号），可直接传给get_definition或get_call_graph","inputSchema":{"type":"object","properties":{"pattern":{"type":"string"},"repo":{"type":"string"},"limit":{"type":"integer","default":20}},"required":["pattern"]}},
        {"name":"codeloom_get_definition","description":"获取符号完整定义（含源码、签名、文件路径、行号、所属类/命名空间）。优先用此而非read_file，因为定位到精确行号","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"}},"required":["name"]}},
        {"name":"codeloom_get_call_graph","description":"递归遍历调用图（谁调用了它/它调用了谁）。grep做不到：理解调用层次，深度可配，支持模糊名自动匹配","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"},"direction":{"type":"string","enum":["callers","callees"]},"max_depth":{"type":"integer","default":3}},"required":["name"]}},
        {"name":"codeloom_semantic_search","description":"自然语言语义搜索代码+文档。用中文描述意图即可（如'数据压缩怎么实现'），不依赖精确关键词。优于grep：理解语义而非文本匹配","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"limit":{"type":"integer","default":10}},"required":["query"]}},
        {"name":"codeloom_search","description":"全文搜索符号名和定义。优于grep：跳过test/vendor/build目录，返回结构化符号信息（名称/类型/文件/行号），比raw grep快且精准","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"limit":{"type":"integer","default":20}},"required":["query"]}},
        {"name":"codeloom_overview","description":"仓库架构全貌：符号/边/文档按类型统计，Top引用排行。优于逐个读文件：快速理解项目结构，一眼看到核心类/函数","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}}}},
        {"name":"codeloom_pull","description":"拉取共享知识库DB","inputSchema":{"type":"object","properties":{"source":{"type":"string"}},"required":["source"]}},
        {"name":"codeloom_push","description":"推送本地DB到共享存储","inputSchema":{"type":"object","properties":{"source":{"type":"string"}}}},
        {"name":"codeloom_switch_branch","description":"切换活跃分支","inputSchema":{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}}
    ]}})
}

// ── Tool dispatch ──────────────────────────────────────────────────────

fn handle_tool_call(id: serde_json::Value, name: &str, args: &serde_json::Value) -> serde_json::Value {
    let result = match name {
        "codeloom_semantic_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let limit = args["limit"].as_u64().unwrap_or(10) as usize;
            semantic_search(query, repo, limit)
        }
        "codeloom_overview" => {
            let repo = args["repo"].as_str().unwrap_or("default");
            overview(repo)
        }
        "codeloom_status" => {
            let repo = args["repo"].as_str().unwrap_or("default");
            status(repo)
        }
        "codeloom_list_symbols" => {
            let pattern = args["pattern"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let limit = args["limit"].as_u64().unwrap_or(20) as usize;
            list_symbols(pattern, repo, limit)
        }
        "codeloom_get_definition" => {
            let sym_name = args["name"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            get_definition(sym_name, repo)
        }
        "codeloom_get_call_graph" => {
            let sym_name = args["name"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let direction = args["direction"].as_str().unwrap_or("callers");
            let max_depth = args["max_depth"].as_u64().unwrap_or(3) as usize;
            get_call_graph(sym_name, repo, direction, max_depth)
        }
        "codeloom_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let limit = args["limit"].as_u64().unwrap_or(20) as usize;
            fulltext_search(query, repo, limit)
        }
        "codeloom_index" => {
            let path = args["path"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            if path.is_empty() { "Usage: provide 'path' and 'repo' to index a code repository".into() }
            else { format!("Indexing not available via MCP. Use CLI: codeloom index {} --repo {}", path, repo) }
        }
        "codeloom_pull" => {
            let source = args["source"].as_str().unwrap_or("");
            if source.is_empty() { "Usage: provide 'source' (remote path or URL)".into() }
            else { format!("Pull not available via MCP. Use CLI: codeloom pull {}", source) }
        }
        "codeloom_push" => {
            let source = args["source"].as_str().unwrap_or("");
            if source.is_empty() { "Push not available via MCP. Use CLI: codeloom push <destination>".into() }
            else { format!("Push not available via MCP. Use CLI: codeloom push {}", source) }
        }
        "codeloom_switch_branch" => {
            let branch = args["name"].as_str().unwrap_or("");
            if branch.is_empty() { "Usage: provide 'name' (branch name)".into() }
            else { format!("Branch switch not available via MCP. Use CLI: codeloom switch-branch --name {}", branch) }
        }
        _ => format!("Unknown tool: {}", name),
    };
    serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":result}]}})
}

// ── DB helper ──────────────────────────────────────────────────────────

fn open_repo_db(repo: &str) -> Result<rusqlite::Connection, String> {
    let dd = crate::config::Config::data_dir().map_err(|e| format!("Config error: {}", e))?;
    let db_path = dd.join(format!("{}.rag.db", repo));
    crate::storage::open(&db_path.to_string_lossy()).map_err(|e| format!("DB error: {}", e))
}

// ── Implementations ────────────────────────────────────────────────────

fn overview(repo: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };

    let total_syms: i64 = conn.query_row("SELECT COUNT(*) FROM symbols WHERE repo=?1", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
    let total_edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap_or(0);
    let total_docs: i64 = conn.query_row("SELECT COUNT(*) FROM doc_nodes WHERE repo=?1", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);

    let mut out = format!("=== {} ===\n", repo);
    out.push_str(&format!("Symbols: {}  |  Edges: {}  |  Docs: {}\n\n", total_syms, total_edges, total_docs));

    // Symbols by kind
    out.push_str("Symbols by kind:\n");
    if let Ok(mut stmt) = conn.prepare("SELECT kind, COUNT(*) FROM symbols WHERE repo=?1 GROUP BY kind ORDER BY COUNT(*) DESC") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?))) {
            for row in rows.flatten() {
                let pct = if total_syms > 0 { row.1 as f64 / total_syms as f64 * 100.0 } else { 0.0 };
                out.push_str(&format!("  {:12}: {:5} ({:.1}%)\n", row.0, row.1, pct));
            }
        }
    }

    // Edge types (grouped by base type)
    out.push_str("\nEdges by type:\n");
    if let Ok(mut stmt) = conn.prepare("SELECT edge_type, COUNT(*) FROM edges GROUP BY edge_type ORDER BY COUNT(*) DESC") {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?))) {
            let mut base_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            let mut samples: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            for row in rows.flatten() {
                let base = row.0.split(':').next().unwrap_or(&row.0).to_string();
                *base_counts.entry(base.clone()).or_default() += row.1;
                samples.entry(base).or_default().push(format!("{}:{}", row.0, row.1));
            }
            let mut sorted: Vec<_> = base_counts.into_iter().collect();
            sorted.sort_by(|a,b| b.1.cmp(&a.1));
            for (base, count) in sorted.iter().take(10) {
                let sample = samples.get(base).and_then(|v| v.first()).map(|s| s.as_str()).unwrap_or("");
                out.push_str(&format!("  {:20}: {:5}  (e.g. {})\n", base, count, sample));
            }
        }
    }

    // Top symbols by reference count
    out.push_str("\nTop symbols (most referenced):\n");
    if let Ok(mut stmt) = conn.prepare(
        "SELECT s.name, s.kind, COUNT(e.id) as refs FROM edges e JOIN symbols s ON s.id=e.target_id WHERE e.target_id!=0 AND s.repo=?1 GROUP BY e.target_id ORDER BY refs DESC LIMIT 10"
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?))) {
            for row in rows.flatten() {
                out.push_str(&format!("  {:40} [{}]  refs={}\n", row.0, row.1, row.2));
            }
        }
    }

    out
}

fn status(repo: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };

    let syms: i64 = conn.query_row("SELECT COUNT(*) FROM symbols WHERE repo=?1", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
    let edges: i64 = conn.query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0)).unwrap_or(0);
    let docs: i64 = conn.query_row("SELECT COUNT(*) FROM doc_nodes WHERE repo=?1", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);
    let links: i64 = conn.query_row("SELECT COUNT(*) FROM doc_code_links WHERE doc_node_id IN (SELECT id FROM doc_nodes WHERE repo=?1)", rusqlite::params![repo], |r| r.get(0)).unwrap_or(0);

    // Resolved edges
    let resolved: i64 = conn.query_row("SELECT COUNT(*) FROM edges WHERE target_id!=0", [], |r| r.get(0)).unwrap_or(0);
    let resolve_pct = if edges > 0 { resolved as f64 / edges as f64 * 100.0 } else { 0.0 };

    format!(
        "Repo: {}\n\
         Symbols: {}  |  Edges: {} (resolved: {:.1}%)  |  Docs: {}\n\
         Doc-Code links: {}\n\
         Database: ~/.codeloom/{}.rag.db\n\
         Status: indexed ✓",
        repo, syms, edges, resolve_pct, docs, links, repo
    )
}

fn list_symbols(pattern: &str, repo: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let like = format!("%{}%", pattern);

    let mut out = format!("Symbols matching '{}' in {}:\n", pattern, repo);
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name, kind, file_path, line_start FROM symbols WHERE repo=?1 AND name LIKE ?2 ORDER BY name LIMIT ?3"
    ) {
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

fn get_definition(name: &str, repo: &str) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };

    let mut out = format!("Definition: '{}' in {}\n", name, repo);

    // First try exact match
    let mut found = false;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name, kind, definition, file_path, line_start, line_end, signature, parent_class, namespace FROM symbols WHERE repo=?1 AND name=?2 LIMIT 5"
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo, name], |r| {
            Ok((
                r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?,
                r.get::<_,String>(3)?, r.get::<_,i64>(4)?, r.get::<_,i64>(5)?,
                r.get::<_,Option<String>>(6)?, r.get::<_,Option<String>>(7)?, r.get::<_,Option<String>>(8)?,
            ))
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

    // Fallback: LIKE search
    if !found {
        let like = format!("%{}%", name);
        if let Ok(mut stmt) = conn.prepare(
            "SELECT name, kind, file_path, line_start FROM symbols WHERE repo=?1 AND name LIKE ?2 ORDER BY name LIMIT 10"
        ) {
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

fn get_call_graph(name: &str, repo: &str, direction: &str, max_depth: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };

    // Find the symbol (exact match)
    let sym_ids: Vec<i64> = match conn.prepare("SELECT id FROM symbols WHERE repo=?1 AND name=?2") {
        Ok(mut stmt) => stmt.query_map(rusqlite::params![repo, name], |r| r.get(0))
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default(),
        Err(_) => return format!("Error querying symbol '{}'", name),
    };

    if sym_ids.is_empty() {
        // Fallback: LIKE search → auto-use first match
        let like = format!("%{}%", name);
        let similar: Vec<(i64, String)> = match conn.prepare("SELECT id, name FROM symbols WHERE repo=?1 AND name LIKE ?2 LIMIT 10") {
            Ok(mut stmt) => stmt.query_map(rusqlite::params![repo, like], |r| Ok((r.get(0)?, r.get(1)?)))
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default(),
            Err(_) => vec![],
        };
        if similar.is_empty() {
            return format!("Symbol '{}' not found in {}", name, repo);
        }
        // Auto-use the first match instead of just listing
        let (first_id, ref first_name) = similar[0];
        let tail = if similar.len() > 1 {
            format!(". Also found: {}", similar.iter().skip(1).take(5).map(|(_,n)| n.as_str()).collect::<Vec<_>>().join(", "))
        } else { String::new() };
        let mut out = format!("Call graph for '{}' → auto-matched '{}' ({}):\n", name, first_name, direction);
        out.push_str(&tail);
        out.push('\n');
        let mut visited = std::collections::HashSet::new();
        visited.insert(first_id);
        out.push_str(&format!("  ● {} (id={})\n", first_name, first_id));
        traverse_calls(&conn, first_id, direction, max_depth, 1, &mut visited, &mut out);
        return out;
    }

    let mut out = format!("Call graph for '{}' ({}):\n", name, direction);
    let mut visited = std::collections::HashSet::new();
    for &root_id in &sym_ids {
        visited.insert(root_id);
        out.push_str(&format!("  ● {} (id={})\n", name, root_id));
        traverse_calls(&conn, root_id, direction, max_depth, 1, &mut visited, &mut out);
    }
    out
}

fn traverse_calls(
    conn: &rusqlite::Connection, sym_id: i64, direction: &str,
    max_depth: usize, depth: usize, visited: &mut std::collections::HashSet<i64>,
    out: &mut String,
) {
    if depth > max_depth { return; }
    let prefix = "  ".repeat(depth + 1);

    let query = match direction {
        "callees" => format!(
            "SELECT e.target_id, e.edge_type FROM edges e WHERE e.source_id={} AND e.target_id!=0 AND e.edge_type LIKE 'calls:%'",
            sym_id
        ),
        _ => format!(
            "SELECT e.source_id, e.edge_type FROM edges e WHERE e.target_id={} AND e.edge_type LIKE 'calls:%'",
            sym_id
        ),
    };

    if let Ok(mut stmt) = conn.prepare(&query) {
        if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?))) {
            for row in rows.flatten() {
                let (other_id, edge_type) = row;
                if visited.contains(&other_id) {
                    let repeated_name = conn.query_row("SELECT name FROM symbols WHERE id=?1", rusqlite::params![other_id], |r| r.get::<_,String>(0)).unwrap_or_default();
                    out.push_str(&format!("{}↳ {} (already shown)\n", prefix, repeated_name));
                    continue;
                }
                visited.insert(other_id);
                let other_name = conn.query_row("SELECT name FROM symbols WHERE id=?1", rusqlite::params![other_id], |r| r.get::<_,String>(0)).unwrap_or_default();
                let called = edge_type.strip_prefix("calls:").unwrap_or(&edge_type);
                out.push_str(&format!("{}↳ {} (calls:{})\n", prefix, other_name, called));
                if depth < max_depth {
                    traverse_calls(conn, other_id, direction, max_depth, depth + 1, visited, out);
                }
            }
        }
    }
}

fn fulltext_search(query: &str, repo: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };
    let like = format!("%{}%", query);

    let mut out = format!("Full-text search: '{}' in {}\n", query, repo);

    // Search in names
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name, kind, file_path, line_start FROM symbols WHERE repo=?1 AND name LIKE ?2 LIMIT ?3"
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo, like, limit as i64], |r| {
            Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,i64>(3)?))
        }) {
            for row in rows.flatten() {
                out.push_str(&format!("  🔧 [{}] {:45}  @ {}:{}\n", row.1, row.0, &row.2[..50.min(row.2.len())], row.3));
            }
        }
    }

    // Search in definitions
    if let Ok(mut stmt) = conn.prepare(
        "SELECT name, kind, file_path, substr(definition,1,120) FROM symbols WHERE repo=?1 AND definition LIKE ?2 AND name NOT LIKE ?3 LIMIT ?4"
    ) {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo, like, like, limit as i64], |r| {
            Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,String>(3)?))
        }) {
            for row in rows.flatten() {
                out.push_str(&format!("  📝 [{}] {:45}  \"{}\"\n", row.1, row.0, &row.3[..80.min(row.3.len())]));
            }
        }
    }

    out
}

// ── Semantic search ────────────────────────────────────────────────────

fn semantic_search(query: &str, repo: &str, limit: usize) -> String {
    let conn = match open_repo_db(repo) { Ok(c) => c, Err(e) => return e };

    let embedder = crate::embedding::TextEmbedder;
    let query_emb = match embedder.embed(query) {
        Ok(e) => e,
        Err(e) => return format!("Error embedding: {}", e),
    };

    // Search doc_nodes
    let mut results = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT id,title,section_path,content FROM doc_nodes WHERE repo=?1") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,String>(3)?))
        }) {
            for row in rows.flatten() {
                let text = format!("{} {} {}", row.1, row.2, row.3);
                let emb = embedder.embed(&text).unwrap_or_default();
                let sim = embedder.similarity(&query_emb, &emb);
                if sim > 0.05 {
                    results.push((sim, format!("📄 {} > {}\n   {}", row.1, row.2, row.3.chars().take(200).collect::<String>())));
                }
            }
        }
    }

    // Search symbols
    if let Ok(mut stmt) = conn.prepare("SELECT id,name,kind,definition FROM symbols WHERE repo=?1") {
        if let Ok(rows) = stmt.query_map(rusqlite::params![repo], |r| {
            Ok((r.get::<_,i64>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?, r.get::<_,String>(3)?))
        }) {
            for row in rows.flatten() {
                let text = format!("{} {} {}", row.2, row.1, row.3);
                let emb = embedder.embed(&text).unwrap_or_default();
                let sim = embedder.similarity(&query_emb, &emb);
                if sim > 0.02 {
                    results.push((sim, format!("🔧 [{}] {}", row.2, row.1)));
                }
            }
        }
    }

    results.sort_by(|a,b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);

    if results.is_empty() {
        return format!("No results for: {}", query);
    }
    let mut out = format!("Semantic search: \"{}\"\n", query);
    for (sim, text) in &results {
        out.push_str(&format!("  [{:.3}] {}\n", sim, text));
    }
    out
}
