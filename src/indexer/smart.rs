use crate::indexer::{
    clang,
    git,
    tree_sitter::{self, FileInfo},
};
use crate::storage::symbols::Symbol;
use rusqlite::Connection;
use crate::{log_info, log_warn};

#[derive(Debug, Default)]
pub struct IndexResult {
    pub files_scanned: usize,
    pub files_changed: usize,
    pub symbols_new: usize,
    pub symbols_inherited: usize,
    pub head_commit: Option<String>,
    pub from_commit: Option<String>,
    pub inherited_from: Option<String>,
}

pub fn smart_index(
    conn: &Connection,
    repo_root: &str,
    repo_name: &str,
    branch: &str,
) -> anyhow::Result<IndexResult> {
    _smart_index(conn, repo_root, repo_name, branch, None)
}

pub fn smart_index_with_parent(
    conn: &Connection,
    repo_root: &str,
    repo_name: &str,
    branch: &str,
    parent: &str,
) -> anyhow::Result<IndexResult> {
    _smart_index(conn, repo_root, repo_name, branch, Some(parent))
}

fn _smart_index(
    conn: &Connection,
    repo_root: &str,
    repo_name: &str,
    branch: &str,
    force_parent: Option<&str>,
) -> anyhow::Result<IndexResult> {
    let mut result = IndexResult::default();
    let head = git::head_commit(repo_root);
    result.head_commit = head.clone();

    // Seed builtin std symbols before first index
    crate::storage::symbols::insert_builtin_symbols(conn)?;

    // Vec0 tables are created by index_vectors() — skip eager init here

    let Some(ref head_commit) = head else {
        full_scan(conn, repo_root, repo_name, branch, &mut result)?;
        return Ok(result);
    };

    let last_state = conn
        .query_row(
            "SELECT head_commit FROM git_index_state WHERE repo=?1 AND branch_name=?2",
            rusqlite::params![repo_name, branch],
            |row| row.get::<_, String>(0),
        )
        .ok();

    if let Some(ref ls) = last_state {
        if *ls == *head_commit {
            println!("Already up to date ({}).", &head_commit[..8]);
            return Ok(result);
        }
    }

    if let Some(ref ls) = last_state {
        let files = git::changed_files(repo_root, ls, head_commit);
        result.from_commit = Some(ls.clone());
        result.files_changed = files.len();
        result.files_scanned = files.len();
        for fp in &files {
            match index_one(conn, fp, repo_root, repo_name, branch) {
                Ok(c) => result.symbols_new += c,
                Err(e) => { log_warn!("indexer", "parse failed: {}: {}", fp, e); eprintln!("Warning: {}: {}", fp, e); }
            }
        }
        update_state(
            conn, repo_name, branch, head_commit, None, result.files_changed,
        )?;
        return Ok(result);
    }

    let parent = force_parent
        .map(|s| s.to_string())
        .or_else(|| find_parent(conn, repo_root, repo_name, head_commit));
    if let Some(ref parent_branch) = parent {
        let base = git::merge_base(repo_root, head_commit, parent_branch)
            .unwrap_or_else(|| head_commit.clone());
        result.inherited_from = Some(format!("{}@{}", parent_branch, &base[..8]));
        result.symbols_inherited = 0;
        let files = git::changed_files(repo_root, &base, head_commit);
        result.files_changed = files.len();
        result.files_scanned = files.len();
        result.from_commit = Some(base);
        for fp in &files {
            match index_one(conn, fp, repo_root, repo_name, branch) {
                Ok(c) => result.symbols_new += c,
                Err(e) => { log_warn!("indexer", "parse failed: {}: {}", fp, e); eprintln!("Warning: {}: {}", fp, e); }
            }
        }
        update_state(
            conn, repo_name, branch, head_commit, Some(parent_branch), result.files_changed,
        )?;
        return Ok(result);
    }

    full_scan(conn, repo_root, repo_name, branch, &mut result)?;
    update_state(
        conn, repo_name, branch, head_commit, None, result.files_changed,
    )?;
    Ok(result)
}

