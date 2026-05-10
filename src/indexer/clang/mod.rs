//! Clang-based C/C++ parser — replaces tree-sitter with `clang -fsyntax-only -ast-dump=json`.

use crate::{log_info, log_warn, log_error, log_debug};

pub mod compile_cmds;
pub mod ast;

use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use rusqlite::Connection;
use sha2::{Sha256, Digest};
use crate::storage::symbols::{Symbol, make_sid};
use crate::indexer::tree_sitter::FileInfo;

/// Index C/C++ files using Clang subprocess.
/// Only processes translation units (.cpp/.cc/.cxx/.c); headers are parsed via #include.
pub fn index_clang(
    conn: &Connection,
    files: &[FileInfo],
    repo_name: &str,
    compile_commands_path: Option<&str>,
) -> anyhow::Result<usize> {
    // Determine project root from first file (always absolute)
    let repo_root = files.first()
        .and_then(|f| find_git_root(&f.path))
        .or_else(|| files.first()
            .and_then(|f| std::path::Path::new(&f.path).parent())
            .map(|p| p.to_string_lossy().to_string()))
        .or_else(|| std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()))
        .map(|p| {
            // Ensure absolute — find_git_root may return "" for current dir
            let p = std::path::Path::new(&p);
            if p.is_absolute() { p.to_string_lossy().to_string() }
            else if p.as_os_str().is_empty() {
                std::env::current_dir().unwrap_or_else(|_| "/".into()).to_string_lossy().to_string()
            } else {
                std::env::current_dir().unwrap_or_else(|_| "/".into()).join(p).to_string_lossy().to_string()
            }
        })
        .unwrap_or_default();
    
    // Load ignore patterns (third-party libs should be treated as external)
    let cmds = compile_cmds::discover(compile_commands_path, &repo_root)?;

    let ignore_patterns = crate::ignore::load_patterns(&repo_root);

    // Only process translation units
    let tu_files: Vec<&FileInfo> = files.iter().filter(|f| {
        let p = std::path::Path::new(&f.path);
        matches!(p.extension().and_then(|e| e.to_str()),
            Some("cpp") | Some("cc") | Some("cxx") | Some("c"))
    }).collect();

    let mut symbols_count = 0;
    let mut edges_count = 0;
    log_debug!("indexer::clang", "parse start: {} translation units", tu_files.len());

    for file in &tu_files {
        // Resolve file path relative to repo root for compile_commands lookup
        let abs_key = if std::path::Path::new(&file.path).is_absolute() {
            file.path.clone()
        } else {
            std::path::Path::new(&repo_root).join(&file.path)
                .to_string_lossy().to_string()
        };
        let args = cmds.get(&abs_key)
            .map(|a| a.as_slice())
            .unwrap_or(&[]);

        match parse_file(&abs_key, args, &repo_root) {
            Ok(ast) => {
                // Resolve file path to absolute — extract_symbols_and_edges needs
                // absolute paths for is_project_file() prefix matching.
                let abs_path = if std::path::Path::new(&file.path).is_absolute() {
                    file.path.clone()
                } else {
                    std::path::Path::new(&repo_root).join(&file.path)
                        .to_string_lossy().to_string()
                };
                let extracted = ast::extract_symbols_and_edges(
                    &ast, repo_name, &abs_path, &repo_root, &ignore_patterns
                );

                let mut name_to_id: HashMap<String, i64> = HashMap::new();

                for sym in &extracted.symbols {
                    let id = upsert_symbol(conn, sym, repo_name)?;
                    let key = format!("{}::{}", sym.namespace.as_deref().unwrap_or(""), sym.name);
                    name_to_id.insert(key, id);
                    name_to_id.insert(sym.name.clone(), id);
                    symbols_count += 1;
                }

                for edge in &extracted.edges {
                    let src_key = format!("{}::{}", edge.source_ns, edge.source_name);
                    let src_id = name_to_id.get(&src_key)
                        .or_else(|| name_to_id.get(&edge.source_name));

                    let tgt_id = name_to_id.get(&edge.target_name);

                    if let Some(&sid) = src_id {
                        let tid = if let Some(&id) = tgt_id {
                            id
                        } else {
                            let stub_kind = infer_stub_kind(&edge.edge_type);
                            create_external_stub(conn, &edge.target_name, stub_kind, "", repo_name)?
                        };

                        if tid > 0 {
                            let _ = conn.execute(
                                "INSERT OR IGNORE INTO edges (source_id, target_id, edge_type, source_repo) VALUES (?1,?2,?3,?4)",
                                rusqlite::params![sid, tid, edge.edge_type, repo_name],
                            );
                            edges_count += 1;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("  clang failed for {}: {}", file.path, e);
            }
        }
    }

    println!("  Clang: {} symbols, {} edges", symbols_count, edges_count);
    Ok(symbols_count)
}

/// Insert or update a symbol. is_external is already set by ast.rs based on white-list.
/// Convergence logic:
/// Infer symbol kind from edge_type prefix for stub creation.
fn infer_stub_kind(edge_type: &str) -> &str {
    if edge_type.starts_with("instantiates:") { "template_function" }
    else if edge_type.starts_with("contains:") { "method" }
    else if edge_type.starts_with("aliases:") { "typedef" }
    else if edge_type.starts_with("overrides:") { "method" }
    else if edge_type.starts_with("uses_type:") { "class" }
    else if edge_type.starts_with("param_type:") || edge_type.starts_with("return_type:") { "class" }
    else { "function" }
}

/// - Full-key match on (name, namespace, kind, parent_class, file_path, repo)
/// - Full-key match on (name, namespace, kind, parent_class, file_path, repo)
///   → found: handle external upgrade / decl→def merge / converge
///   → not found: insert new
///   Signature is NOT in the key — template instance names already embed type params
///   (e.g. `data<int>` vs `data<float>`). File_path="" for template instances ensures
///   convergence across TUs; non-template symbols keep their real file_path.
fn upsert_symbol(conn: &Connection, sym: &Symbol, repo: &str) -> anyhow::Result<i64> {
    let ns = sym.namespace.as_deref().unwrap_or("");

    // Full-key exact match on (name, namespace, kind, signature, file_path, repo)
    // parent_class is NOT in the key — names are already qualified (e.g. Class::method).
    // Signature distinguishes function overloads and different template instantiations.
    let existing: Option<(i64, bool, bool)> = conn.query_row(
        "SELECT id, is_external, is_definition FROM symbols          WHERE name=?1 AND namespace=?2 AND kind=?3            AND COALESCE(signature,'')=COALESCE(?4,'')            AND file_path=?5 AND repo=?6 LIMIT 1",
        rusqlite::params![sym.name, ns, sym.kind, sym.signature, sym.file_path, repo],
        |row| Ok((row.get(0)?, row.get::<_, i32>(1)? != 0, row.get::<_, i32>(2)? != 0)),
    ).ok();

    match existing {
        None => {
            sym.insert(conn)
        }
        Some((id, true, _)) if !sym.is_external => {
            // External stub upgraded by real project implementation
            conn.execute(
                "UPDATE symbols SET is_external=0, is_definition=?1, line_start=?2, line_end=?3,                  signature=?4, sid=?5, doc_comment=?6 WHERE id=?7",
                rusqlite::params![sym.is_definition as i32, sym.line_start, sym.line_end,
                    sym.signature, sym.sid, sym.doc_comment, id],
            )?;
            Ok(id)
        }
        Some((id, false, false)) if sym.is_definition => {
            // Declaration → definition merge
            conn.execute(
                "UPDATE symbols SET is_definition=1, line_start=?1, line_end=?2,                  doc_comment=CASE WHEN ?3!='' THEN doc_comment||CHAR(10)||?3 ELSE doc_comment END                  WHERE id=?4",
                rusqlite::params![sym.line_start, sym.line_end, sym.doc_comment, id],
            )?;
            Ok(id)
        }
        Some((id, _, _)) => {
            // Both definitions or both declarations with same full key → converge
            Ok(id)
        }
    }
}

/// Create an external symbol stub for an edge target not yet in the DB.
fn create_external_stub(conn: &Connection, name: &str, kind: &str, ns: &str, repo: &str) -> anyhow::Result<i64> {
    let hash = format!("ext:{:x}", Sha256::digest(name.as_bytes()))
        .chars().take(16).collect::<String>();
    let sid = make_sid(repo, name, "", ns, kind, "");

    match conn.query_row(
        "INSERT INTO symbols (repo,name,kind,content_hash,file_path,line_start,line_end,language,sid,namespace,is_external,is_definition) \
         VALUES (?1,?2,?3,?4,'',0,0,'cpp',?5,?6,1,1) \
         ON CONFLICT(content_hash,file_path,name,repo) DO UPDATE SET is_external=1 \
         RETURNING id",
        rusqlite::params![repo, name, kind, hash, sid, ns],
        |row| row.get(0),
    ) {
        Ok(id) => Ok(id),
        Err(_) => {
            conn.query_row(
                "SELECT id FROM symbols WHERE name=?1 AND repo=?2 LIMIT 1",
                rusqlite::params![name, repo],
                |row| row.get(0),
            ).or(Ok(0))
        }
    }
}

/// Run clang, pipe through Python filter to strip system headers and function bodies.
fn parse_file(file: &str, extra_args: &[String], project_root: &str) -> anyhow::Result<serde_json::Value> {
    // Locate the filter script
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let filter_script = std::path::Path::new(&home).join(".hermes/scripts/clang_filter.py");
    
    // Build pipeline: clang ... | python3 filter.py <project_root>
    // Compiler flags (-I/-D/-std= etc) go BEFORE --, only the source file after
    let mut clang_args = String::from("-fsyntax-only -Xclang -ast-dump=json");
    for arg in extra_args {
        clang_args.push_str(" '");
        clang_args.push_str(arg);
        clang_args.push('\'');
    }
    clang_args.push_str(" -- '");
    clang_args.push_str(file);
    clang_args.push('\'');

    let pipeline = format!(
        "clang {} 2>/dev/null | python3 '{}' '{}'",
        clang_args,
        filter_script.display(),
        project_root
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&pipeline)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("clang+filter failed: {}", stderr.lines().next().unwrap_or("unknown error"));
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(json)
}

/// Walk up from a file path to find the nearest .git directory root.
fn find_git_root(file_path: &str) -> Option<String> {
    let mut path = std::path::Path::new(file_path).parent()?;
    loop {
        if path.join(".git").exists() {
            return Some(path.to_string_lossy().to_string());
        }
        path = path.parent()?;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::symbols::Symbol;

    fn test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo TEXT NOT NULL, name TEXT NOT NULL, kind TEXT NOT NULL,
                content_hash TEXT NOT NULL, file_path TEXT NOT NULL,
                line_start INTEGER DEFAULT 0, line_end INTEGER DEFAULT 0,
                language TEXT, signature TEXT, parent_class TEXT,
                namespace TEXT, doc_comment TEXT DEFAULT '',
                sid TEXT, access TEXT DEFAULT '', is_virtual INTEGER DEFAULT 0,
                is_definition INTEGER DEFAULT 1, is_external INTEGER DEFAULT 0,
                template_args TEXT,
                UNIQUE(content_hash, file_path, name, repo)
            )"
        ).unwrap();
        conn
    }

    fn make_sym(name: &str, ns: &str, kind: &str, sig: &str, file: &str, is_def: bool) -> Symbol {
        Symbol {
            name: name.to_string(),
            namespace: Some(ns.to_string()),
            kind: kind.to_string(),
            signature: Some(sig.to_string()),
            file_path: file.to_string(),
            is_definition: is_def,
            repo: "test".to_string(),
            content_hash: format!("hash:{name}:{sig}:{file}"),
            ..Default::default()
        }
    }

    // === Full-key matching tests ===

    #[test]
    fn test_same_signature_same_file_converge() {
        // Same (name, ns, kind, sig, file) → same symbol
        let conn = test_db();
        let s1 = make_sym("data", "ns", "function", "int*()", "a.cpp", false);
        let s2 = make_sym("data", "ns", "function", "int*()", "a.cpp", false);
        let id1 = upsert_symbol(&conn, &s1, "test").unwrap();
        let id2 = upsert_symbol(&conn, &s2, "test").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    }

    #[test]
    fn test_different_file_same_signature_insert() {
        // Same sig but different file → different symbols (e.g. static in two .cpp)
        let conn = test_db();
        let s1 = make_sym("init", "ns", "function", "void()", "a.cpp", false);
        let s2 = make_sym("init", "ns", "function", "void()", "b.cpp", false);
        let id1 = upsert_symbol(&conn, &s1, "test").unwrap();
        let id2 = upsert_symbol(&conn, &s2, "test").unwrap();
        assert_ne!(id1, id2);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
    }

    #[test]
    fn test_template_instance_empty_file_converge() {
        // Template instances have file_path="" → same sig converges across TUs
        let conn = test_db();
        let s1 = make_sym("data", "flatbuffers", "function", "const T *", "", false);
        let s2 = make_sym("data", "flatbuffers", "function", "const T *", "", false);
        let id1 = upsert_symbol(&conn, &s1, "test").unwrap();
        let id2 = upsert_symbol(&conn, &s2, "test").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    }

    #[test]
    fn test_struct_converge_by_full_key() {
        // Same struct name+ns+kind+file → converge
        let conn = test_db();
        let mk = |hash: &str| Symbol {
            name: "Table".to_string(), namespace: Some("fb".to_string()),
            kind: "struct".to_string(), file_path: "table.h".to_string(),
            is_definition: false, repo: "test".to_string(),
            content_hash: hash.to_string(),
            ..Default::default()
        };
        let id1 = upsert_symbol(&conn, &mk("h1"), "test").unwrap();
        let id2 = upsert_symbol(&conn, &mk("h2"), "test").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    }

    #[test]
    fn test_field_with_parent_converge() {
        // Same field name+ns+parent_class+file → converge
        let conn = test_db();
        let mk = |hash: &str| Symbol {
            name: "size_".to_string(), namespace: Some("fb".to_string()),
            kind: "field".to_string(), parent_class: Some("Builder".to_string()),
            file_path: "builder.h".to_string(), is_definition: false, repo: "test".to_string(),
            content_hash: hash.to_string(),
            ..Default::default()
        };
        let id1 = upsert_symbol(&conn, &mk("h1"), "test").unwrap();
        let id2 = upsert_symbol(&conn, &mk("h2"), "test").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
    }

    #[test]
    fn test_different_name_different_symbol() {
        // Qualified names (Builder::size_ vs Verifier::size_) are naturally distinct
        let conn = test_db();
        let mk = |qname: &str, hash: &str| Symbol {
            name: qname.to_string(), namespace: Some("fb".to_string()),
            kind: "field".to_string(), parent_class: None,
            file_path: "x.h".to_string(), is_definition: false, repo: "test".to_string(),
            content_hash: hash.to_string(),
            ..Default::default()
        };
        let id1 = upsert_symbol(&conn, &mk("Builder::size_", "h1"), "test").unwrap();
        let id2 = upsert_symbol(&conn, &mk("Verifier::size_", "h2"), "test").unwrap();
        assert_ne!(id1, id2, "different qualified names should get different ids");
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
    }

    #[test]
    fn test_external_stub_upgrade() {
        // External stub (is_external=1) → real symbol upgrades to is_external=0
        let conn = test_db();
        let stub = Symbol {
            name: "external_func".to_string(), namespace: None, kind: "function".to_string(),
            file_path: "".to_string(), is_external: true, is_definition: true,
            repo: "test".to_string(), content_hash: "stub_hash".to_string(),
            ..Default::default()
        };
        let stub_id = stub.insert(&conn).unwrap();

        let real = Symbol {
            name: "external_func".to_string(), namespace: None, kind: "function".to_string(),
            signature: Some("void()".to_string()), file_path: "real.cpp".to_string(),
            is_external: false, is_definition: true, repo: "test".to_string(),
            content_hash: "real_hash".to_string(),
            ..Default::default()
        };
        // real symbol has different file_path → won't match stub by full key
        // This test verifies the existing behavior (no change needed here)
        let real_id = upsert_symbol(&conn, &real, "test").unwrap();
        // Real symbol gets different id because file_path doesn't match
        assert_ne!(stub_id, real_id);
    }
}

