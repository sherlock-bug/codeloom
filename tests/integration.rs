// Integration tests — run with: cargo test --test integration
// Requires: models/bge-small-zh/ (from build.rs)
// Fixtures: tests/fixtures/ — minimal handcrafted test data per scenario
use std::process::Command;

const CODELOOM: &str = "target/release/codeloom";

fn codeloom(args: &[&str]) -> std::process::Output {
    Command::new(CODELOOM).args(args).output().expect("codeloom not found — build first with: cargo build --release")
}

fn codeloom_mcp(json: &str) -> String {
    let mut child = Command::new(CODELOOM)
        .arg("mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("codeloom mcp failed to start");
    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(json.as_bytes()).unwrap();
    child.stdin.as_mut().unwrap().write_all(b"\n").unwrap();
    let output = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}

// ── Fast tests (< 3s total) ──────────────────────────────────────────

#[test]
fn test_index_and_status() {
    let fixture = "tests/fixtures/index_status";
    let out = codeloom(&["index", fixture, "--repo", "ix", "--branch", "main"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Done:"), "index should succeed: {}", stdout);
    assert!(stdout.contains("symbols"), "should have symbols");

    let out = codeloom(&["status", "--repo", "ix"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Symbols:"), "should show symbols stat");
    assert!(stdout.contains("Docs:"), "should show docs stat");
}

#[test]
fn test_branch_filtering() {
    let fixture = "tests/fixtures/branch_filter";
    // Index on main
    codeloom(&["index", fixture, "--repo", "bf", "--branch", "main"]);
    let resp_main = codeloom_mcp(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_status","arguments":{"repo":"bf","branch":"main"}}}"#);
    assert!(!resp_main.contains("error"), "main should succeed");

    // Index on feature branch (same code, different branch)
    codeloom(&["index", fixture, "--repo", "bf", "--branch", "feature-x"]);
    let resp_feat = codeloom_mcp(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_status","arguments":{"repo":"bf","branch":"feature-x"}}}"#);
    assert!(!resp_feat.contains("error"), "feature-x should succeed");

    // Query non-existent branch should not error
    let resp = codeloom_mcp(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_status","arguments":{"repo":"bf","branch":"nonexistent"}}}"#);
    assert!(!resp.contains("error"), "should not error on missing branch");
}

#[test]
fn test_doc_indexing() {
    let fixture = "tests/fixtures/doc_index";
    codeloom(&["index", fixture, "--repo", "doc", "--branch", "main"]);
    let out = codeloom(&["status", "--repo", "doc"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Docs:"), "should show docs stat");
}

#[test]
fn test_mcp_tools_list() {
    let resp = codeloom_mcp(r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#);
    assert!(resp.contains("codeloom_index"));
    assert!(resp.contains("codeloom_semantic_search"));
    assert!(!resp.contains("codeloom_pull"), "pull should be removed");
    assert!(!resp.contains("codeloom_push"), "push should be removed");
}

#[test]
fn test_mcp_missing_branch_error() {
    let resp = codeloom_mcp(r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"codeloom_status","arguments":{"repo":"default"}}}"#);
    assert!(resp.contains("branch is required"), "should error on missing branch, got: {}", &resp[..200.min(resp.len())]);
}

#[test]
fn test_semantic_search_candle_mode() {
    // Verify candle mode via single MCP process (model loads once, cached)
    let fixture = "tests/fixtures/semantic_search";
    codeloom(&["index", fixture, "--repo", "ss", "--branch", "main"]);

    // Spawn one MCP process, send multiple requests — model reuses OnceLock cache
    use std::io::{BufRead, BufReader, Write};
    let mut child = Command::new(CODELOOM)
        .arg("mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("mcp failed");

    let stdin = child.stdin.as_mut().unwrap();
    let mut stdout = BufReader::new(child.stdout.as_mut().unwrap());

    // Send initialize
    write!(stdin, r#"{{"jsonrpc":"2.0","id":0,"method":"initialize","params":{{}}}}"#).unwrap();
    write!(stdin, "
").unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();

    // Send semantic search
    write!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"codeloom_semantic_search","arguments":{{"query":"user login authentication","branch":"main","repo":"ss","limit":5}}}}}}"#).unwrap();
    write!(stdin, "
").unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();

    let _ = child.kill();
    assert!(!line.contains("Jaccard fallback"), "candle mode should be active");
    assert!(line.contains("Auth"), "should find auth: {}", &line[..200.min(line.len())]);
}