fn full_scan(
    conn: &Connection,
    repo_root: &str,
    repo_name: &str,
    branch: &str,
    result: &mut IndexResult,
) -> anyhow::Result<()> {
    let files: Vec<String> = tree_sitter::collect_files(repo_root)
        .into_iter()
        .map(|f| f.path)
        .collect();
    let total = files.len();
    result.files_changed = total;
    result.files_scanned = total;
    for (i, fp) in files.iter().enumerate() {
        if total > 20 && (i % 20 == 0 || i == total - 1) {
            eprintln!("  [{:>3}/{}] parsing + vectorizing...", i + 1, total);
        }
        match index_one(conn, fp, repo_root, repo_name, branch) {
            Ok(c) => result.symbols_new += c,
            Err(e) => eprintln!("Warning: {}: {}", fp, e),
        }
    }
    let branch_id = crate::storage::resolve_branch_id(conn, repo_name, branch)?;
    conn.execute(
        "INSERT OR IGNORE INTO branches (node_id,repo,branch_id,override_def,override_hash) SELECT id,repo,?1,NULL,NULL FROM nodes WHERE repo=?2 AND node_type='sym'",
        rusqlite::params![branch_id, repo_name],
    )?;
    Ok(())
}

fn index_one(
    conn: &Connection,
    file_path: &str,
    _repo_root: &str,
    repo_name: &str,
    branch: &str,
) -> anyhow::Result<usize> {
    let lang = match tree_sitter::detect_language(file_path) {
        Some(l) => l,
        None => return Ok(0),
    };

    // Route C++ files to Clang parser (with compile_commands from env var)
    if lang == "cpp" {
        let fi = FileInfo {
            path: file_path.to_string(),
            language: "cpp",
            modified: std::time::SystemTime::now(),
        };
        let cc = std::env::var("CODELOOM_COMPILE_COMMANDS").ok();
        return clang::index_clang(conn, &[fi], repo_name, cc.as_deref(), branch);
    }

    let mut parser = match tree_sitter::create_parser(lang) {
        Some(p) => p,
        None => return Ok(0),
    };
    let fi = FileInfo {
        path: file_path.to_string(),
        language: lang,
        modified: std::time::SystemTime::now(),
    };
    let (symbols, edges_data) = tree_sitter::parse_file(&fi, &mut parser, repo_name)?;
    let count = symbols.len();
    let mut id_map = Vec::new();

    for sym in &symbols {
        let db_id = sym.insert(conn, branch)?;
        id_map.push(db_id);
        let branch_id = crate::storage::resolve_branch_id(conn, repo_name, branch)?;
        conn.execute(
            "INSERT OR IGNORE INTO branches (node_id,repo,branch_id,override_def,override_hash) VALUES (?1,?2,?3,NULL,NULL)",
            rusqlite::params![db_id, repo_name, branch_id],
        )?;
    }
    // Resolve target IDs: respect extractor's usize::MAX as unresolved,
    // then try id_map lookup, then fall back to name-based resolution
    for (src_idx, tgt_idx, edge) in &edges_data {
        if let Some(&src_db) = id_map.get(*src_idx) {
            let target_db = if *tgt_idx == usize::MAX {
                0 // extractor says unresolved — trust it
            } else if let Some(&tgt_db) = id_map.get(*tgt_idx) {
                tgt_db // extractor resolved within same batch
            } else {
                resolve_target(conn, &symbols, &id_map, edge, repo_name)
            };
            let _ = conn.execute(
                "INSERT OR IGNORE INTO edges (source_id,target_id,edge_type,source_repo) VALUES (?1,?2,?3,?4)",
                rusqlite::params![src_db, target_db, edge, repo_name],
            );
        }
    }
    // Create file node — extract leading comment block as summary (not all symbol doc_comments)
    let source = crate::util::read_file_smart(file_path)?;
    let summary = extract_leading_comment(&source);
    let fn_hash = crate::storage::dedup::hash_content(file_path);
    let branch_id = crate::storage::resolve_branch_id(conn, repo_name, branch)?;
    crate::storage::files::FileNode {
        id: None,
        repo: repo_name.to_string(),
        file_path: file_path.to_string(),
        file_type: "code".to_string(),
        summary,
        content_hash: fn_hash,
        branch_id,
    }
    .insert(conn, branch)?;

    Ok(count)
}

