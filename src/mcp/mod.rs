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
        {"name":"codeloom_status","description":"查看索引状态","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}}}},
        {"name":"codeloom_list_symbols","description":"模糊搜索符号","inputSchema":{"type":"object","properties":{"pattern":{"type":"string"},"repo":{"type":"string"}},"required":["pattern"]}},
        {"name":"codeloom_get_definition","description":"获取符号完整定义","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"}},"required":["name"]}},
        {"name":"codeloom_get_call_graph","description":"获取调用图","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"},"max_depth":{"type":"integer","default":5}},"required":["name"]}},
        {"name":"codeloom_semantic_search","description":"自然语言语义搜索代码和文档","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"repo":{"type":"string"},"limit":{"type":"integer","default":10}},"required":["query"]}},
        {"name":"codeloom_search","description":"全文搜索FTS5","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","default":10}},"required":["query"]}},
        {"name":"codeloom_overview","description":"架构全貌","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}}}},
        {"name":"codeloom_pull","description":"拉取共享DB","inputSchema":{"type":"object","properties":{"source":{"type":"string"}},"required":["source"]}},
        {"name":"codeloom_push","description":"推送baseDB","inputSchema":{"type":"object","properties":{"source":{"type":"string"}}}},
        {"name":"codeloom_switch_branch","description":"切换分支","inputSchema":{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}}
    ]}})
}

fn handle_tool_call(id: serde_json::Value, name: &str, args: &serde_json::Value) -> serde_json::Value {
    let result = match name {
        "codeloom_semantic_search" => {
            let query = args["query"].as_str().unwrap_or("");
            let repo = args["repo"].as_str().unwrap_or("default");
            let limit = args["limit"].as_u64().unwrap_or(10) as usize;
            semantic_search(query, repo, limit)
        }
        _ => "ok".to_string(),
    };
    serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":result}]}})
}

fn semantic_search(query: &str, repo: &str, limit: usize) -> String {
    let dd = match crate::config::Config::data_dir() {
        Ok(d) => d,
        Err(e) => return format!("Error: {}", e),
    };
    let db_path = dd.join(format!("{}.rag.db", repo));
    let conn = match crate::storage::open(&db_path.to_string_lossy()) {
        Ok(c) => c,
        Err(e) => return format!("Error opening DB: {}", e),
    };

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
