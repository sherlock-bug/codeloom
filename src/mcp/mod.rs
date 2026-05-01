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
            "tools/list" => serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"tools":[
                {"name":"codeloom_index","description":"增量索引代码库","inputSchema":{"type":"object","properties":{"path":{"type":"string"},"branch":{"type":"string"},"repo":{"type":"string"}}}},
                {"name":"codeloom_status","description":"查看索引状态","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}}}},
                {"name":"codeloom_list_symbols","description":"模糊搜索符号","inputSchema":{"type":"object","properties":{"pattern":{"type":"string"},"repo":{"type":"string"}},"required":["pattern"]}},
                {"name":"codeloom_get_definition","description":"获取符号完整定义","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"}},"required":["name"]}},
                {"name":"codeloom_get_call_graph","description":"获取调用图","inputSchema":{"type":"object","properties":{"name":{"type":"string"},"repo":{"type":"string"},"max_depth":{"type":"integer","default":5}},"required":["name"]}},
                {"name":"codeloom_search","description":"全文搜索FTS5","inputSchema":{"type":"object","properties":{"query":{"type":"string"},"limit":{"type":"integer","default":10}},"required":["query"]}},
                {"name":"codeloom_overview","description":"架构全貌","inputSchema":{"type":"object","properties":{"repo":{"type":"string"}}}},
                {"name":"codeloom_pull","description":"拉取共享DB","inputSchema":{"type":"object","properties":{"source":{"type":"string"}},"required":["source"]}},
                {"name":"codeloom_push","description":"推送baseDB","inputSchema":{"type":"object","properties":{"source":{"type":"string"}}}},
                {"name":"codeloom_switch_branch","description":"切换分支","inputSchema":{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}}
            ]}}),
            "tools/call" => serde_json::json!({"jsonrpc":"2.0","id":id,"result":{"content":[{"type":"text","text":"ok"}]}}),
            _ => serde_json::json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("unknown: {}",method)}}),
        };
        writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
        stdout.flush()?;
    }
    Ok(())
}