/// Extract target name from edge_type (e.g. "inherits:std::exception" → "std::exception"),
/// resolve to symbol ID: same-file → DB lookup → LIKE fallback → 0 if not found.
fn resolve_target(
    conn: &Connection,
    symbols: &[Symbol],
    id_map: &[i64],
    edge: &str,
    repo: &str,
) -> i64 {
    let target_name = edge.find(':').map(|i| &edge[i + 1..]).unwrap_or("");
    if target_name.is_empty() {
        return 0;
    }
    // 0. Strip overload suffix (N) for matching — "func(2)" → "func"
    let base_name = if let Some(open) = target_name.find('(') {
        if target_name.ends_with(')') { &target_name[..open] } else { target_name }
    } else { target_name };
    // 1. same-file lookup
    if let Some(idx) = symbols.iter().position(|s| s.name == target_name || s.name == base_name) {
        if let Some(&id) = id_map.get(idx) {
            return id;
        }
    }
    // 1b. same-file lookup with base_name (sans overload suffix)
    if base_name != target_name {
        if let Some(idx) = symbols.iter().position(|s| s.name == base_name) {
            if let Some(&id) = id_map.get(idx) {
                return id;
            }
        }
    }
    // 2. DB lookup by exact name+repo — try both target_name and base_name
    let query_name = if base_name != target_name { base_name } else { target_name };
    if let Ok(id) = conn.query_row(
        "SELECT id FROM nodes WHERE name=?1 AND repo=?2 AND node_type='sym' LIMIT 1",
        rusqlite::params![target_name, repo],
        |row| row.get(0),
    ) {
        return id;
    }
    // 3. LIKE fallback: match by qualified-name suffix (e.g., "size" matches "MyVector::size")
    // Use '%::X' not '%X%' to avoid single-char type names matching everything
    let like_pat = format!("%::{}", target_name);
    if let Ok(id) = conn.query_row(
        "SELECT id FROM nodes WHERE name LIKE ?1 AND repo=?2 AND node_type='sym' LIMIT 1",
        rusqlite::params![like_pat, repo],
        |row| row.get(0),
    ) {
        return id;
    }
    // 4. Check builtin symbols (__builtin__ repo) for fully-qualified names
    if let Ok(id) = conn.query_row(
        "SELECT id FROM nodes WHERE name=?1 AND repo='__builtin__' AND node_type='sym' LIMIT 1",
        rusqlite::params![target_name],
        |row| row.get(0),
    ) {
        return id;
    }
    // 5. LIKE fallback on builtin too
    if let Ok(id) = conn.query_row(
        "SELECT id FROM nodes WHERE name LIKE ?1 AND repo='__builtin__' AND node_type='sym' LIMIT 1",
        rusqlite::params![like_pat, repo],
        |row| row.get(0),
    ) {
        return id;
    }
    0
}

fn find_parent(
    conn: &Connection,
    repo_root: &str,
    repo_name: &str,
    head: &str,
) -> Option<String> {
    let mut stmt = conn
        .prepare("SELECT branch_name,head_commit FROM git_index_state WHERE repo=?1")
        .ok()?;
    let rows = stmt
        .query_map(rusqlite::params![repo_name], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .ok()?;
    let mut candidates: Vec<(String, i64)> = Vec::new();
    for row in rows {
        if let Ok((bn, ih)) = row {
            if ih == head {
                continue;
            }
            let base = git::merge_base(repo_root, head, &bn)?;
            if git::is_ancestor(repo_root, &base, &ih) {
                candidates.push((bn, commit_time(repo_root, &base)));
            }
        }
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates.into_iter().next().map(|(bn, _)| bn)
}

fn commit_time(repo_root: &str, commit: &str) -> i64 {
    std::process::Command::new("git")
        .args(["log", "-1", "--format=%ct", commit])
        .current_dir(repo_root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn update_state(
    conn: &Connection,
    repo: &str,
    branch: &str,
    head: &str,
    parent: Option<&str>,
    file_count: usize,
) -> anyhow::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR REPLACE INTO git_index_state (repo,branch_name,head_commit,parent_ref,indexed_files,indexed_at) VALUES (?1,?2,?3,?4,?5,?6)",
        rusqlite::params![repo, branch, head, parent, file_count as i64, now],
    )?;
    Ok(())
}

/// Extract leading consecutive comment lines from source code.
/// Returns the comment block (joined with " | "), empty if no leading comments.
fn extract_leading_comment(source: &str) -> String {
    let mut lines = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !lines.is_empty() {
                lines.push(""); // preserve blank lines within comment block
            }
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with('#') {
            lines.push(trimmed);
        } else {
            break; // first non-comment, non-blank line → end of header comment
        }
    }
    // Remove trailing blank lines
    while lines.last().map_or(false, |l| l.is_empty()) {
        lines.pop();
    }
    lines.join(" | ")
}
